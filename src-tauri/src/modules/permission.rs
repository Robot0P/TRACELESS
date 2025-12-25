use std::process::Command;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

// ==================== Platform-specific imports ====================

#[cfg(target_os = "macos")]
use security_framework::authorization::{Authorization, AuthorizationItemSetBuilder, Flags};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

// ==================== Global State ====================

/// Authorization status flag - tracks whether authorization has been granted
static AUTHORIZATION_GRANTED: AtomicBool = AtomicBool::new(false);

/// Permission status structure
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct PermissionStatus {
    pub is_admin: bool,
    pub has_full_disk_access: bool,
    pub has_authorization: bool,
    pub has_system_privileges: bool,
    pub platform: String,
}

/// Get permission status summary
pub fn get_permission_status() -> PermissionStatus {
    PermissionStatus {
        is_admin: is_elevated(),
        has_full_disk_access: check_full_disk_access(),
        has_authorization: check_authorization_valid(),
        has_system_privileges: check_system_privileges(),
        platform: get_platform_name(),
    }
}

fn get_platform_name() -> String {
    #[cfg(target_os = "macos")]
    { "macOS".to_string() }
    #[cfg(target_os = "windows")]
    { "Windows".to_string() }
    #[cfg(target_os = "linux")]
    { "Linux".to_string() }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    { "Unknown".to_string() }
}

// ==================== Cross-Platform Authorization ====================

/// Check if authorization is valid
pub fn check_authorization_valid() -> bool {
    #[cfg(target_os = "macos")]
    {
        AUTHORIZATION_GRANTED.load(Ordering::SeqCst)
    }
    #[cfg(target_os = "windows")]
    {
        is_elevated()
    }
    #[cfg(target_os = "linux")]
    {
        is_elevated()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        false
    }
}

/// Check if the process has SYSTEM-level privileges (Windows) or equivalent
pub fn check_system_privileges() -> bool {
    #[cfg(target_os = "windows")]
    {
        // Check if running as SYSTEM or with high integrity level
        check_windows_system_privileges()
    }
    #[cfg(target_os = "macos")]
    {
        // On macOS, check if running as root
        unsafe { libc::geteuid() == 0 }
    }
    #[cfg(target_os = "linux")]
    {
        // On Linux, check if running as root
        unsafe { libc::geteuid() == 0 }
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        false
    }
}

// ==================== macOS Authorization Services ====================

#[cfg(target_os = "macos")]
pub fn create_authorization() -> Result<(), String> {
    if check_authorization_valid() {
        return Ok(());
    }

    let rights = AuthorizationItemSetBuilder::new()
        .add_right("system.privilege.admin")
        .map_err(|e| format!("Failed to create authorization rights: {}", e))?
        .build();

    let _auth = Authorization::new(
        Some(rights),
        None,
        Flags::INTERACTION_ALLOWED | Flags::EXTEND_RIGHTS | Flags::PREAUTHORIZE,
    ).map_err(|e| format!("Authorization request failed: {}", e))?;

    AUTHORIZATION_GRANTED.store(true, Ordering::SeqCst);
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn execute_with_authorization(command: &str, args: &[&str]) -> Result<String, String> {
    if !check_authorization_valid() {
        create_authorization()?;
    }

    let full_cmd = if args.is_empty() {
        command.to_string()
    } else {
        format!("{} {}", command, args.join(" "))
    };

    execute_privileged_command_macos(&full_cmd)
}

#[cfg(target_os = "macos")]
fn execute_privileged_command_macos(command: &str) -> Result<String, String> {
    let script = format!(
        r#"do shell script "{}" with administrator privileges"#,
        command.replace("\\", "\\\\").replace("\"", "\\\"")
    );

    let output = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|e| format!("Execution failed: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        if error.contains("User canceled") || error.contains("user canceled") {
            Err("User cancelled authorization".to_string())
        } else {
            Err(format!("Execution failed: {}", error))
        }
    }
}

#[cfg(target_os = "macos")]
pub fn release_authorization() {
    AUTHORIZATION_GRANTED.store(false, Ordering::SeqCst);
}

// ==================== Windows System-Level Privileges ====================

#[cfg(target_os = "windows")]
pub fn create_authorization() -> Result<(), String> {
    if is_elevated() {
        AUTHORIZATION_GRANTED.store(true, Ordering::SeqCst);
        Ok(())
    } else {
        // Try to restart with admin privileges
        request_elevation()
    }
}

