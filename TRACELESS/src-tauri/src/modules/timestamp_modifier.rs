use std::collections::HashMap;
use std::path::Path;

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{FILETIME, HANDLE};
#[cfg(target_os = "windows")]
use windows::Win32::Storage::FileSystem::{
    CreateFileW, SetFileTime, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_WRITE, FILE_SHARE_READ,
    FILE_SHARE_WRITE, OPEN_EXISTING,
};
#[cfg(target_os = "windows")]
use windows::core::PCWSTR;
#[cfg(target_os = "windows")]
use std::ffi::OsStr;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;

#[cfg(target_family = "unix")]
use filetime::{FileTime, set_file_times};

#[cfg(target_os = "macos")]
use std::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FileTimestamps {
    pub modified: Option<String>,
    pub accessed: Option<String>,
    pub created: Option<String>,
}

/// 获取文件时间戳
pub fn get_file_timestamps(file_path: &str) -> Result<FileTimestamps, String> {
    let path = Path::new(file_path);

    if !path.exists() {
        return Err(format!("文件不存在: {}", file_path));
    }

    let metadata = std::fs::metadata(path)
        .map_err(|e| format!("无法获取文件元数据: {}", e))?;

    let modified = metadata.modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| {
            chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                .unwrap_or_default()
                .to_rfc3339()
        });

    let accessed = metadata.accessed()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| {
            chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                .unwrap_or_default()
                .to_rfc3339()
        });

    let created = metadata.created()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| {
            chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                .unwrap_or_default()
                .to_rfc3339()
        });

    Ok(FileTimestamps {
        modified,
        accessed,
        created,
    })
}

/// 修改文件时间戳
pub fn modify_file_timestamps(
    file_path: &str,
    timestamps: HashMap<String, String>,
) -> Result<(), String> {
    let path = Path::new(file_path);

    if !path.exists() {
        return Err(format!("文件不存在: {}", file_path));
    }

    #[cfg(target_os = "windows")]
    {
        modify_timestamps_windows(file_path, timestamps)
    }

    #[cfg(target_os = "macos")]
    {
        modify_timestamps_macos(file_path, timestamps)
    }

    #[cfg(all(target_family = "unix", not(target_os = "macos")))]
    {
        modify_timestamps_linux(file_path, timestamps)
    }
}

