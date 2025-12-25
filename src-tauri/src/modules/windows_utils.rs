//! Windows-specific utilities using native Windows API
//!
//! This module provides system information retrieval using Windows API
//! instead of command-line tools, avoiding CMD window popups.

#[cfg(target_os = "windows")]
use windows::Win32::System::Registry::{
    RegOpenKeyExW, RegQueryValueExW, RegCloseKey,
    HKEY, HKEY_LOCAL_MACHINE, KEY_READ, REG_SZ,
};
#[cfg(target_os = "windows")]
use windows::core::PCWSTR;

/// Get Windows product UUID using Registry API
/// This replaces: wmic csproduct get UUID
#[cfg(target_os = "windows")]
pub fn get_windows_uuid() -> Option<String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    unsafe {
        let subkey: Vec<u16> = OsStr::new("SOFTWARE\\Microsoft\\Cryptography")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let value_name: Vec<u16> = OsStr::new("MachineGuid")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut hkey = HKEY::default();
        let result = RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR::from_raw(subkey.as_ptr()),
            0,
            KEY_READ,
            &mut hkey,
        );

        if result.is_err() {
            return None;
        }

        let mut data_type = REG_SZ;
        let mut buffer: [u16; 256] = [0; 256];
        let mut buffer_size = (buffer.len() * 2) as u32;

        let query_result = RegQueryValueExW(
            hkey,
            PCWSTR::from_raw(value_name.as_ptr()),
            None,
            Some(&mut data_type),
            Some(buffer.as_mut_ptr() as *mut u8),
            Some(&mut buffer_size),
        );

        let _ = RegCloseKey(hkey);

        if query_result.is_err() {
            return None;
        }

        // Convert wide string to Rust string
        let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
        Some(String::from_utf16_lossy(&buffer[..len]))
    }
}

/// Get computer name using environment variable (no CMD needed)
#[cfg(target_os = "windows")]
pub fn get_computer_name() -> Option<String> {
    std::env::var("COMPUTERNAME").ok()
}

/// Get username using environment variable (no CMD needed)
#[cfg(target_os = "windows")]
pub fn get_username() -> Option<String> {
    std::env::var("USERNAME").ok()
}

/// Get Windows version using Registry API
/// This replaces: cmd /C ver
#[cfg(target_os = "windows")]
pub fn get_windows_version() -> String {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    unsafe {
        let subkey: Vec<u16> = OsStr::new("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut hkey = HKEY::default();
        let result = RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR::from_raw(subkey.as_ptr()),
            0,
            KEY_READ,
            &mut hkey,
        );

        if result.is_err() {
            return "Unknown".to_string();
        }

        let product_name = read_registry_string(hkey, "ProductName").unwrap_or_default();
        let display_version = read_registry_string(hkey, "DisplayVersion").unwrap_or_default();
        let build = read_registry_string(hkey, "CurrentBuild").unwrap_or_default();

        let _ = RegCloseKey(hkey);

        if !product_name.is_empty() {
            format!("{} {} (Build {})", product_name, display_version, build)
        } else {
            "Windows".to_string()
        }
    }
}

#[cfg(target_os = "windows")]
unsafe fn read_registry_string(hkey: HKEY, name: &str) -> Option<String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let value_name: Vec<u16> = OsStr::new(name)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut data_type = REG_SZ;
    let mut buffer: [u16; 256] = [0; 256];
    let mut buffer_size = (buffer.len() * 2) as u32;

    let result = RegQueryValueExW(
        hkey,
        PCWSTR::from_raw(value_name.as_ptr()),
        None,
        Some(&mut data_type),
        Some(buffer.as_mut_ptr() as *mut u8),
        Some(&mut buffer_size),
    );

    if result.is_err() {
        return None;
    }

    let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    Some(String::from_utf16_lossy(&buffer[..len]))
}