#[cfg(target_os = "windows")]
pub fn execute_with_authorization(command: &str, args: &[&str]) -> Result<String, String> {
    if is_elevated() {
        // Already elevated, execute directly
        let output = Command::new(command)
            .args(args)
            .output()
            .map_err(|e| format!("Command execution failed: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    } else {
        Err("Administrator privileges required. Please restart as Administrator.".to_string())
    }
}

/// Execute command as SYSTEM user using PsExec or scheduled task
#[cfg(target_os = "windows")]
pub fn execute_as_system(command: &str, args: &[&str]) -> Result<String, String> {
    use windows::Win32::System::Threading::CREATE_NO_WINDOW;

    if !is_elevated() {
        return Err("Administrator privileges required to run as SYSTEM".to_string());
    }

    // Method 1: Use scheduled task to run as SYSTEM
    let task_name = format!("TracelessTask_{}", std::process::id());
    let full_command = if args.is_empty() {
        command.to_string()
    } else {
        format!("{} {}", command, args.join(" "))
    };

    // Create a scheduled task that runs as SYSTEM
    let create_task = Command::new("schtasks")
        .args([
            "/Create",
            "/TN", &task_name,
            "/TR", &full_command,
            "/SC", "ONCE",
            "/ST", "00:00",
            "/RU", "SYSTEM",
            "/F",
        ])
        .creation_flags(CREATE_NO_WINDOW.0)
        .output()
        .map_err(|e| format!("Failed to create scheduled task: {}", e))?;

    if !create_task.status.success() {
        return Err(format!("Failed to create scheduled task: {}",
            String::from_utf8_lossy(&create_task.stderr)));
    }

    // Run the task immediately
    let run_task = Command::new("schtasks")
        .args(["/Run", "/TN", &task_name])
        .creation_flags(CREATE_NO_WINDOW.0)
        .output()
        .map_err(|e| format!("Failed to run scheduled task: {}", e))?;

    // Wait a moment for task to complete
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Delete the task
    let _ = Command::new("schtasks")
        .args(["/Delete", "/TN", &task_name, "/F"])
        .creation_flags(CREATE_NO_WINDOW.0)
        .output();

    if run_task.status.success() {
        Ok("Command executed as SYSTEM".to_string())
    } else {
        Err(format!("Failed to run as SYSTEM: {}",
            String::from_utf8_lossy(&run_task.stderr)))
    }
}

/// Execute command with TrustedInstaller privileges
#[cfg(target_os = "windows")]
pub fn execute_as_trustedinstaller(command: &str, args: &[&str]) -> Result<String, String> {
    use windows::Win32::System::Threading::CREATE_NO_WINDOW;

    if !is_elevated() {
        return Err("Administrator privileges required to run as TrustedInstaller".to_string());
    }

    let full_command = if args.is_empty() {
        command.to_string()
    } else {
        format!("{} {}", command, args.join(" "))
    };

    // Use PowerShell to access TrustedInstaller through token manipulation
    // This requires the TrustedInstaller service to be running
    let ps_script = format!(r#"
        $ErrorActionPreference = 'Stop'

        # Start TrustedInstaller service if not running
        $service = Get-Service TrustedInstaller -ErrorAction SilentlyContinue
        if ($service.Status -ne 'Running') {{
            Start-Service TrustedInstaller
            Start-Sleep -Milliseconds 500
        }}

        # Create a scheduled task to run as TrustedInstaller
        $taskName = 'TracelessTI_{}'
        $action = New-ScheduledTaskAction -Execute 'cmd.exe' -Argument '/c {}'
        $principal = New-ScheduledTaskPrincipal -UserId 'NT SERVICE\TrustedInstaller' -LogonType ServiceAccount -RunLevel Highest
        $settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries

        Register-ScheduledTask -TaskName $taskName -Action $action -Principal $principal -Settings $settings -Force | Out-Null
        Start-ScheduledTask -TaskName $taskName
        Start-Sleep -Milliseconds 1000
        Unregister-ScheduledTask -TaskName $taskName -Confirm:$false

        Write-Output 'Command executed as TrustedInstaller'
    "#, std::process::id(), full_command.replace("'", "''"));

    let output = Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &ps_script])
        .creation_flags(CREATE_NO_WINDOW.0)
        .output()
        .map_err(|e| format!("PowerShell execution failed: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(format!("TrustedInstaller execution failed: {}",
            String::from_utf8_lossy(&output.stderr)))
    }
}