#[cfg(target_os = "windows")]
fn modify_timestamps_windows(
    file_path: &str,
    timestamps: HashMap<String, String>,
) -> Result<(), String> {
    use chrono::DateTime;

    unsafe {
        let path_wide: Vec<u16> = OsStr::new(file_path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let handle = CreateFileW(
            PCWSTR::from_raw(path_wide.as_ptr()),
            FILE_GENERIC_WRITE.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            HANDLE::default(),
        )
        .map_err(|e| format!("无法打开文件: {}", e))?;

        let mut creation_time: Option<FILETIME> = None;
        let mut last_access_time: Option<FILETIME> = None;
        let mut last_write_time: Option<FILETIME> = None;

        // 解析时间戳
        if let Some(created_str) = timestamps.get("created") {
            if let Ok(dt) = DateTime::parse_from_rfc3339(created_str) {
                creation_time = Some(datetime_to_filetime(dt.timestamp()));
            }
        }

        if let Some(accessed_str) = timestamps.get("accessed") {
            if let Ok(dt) = DateTime::parse_from_rfc3339(accessed_str) {
                last_access_time = Some(datetime_to_filetime(dt.timestamp()));
            }
        }

        if let Some(modified_str) = timestamps.get("modified") {
            if let Ok(dt) = DateTime::parse_from_rfc3339(modified_str) {
                last_write_time = Some(datetime_to_filetime(dt.timestamp()));
            }
        }

        let result = SetFileTime(
            handle,
            creation_time.as_ref(),
            last_access_time.as_ref(),
            last_write_time.as_ref(),
        );

        windows::Win32::Foundation::CloseHandle(handle).ok();

        result.map_err(|e| format!("修改时间戳失败: {}", e))?;
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn datetime_to_filetime(timestamp: i64) -> FILETIME {
    // Convert Unix timestamp to Windows FILETIME
    // FILETIME is 100-nanosecond intervals since January 1, 1601
    const UNIX_EPOCH_FILETIME: i64 = 116444736000000000;

    let filetime_value = (timestamp * 10000000) + UNIX_EPOCH_FILETIME;

    FILETIME {
        dwLowDateTime: (filetime_value & 0xFFFFFFFF) as u32,
        dwHighDateTime: ((filetime_value >> 32) & 0xFFFFFFFF) as u32,
    }
}

#[cfg(target_family = "unix")]
fn modify_timestamps_unix(
    file_path: &str,
    timestamps: HashMap<String, String>,
) -> Result<(), String> {
    use chrono::DateTime;

    let path = Path::new(file_path);

    // Get current timestamps
    let metadata = std::fs::metadata(path)
        .map_err(|e| format!("无法获取文件元数据: {}", e))?;

    let mut atime = FileTime::from_last_access_time(&metadata);
    let mut mtime = FileTime::from_last_modification_time(&metadata);

    // Parse new timestamps
    if let Some(accessed_str) = timestamps.get("accessed") {
        if let Ok(dt) = DateTime::parse_from_rfc3339(accessed_str) {
            atime = FileTime::from_unix_time(dt.timestamp(), 0);
        }
    }

    if let Some(modified_str) = timestamps.get("modified") {
        if let Ok(dt) = DateTime::parse_from_rfc3339(modified_str) {
            mtime = FileTime::from_unix_time(dt.timestamp(), 0);
        }
    }

    // Note: Unix systems don't generally support modifying creation time
    if timestamps.contains_key("created") {
        // Creation time modification is not supported on Unix systems
    }

    set_file_times(path, atime, mtime)
        .map_err(|e| format!("修改时间戳失败: {}", e))?;

    Ok(())
}

/// macOS 专用时间戳修改，支持创建时间
#[cfg(target_os = "macos")]
fn modify_timestamps_macos(
    file_path: &str,
    timestamps: HashMap<String, String>,
) -> Result<(), String> {
    use chrono::DateTime;

    let path = Path::new(file_path);

    // Get current timestamps
    let metadata = std::fs::metadata(path)
        .map_err(|e| format!("无法获取文件元数据: {}", e))?;

    let mut atime = FileTime::from_last_access_time(&metadata);
    let mut mtime = FileTime::from_last_modification_time(&metadata);

    // Parse new timestamps for atime and mtime
    if let Some(accessed_str) = timestamps.get("accessed") {
        if let Ok(dt) = DateTime::parse_from_rfc3339(accessed_str) {
            atime = FileTime::from_unix_time(dt.timestamp(), 0);
        }
    }

    if let Some(modified_str) = timestamps.get("modified") {
        if let Ok(dt) = DateTime::parse_from_rfc3339(modified_str) {
            mtime = FileTime::from_unix_time(dt.timestamp(), 0);
        }
    }

    // Set access and modification time
    set_file_times(path, atime, mtime)
        .map_err(|e| format!("修改访问/修改时间戳失败: {}", e))?;

    // Handle creation time (birth time) on macOS using SetFile command
    if let Some(created_str) = timestamps.get("created") {
        if let Ok(dt) = DateTime::parse_from_rfc3339(created_str) {
            // Format: MM/DD/YYYY HH:MM:SS for SetFile -d
            let formatted = dt.format("%m/%d/%Y %H:%M:%S").to_string();

            let output = Command::new("SetFile")
                .args(["-d", &formatted, file_path])
                .output();

            match output {
                Ok(result) => {
                    if !result.status.success() {
                        // Try using touch command as fallback
                        let touch_format = dt.format("%Y%m%d%H%M.%S").to_string();
                        let touch_result = Command::new("touch")
                            .args(["-t", &touch_format, file_path])
                            .output();

                        if touch_result.is_err() || !touch_result.unwrap().status.success() {
                            // Failed to set creation time, SetFile may not be available
                        }
                    }
                }
                Err(_) => {
                    // SetFile not available, creation time cannot be modified
                }
            }
        }
    }

    Ok(())
}

/// Linux 专用时间戳修改
#[cfg(all(target_family = "unix", not(target_os = "macos")))]
fn modify_timestamps_linux(
    file_path: &str,
    timestamps: HashMap<String, String>,
) -> Result<(), String> {
    use chrono::DateTime;

    let path = Path::new(file_path);

    // Get current timestamps
    let metadata = std::fs::metadata(path)
        .map_err(|e| format!("无法获取文件元数据: {}", e))?;

    let mut atime = FileTime::from_last_access_time(&metadata);
    let mut mtime = FileTime::from_last_modification_time(&metadata);

    // Parse new timestamps
    if let Some(accessed_str) = timestamps.get("accessed") {
        if let Ok(dt) = DateTime::parse_from_rfc3339(accessed_str) {
            atime = FileTime::from_unix_time(dt.timestamp(), 0);
        }
    }

    if let Some(modified_str) = timestamps.get("modified") {
        if let Ok(dt) = DateTime::parse_from_rfc3339(modified_str) {
            mtime = FileTime::from_unix_time(dt.timestamp(), 0);
        }
    }

    // Note: Linux doesn't support modifying creation time (birth time)
    // without using debugfs which requires root
    if timestamps.contains_key("created") {
        // Creation time modification is not supported on Linux
    }

    set_file_times(path, atime, mtime)
        .map_err(|e| format!("修改时间戳失败: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_file_timestamps() {
        // Create a temporary file
        let temp_file = "test_timestamp.txt";
        std::fs::write(temp_file, "test").unwrap();

        let result = get_file_timestamps(temp_file);
        assert!(result.is_ok());

        let timestamps = result.unwrap();
        assert!(timestamps.modified.is_some());
        assert!(timestamps.accessed.is_some());

        // Clean up
        std::fs::remove_file(temp_file).ok();
    }
}