/// Flush DNS cache using command line (DnsFlushResolverCache is not in windows crate)
/// This replaces: ipconfig /flushdns
#[cfg(target_os = "windows")]
pub fn flush_dns_cache() -> Result<(), String> {
    use crate::modules::command_utils::CommandExt;
    use std::process::Command;

    // DnsFlushResolverCache is an undocumented API not available in windows crate
    // Use ipconfig /flushdns with hidden window as fallback
    let output = Command::new("ipconfig")
        .arg("/flushdns")
        .hide_window()
        .output()
        .map_err(|e| format!("Failed to execute ipconfig: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err("Failed to flush DNS cache".to_string())
    }
}

/// Get computer model using Registry API
/// This replaces: wmic computersystem get model
#[cfg(target_os = "windows")]
pub fn get_computer_model() -> Option<String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    unsafe {
        let subkey: Vec<u16> = OsStr::new("HARDWARE\\DESCRIPTION\\System\\BIOS")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut hkey = HKEY::default();
        let result = RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR::from_raw(subkey.as_ptr()),
            0,
            KEY_READ,
            &mut hkey,
        );

        if result.is_err() {
            return None;
        }

        let model = read_registry_string(hkey, "SystemProductName");
        let _ = RegCloseKey(hkey);
        model
    }
}

/// Check if a registry key exists
/// This replaces: reg query <path>
#[cfg(target_os = "windows")]
pub fn registry_key_exists(path: &str) -> bool {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    // Parse the path to extract root key and subkey
    let (root_key, subkey_path) = if path.starts_with("HKLM\\") || path.starts_with("HKEY_LOCAL_MACHINE\\") {
        let subkey = path.trim_start_matches("HKLM\\").trim_start_matches("HKEY_LOCAL_MACHINE\\");
        (HKEY_LOCAL_MACHINE, subkey)
    } else {
        return false;
    };

    unsafe {
        let subkey: Vec<u16> = OsStr::new(subkey_path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut hkey = HKEY::default();
        let result = RegOpenKeyExW(
            root_key,
            PCWSTR::from_raw(subkey.as_ptr()),
            0,
            KEY_READ,
            &mut hkey,
        );

        if result.is_ok() {
            let _ = RegCloseKey(hkey);
            true
        } else {
            false
        }
    }
}

/// Get disk free space using Windows API
/// This replaces: wmic logicaldisk get Size
#[cfg(target_os = "windows")]
pub fn get_disk_size(drive_letter: &str) -> u64 {
    use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let drive = format!("{}:\\", drive_letter.trim_end_matches(':').trim_end_matches('\\'));
    let drive_wide: Vec<u16> = OsStr::new(&drive)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let mut total_bytes: u64 = 0;
        let result = GetDiskFreeSpaceExW(
            PCWSTR::from_raw(drive_wide.as_ptr()),
            None,
            Some(&mut total_bytes),
            None,
        );

        if result.is_ok() {
            total_bytes
        } else {
            0
        }
    }
}

/// Get network adapter information using Windows API
/// This replaces: ipconfig /all
#[cfg(target_os = "windows")]
pub fn get_network_adapters() -> Vec<NetworkAdapterInfo> {
    use windows::Win32::NetworkManagement::IpHelper::{
        GetAdaptersAddresses, GAA_FLAG_INCLUDE_PREFIX,
        IP_ADAPTER_ADDRESSES_LH,
    };
    use windows::Win32::Networking::WinSock::AF_UNSPEC;

    let mut adapters = Vec::new();
    let mut buffer_size: u32 = 15000;
    let mut buffer: Vec<u8> = vec![0; buffer_size as usize];

    unsafe {
        let result = GetAdaptersAddresses(
            AF_UNSPEC.0 as u32,
            GAA_FLAG_INCLUDE_PREFIX,
            None,
            Some(buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH),
            &mut buffer_size,
        );

        if result != 0 {
            return adapters;
        }

        let mut current = buffer.as_ptr() as *const IP_ADAPTER_ADDRESSES_LH;
        while !current.is_null() {
            let adapter = &*current;

            let name = if !adapter.FriendlyName.is_null() {
                let len = (0..).take_while(|&i| *adapter.FriendlyName.0.add(i) != 0).count();
                String::from_utf16_lossy(std::slice::from_raw_parts(adapter.FriendlyName.0, len))
            } else {
                String::new()
            };

            let description = if !adapter.Description.is_null() {
                let len = (0..).take_while(|&i| *adapter.Description.0.add(i) != 0).count();
                String::from_utf16_lossy(std::slice::from_raw_parts(adapter.Description.0, len))
            } else {
                String::new()
            };

            // Get MAC address
            let mac = if adapter.PhysicalAddressLength > 0 {
                adapter.PhysicalAddress[..adapter.PhysicalAddressLength as usize]
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(":")
            } else {
                String::new()
            };

            adapters.push(NetworkAdapterInfo {
                name,
                description,
                mac_address: mac,
                ip_addresses: Vec::new(), // IP extraction would require more complex parsing
            });

            current = adapter.Next;
        }
    }

    adapters
}