/// Check if current process has SYSTEM-level privileges on Windows
#[cfg(target_os = "windows")]
fn check_windows_system_privileges() -> bool {
    use windows::Win32::Foundation::{HANDLE, CloseHandle};
    use windows::Win32::Security::{
        GetTokenInformation, TokenIntegrityLevel, TOKEN_MANDATORY_LABEL, TOKEN_QUERY,
    };
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }

        let mut return_length = 0u32;
        let _ = GetTokenInformation(
            token,
            TokenIntegrityLevel,
            None,
            0,
            &mut return_length,
        );

        if return_length == 0 {
            let _ = CloseHandle(token);
            return false;
        }

        let mut buffer = vec![0u8; return_length as usize];
        let result = GetTokenInformation(
            token,
            TokenIntegrityLevel,
            Some(buffer.as_mut_ptr() as *mut _),
            return_length,
            &mut return_length,
        );

        let _ = CloseHandle(token);

        if result.is_err() {
            return false;
        }

        // Parse the integrity level
        let label = &*(buffer.as_ptr() as *const TOKEN_MANDATORY_LABEL);
        let sid = label.Label.Sid;

        if sid.0.is_null() {
            return false;
        }

        // Get the last sub-authority (integrity level RID)
        use windows::Win32::Security::{GetSidSubAuthorityCount, GetSidSubAuthority};
        let sub_auth_count = *GetSidSubAuthorityCount(sid);
        if sub_auth_count == 0 {
            return false;
        }

        let integrity_level = *GetSidSubAuthority(sid, (sub_auth_count - 1) as u32);

        // SECURITY_MANDATORY_SYSTEM_RID = 0x4000
        integrity_level >= 0x4000
    }
}

#[cfg(target_os = "windows")]
pub fn release_authorization() {
    AUTHORIZATION_GRANTED.store(false, Ordering::SeqCst);
}

// ==================== Linux Privileges ====================

#[cfg(target_os = "linux")]
pub fn create_authorization() -> Result<(), String> {
    if is_elevated() {
        AUTHORIZATION_GRANTED.store(true, Ordering::SeqCst);
        Ok(())
    } else {
        Err("Root privileges required. Please run with sudo.".to_string())
    }
}

