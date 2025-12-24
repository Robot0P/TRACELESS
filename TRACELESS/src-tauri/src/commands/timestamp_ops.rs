use crate::modules::timestamp_modifier::{self, FileTimestamps};
use crate::modules::PathValidator;
use std::collections::HashMap;

#[tauri::command]
pub fn get_file_timestamps(file_path: String) -> Result<FileTimestamps, String> {
    // Validate path security
    let validator = PathValidator::new().require_exists(true);
    let validated_path = validator.validate_file(&file_path)
        .map_err(|e| format!("PATH_VALIDATION:{}", e))?;

    timestamp_modifier::get_file_timestamps(&validated_path.to_string_lossy())
}

#[tauri::command]
pub fn modify_file_timestamps(
    file_path: String,
    timestamps: HashMap<String, String>,
) -> Result<String, String> {
    // Validate path security
    let validator = PathValidator::new().require_exists(true);
    let validated_path = validator.validate_file(&file_path)
        .map_err(|e| format!("PATH_VALIDATION:{}", e))?;

    let path_str = validated_path.to_string_lossy().to_string();
    timestamp_modifier::modify_file_timestamps(&path_str, timestamps)?;
    Ok(format!("OK:timestampModified:{}", path_str))
}
