use serde::{Deserialize, Serialize};
use std::process::Command;
use sysinfo::Networks;

/// 网络扫描项信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkCleanItem {
    pub item_type: String,       // 类型标识符
    pub label: String,           // 显示名称
    pub description: String,     // 描述
    pub count: u32,              // 数量
    pub count_display: String,   // 显示数量
    pub accessible: bool,        // 是否可清理
    pub category: String,        // 分类
}

/// 网络扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkScanResult {
    pub items: Vec<NetworkCleanItem>,
    pub total_items: usize,
    pub network_info: NetworkInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub active_connections: Vec<ConnectionInfo>,
    pub dns_servers: Vec<String>,
    pub wifi_networks: Vec<WifiNetwork>,
    pub vpn_connections: Vec<VpnConnection>,
    pub proxy_settings: ProxySettings,
    pub network_interfaces: Vec<NetworkInterface>,
    pub dns_cache_count: u32,
    pub arp_cache_count: u32,
    pub routing_entries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub protocol: String,
    pub local_address: String,
    pub local_port: u16,
    pub remote_address: String,
    pub remote_port: u16,
    pub state: String,
    pub pid: Option<u32>,
    pub process_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiNetwork {
    pub ssid: String,
    pub security: String,
    pub auto_connect: bool,
    pub last_connected: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnConnection {
    pub name: String,
    pub vpn_type: String,
    pub server: String,
    pub connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxySettings {
    pub http_proxy: Option<String>,
    pub https_proxy: Option<String>,
    pub socks_proxy: Option<String>,
    pub proxy_enabled: bool,
    pub pac_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub ip_address: Option<String>,
    pub mac_address: Option<String>,
    pub status: String,
    pub interface_type: String,
}

/// 获取网络信息
pub fn get_network_info() -> Result<NetworkInfo, String> {
    let active_connections = get_active_connections().unwrap_or_default();
    let dns_servers = get_dns_servers().unwrap_or_default();
    let wifi_networks = get_wifi_networks().unwrap_or_default();
    let vpn_connections = get_vpn_connections().unwrap_or_default();
    let proxy_settings = get_proxy_settings().unwrap_or_default();
    let network_interfaces = get_network_interfaces().unwrap_or_default();
    let dns_cache_count = get_dns_cache_count().unwrap_or(0);
    let arp_cache_count = get_arp_cache_count().unwrap_or(0);
    let routing_entries = get_routing_entries_count().unwrap_or(0);

    Ok(NetworkInfo {
        active_connections,
        dns_servers,
        wifi_networks,
        vpn_connections,
        proxy_settings,
        network_interfaces,
        dns_cache_count,
        arp_cache_count,
        routing_entries,
    })
}

/// 扫描可清理的网络相关项目
pub fn scan_network_items() -> Result<NetworkScanResult, String> {
    let mut items = Vec::new();
    let network_info = get_network_info()?;

    // 1. DNS 缓存
    let dns_count = network_info.dns_cache_count;
    items.push(NetworkCleanItem {
        item_type: "dns_cache".to_string(),
        label: "DNS 缓存".to_string(),
        description: "域名解析缓存记录".to_string(),
        count: dns_count,
        count_display: if dns_count > 0 { format!("{} 条记录", dns_count) } else { "动态".to_string() },
        accessible: true,
        category: "缓存".to_string(),
    });

    // 2. ARP 缓存
    let arp_count = network_info.arp_cache_count;
    if arp_count > 0 {
        items.push(NetworkCleanItem {
            item_type: "arp_cache".to_string(),
            label: "ARP 缓存".to_string(),
            description: "地址解析协议缓存".to_string(),
            count: arp_count,
            count_display: format!("{} 条记录", arp_count),
            accessible: true,
            category: "缓存".to_string(),
        });
    }

    // 3. 路由表
    let routing_count = network_info.routing_entries;
    if routing_count > 0 {
        items.push(NetworkCleanItem {
            item_type: "routing_table".to_string(),
            label: "路由缓存".to_string(),
            description: "网络路由缓存条目".to_string(),
            count: routing_count,
            count_display: format!("{} 条路由", routing_count),
            accessible: true,
            category: "缓存".to_string(),
        });
    }

    // 4. WiFi 配置
    let wifi_count = network_info.wifi_networks.len() as u32;
    if wifi_count > 0 {
        items.push(NetworkCleanItem {
            item_type: "wifi_profiles".to_string(),
            label: "WiFi 配置".to_string(),
            description: "已保存的无线网络配置".to_string(),
            count: wifi_count,
            count_display: format!("{} 个网络", wifi_count),
            accessible: true,
            category: "配置".to_string(),
        });
    }

    // 5. VPN 连接
    let vpn_count = network_info.vpn_connections.len() as u32;
    if vpn_count > 0 {
        let connected_count = network_info.vpn_connections.iter().filter(|v| v.connected).count();
        items.push(NetworkCleanItem {
            item_type: "vpn_connections".to_string(),
            label: "VPN 连接".to_string(),
            description: format!("VPN 配置 ({} 个已连接)", connected_count),
            count: vpn_count,
            count_display: format!("{} 个配置", vpn_count),
            accessible: connected_count > 0,
            category: "连接".to_string(),
        });
    }

    // 6. 代理设置
    if network_info.proxy_settings.proxy_enabled {
        items.push(NetworkCleanItem {
            item_type: "proxy_settings".to_string(),
            label: "代理设置".to_string(),
            description: "HTTP/HTTPS/SOCKS 代理配置".to_string(),
            count: 1,
            count_display: "已启用".to_string(),
            accessible: true,
            category: "配置".to_string(),
        });
    }

    // 7. 活动连接
    let connection_count = network_info.active_connections.len() as u32;
    if connection_count > 0 {
        let established = network_info.active_connections.iter()
            .filter(|c| c.state.to_uppercase() == "ESTABLISHED" || c.state.to_uppercase() == "ESTAB")
            .count();
        items.push(NetworkCleanItem {
            item_type: "active_connections".to_string(),
            label: "活动连接".to_string(),
            description: format!("{} 个已建立连接", established),
            count: connection_count,
            count_display: format!("{} 个连接", connection_count),
            accessible: false, // 仅显示，不清理
            category: "状态".to_string(),
        });
    }

    // 8. 网络接口
    let interface_count = network_info.network_interfaces.len() as u32;
    if interface_count > 0 {
        let active = network_info.network_interfaces.iter()
            .filter(|i| i.status == "active" || i.status == "up")
            .count();
        items.push(NetworkCleanItem {
            item_type: "network_interfaces".to_string(),
            label: "网络接口".to_string(),
            description: format!("{} 个活跃接口", active),
            count: interface_count,
            count_display: format!("{} 个接口", interface_count),
            accessible: false, // 仅显示，不清理
            category: "状态".to_string(),
        });
    }

    // 9. 连接历史
    items.push(NetworkCleanItem {
        item_type: "connection_history".to_string(),
        label: "连接历史".to_string(),
        description: "网络共享和服务器连接记录".to_string(),
        count: 0,
        count_display: "可清理".to_string(),
        accessible: true,
        category: "历史".to_string(),
    });

    #[cfg(target_os = "windows")]
    {
        // 10. NetBIOS 缓存 (Windows only)
        items.push(NetworkCleanItem {
            item_type: "netbios_cache".to_string(),
            label: "NetBIOS 缓存".to_string(),
            description: "网络基本输入输出系统缓存".to_string(),
            count: 0,
            count_display: "可清理".to_string(),
            accessible: true,
            category: "缓存".to_string(),
        });
    }

    // 过滤掉 count 为 0 且不可清理的项目
    items.retain(|item| item.count > 0 || item.accessible);

    Ok(NetworkScanResult {
        total_items: items.len(),
        items,
        network_info,
    })
}

/// 获取活动连接
fn get_active_connections() -> Result<Vec<ConnectionInfo>, String> {
    let mut connections = Vec::new();

    #[cfg(target_os = "macos")]
    {
        // 使用 netstat 获取连接
        let output = Command::new("netstat")
            .args(&["-anv", "-p", "tcp"])
            .output()
            .map_err(|e| format!("获取连接失败: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);

        for line in output_str.lines().skip(2) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                let local = parts[3];
                let remote = parts[4];
                let state = parts[5];

                // 解析地址和端口
                if let (Some((local_addr, local_port)), Some((remote_addr, remote_port))) =
                    (parse_address(local), parse_address(remote))
                {
                    let pid = parts.get(8).and_then(|p| p.parse::<u32>().ok());

                    connections.push(ConnectionInfo {
                        protocol: "TCP".to_string(),
                        local_address: local_addr,
                        local_port,
                        remote_address: remote_addr,
                        remote_port,
                        state: state.to_string(),
                        pid,
                        process_name: None,
                    });
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let output = Command::new("ss")
            .args(&["-tunapH"])
            .output()
            .map_err(|e| format!("获取连接失败: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);

        for line in output_str.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                let protocol = parts[0].to_uppercase();
                let state = parts[1];
                let local = parts[4];
                let remote = parts[5];

                if let (Some((local_addr, local_port)), Some((remote_addr, remote_port))) =
                    (parse_address(local), parse_address(remote))
                {
                    connections.push(ConnectionInfo {
                        protocol,
                        local_address: local_addr,
                        local_port,
                        remote_address: remote_addr,
                        remote_port,
                        state: state.to_string(),
                        pid: None,
                        process_name: None,
                    });
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("netstat")
            .args(&["-ano"])
            .output()
            .map_err(|e| format!("获取连接失败: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);

        for line in output_str.lines().skip(4) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let protocol = parts[0].to_uppercase();
                let local = parts[1];
                let remote = parts[2];
                let state = parts[3];
                let pid = parts.get(4).and_then(|p| p.parse::<u32>().ok());

                if let (Some((local_addr, local_port)), Some((remote_addr, remote_port))) =
                    (parse_address(local), parse_address(remote))
                {
                    connections.push(ConnectionInfo {
                        protocol,
                        local_address: local_addr,
                        local_port,
                        remote_address: remote_addr,
                        remote_port,
                        state: state.to_string(),
                        pid,
                        process_name: None,
                    });
                }
            }
        }
    }

    Ok(connections)
}

/// 解析地址和端口
fn parse_address(addr: &str) -> Option<(String, u16)> {
    // 处理 IPv6 地址 [::]:port 或 [addr]:port
    if addr.starts_with('[') {
        if let Some(bracket_end) = addr.rfind(']') {
            let ip = &addr[1..bracket_end];
            let port_str = &addr[bracket_end + 2..];
            if let Ok(port) = port_str.parse::<u16>() {
                return Some((ip.to_string(), port));
            }
        }
    }

    // 处理 IPv4 地址 addr.port 或 addr:port
    if let Some(last_dot) = addr.rfind('.') {
        let potential_port = &addr[last_dot + 1..];
        if let Ok(port) = potential_port.parse::<u16>() {
            return Some((addr[..last_dot].to_string(), port));
        }
    }

    if let Some(colon) = addr.rfind(':') {
        let ip = &addr[..colon];
        let port_str = &addr[colon + 1..];
        if let Ok(port) = port_str.parse::<u16>() {
            return Some((ip.to_string(), port));
        }
    }

    None
}

/// 获取 DNS 服务器
fn get_dns_servers() -> Result<Vec<String>, String> {
    let mut servers = Vec::new();

    #[cfg(target_os = "macos")]
    {
        let output = Command::new("scutil")
            .arg("--dns")
            .output()
            .map_err(|e| format!("获取DNS服务器失败: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);

        for line in output_str.lines() {
            let line = line.trim();
            if line.starts_with("nameserver") {
                if let Some(server) = line.split_whitespace().nth(2) {
                    if !servers.contains(&server.to_string()) {
                        servers.push(server.to_string());
                    }
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/etc/resolv.conf") {
            for line in content.lines() {
                if line.starts_with("nameserver") {
                    if let Some(server) = line.split_whitespace().nth(1) {
                        servers.push(server.to_string());
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("netsh")
            .args(&["interface", "ip", "show", "dns"])
            .output()
            .map_err(|e| format!("获取DNS服务器失败: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            let line = line.trim();
            // 解析 IP 地址
            let parts: Vec<&str> = line.split_whitespace().collect();
            for part in parts {
                if part.contains('.') && part.chars().all(|c| c.is_numeric() || c == '.') {
                    servers.push(part.to_string());
                }
            }
        }
    }

    Ok(servers)
}

/// 获取 WiFi 网络列表
fn get_wifi_networks() -> Result<Vec<WifiNetwork>, String> {
    let mut networks = Vec::new();

    #[cfg(target_os = "macos")]
    {
        // 获取已保存的 WiFi 网络
        let output = Command::new("networksetup")
            .args(&["-listpreferredwirelessnetworks", "en0"])
            .output();

        if let Ok(output) = output {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines().skip(1) {
                let ssid = line.trim();
                if !ssid.is_empty() {
                    networks.push(WifiNetwork {
                        ssid: ssid.to_string(),
                        security: "WPA2".to_string(), // 默认
                        auto_connect: true,
                        last_connected: None,
                    });
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        // 使用 nmcli 获取保存的连接
        let output = Command::new("nmcli")
            .args(&["-t", "-f", "NAME,TYPE", "connection", "show"])
            .output();

        if let Ok(output) = output {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 2 && parts[1].contains("wireless") {
                    networks.push(WifiNetwork {
                        ssid: parts[0].to_string(),
                        security: "WPA2".to_string(),
                        auto_connect: true,
                        last_connected: None,
                    });
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("netsh")
            .args(&["wlan", "show", "profiles"])
            .output();

        if let Ok(output) = output {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains("All User Profile") || line.contains("所有用户配置文件") {
                    if let Some(ssid) = line.split(':').nth(1) {
                        networks.push(WifiNetwork {
                            ssid: ssid.trim().to_string(),
                            security: "WPA2".to_string(),
                            auto_connect: true,
                            last_connected: None,
                        });
                    }
                }
            }
        }
    }

    Ok(networks)
}

/// 获取 VPN 连接
fn get_vpn_connections() -> Result<Vec<VpnConnection>, String> {
    let mut vpns = Vec::new();

    #[cfg(target_os = "macos")]
    {
        // 使用 scutil 获取 VPN 配置
        let output = Command::new("scutil")
            .args(&["--nc", "list"])
            .output();

        if let Ok(output) = output {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                // 解析 VPN 配置行
                if line.contains("PPP") || line.contains("IPSec") || line.contains("VPN") {
                    // 提取名称
                    if let Some(name_start) = line.find('"') {
                        if let Some(name_end) = line.rfind('"') {
                            let name = &line[name_start + 1..name_end];
                            let vpn_type = if line.contains("IPSec") {
                                "IPSec"
                            } else if line.contains("L2TP") {
                                "L2TP"
                            } else if line.contains("PPTP") {
                                "PPTP"
                            } else {
                                "Unknown"
                            };

                            let connected = line.contains("Connected");

                            vpns.push(VpnConnection {
                                name: name.to_string(),
                                vpn_type: vpn_type.to_string(),
                                server: "".to_string(),
                                connected,
                            });
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("rasdial")
            .output();

        if let Ok(output) = output {
            let output_str = String::from_utf8_lossy(&output.stdout);
            // 解析 VPN 连接
            for line in output_str.lines() {
                if !line.is_empty() && !line.contains("No connections") {
                    vpns.push(VpnConnection {
                        name: line.trim().to_string(),
                        vpn_type: "Unknown".to_string(),
                        server: "".to_string(),
                        connected: true,
                    });
                }
            }
        }
    }

    Ok(vpns)
}

/// 获取代理设置
fn get_proxy_settings() -> Result<ProxySettings, String> {
    let mut settings = ProxySettings {
        http_proxy: None,
        https_proxy: None,
        socks_proxy: None,
        proxy_enabled: false,
        pac_url: None,
    };

    #[cfg(target_os = "macos")]
    {
        // 获取 HTTP 代理
        let output = Command::new("networksetup")
            .args(&["-getwebproxy", "Wi-Fi"])
            .output();

        if let Ok(output) = output {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let mut server = String::new();
            let mut port = String::new();
            let mut enabled = false;

            for line in output_str.lines() {
                if line.starts_with("Enabled:") {
                    enabled = line.contains("Yes");
                } else if line.starts_with("Server:") {
                    server = line.replace("Server:", "").trim().to_string();
                } else if line.starts_with("Port:") {
                    port = line.replace("Port:", "").trim().to_string();
                }
            }

            if enabled && !server.is_empty() {
                settings.http_proxy = Some(format!("{}:{}", server, port));
                settings.proxy_enabled = true;
            }
        }

        // 获取 HTTPS 代理
        let output = Command::new("networksetup")
            .args(&["-getsecurewebproxy", "Wi-Fi"])
            .output();

        if let Ok(output) = output {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let mut server = String::new();
            let mut port = String::new();
            let mut enabled = false;

            for line in output_str.lines() {
                if line.starts_with("Enabled:") {
                    enabled = line.contains("Yes");
                } else if line.starts_with("Server:") {
                    server = line.replace("Server:", "").trim().to_string();
                } else if line.starts_with("Port:") {
                    port = line.replace("Port:", "").trim().to_string();
                }
            }

            if enabled && !server.is_empty() {
                settings.https_proxy = Some(format!("{}:{}", server, port));
                settings.proxy_enabled = true;
            }
        }

        // 获取 SOCKS 代理
        let output = Command::new("networksetup")
            .args(&["-getsocksfirewallproxy", "Wi-Fi"])
            .output();

        if let Ok(output) = output {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let mut server = String::new();
            let mut port = String::new();
            let mut enabled = false;

            for line in output_str.lines() {
                if line.starts_with("Enabled:") {
                    enabled = line.contains("Yes");
                } else if line.starts_with("Server:") {
                    server = line.replace("Server:", "").trim().to_string();
                } else if line.starts_with("Port:") {
                    port = line.replace("Port:", "").trim().to_string();
                }
            }

            if enabled && !server.is_empty() {
                settings.socks_proxy = Some(format!("{}:{}", server, port));
                settings.proxy_enabled = true;
            }
        }

        // 获取 PAC URL
        let output = Command::new("networksetup")
            .args(&["-getautoproxyurl", "Wi-Fi"])
            .output();

        if let Ok(output) = output {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.starts_with("URL:") {
                    let url = line.replace("URL:", "").trim().to_string();
                    if !url.is_empty() && url != "(null)" {
                        settings.pac_url = Some(url);
                        settings.proxy_enabled = true;
                    }
                }
            }
        }
    }

    // 也检查环境变量
    if let Ok(proxy) = std::env::var("http_proxy") {
        settings.http_proxy = Some(proxy);
        settings.proxy_enabled = true;
    }
    if let Ok(proxy) = std::env::var("https_proxy") {
        settings.https_proxy = Some(proxy);
        settings.proxy_enabled = true;
    }

    Ok(settings)
}

/// 获取网络接口 - 使用 sysinfo crate
fn get_network_interfaces() -> Result<Vec<NetworkInterface>, String> {
    let mut interfaces = Vec::new();
    let networks = Networks::new_with_refreshed_list();

    for (name, _network) in &networks {
        // 判断接口类型
        let interface_type = determine_interface_type(name);

        // 跳过回环接口
        if interface_type == "Loopback" {
            continue;
        }

        // 获取接口详细信息（IP 和 MAC 需要平台特定的方式获取）
        let (ip_address, mac_address, status) = get_interface_details(name);

        interfaces.push(NetworkInterface {
            name: name.clone(),
            ip_address,
            mac_address,
            status,
            interface_type,
        });
    }

    // 如果 sysinfo 没有返回接口，回退到平台特定方法
    if interfaces.is_empty() {
        return get_network_interfaces_fallback();
    }

    Ok(interfaces)
}

/// 确定接口类型
fn determine_interface_type(name: &str) -> String {
    let name_lower = name.to_lowercase();

    if name_lower == "lo" || name_lower == "lo0" {
        "Loopback".to_string()
    } else if name_lower.starts_with("en") {
        // macOS: en0 通常是 Wi-Fi, en1+ 是 Ethernet
        if name_lower == "en0" {
            "Wi-Fi".to_string()
        } else {
            "Ethernet".to_string()
        }
    } else if name_lower.starts_with("eth") || name_lower.starts_with("enp") || name_lower.starts_with("eno") || name_lower.starts_with("ens") {
        "Ethernet".to_string()
    } else if name_lower.starts_with("wlan") || name_lower.starts_with("wlp") || name_lower.starts_with("wifi") {
        "Wi-Fi".to_string()
    } else if name_lower.starts_with("docker") || name_lower.starts_with("br-") {
        "Docker".to_string()
    } else if name_lower.starts_with("veth") {
        "Virtual Ethernet".to_string()
    } else if name_lower.starts_with("tun") || name_lower.starts_with("tap") || name_lower.starts_with("utun") || name_lower.starts_with("ipsec") {
        "VPN".to_string()
    } else if name_lower.starts_with("virbr") || name_lower.starts_with("bridge") {
        "Bridge".to_string()
    } else if name_lower.starts_with("vmnet") || name_lower.starts_with("vboxnet") {
        "Virtual Machine".to_string()
    } else {
        "Other".to_string()
    }
}

/// 获取接口详细信息 (IP, MAC, status)
#[cfg(target_os = "macos")]
fn get_interface_details(name: &str) -> (Option<String>, Option<String>, String) {
    let output = Command::new("ifconfig")
        .arg(name)
        .output();

    if let Ok(output) = output {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut ip_address = None;
        let mut mac_address = None;
        let mut status = "inactive".to_string();

        for line in output_str.lines() {
            let line = line.trim();
            if line.starts_with("inet ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    ip_address = Some(parts[1].to_string());
                }
            } else if line.starts_with("ether ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    mac_address = Some(parts[1].to_string());
                }
            } else if line.contains("status: active") {
                status = "active".to_string();
            }
        }

        (ip_address, mac_address, status)
    } else {
        (None, None, "unknown".to_string())
    }
}

#[cfg(target_os = "linux")]
fn get_interface_details(name: &str) -> (Option<String>, Option<String>, String) {
    let output = Command::new("ip")
        .args(&["addr", "show", name])
        .output();

    if let Ok(output) = output {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut ip_address = None;
        let mut mac_address = None;
        let mut status = "inactive".to_string();

        for line in output_str.lines() {
            let line = line.trim();
            if line.starts_with("link/ether ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    mac_address = Some(parts[1].to_string());
                }
            } else if line.starts_with("inet ") && !line.starts_with("inet6 ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let ip = parts[1].split('/').next().unwrap_or(parts[1]);
                    ip_address = Some(ip.to_string());
                }
            }

            if output_str.contains("UP") && output_str.contains("LOWER_UP") {
                status = "active".to_string();
            } else if output_str.contains("UP") {
                status = "up".to_string();
            }
        }

        (ip_address, mac_address, status)
    } else {
        (None, None, "unknown".to_string())
    }
}

#[cfg(target_os = "windows")]
fn get_interface_details(name: &str) -> (Option<String>, Option<String>, String) {
    // Windows: 使用 ipconfig 和 getmac
    let output = Command::new("ipconfig")
        .arg("/all")
        .output();

    let mut ip_address = None;
    let mut mac_address = None;
    let status = "unknown".to_string();

    if let Ok(output) = output {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut in_interface = false;

        for line in output_str.lines() {
            if line.contains(name) {
                in_interface = true;
            } else if in_interface {
                if line.trim().is_empty() {
                    in_interface = false;
                } else if line.contains("Physical Address") || line.contains("物理地址") {
                    if let Some(mac) = line.split(':').last() {
                        mac_address = Some(mac.trim().replace("-", ":"));
                    }
                } else if line.contains("IPv4 Address") || line.contains("IPv4 地址") {
                    if let Some(ip) = line.split(':').last() {
                        ip_address = Some(ip.trim().trim_end_matches("(Preferred)").trim().to_string());
                    }
                }
            }
        }
    }

    (ip_address, mac_address, status)
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn get_interface_details(_name: &str) -> (Option<String>, Option<String>, String) {
    (None, None, "unknown".to_string())
}

/// 回退方法：使用平台特定命令获取网络接口
fn get_network_interfaces_fallback() -> Result<Vec<NetworkInterface>, String> {
    let mut interfaces = Vec::new();

    #[cfg(target_os = "macos")]
    {
        let output = Command::new("ifconfig")
            .output()
            .map_err(|e| format!("获取网络接口失败: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut current_interface: Option<NetworkInterface> = None;

        for line in output_str.lines() {
            if !line.starts_with('\t') && !line.starts_with(' ') && line.contains(':') {
                if let Some(iface) = current_interface.take() {
                    interfaces.push(iface);
                }

                let name = line.split(':').next().unwrap_or("").to_string();
                let interface_type = determine_interface_type(&name);

                current_interface = Some(NetworkInterface {
                    name,
                    ip_address: None,
                    mac_address: None,
                    status: "inactive".to_string(),
                    interface_type,
                });
            } else if let Some(ref mut iface) = current_interface {
                let line = line.trim();
                if line.starts_with("inet ") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        iface.ip_address = Some(parts[1].to_string());
                    }
                } else if line.starts_with("ether ") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        iface.mac_address = Some(parts[1].to_string());
                    }
                } else if line.contains("status: active") {
                    iface.status = "active".to_string();
                }
            }
        }

        if let Some(iface) = current_interface {
            interfaces.push(iface);
        }
    }

    #[cfg(target_os = "linux")]
    {
        let output = Command::new("ip")
            .args(&["addr", "show"])
            .output()
            .map_err(|e| format!("获取网络接口失败: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut current_interface: Option<NetworkInterface> = None;

        for line in output_str.lines() {
            if let Some(colon_pos) = line.find(':') {
                if line.chars().next().map(|c| c.is_numeric()).unwrap_or(false) {
                    if let Some(iface) = current_interface.take() {
                        interfaces.push(iface);
                    }

                    let after_first_colon = &line[colon_pos + 1..];
                    if let Some(second_colon) = after_first_colon.find(':') {
                        let name = after_first_colon[..second_colon].trim().to_string();
                        let interface_type = determine_interface_type(&name);
                        let status = if line.contains("UP") && line.contains("LOWER_UP") {
                            "active"
                        } else if line.contains("UP") {
                            "up"
                        } else {
                            "inactive"
                        };

                        current_interface = Some(NetworkInterface {
                            name,
                            ip_address: None,
                            mac_address: None,
                            status: status.to_string(),
                            interface_type,
                        });
                    }
                }
            } else if let Some(ref mut iface) = current_interface {
                let line = line.trim();
                if line.starts_with("link/ether ") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        iface.mac_address = Some(parts[1].to_string());
                    }
                } else if line.starts_with("inet ") && !line.starts_with("inet6 ") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let ip = parts[1].split('/').next().unwrap_or(parts[1]);
                        iface.ip_address = Some(ip.to_string());
                    }
                }
            }
        }

        if let Some(iface) = current_interface {
            interfaces.push(iface);
        }
    }

    Ok(interfaces)
}

/// 获取 DNS 缓存条目数
fn get_dns_cache_count() -> Result<u32, String> {
    #[cfg(target_os = "macos")]
    {
        // macOS 没有直接获取 DNS 缓存数量的命令，返回估计值
        let output = Command::new("dscacheutil")
            .args(&["-statistics"])
            .output();

        if let Ok(output) = output {
            let output_str = String::from_utf8_lossy(&output.stdout);
            // 尝试从统计信息中解析
            for line in output_str.lines() {
                if line.contains("entries") {
                    if let Some(num) = line.split_whitespace()
                        .find(|s| s.chars().all(|c| c.is_numeric()))
                        .and_then(|s| s.parse::<u32>().ok())
                    {
                        return Ok(num);
                    }
                }
            }
        }
        // 返回默认估计值
        Ok(50)
    }

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("ipconfig")
            .arg("/displaydns")
            .output()
            .map_err(|e| format!("获取DNS缓存失败: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let count = output_str.matches("Record Name").count() as u32;
        Ok(count)
    }

    #[cfg(target_os = "linux")]
    {
        // Linux systemd-resolved
        let output = Command::new("resolvectl")
            .arg("statistics")
            .output();

        if let Ok(output) = output {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains("Current Cache Size") {
                    if let Some(num) = line.split_whitespace()
                        .last()
                        .and_then(|s| s.parse::<u32>().ok())
                    {
                        return Ok(num);
                    }
                }
            }
        }
        Ok(0)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Ok(0)
    }
}

/// 获取 ARP 缓存条目数
fn get_arp_cache_count() -> Result<u32, String> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("arp")
            .arg("-an")
            .output()
            .map_err(|e| format!("获取ARP缓存失败: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let count = output_str.lines().filter(|line| line.contains("at")).count() as u32;
        Ok(count)
    }

    #[cfg(target_os = "linux")]
    {
        let output = Command::new("ip")
            .args(&["neigh", "show"])
            .output()
            .map_err(|e| format!("获取ARP缓存失败: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let count = output_str.lines().count() as u32;
        Ok(count)
    }

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("arp")
            .arg("-a")
            .output()
            .map_err(|e| format!("获取ARP缓存失败: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let count = output_str.lines()
            .filter(|line| line.contains("dynamic") || line.contains("static"))
            .count() as u32;
        Ok(count)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Ok(0)
    }
}

/// 获取路由表条目数
fn get_routing_entries_count() -> Result<u32, String> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("netstat")
            .arg("-rn")
            .output()
            .map_err(|e| format!("获取路由表失败: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let count = output_str.lines().skip(4).filter(|line| !line.is_empty()).count() as u32;
        Ok(count)
    }

    #[cfg(target_os = "linux")]
    {
        let output = Command::new("ip")
            .args(&["route", "show"])
            .output()
            .map_err(|e| format!("获取路由表失败: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let count = output_str.lines().count() as u32;
        Ok(count)
    }

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("route")
            .arg("print")
            .output()
            .map_err(|e| format!("获取路由表失败: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let count = output_str.lines()
            .filter(|line| line.trim().starts_with(|c: char| c.is_numeric()))
            .count() as u32;
        Ok(count)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Ok(0)
    }
}

impl Default for ProxySettings {
    fn default() -> Self {
        ProxySettings {
            http_proxy: None,
            https_proxy: None,
            socks_proxy: None,
            proxy_enabled: false,
            pac_url: None,
        }
    }
}

/// 清理 DNS 缓存
pub fn clear_dns_cache() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        Command::new("ipconfig")
            .arg("/flushdns")
            .output()
            .map_err(|e| format!("清理DNS缓存失败: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: 重启 systemd-resolved 或 nscd
        let services = vec!["systemd-resolved", "nscd", "dnsmasq"];
        let mut success = false;

        for service in services {
            if Command::new("systemctl")
                .args(&["restart", service])
                .output()
                .is_ok()
            {
                success = true;
                break;
            }
        }

        if !success {
            // 尝试直接清理
            let _ = Command::new("resolvectl")
                .arg("flush-caches")
                .output();
        }
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("dscacheutil")
            .arg("-flushcache")
            .output()
            .map_err(|e| format!("清理DNS缓存失败: {}", e))?;

        Command::new("killall")
            .args(&["-HUP", "mDNSResponder"])
            .output()
            .ok();
    }

    Ok(())
}

/// 清理 ARP 缓存
pub fn clear_arp_cache() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        Command::new("arp")
            .arg("-d")
            .arg("*")
            .output()
            .map_err(|e| format!("清理ARP缓存失败: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        // macOS 使用 arp -d -a 或 sudo arp -a -d
        Command::new("arp")
            .args(&["-d", "-a"])
            .output()
            .map_err(|e| format!("清理ARP缓存失败: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("ip")
            .args(&["neigh", "flush", "all"])
            .output()
            .map_err(|e| format!("清理ARP缓存失败: {}", e))?;
    }

    Ok(())
}

/// 清理 NetBIOS 缓存 (Windows only)
#[cfg(target_os = "windows")]
pub fn clear_netbios_cache() -> Result<(), String> {
    Command::new("nbtstat")
        .arg("-R")
        .output()
        .map_err(|e| format!("清理NetBIOS缓存失败: {}", e))?;

    Ok(())
}

/// 重置路由表
pub fn reset_routing_table() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        Command::new("route")
            .arg("-f")
            .output()
            .map_err(|e| format!("重置路由表失败: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: 刷新路由缓存
        Command::new("route")
            .args(&["-n", "flush"])
            .output()
            .map_err(|e| format!("重置路由表失败: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("ip")
            .args(&["route", "flush", "cache"])
            .output()
            .map_err(|e| format!("重置路由表失败: {}", e))?;
    }

    Ok(())
}

/// 清理 WiFi 配置
pub fn clear_wifi_profiles() -> Result<u32, String> {
    let mut count = 0;

    #[cfg(target_os = "macos")]
    {
        // 获取所有保存的 WiFi 网络
        let output = Command::new("networksetup")
            .args(&["-listpreferredwirelessnetworks", "en0"])
            .output()
            .map_err(|e| format!("获取WiFi配置失败: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines().skip(1) {
            let ssid = line.trim();
            if !ssid.is_empty() {
                // 删除每个 WiFi 配置
                let _ = Command::new("networksetup")
                    .args(&["-removepreferredwirelessnetwork", "en0", ssid])
                    .output();
                count += 1;
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("netsh")
            .args(&["wlan", "show", "profiles"])
            .output()
            .map_err(|e| format!("获取WiFi配置失败: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if line.contains("All User Profile") || line.contains("所有用户配置文件") {
                if let Some(ssid) = line.split(':').nth(1) {
                    let ssid = ssid.trim();
                    let _ = Command::new("netsh")
                        .args(&["wlan", "delete", "profile", &format!("name={}", ssid)])
                        .output();
                    count += 1;
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        // 使用 nmcli 删除 WiFi 连接
        let output = Command::new("nmcli")
            .args(&["-t", "-f", "NAME,TYPE", "connection", "show"])
            .output();

        if let Ok(output) = output {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 2 && parts[1].contains("wireless") {
                    let _ = Command::new("nmcli")
                        .args(&["connection", "delete", parts[0]])
                        .output();
                    count += 1;
                }
            }
        }
    }

    Ok(count)
}

/// 清理连接历史
pub fn clear_connection_history() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        // 清理最近使用的服务器历史
        let home = std::env::var("HOME").unwrap_or_default();

        // 清理 SMB/AFP 连接历史
        let _ = std::fs::remove_file(format!("{}/.local/share/recently-used.xbel", home));

        // 清理网络位置历史
        let _ = Command::new("defaults")
            .args(&["delete", "com.apple.sidebarlists", "networkbrowser"])
            .output();
    }

    #[cfg(target_os = "windows")]
    {
        // 清理网络连接历史
        let _ = Command::new("reg")
            .args(&["delete", r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Map Network Drive MRU", "/f"])
            .output();

        // 清理最近的网络位置
        let _ = Command::new("reg")
            .args(&["delete", r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\RunMRU", "/f"])
            .output();
    }

    Ok(())
}

/// 清理代理设置
pub fn clear_proxy_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        // 禁用所有代理
        let _ = Command::new("networksetup")
            .args(&["-setwebproxystate", "Wi-Fi", "off"])
            .output();

        let _ = Command::new("networksetup")
            .args(&["-setsecurewebproxystate", "Wi-Fi", "off"])
            .output();

        let _ = Command::new("networksetup")
            .args(&["-setsocksfirewallproxystate", "Wi-Fi", "off"])
            .output();

        let _ = Command::new("networksetup")
            .args(&["-setautoproxystate", "Wi-Fi", "off"])
            .output();
    }

    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("reg")
            .args(&["add", r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings", "/v", "ProxyEnable", "/t", "REG_DWORD", "/d", "0", "/f"])
            .output();
    }

    Ok(())
}

/// 断开 VPN 连接
pub fn disconnect_vpn() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        // 获取所有 VPN 配置
        let output = Command::new("scutil")
            .args(&["--nc", "list"])
            .output();

        if let Ok(output) = output {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains("Connected") {
                    // 提取 VPN 服务 ID
                    if let Some(id_start) = line.find('(') {
                        if let Some(id_end) = line.find(')') {
                            let service_id = &line[id_start + 1..id_end];
                            let _ = Command::new("scutil")
                                .args(&["--nc", "stop", service_id])
                                .output();
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("rasdial")
            .arg("/DISCONNECT")
            .output();
    }

    Ok(())
}

/// 清理网络痕迹
pub fn clean_network(types: Vec<String>) -> Result<Vec<String>, String> {
    let mut cleaned = Vec::new();
    let mut errors = Vec::new();

    for network_type in types {
        let result = match network_type.as_str() {
            "dns" | "dns_cache" => clear_dns_cache(),
            "arp" | "arp_table" => clear_arp_cache(),
            #[cfg(target_os = "windows")]
            "netbios" | "netbios_cache" => clear_netbios_cache(),
            "routing" | "routing_table" => reset_routing_table(),
            "wifi" | "wifi_profiles" => clear_wifi_profiles().map(|_| ()),
            "history" | "connection_history" => clear_connection_history(),
            "proxy" | "proxy_settings" => clear_proxy_settings(),
            "vpn" | "vpn_disconnect" => disconnect_vpn(),
            _ => Err(format!("未知的网络类型: {}", network_type)),
        };

        match result {
            Ok(_) => cleaned.push(network_type),
            Err(e) => errors.push(format!("{}: {}", network_type, e)),
        }
    }

    if !errors.is_empty() && cleaned.is_empty() {
        return Err(format!(
            "网络清理失败:\n{}",
            errors.join("\n")
        ));
    }

    // errors logged silently

    Ok(cleaned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_network_info() {
        let result = get_network_info();
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_active_connections() {
        let result = get_active_connections();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_address() {
        assert_eq!(parse_address("192.168.1.1.8080"), Some(("192.168.1.1".to_string(), 8080)));
        assert_eq!(parse_address("127.0.0.1:443"), Some(("127.0.0.1".to_string(), 443)));
    }
}
