use crate::modules::anti_analysis::{self, EnvironmentCheck};

#[tauri::command]
pub fn check_environment() -> Result<EnvironmentCheck, String> {
    Ok(anti_analysis::check_environment())
}
