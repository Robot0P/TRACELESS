use crate::modules::network_cleaner;
use crate::commands::permission_ops::require_admin_for_operation;

#[tauri::command]
pub fn get_network_info() -> Result<network_cleaner::NetworkInfo, String> {
    network_cleaner::get_network_info()
}

#[tauri::command]
pub fn scan_network_items() -> Result<network_cleaner::NetworkScanResult, String> {
    network_cleaner::scan_network_items()
}

#[tauri::command]
pub fn clean_network(types: Vec<String>) -> Result<Vec<String>, String> {
    // Check permission for operations that require elevation
    for network_type in &types {
        let op_name = format!("network_{}", network_type);
        require_admin_for_operation(&op_name)?;
    }

    network_cleaner::clean_network(types)
}
