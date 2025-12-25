use crate::modules::{WipeMethod, secure_delete, PathValidator, is_safe_for_deletion};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::collections::HashSet;

// 全局锁，用于跟踪正在删除的路径
static DELETING_PATHS: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub modified: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteProgress {
    pub current_file: String,
    pub total_files: usize,
    pub completed_files: usize,
    pub current_pass: u32,
    pub total_passes: u32,
    pub percentage: f64,
}

/// 安全删除文件的 Tauri 命令
#[tauri::command]
pub async fn secure_delete_file(
    app: AppHandle,
    path: String,
    method: String,
) -> Result<String, String> {
    // 使用路径验证器验证路径安全性
    let validator = PathValidator::strict();
    let validated_path = validator.validate(&path)
        .map_err(|e| format!("PATH_VALIDATION:{}", e))?;

    // Check if path is safe for deletion (not system critical path)
    if !is_safe_for_deletion(&validated_path) {
        return Err(format!("PERMISSION_DENIED:systemCriticalPath:{}", path));
    }

    let canonical_path = validated_path.to_string_lossy().to_string();

    // Check if this path is being deleted
    {
        let mut deleting = DELETING_PATHS.lock().unwrap();
        if deleting.contains(&canonical_path) {
            return Err("RESOURCE_BUSY:pathBeingDeleted".to_string());
        }
        // 标记该路径为正在删除
        deleting.insert(canonical_path.clone());
    }

    // 使用 scopeguard 确保无论成功还是失败都会移除路径标记
    let _guard = scopeguard::guard(canonical_path.clone(), |path| {
        let mut deleting = DELETING_PATHS.lock().unwrap();
        deleting.remove(&path);
    });

    // 解析擦除方法
    let wipe_method = WipeMethod::from_string(&method)?;

    // 创建进度回调
    let progress_callback = move |current_file: String, completed_files: usize, total_files: usize, current_pass: u32, total_passes: u32| {
        let percentage = if total_files > 0 {
            ((completed_files as f64 + (current_pass as f64 / total_passes as f64)) / total_files as f64) * 100.0
        } else {
            0.0
        };

        let progress = DeleteProgress {
            current_file: current_file.clone(),
            total_files,
            completed_files,
            current_pass,
            total_passes,
            percentage,
        };

        let _ = app.emit("delete-progress", progress);
    };

    // 执行安全删除
    crate::modules::secure_delete_with_progress(&canonical_path, wipe_method, progress_callback)?;

    Ok(format!("OK:fileDeleted:{}", path))
}

/// 获取文件信息的 Tauri 命令
#[tauri::command]
pub async fn get_file_info(path: String) -> Result<FileInfo, String> {
    // 验证路径安全性
    let validator = PathValidator::new().require_exists(true);
    let validated_path = validator.validate(&path)
        .map_err(|e| format!("PATH_VALIDATION:{}", e))?;

    let metadata = std::fs::metadata(&validated_path)
        .map_err(|e| format!("IO_ERROR:{}", e))?;

    let modified = metadata
        .modified()
        .map(|time| format!("{:?}", time))
        .unwrap_or_else(|_| "Unknown".to_string());

    Ok(FileInfo {
        path: validated_path.to_string_lossy().to_string(),
        size: metadata.len(),
        is_dir: metadata.is_dir(),
        modified,
    })
}
