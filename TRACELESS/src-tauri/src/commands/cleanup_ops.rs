use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::modules::{
    self_cleaning::{perform_self_cleaning, get_app_traces_size, SelfCleanResult},
    dry_run::{
        preview_file_deletion, preview_log_cleaning, preview_browser_cleaning,
        preview_shell_history_cleaning, preview_dns_flush, preview_trash_cleaning,
        preview_full_cleanup, PreviewResult,
    },
};

/// Self-clean result for frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct SelfCleanResponse {
    pub success: bool,
    pub cleaned_files: u32,
    pub cleaned_bytes: u64,
    pub cleaned_size_formatted: String,
    pub errors: Vec<String>,
}

/// App traces info for frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct AppTracesInfo {
    pub total_size: u64,
    pub file_count: u32,
    pub size_formatted: String,
}

/// Format bytes for display
fn format_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

/// Perform self-cleaning of application traces
#[tauri::command]
pub async fn perform_app_self_clean() -> Result<SelfCleanResponse, String> {
    let result = perform_self_cleaning();

    Ok(SelfCleanResponse {
        success: result.errors.is_empty(),
        cleaned_files: result.cleaned_files,
        cleaned_bytes: result.cleaned_bytes,
        cleaned_size_formatted: format_size(result.cleaned_bytes),
        errors: result.errors,
    })
}

/// Get info about application traces that can be cleaned
#[tauri::command]
pub async fn get_app_traces_info() -> Result<AppTracesInfo, String> {
    let (total_size, file_count) = get_app_traces_size();

    Ok(AppTracesInfo {
        total_size,
        file_count,
        size_formatted: format_size(total_size),
    })
}

/// Preview file deletion operation (dry-run)
#[tauri::command]
pub async fn preview_deletion(paths: Vec<String>, secure_delete: bool) -> Result<PreviewResult, String> {
    let path_bufs: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
    Ok(preview_file_deletion(&path_bufs, secure_delete))
}

/// Preview log cleaning operation (dry-run)
#[tauri::command]
pub async fn preview_log_clean() -> Result<PreviewResult, String> {
    Ok(preview_log_cleaning())
}

/// Preview browser data cleaning operation (dry-run)
#[tauri::command]
pub async fn preview_browser_clean() -> Result<PreviewResult, String> {
    Ok(preview_browser_cleaning())
}

/// Preview shell history cleaning operation (dry-run)
#[tauri::command]
pub async fn preview_shell_clean() -> Result<PreviewResult, String> {
    Ok(preview_shell_history_cleaning())
}

/// Preview DNS flush operation (dry-run)
#[tauri::command]
pub async fn preview_dns_clean() -> Result<PreviewResult, String> {
    Ok(preview_dns_flush())
}

/// Preview trash cleaning operation (dry-run)
#[tauri::command]
pub async fn preview_trash_clean() -> Result<PreviewResult, String> {
    Ok(preview_trash_cleaning())
}

/// Preview full cleanup operation (dry-run)
#[tauri::command]
pub async fn preview_all_cleanup() -> Result<PreviewResult, String> {
    Ok(preview_full_cleanup())
}