#[cfg(target_os = "linux")]
pub fn execute_with_authorization(command: &str, args: &[&str]) -> Result<String, String> {
    if is_elevated() {
        let output = Command::new(command)
            .args(args)
            .output()
            .map_err(|e| format!("Command execution failed: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    } else {
        // Try using pkexec for graphical sudo
        execute_with_pkexec(command, args)
    }
}

#[cfg(target_os = "linux")]
fn execute_with_pkexec(command: &str, args: &[&str]) -> Result<String, String> {
    let mut cmd_args = vec![command];
    cmd_args.extend(args);

    let output = Command::new("pkexec")
        .args(&cmd_args)
        .output()
        .map_err(|e| format!("pkexec execution failed: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        if error.contains("dismissed") || error.contains("cancelled") {
            Err("User cancelled authorization".to_string())
        } else {
            Err(format!("Execution failed: {}", error))
        }
    }
}

#[cfg(target_os = "linux")]
pub fn release_authorization() {
    AUTHORIZATION_GRANTED.store(false, Ordering::SeqCst);
}

// ==================== Fallback implementations ====================

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub fn create_authorization() -> Result<(), String> {
    Err("Unsupported operating system".to_string())
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub fn execute_with_authorization(command: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(command)
        .args(args)
        .output()
        .map_err(|e| format!("Command execution failed: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub fn release_authorization() {}

// ==================== Cross-Platform Privileged Command Execution ====================

/// Run command with admin privileges (cross-platform)
pub fn run_with_admin_privileges(command: &str) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        if check_authorization_valid() {
            return execute_privileged_command_macos(command);
        }

        let script = format!(
            r#"do shell script "{}" with administrator privileges"#,
            command.replace("\\", "\\\\").replace("\"", "\\\"")
        );

        let output = Command::new("osascript")
            .args(["-e", &script])
            .output()
            .map_err(|e| format!("Execution failed: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            if error.contains("User canceled") || error.contains("user canceled") {
                Err("User cancelled authorization".to_string())
            } else {
                Err(format!("Execution failed: {}", error))
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::Threading::CREATE_NO_WINDOW;

        if is_elevated() {
            // Already elevated, run directly via cmd
            let output = Command::new("cmd")
                .args(["/C", command])
                .creation_flags(CREATE_NO_WINDOW.0)
                .output()
                .map_err(|e| format!("Execution failed: {}", e))?;

            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(String::from_utf8_lossy(&output.stderr).to_string())
            }
        } else {
            // Need to elevate - use PowerShell Start-Process with -Verb RunAs
            let ps_command = format!(
                "Start-Process cmd -ArgumentList '/C {}' -Verb RunAs -Wait -WindowStyle Hidden",
                command.replace("'", "''")
            );

            let output = Command::new("powershell")
                .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &ps_command])
                .creation_flags(CREATE_NO_WINDOW.0)
                .output()
                .map_err(|e| format!("PowerShell execution failed: {}", e))?;

            if output.status.success() {
                Ok("Command executed with elevated privileges".to_string())
            } else {
                Err(format!("Elevation failed: {}", String::from_utf8_lossy(&output.stderr)))
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if is_elevated() {
            let output = Command::new("sh")
                .args(["-c", command])
                .output()
                .map_err(|e| format!("Execution failed: {}", e))?;

            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(String::from_utf8_lossy(&output.stderr).to_string())
            }
        } else {
            // Try pkexec first, then sudo
            let output = Command::new("pkexec")
                .args(["sh", "-c", command])
                .output();

            match output {
                Ok(out) if out.status.success() => {
                    Ok(String::from_utf8_lossy(&out.stdout).to_string())
                }
                _ => {
                    // Fallback to sudo (may not work without terminal)
                    Err("Root privileges required. Please run with sudo.".to_string())
                }
            }
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Err("Unsupported operating system".to_string())
    }
}

// ==================== Full Disk Access (macOS) / Protected Files ====================

/// Check full disk access (macOS) or equivalent permissions
pub fn check_full_disk_access() -> bool {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_default();

        let protected_paths = [
            format!("{}/Library/Mail", home),
            format!("{}/Library/Messages", home),
            format!("{}/Library/Safari/History.db", home),
        ];

        for path in &protected_paths {
            let p = Path::new(path);
            if p.exists() {
                if std::fs::read_dir(p.parent().unwrap_or(p)).is_ok() {
                    if std::fs::metadata(p).is_ok() {
                        return true;
                    }
                }
            }
        }

        let tcc_path = format!("{}/Library/Application Support/com.apple.TCC/TCC.db", home);
        if std::fs::read(&tcc_path).is_ok() {
            return true;
        }

        false
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, check if we can access protected system files
        let system_root = std::env::var("SYSTEMROOT").unwrap_or_else(|_| "C:\\Windows".to_string());
        let protected_path = format!("{}\\System32\\config\\SAM", system_root);

        std::fs::metadata(&protected_path).is_ok()
    }

    #[cfg(target_os = "linux")]
    {
        // On Linux, check if we can access /etc/shadow
        std::fs::metadata("/etc/shadow").is_ok()
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        true
    }
}

// ==================== System Settings ====================

/// Open full disk access settings (macOS) or equivalent
pub fn open_full_disk_access_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles")
            .spawn()
            .map_err(|e| format!("Failed to open system settings: {}", e))?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        // Open Windows Security settings
        Command::new("cmd")
            .args(["/C", "start", "windowsdefender://threatsettings"])
            .spawn()
            .map_err(|e| format!("Failed to open security settings: {}", e))?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        // Try to open system settings
        let result = Command::new("gnome-control-center")
            .arg("privacy")
            .spawn();

        if result.is_err() {
            let _ = Command::new("systemsettings5")
                .spawn();
        }
        Ok(())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Err("Unsupported operating system".to_string())
    }
}

/// Open accessibility settings
pub fn open_accessibility_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .spawn()
            .map_err(|e| format!("Failed to open system settings: {}", e))?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "ms-settings:easeofaccess"])
            .spawn()
            .map_err(|e| format!("Failed to open accessibility settings: {}", e))?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        let result = Command::new("gnome-control-center")
            .arg("universal-access")
            .spawn();

        if result.is_err() {
            let _ = Command::new("systemsettings5")
                .arg("kcm_access")
                .spawn();
        }
        Ok(())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Ok(())
    }
}