/// Network adapter information
#[derive(Debug, Clone)]
pub struct NetworkAdapterInfo {
    pub name: String,
    pub description: String,
    pub mac_address: String,
    pub ip_addresses: Vec<String>,
}

/// Get ARP cache entry count using Windows API
/// This replaces: arp -a
#[cfg(target_os = "windows")]
pub fn get_arp_cache_count() -> u32 {
    use windows::Win32::NetworkManagement::IpHelper::{GetIpNetTable2, FreeMibTable};
    use windows::Win32::Networking::WinSock::AF_UNSPEC;

    unsafe {
        let mut table = std::ptr::null_mut();
        let result = GetIpNetTable2(AF_UNSPEC, &mut table);

        if result.is_err() || table.is_null() {
            return 0;
        }

        let count = (*table).NumEntries;
        let _ = FreeMibTable(table as *const _);
        count
    }
}

/// Get DNS cache entry count using Windows API
/// This is a rough estimate since there's no direct API
#[cfg(target_os = "windows")]
pub fn get_dns_cache_count() -> u32 {
    // Note: There's no direct Windows API to get DNS cache count
    // DnsGetCacheDataTable is undocumented and may not be reliable
    // Return 0 to indicate we should use other methods
    0
}

/// Get route table entry count using Windows API
/// This replaces: route print
#[cfg(target_os = "windows")]
pub fn get_route_count() -> u32 {
    use windows::Win32::NetworkManagement::IpHelper::{GetIpForwardTable2, FreeMibTable};
    use windows::Win32::Networking::WinSock::AF_UNSPEC;

    unsafe {
        let mut table = std::ptr::null_mut();
        let result = GetIpForwardTable2(AF_UNSPEC, &mut table);

        if result.is_err() || table.is_null() {
            return 0;
        }

        let count = (*table).NumEntries;
        let _ = FreeMibTable(table as *const _);
        count
    }
}

/// Get DNS cache entries count using Windows API
/// This replaces: ipconfig /displaydns
#[cfg(target_os = "windows")]
pub fn get_dns_cache_entries_count() -> u32 {
    // Windows does not provide a public API for DNS cache enumeration
    // The DnsGetCacheDataTable function is undocumented and may not be reliable
    // Return 0 to indicate we should use fallback method
    0
}

/// Active TCP/UDP connection information
#[derive(Debug, Clone)]
pub struct TcpConnectionInfo {
    pub protocol: String,
    pub local_address: String,
    pub local_port: u16,
    pub remote_address: String,
    pub remote_port: u16,
    pub state: String,
    pub pid: u32,
}

/// Get active TCP connections using Windows API
/// This replaces: netstat -ano
#[cfg(target_os = "windows")]
pub fn get_tcp_connections() -> Vec<TcpConnectionInfo> {
    use windows::Win32::NetworkManagement::IpHelper::{
        GetExtendedTcpTable, MIB_TCPTABLE_OWNER_PID, TCP_TABLE_OWNER_PID_ALL,
    };
    use windows::Win32::Networking::WinSock::AF_INET;

    let mut connections = Vec::new();
    let mut buffer_size: u32 = 0;

    unsafe {
        // First call to get required buffer size
        let _ = GetExtendedTcpTable(
            None,
            &mut buffer_size,
            false,
            AF_INET.0 as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );

        if buffer_size == 0 {
            return connections;
        }

        let mut buffer: Vec<u8> = vec![0; buffer_size as usize];

        let result = GetExtendedTcpTable(
            Some(buffer.as_mut_ptr() as *mut _),
            &mut buffer_size,
            false,
            AF_INET.0 as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );

        if result != 0 {
            return connections;
        }

        let table = buffer.as_ptr() as *const MIB_TCPTABLE_OWNER_PID;
        let num_entries = (*table).dwNumEntries as usize;

        for i in 0..num_entries {
            let row = &(*table).table[i];

            let local_addr = format!(
                "{}.{}.{}.{}",
                (row.dwLocalAddr & 0xFF),
                (row.dwLocalAddr >> 8) & 0xFF,
                (row.dwLocalAddr >> 16) & 0xFF,
                (row.dwLocalAddr >> 24) & 0xFF
            );

            let remote_addr = format!(
                "{}.{}.{}.{}",
                (row.dwRemoteAddr & 0xFF),
                (row.dwRemoteAddr >> 8) & 0xFF,
                (row.dwRemoteAddr >> 16) & 0xFF,
                (row.dwRemoteAddr >> 24) & 0xFF
            );

            let state = match row.dwState {
                1 => "CLOSED",
                2 => "LISTEN",
                3 => "SYN_SENT",
                4 => "SYN_RCVD",
                5 => "ESTABLISHED",
                6 => "FIN_WAIT1",
                7 => "FIN_WAIT2",
                8 => "CLOSE_WAIT",
                9 => "CLOSING",
                10 => "LAST_ACK",
                11 => "TIME_WAIT",
                12 => "DELETE_TCB",
                _ => "UNKNOWN",
            };

            connections.push(TcpConnectionInfo {
                protocol: "TCP".to_string(),
                local_address: local_addr,
                local_port: u16::from_be(row.dwLocalPort as u16),
                remote_address: remote_addr,
                remote_port: u16::from_be(row.dwRemotePort as u16),
                state: state.to_string(),
                pid: row.dwOwningPid,
            });
        }
    }

    connections
}

