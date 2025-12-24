use crate::modules::settings_storage::{self, AppSettings, SettingsDbInfo};

/// 保存所有设置
#[tauri::command]
pub fn save_settings(settings: AppSettings) -> Result<(), String> {
    settings_storage::save_all_settings(&settings)
}

/// 加载所有设置
#[tauri::command]
pub fn load_settings() -> Result<AppSettings, String> {
    settings_storage::load_all_settings()
}

/// 重置所有设置为默认值
#[tauri::command]
pub fn reset_settings() -> Result<AppSettings, String> {
    settings_storage::reset_all_settings()
}

/// 获取设置数据库信息
#[tauri::command]
pub fn get_settings_info() -> Result<SettingsDbInfo, String> {
    settings_storage::get_settings_db_info()
}
