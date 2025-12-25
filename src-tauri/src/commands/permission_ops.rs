use crate::modules::permission::{self, PermissionStatus};
use std::sync::atomic::{AtomicBool, Ordering};
use std::path::PathBuf;
use std::fs;

// Global permission state - marks whether permissions have been initialized
pub static PERMISSION_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Check if the current operation requires and has admin privileges
/// Returns Ok(()) if permission is available, Err with error code if not
/// On Windows, if running without elevation, returns an error prompting user to restart with admin rights
pub fn require_admin_for_operation(operation: &str) -> Result<(), String> {
    if !permission::requires_elevation(operation) {
        return Ok(());
    }

    // Check all sources of authorization
    if has_persistent_auth() || permission::check_authorization_valid() ||
       PERMISSION_INITIALIZED.load(Ordering::SeqCst) || permission::is_elevated() {
        return Ok(());
    }

    // On Windows, provide a more helpful error message
    #[cfg(target_os = "windows")]
    {
        return Err(format!("PERMISSION_DENIED:{}:请以管理员身份运行此应用程序。右键点击应用图标，选择"以管理员身份运行"。", operation));
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err(format!("PERMISSION_DENIED:{}:adminRequired", operation))
    }
}

/// Get authorization state file path
fn get_auth_state_file() -> Option<PathBuf> {
    dirs::data_dir().map(|p| p.join("traceless").join(".auth_state"))
}

/// Check if there is persistent authorization state
fn has_persistent_auth() -> bool {
    if let Some(path) = get_auth_state_file() {
        path.exists()
    } else {
        false
    }
}

/// Save authorization state to file
fn save_auth_state() -> Result<(), String> {
    if let Some(path) = get_auth_state_file() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("IO_ERROR:{}", e))?;
        }
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        fs::write(&path, timestamp.to_string()).map_err(|e| format!("IO_ERROR:{}", e))?;
        Ok(())
    } else {
        Err("IO_ERROR:cannotGetDataDir".to_string())
    }
}

/// Check if admin permission is available
#[tauri::command]
pub fn check_admin_permission() -> Result<bool, String> {
    if has_persistent_auth() {
        return Ok(true);
    }

    if permission::check_authorization_valid() {
        return Ok(true);
    }

    if PERMISSION_INITIALIZED.load(Ordering::SeqCst) {
        return Ok(true);
    }

    Ok(permission::is_elevated())
}

#[tauri::command]
pub fn get_elevation_guide() -> Result<String, String> {
    Ok(permission::get_elevation_guide())
}

#[tauri::command]
pub fn requires_admin(operation: String) -> Result<bool, String> {
    Ok(permission::requires_elevation(&operation))
}

#[tauri::command]
pub fn request_admin_elevation() -> Result<(), String> {
    permission::request_elevation()
}

#[tauri::command]
pub fn open_privacy_settings() -> Result<(), String> {
    permission::open_privacy_settings()
}

/// Check if permission is initialized
#[tauri::command]
pub fn check_permission_initialized() -> bool {
    if has_persistent_auth() {
        return true;
    }
    PERMISSION_INITIALIZED.load(Ordering::SeqCst)
}

/// Initialize application permissions
/// On macOS: Uses Authorization Services
/// On Windows: Checks for admin elevation
/// On Linux: Checks for root privileges
#[tauri::command]
pub async fn initialize_permissions() -> Result<String, String> {
    if has_persistent_auth() {
        PERMISSION_INITIALIZED.store(true, Ordering::SeqCst);
        #[cfg(target_os = "macos")]
        {
            let _ = permission::create_authorization();
        }
        return Ok("OK:authorizationExists".to_string());
    }

    #[cfg(target_os = "macos")]
    {
        match permission::create_authorization() {
            Ok(()) => {
                PERMISSION_INITIALIZED.store(true, Ordering::SeqCst);
                let _ = save_auth_state();
                Ok("OK:permissionInitialized".to_string())
            }
            Err(e) => Err(format!("PERMISSION_DENIED:{}", e))
        }
    }

    #[cfg(target_os = "windows")]
    {
        if permission::is_elevated() {
            PERMISSION_INITIALIZED.store(true, Ordering::SeqCst);
            let _ = save_auth_state();
            Ok("OK:adminConfirmed".to_string())
        } else {
            // Try to restart with elevated privileges
            match permission::request_elevation() {
                Ok(_) => {
                    // If request_elevation succeeds, it will restart the app
                    // This line should not be reached as the process will exit
                    Ok("OK:elevationRequested".to_string())
                }
                Err(e) => {
                    // User cancelled or elevation failed
                    Err(format!("PERMISSION_DENIED:{}", e))
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if permission::is_elevated() {
            PERMISSION_INITIALIZED.store(true, Ordering::SeqCst);
            let _ = save_auth_state();
            Ok("OK:rootConfirmed".to_string())
        } else {
            Err("PERMISSION_DENIED:runWithSudo".to_string())
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Err("PLATFORM_NOT_SUPPORTED".to_string())
    }
}

/// Get complete permission status
#[tauri::command]
pub fn get_permission_status() -> Result<PermissionStatus, String> {
    Ok(permission::get_permission_status())
}

/// Check full disk access (macOS) or equivalent permissions
#[tauri::command]
pub fn check_full_disk_access() -> Result<bool, String> {
    Ok(permission::check_full_disk_access())
}

/// Check if running with SYSTEM-level privileges
#[tauri::command]
pub fn check_system_privileges() -> Result<bool, String> {
    Ok(permission::check_system_privileges())
}

/// Open full disk access settings
#[tauri::command]
pub fn open_full_disk_access_settings() -> Result<(), String> {
    permission::open_full_disk_access_settings()
}

/// Open accessibility settings
#[tauri::command]
pub fn open_accessibility_settings() -> Result<(), String> {
    permission::open_accessibility_settings()
}

/// Run command with admin privileges
#[tauri::command]
pub async fn run_with_admin(command: String) -> Result<String, String> {
    permission::run_with_admin_privileges(&command)
}

/// Run command as SYSTEM user (Windows only)
#[tauri::command]
pub async fn run_as_system(command: String, args: Vec<String>) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        permission::execute_as_system(&command, &args_refs)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (command, args);
        Err("PLATFORM_NOT_SUPPORTED:windowsOnly".to_string())
    }
}

/// Run command as TrustedInstaller (Windows only)
#[tauri::command]
pub async fn run_as_trustedinstaller(command: String, args: Vec<String>) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        permission::execute_as_trustedinstaller(&command, &args_refs)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (command, args);
        Err("PLATFORM_NOT_SUPPORTED:windowsOnly".to_string())
    }
}