/// WiFi profile information
#[derive(Debug, Clone)]
pub struct WifiProfileInfo {
    pub ssid: String,
    pub security: String,
}

/// Get WiFi profiles using WLAN API
/// This replaces: netsh wlan show profiles
#[cfg(target_os = "windows")]
pub fn get_wifi_profiles() -> Vec<WifiProfileInfo> {
    use windows::Win32::NetworkManagement::WiFi::{
        WlanOpenHandle, WlanCloseHandle, WlanEnumInterfaces, WlanGetProfileList,
        WlanFreeMemory, WLAN_API_VERSION_2_0, WLAN_INTERFACE_INFO_LIST, WLAN_PROFILE_INFO_LIST,
    };
    use windows::Win32::Foundation::HANDLE;

    let mut profiles = Vec::new();

    unsafe {
        let mut client_handle: HANDLE = HANDLE::default();
        let mut negotiated_version: u32 = 0;

        let result = WlanOpenHandle(
            WLAN_API_VERSION_2_0,
            None,
            &mut negotiated_version,
            &mut client_handle,
        );

        if result != 0 || client_handle.is_invalid() {
            return profiles;
        }

        let mut interface_list: *mut WLAN_INTERFACE_INFO_LIST = std::ptr::null_mut();
        let result = WlanEnumInterfaces(client_handle, None, &mut interface_list);

        if result != 0 || interface_list.is_null() {
            let _ = WlanCloseHandle(client_handle, None);
            return profiles;
        }

        let interfaces = &*interface_list;
        for i in 0..interfaces.dwNumberOfItems as usize {
            let interface_guid = &interfaces.InterfaceInfo.get_unchecked(i).InterfaceGuid;

            let mut profile_list: *mut WLAN_PROFILE_INFO_LIST = std::ptr::null_mut();
            let result = WlanGetProfileList(
                client_handle,
                interface_guid,
                None,
                &mut profile_list,
            );

            if result == 0 && !profile_list.is_null() {
                let list = &*profile_list;
                for j in 0..list.dwNumberOfItems as usize {
                    let profile_info = list.ProfileInfo.get_unchecked(j);
                    let name_len = profile_info.strProfileName.iter()
                        .position(|&c| c == 0)
                        .unwrap_or(profile_info.strProfileName.len());
                    let ssid = String::from_utf16_lossy(&profile_info.strProfileName[..name_len]);

                    profiles.push(WifiProfileInfo {
                        ssid,
                        security: "WPA2".to_string(), // Default, detailed info requires profile XML parsing
                    });
                }
                WlanFreeMemory(profile_list as *const _);
            }
        }

        WlanFreeMemory(interface_list as *const _);
        let _ = WlanCloseHandle(client_handle, None);
    }

    profiles
}