/// Open system privacy settings
pub fn open_privacy_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy")
            .spawn()
            .map_err(|e| format!("Failed to open system settings: {}", e))?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "ms-settings:privacy"])
            .spawn()
            .map_err(|e| format!("Failed to open system settings: {}", e))?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        let result = Command::new("gnome-control-center")
            .arg("privacy")
            .spawn();

        if result.is_err() {
            Command::new("systemsettings5")
                .spawn()
                .map_err(|e| format!("Failed to open system settings: {}", e))?;
        }
        Ok(())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Err("Unsupported operating system".to_string())
    }
}

// ==================== Privilege Checks ====================

/// Check if running with admin/root privileges
pub fn is_elevated() -> bool {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Foundation::BOOL;
        use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
        use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

        unsafe {
            let mut token = windows::Win32::Foundation::HANDLE::default();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
                return false;
            }

            let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
            let mut return_length = 0u32;

            if GetTokenInformation(
                token,
                TokenElevation,
                Some(&mut elevation as *mut _ as *mut _),
                std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut return_length,
            ).is_ok() {
                let _ = windows::Win32::Foundation::CloseHandle(token);
                elevation.TokenIsElevated != 0
            } else {
                let _ = windows::Win32::Foundation::CloseHandle(token);
                false
            }
        }
    }

    #[cfg(target_family = "unix")]
    {
        unsafe { libc::geteuid() == 0 }
    }
}

/// Get elevation guide text
pub fn get_elevation_guide() -> String {
    #[cfg(target_os = "windows")]
    {
        "Please right-click the application icon and select 'Run as Administrator' to obtain necessary privileges.".to_string()
    }

    #[cfg(target_os = "macos")]
    {
        "The application will use macOS Authorization Services to request administrator privileges. Authorization remains valid during the application session.".to_string()
    }

    #[cfg(target_os = "linux")]
    {
        "Please run this application with sudo to obtain root privileges.".to_string()
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        "Please run the program as administrator.".to_string()
    }
}

/// Check if operation requires elevated privileges
pub fn requires_elevation(operation: &str) -> bool {
    match operation {
        "file_delete" => false,
        "system_logs" => true,
        "memory_clipboard" => false,
        "memory_pagefile" => true,
        "memory_hibernation" => true,
        "memory_swap" => true,
        "memory_working_set" => false,
        "memory_standby" => true,
        "network_dns" => true,
        "network_arp" => true,
        "network_netbios" => true,
        "network_routing" => true,
        "network_wifi" => true,
        "network_history" => false,
        "registry" => cfg!(target_os = "windows"),
        "timestamp" => false,
        "anti_analysis" => false,
        "disk_encryption" => true,
        "event_logs" => true,
        "prefetch" => true,
        "thumbnail_cache" => false,
        "recent_docs" => false,
        "usn_journal" => true,
        "mft" => true,
        _ => false,
    }
}

/// Request privilege elevation
pub fn request_elevation() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::Threading::CREATE_NO_WINDOW;

        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get executable path: {}", e))?;

        let status = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!("Start-Process '{}' -Verb RunAs", exe_path.display())
            ])
            .creation_flags(CREATE_NO_WINDOW.0)
            .status()
            .map_err(|e| format!("Failed to request administrator privileges: {}", e))?;

        if status.success() {
            std::process::exit(0);
        } else {
            Err("User cancelled the permission request".to_string())
        }
    }

    #[cfg(target_os = "macos")]
    {
        create_authorization()
    }

    #[cfg(target_os = "linux")]
    {
        Err("Please restart the application with sudo".to_string())
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err("Unsupported operating system".to_string())
    }
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_elevated() {
        let _elevated = is_elevated();
    }

    #[test]
    fn test_get_elevation_guide() {
        let guide = get_elevation_guide();
        assert!(!guide.is_empty());
    }

    #[test]
    fn test_requires_elevation() {
        assert!(requires_elevation("system_logs"));
        assert!(requires_elevation("disk_encryption"));
        assert!(!requires_elevation("file_delete"));
    }

    #[test]
    fn test_get_platform_name() {
        let platform = get_platform_name();
        assert!(!platform.is_empty());
        #[cfg(target_os = "macos")]
        assert_eq!(platform, "macOS");
        #[cfg(target_os = "windows")]
        assert_eq!(platform, "Windows");
        #[cfg(target_os = "linux")]
        assert_eq!(platform, "Linux");
    }
}
