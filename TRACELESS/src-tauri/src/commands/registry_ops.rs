use crate::modules::registry_cleaner;
use crate::commands::permission_ops::require_admin_for_operation;

#[tauri::command]
pub fn clean_registry(types: Vec<String>) -> Result<Vec<String>, String> {
    // Check permission - registry cleanup requires elevation on Windows
    require_admin_for_operation("registry")?;

    registry_cleaner::clean_registry(types)
}

#[tauri::command]
pub fn get_registry_info() -> Result<registry_cleaner::RegistryStatus, String> {
    registry_cleaner::get_registry_info()
}