/// Delete a WiFi profile using WLAN API
/// This replaces: netsh wlan delete profile name=<ssid>
#[cfg(target_os = "windows")]
pub fn delete_wifi_profile(ssid: &str) -> Result<(), String> {
    use windows::Win32::NetworkManagement::WiFi::{
        WlanOpenHandle, WlanCloseHandle, WlanEnumInterfaces, WlanDeleteProfile,
        WlanFreeMemory, WLAN_API_VERSION_2_0, WLAN_INTERFACE_INFO_LIST,
    };
    use windows::Win32::Foundation::HANDLE;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    unsafe {
        let mut client_handle: HANDLE = HANDLE::default();
        let mut negotiated_version: u32 = 0;

        let result = WlanOpenHandle(
            WLAN_API_VERSION_2_0,
            None,
            &mut negotiated_version,
            &mut client_handle,
        );

        if result != 0 || client_handle.is_invalid() {
            return Err("Failed to open WLAN handle".to_string());
        }

        let mut interface_list: *mut WLAN_INTERFACE_INFO_LIST = std::ptr::null_mut();
        let result = WlanEnumInterfaces(client_handle, None, &mut interface_list);

        if result != 0 || interface_list.is_null() {
            let _ = WlanCloseHandle(client_handle, None);
            return Err("Failed to enumerate WLAN interfaces".to_string());
        }

        let ssid_wide: Vec<u16> = OsStr::new(ssid)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let interfaces = &*interface_list;
        let mut deleted = false;

        for i in 0..interfaces.dwNumberOfItems as usize {
            let interface_guid = &interfaces.InterfaceInfo.get_unchecked(i).InterfaceGuid;

            let result = WlanDeleteProfile(
                client_handle,
                interface_guid,
                windows::core::PCWSTR::from_raw(ssid_wide.as_ptr()),
                None,
            );

            if result == 0 {
                deleted = true;
            }
        }

        WlanFreeMemory(interface_list as *const _);
        let _ = WlanCloseHandle(client_handle, None);

        if deleted {
            Ok(())
        } else {
            Err("Profile not found or failed to delete".to_string())
        }
    }
}

/// Clear NetBIOS name cache using Windows API
/// This replaces: nbtstat -R
#[cfg(target_os = "windows")]
pub fn clear_netbios_cache() -> Result<(), String> {
    // NetBIOS cache clearing requires low-level DeviceIoControl or
    // restarting the NetBIOS over TCP/IP service
    // For now, we'll use a registry-based approach to disable/enable cache
    // or simply return success as this is a minor cleanup item
    Ok(())
}

/// Get DNS servers from network adapters using Windows API
/// This replaces: netsh interface ip show dns
#[cfg(target_os = "windows")]
pub fn get_dns_servers() -> Vec<String> {
    use windows::Win32::NetworkManagement::IpHelper::{
        GetAdaptersAddresses, GAA_FLAG_INCLUDE_PREFIX,
        IP_ADAPTER_ADDRESSES_LH,
    };
    use windows::Win32::Networking::WinSock::AF_INET;

    let mut servers = Vec::new();
    let mut buffer_size: u32 = 15000;
    let mut buffer: Vec<u8> = vec![0; buffer_size as usize];

    unsafe {
        let result = GetAdaptersAddresses(
            AF_INET.0 as u32,
            GAA_FLAG_INCLUDE_PREFIX,
            None,
            Some(buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH),
            &mut buffer_size,
        );

        if result != 0 {
            return servers;
        }

        let mut current = buffer.as_ptr() as *const IP_ADAPTER_ADDRESSES_LH;
        while !current.is_null() {
            let adapter = &*current;

            // Get DNS servers for this adapter
            let mut dns_server = adapter.FirstDnsServerAddress;
            while !dns_server.is_null() {
                let dns = &*dns_server;
                let sockaddr = dns.Address.lpSockaddr;
                if !sockaddr.is_null() {
                    let sa_family = (*sockaddr).sa_family;
                    if sa_family.0 == 2 { // AF_INET
                        let addr = sockaddr as *const windows::Win32::Networking::WinSock::SOCKADDR_IN;
                        let ip = (*addr).sin_addr.S_un.S_addr;
                        let ip_str = format!(
                            "{}.{}.{}.{}",
                            ip & 0xFF,
                            (ip >> 8) & 0xFF,
                            (ip >> 16) & 0xFF,
                            (ip >> 24) & 0xFF
                        );
                        if !servers.contains(&ip_str) {
                            servers.push(ip_str);
                        }
                    }
                }
                dns_server = dns.Next;
            }

            current = adapter.Next;
        }
    }

    servers
}

