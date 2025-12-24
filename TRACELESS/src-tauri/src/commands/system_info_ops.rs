use crate::modules::system_info::{get_system_info, get_network_speed, get_disks_info, SystemInfo, NetworkSpeed, DiskInfo};

#[tauri::command]
pub async fn get_system_info_api() -> Result<SystemInfo, String> {
    get_system_info()
}

#[tauri::command]
pub async fn get_network_speed_api() -> Result<NetworkSpeed, String> {
    get_network_speed()
}

#[tauri::command]
pub async fn get_disks_info_api() -> Result<Vec<DiskInfo>, String> {
    get_disks_info()
}