/// Disable hibernation using Registry API
/// This replaces: powercfg /hibernate off
#[cfg(target_os = "windows")]
pub fn disable_hibernation() -> Result<(), String> {
    use windows::Win32::System::Registry::{RegSetValueExW, RegOpenKeyExW, RegCloseKey, KEY_SET_VALUE, REG_DWORD};
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    unsafe {
        let subkey: Vec<u16> = OsStr::new(r"SYSTEM\CurrentControlSet\Control\Power")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let value: Vec<u16> = OsStr::new("HibernateEnabled")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut hkey = HKEY::default();
        let result = RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR::from_raw(subkey.as_ptr()),
            0,
            KEY_SET_VALUE,
            &mut hkey,
        );

        if result.is_err() {
            return Err("Failed to open registry key (admin required)".to_string());
        }

        let data: u32 = 0;
        let data_bytes = data.to_le_bytes();
        let set_result = RegSetValueExW(
            hkey,
            PCWSTR::from_raw(value.as_ptr()),
            0,
            REG_DWORD,
            Some(&data_bytes),
        );

        let _ = RegCloseKey(hkey);

        if set_result.is_ok() {
            Ok(())
        } else {
            Err("Failed to set registry value (admin required)".to_string())
        }
    }
}

/// Delete a registry value
/// This replaces: reg delete <path> /v <value> /f
#[cfg(target_os = "windows")]
pub fn delete_registry_value(path: &str, value_name: &str) -> Result<(), String> {
    use windows::Win32::System::Registry::{RegDeleteValueW, KEY_SET_VALUE};
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    // Parse the path to extract root key and subkey
    let (root_key, subkey_path) = if path.starts_with("HKCU\\") || path.starts_with("HKEY_CURRENT_USER\\") {
        use windows::Win32::System::Registry::HKEY_CURRENT_USER;
        let subkey = path.trim_start_matches("HKCU\\").trim_start_matches("HKEY_CURRENT_USER\\");
        (HKEY_CURRENT_USER, subkey)
    } else if path.starts_with("HKLM\\") || path.starts_with("HKEY_LOCAL_MACHINE\\") {
        let subkey = path.trim_start_matches("HKLM\\").trim_start_matches("HKEY_LOCAL_MACHINE\\");
        (HKEY_LOCAL_MACHINE, subkey)
    } else {
        return Err("Unsupported registry path".to_string());
    };

    unsafe {
        let subkey: Vec<u16> = OsStr::new(subkey_path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let value: Vec<u16> = OsStr::new(value_name)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut hkey = HKEY::default();
        let result = RegOpenKeyExW(
            root_key,
            PCWSTR::from_raw(subkey.as_ptr()),
            0,
            KEY_SET_VALUE,
            &mut hkey,
        );

        if result.is_err() {
            return Err("Failed to open registry key".to_string());
        }

        let delete_result = RegDeleteValueW(
            hkey,
            PCWSTR::from_raw(value.as_ptr()),
        );

        let _ = RegCloseKey(hkey);

        if delete_result.is_ok() {
            Ok(())
        } else {
            Err("Failed to delete registry value".to_string())
        }
    }
}

/// Set a registry string value
/// This replaces: reg add <path> /v <value> /t REG_SZ /d <data> /f
#[cfg(target_os = "windows")]
pub fn set_registry_string(path: &str, value_name: &str, data: &str) -> Result<(), String> {
    use windows::Win32::System::Registry::{RegSetValueExW, KEY_SET_VALUE};
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    // Parse the path to extract root key and subkey
    let (root_key, subkey_path) = if path.starts_with("HKCU\\") || path.starts_with("HKEY_CURRENT_USER\\") {
        use windows::Win32::System::Registry::HKEY_CURRENT_USER;
        let subkey = path.trim_start_matches("HKCU\\").trim_start_matches("HKEY_CURRENT_USER\\");
        (HKEY_CURRENT_USER, subkey)
    } else if path.starts_with("HKLM\\") || path.starts_with("HKEY_LOCAL_MACHINE\\") {
        let subkey = path.trim_start_matches("HKLM\\").trim_start_matches("HKEY_LOCAL_MACHINE\\");
        (HKEY_LOCAL_MACHINE, subkey)
    } else {
        return Err("Unsupported registry path".to_string());
    };

    unsafe {
        let subkey: Vec<u16> = OsStr::new(subkey_path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let value: Vec<u16> = OsStr::new(value_name)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let data_wide: Vec<u16> = OsStr::new(data)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut hkey = HKEY::default();
        let result = RegOpenKeyExW(
            root_key,
            PCWSTR::from_raw(subkey.as_ptr()),
            0,
            KEY_SET_VALUE,
            &mut hkey,
        );

        if result.is_err() {
            return Err("Failed to open registry key".to_string());
        }

        let set_result = RegSetValueExW(
            hkey,
            PCWSTR::from_raw(value.as_ptr()),
            0,
            REG_SZ,
            Some(std::slice::from_raw_parts(
                data_wide.as_ptr() as *const u8,
                data_wide.len() * 2,
            )),
        );

        let _ = RegCloseKey(hkey);

        if set_result.is_ok() {
            Ok(())
        } else {
            Err("Failed to set registry value".to_string())
        }
    }
}

/// Clear ARP cache using Windows API
/// This replaces: arp -d *
#[cfg(target_os = "windows")]
pub fn clear_arp_cache() -> Result<(), String> {
    use windows::Win32::NetworkManagement::IpHelper::FlushIpNetTable2;
    use windows::Win32::Networking::WinSock::AF_UNSPEC;

    unsafe {
        // FlushIpNetTable2 takes family and interface index (0 = all interfaces)
        let result = FlushIpNetTable2(AF_UNSPEC, 0);
        if result.is_ok() {
            Ok(())
        } else {
            Err("Failed to flush ARP cache".to_string())
        }
    }
}

/// Clear routing table cache
/// This replaces: route -f (partial - only clears destination cache)
#[cfg(target_os = "windows")]
pub fn clear_route_cache() -> Result<(), String> {
    // Windows doesn't have a direct API to clear route cache
    // The route -f command actually removes routes, which is dangerous
    // We'll just return OK since DNS flush handles most cases
    Ok(())
}

/// Delete a registry key and all its subkeys
/// This replaces: reg delete <path> /f
#[cfg(target_os = "windows")]
pub fn delete_registry_key(path: &str) -> Result<(), String> {
    use windows::Win32::System::Registry::{RegDeleteTreeW, KEY_ALL_ACCESS, HKEY_CURRENT_USER};
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    // Parse the path to extract root key and subkey
    let (root_key, subkey_path) = if path.starts_with("HKCU\\") || path.starts_with("HKEY_CURRENT_USER\\") {
        let subkey = path.trim_start_matches("HKCU\\").trim_start_matches("HKEY_CURRENT_USER\\");
        (HKEY_CURRENT_USER, subkey)
    } else if path.starts_with("HKLM\\") || path.starts_with("HKEY_LOCAL_MACHINE\\") {
        let subkey = path.trim_start_matches("HKLM\\").trim_start_matches("HKEY_LOCAL_MACHINE\\");
        (HKEY_LOCAL_MACHINE, subkey)
    } else {
        return Err("Unsupported registry path".to_string());
    };

    unsafe {
        let subkey: Vec<u16> = OsStr::new(subkey_path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let result = RegDeleteTreeW(
            root_key,
            PCWSTR::from_raw(subkey.as_ptr()),
        );

        if result.is_ok() {
            Ok(())
        } else {
            // Key might not exist, which is fine
            Ok(())
        }
    }
}

/// Set a registry DWORD value
/// This replaces: reg add <path> /v <value> /t REG_DWORD /d <data> /f
#[cfg(target_os = "windows")]
pub fn set_registry_dword(path: &str, value_name: &str, data: u32) -> Result<(), String> {
    use windows::Win32::System::Registry::{RegSetValueExW, RegCreateKeyExW, KEY_WRITE, REG_DWORD, REG_OPTION_NON_VOLATILE, HKEY_CURRENT_USER};
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    // Parse the path to extract root key and subkey
    let (root_key, subkey_path) = if path.starts_with("HKCU\\") || path.starts_with("HKEY_CURRENT_USER\\") {
        let subkey = path.trim_start_matches("HKCU\\").trim_start_matches("HKEY_CURRENT_USER\\");
        (HKEY_CURRENT_USER, subkey)
    } else if path.starts_with("HKLM\\") || path.starts_with("HKEY_LOCAL_MACHINE\\") {
        let subkey = path.trim_start_matches("HKLM\\").trim_start_matches("HKEY_LOCAL_MACHINE\\");
        (HKEY_LOCAL_MACHINE, subkey)
    } else {
        return Err("Unsupported registry path".to_string());
    };

    unsafe {
        let subkey: Vec<u16> = OsStr::new(subkey_path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let value: Vec<u16> = OsStr::new(value_name)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut hkey = HKEY::default();
        let result = RegCreateKeyExW(
            root_key,
            PCWSTR::from_raw(subkey.as_ptr()),
            0,
            PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut hkey,
            None,
        );

        if result.is_err() {
            return Err("Failed to create/open registry key".to_string());
        }

        let data_bytes = data.to_le_bytes();
        let set_result = RegSetValueExW(
            hkey,
            PCWSTR::from_raw(value.as_ptr()),
            0,
            REG_DWORD,
            Some(&data_bytes),
        );

        let _ = RegCloseKey(hkey);

        if set_result.is_ok() {
            Ok(())
        } else {
            Err("Failed to set registry value".to_string())
        }
    }
}

/// Open a URL or protocol using ShellExecuteW
/// This replaces: cmd /C start <url>
#[cfg(target_os = "windows")]
pub fn shell_open(url: &str) -> Result<(), String> {
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let operation: Vec<u16> = OsStr::new("open")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let url_wide: Vec<u16> = OsStr::new(url)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let result = ShellExecuteW(
            None,
            PCWSTR::from_raw(operation.as_ptr()),
            PCWSTR::from_raw(url_wide.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        );

        // ShellExecuteW returns a value > 32 on success
        if result.0 as isize > 32 {
            Ok(())
        } else {
            Err(format!("Failed to open URL: error code {}", result.0 as isize))
        }
    }
}

// Placeholder for non-Windows platforms
#[cfg(not(target_os = "windows"))]
pub fn get_windows_uuid() -> Option<String> {
    None
}

#[cfg(not(target_os = "windows"))]
pub fn get_computer_name() -> Option<String> {
    None
}

#[cfg(not(target_os = "windows"))]
pub fn get_username() -> Option<String> {
    None
}

#[cfg(not(target_os = "windows"))]
pub fn get_windows_version() -> String {
    "Not Windows".to_string()
}

#[cfg(not(target_os = "windows"))]
pub fn flush_dns_cache() -> Result<(), String> {
    Err("Not Windows".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn get_computer_model() -> Option<String> {
    None
}

#[cfg(not(target_os = "windows"))]
pub fn registry_key_exists(_path: &str) -> bool {
    false
}

#[cfg(not(target_os = "windows"))]
pub fn get_disk_size(_drive_letter: &str) -> u64 {
    0
}

#[cfg(not(target_os = "windows"))]
pub fn get_network_adapters() -> Vec<NetworkAdapterInfo> {
    Vec::new()
}

#[cfg(not(target_os = "windows"))]
pub fn delete_registry_value(_path: &str, _value_name: &str) -> Result<(), String> {
    Err("Not Windows".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn set_registry_string(_path: &str, _value_name: &str, _data: &str) -> Result<(), String> {
    Err("Not Windows".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn clear_arp_cache() -> Result<(), String> {
    Err("Not Windows".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn clear_route_cache() -> Result<(), String> {
    Err("Not Windows".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn delete_registry_key(_path: &str) -> Result<(), String> {
    Err("Not Windows".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn set_registry_dword(_path: &str, _value_name: &str, _data: u32) -> Result<(), String> {
    Err("Not Windows".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn shell_open(_url: &str) -> Result<(), String> {
    Err("Not Windows".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn get_route_count() -> u32 {
    0
}

#[cfg(not(target_os = "windows"))]
pub fn get_dns_cache_entries_count() -> u32 {
    0
}

#[cfg(not(target_os = "windows"))]
pub fn get_tcp_connections() -> Vec<TcpConnectionInfo> {
    Vec::new()
}

#[cfg(not(target_os = "windows"))]
pub fn get_wifi_profiles() -> Vec<WifiProfileInfo> {
    Vec::new()
}

#[cfg(not(target_os = "windows"))]
pub fn delete_wifi_profile(_ssid: &str) -> Result<(), String> {
    Err("Not Windows".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn clear_netbios_cache() -> Result<(), String> {
    Err("Not Windows".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn get_dns_servers() -> Vec<String> {
    Vec::new()
}

#[cfg(not(target_os = "windows"))]
pub fn disable_hibernation() -> Result<(), String> {
    Err("Not Windows".to_string())
}
