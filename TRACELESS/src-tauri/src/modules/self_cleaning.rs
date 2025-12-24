//! Self-cleaning module for removing application traces
//!
//! Cleans up Tauri application cache, logs, and temporary files
//! to prevent leaving forensic traces on the system.

use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Result of self-cleaning operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SelfCleanResult {
    pub cleaned_files: u32,
    pub cleaned_bytes: u64,
    pub errors: Vec<String>,
}

/// Get paths to clean for the current application
pub fn get_app_cache_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let home_dir = std::env::var("HOME").unwrap_or_default();
    let app_name = "wuhen"; // Application name

    #[cfg(target_os = "macos")]
    {
        // Tauri/WebKit cache directories
        paths.push(PathBuf::from(format!(
            "{}/Library/Caches/com.wuhen.app",
            home_dir
        )));
        paths.push(PathBuf::from(format!(
            "{}/Library/WebKit/com.wuhen.app",
            home_dir
        )));
        paths.push(PathBuf::from(format!(
            "{}/Library/Application Support/com.wuhen.app/Cache",
            home_dir
        )));
        paths.push(PathBuf::from(format!(
            "{}/Library/Application Support/com.wuhen.app/WebKit",
            home_dir
        )));
        paths.push(PathBuf::from(format!(
            "{}/Library/Application Support/com.wuhen.app/GPUCache",
            home_dir
        )));
        paths.push(PathBuf::from(format!(
            "{}/Library/Application Support/com.wuhen.app/Code Cache",
            home_dir
        )));
        paths.push(PathBuf::from(format!(
            "{}/Library/Application Support/com.wuhen.app/logs",
            home_dir
        )));

        // Tauri 2.x specific paths
        paths.push(PathBuf::from(format!(
            "{}/Library/Caches/{}", home_dir, app_name
        )));
        paths.push(PathBuf::from(format!(
            "{}/Library/Application Support/{}/EBWebView", home_dir, app_name
        )));
    }

    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").unwrap_or_default();
        let localappdata = std::env::var("LOCALAPPDATA").unwrap_or_default();

        // Tauri/WebView2 cache directories
        paths.push(PathBuf::from(format!(
            "{}\\com.wuhen.app\\Cache",
            localappdata
        )));
        paths.push(PathBuf::from(format!(
            "{}\\com.wuhen.app\\EBWebView",
            localappdata
        )));
        paths.push(PathBuf::from(format!(
            "{}\\com.wuhen.app\\GPUCache",
            localappdata
        )));
        paths.push(PathBuf::from(format!(
            "{}\\com.wuhen.app\\Code Cache",
            localappdata
        )));
        paths.push(PathBuf::from(format!(
            "{}\\com.wuhen.app\\logs",
            appdata
        )));

        // Tauri 2.x specific paths
        paths.push(PathBuf::from(format!(
            "{}\\{}\\EBWebView", localappdata, app_name
        )));
    }

    #[cfg(target_os = "linux")]
    {
        // Tauri/WebKitGTK cache directories
        paths.push(PathBuf::from(format!(
            "{}/.cache/com.wuhen.app",
            home_dir
        )));
        paths.push(PathBuf::from(format!(
            "{}/.local/share/com.wuhen.app/Cache",
            home_dir
        )));
        paths.push(PathBuf::from(format!(
            "{}/.local/share/com.wuhen.app/WebKit",
            home_dir
        )));
        paths.push(PathBuf::from(format!(
            "{}/.local/share/com.wuhen.app/logs",
            home_dir
        )));

        // Tauri 2.x specific paths
        paths.push(PathBuf::from(format!(
            "{}/.cache/{}", home_dir, app_name
        )));
        paths.push(PathBuf::from(format!(
            "{}/.local/share/{}", home_dir, app_name
        )));
    }

    // Filter to only existing paths
    paths.into_iter().filter(|p| p.exists()).collect()
}

/// Get temporary files created by the application
pub fn get_temp_files() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // System temp directory
    if let Ok(temp_dir) = std::env::var("TMPDIR")
        .or_else(|_| std::env::var("TEMP"))
        .or_else(|_| std::env::var("TMP"))
    {
        let temp_path = Path::new(&temp_dir);
        if temp_path.exists() {
            // Look for files matching our app patterns
            if let Ok(entries) = fs::read_dir(temp_path) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    // Match Tauri/WebKit temp files
                    if file_name.contains("wuhen")
                        || file_name.contains("tauri")
                        || file_name.contains("webkit")
                        || file_name.starts_with(".com.wuhen")
                        || file_name.starts_with("com.wuhen")
                    {
                        paths.push(entry.path());
                    }
                }
            }
        }
    }

    paths
}

/// Clean application cache directories
pub fn clean_app_cache() -> SelfCleanResult {
    let mut result = SelfCleanResult {
        cleaned_files: 0,
        cleaned_bytes: 0,
        errors: Vec::new(),
    };

    let cache_paths = get_app_cache_paths();

    for cache_path in cache_paths {
        match clean_directory(&cache_path) {
            Ok((files, bytes)) => {
                result.cleaned_files += files;
                result.cleaned_bytes += bytes;
            }
            Err(e) => {
                result.errors.push(format!("{}: {}", cache_path.display(), e));
            }
        }
    }

    // Also clean temp files
    let temp_files = get_temp_files();
    for temp_file in temp_files {
        if let Ok(metadata) = fs::metadata(&temp_file) {
            let size = metadata.len();
            if temp_file.is_dir() {
                if fs::remove_dir_all(&temp_file).is_ok() {
                    result.cleaned_files += 1;
                    result.cleaned_bytes += size;
                }
            } else if fs::remove_file(&temp_file).is_ok() {
                result.cleaned_files += 1;
                result.cleaned_bytes += size;
            }
        }
    }

    result
}

/// Clean a directory recursively
fn clean_directory(path: &Path) -> Result<(u32, u64), String> {
    if !path.exists() {
        return Ok((0, 0));
    }

    let mut cleaned_files: u32 = 0;
    let mut cleaned_bytes: u64 = 0;

    // Walk directory and collect files first
    let entries: Vec<_> = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .collect();

    for entry in entries {
        if let Ok(metadata) = entry.metadata() {
            let size = metadata.len();
            if fs::remove_file(entry.path()).is_ok() {
                cleaned_files += 1;
                cleaned_bytes += size;
            }
        }
    }

    // Remove empty directories (bottom-up)
    let mut dirs: Vec<PathBuf> = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .map(|e| e.path().to_path_buf())
        .collect();

    // Sort by depth (deepest first)
    dirs.sort_by(|a, b| b.components().count().cmp(&a.components().count()));

    for dir in dirs {
        let _ = fs::remove_dir(&dir);
    }

    Ok((cleaned_files, cleaned_bytes))
}

/// Clean browser-specific WebView data
pub fn clean_webview_data() -> SelfCleanResult {
    let mut result = SelfCleanResult {
        cleaned_files: 0,
        cleaned_bytes: 0,
        errors: Vec::new(),
    };

    let home_dir = std::env::var("HOME").unwrap_or_default();

    #[cfg(target_os = "macos")]
    {
        // WebKit LocalStorage, IndexedDB, ServiceWorkers
        let webview_paths = vec![
            format!("{}/Library/Application Support/com.wuhen.app/Local Storage", home_dir),
            format!("{}/Library/Application Support/com.wuhen.app/IndexedDB", home_dir),
            format!("{}/Library/Application Support/com.wuhen.app/Service Workers", home_dir),
            format!("{}/Library/Application Support/com.wuhen.app/Session Storage", home_dir),
            format!("{}/Library/Application Support/com.wuhen.app/Network", home_dir),
        ];

        for path_str in webview_paths {
            let path = PathBuf::from(&path_str);
            if path.exists() {
                match clean_directory(&path) {
                    Ok((files, bytes)) => {
                        result.cleaned_files += files;
                        result.cleaned_bytes += bytes;
                    }
                    Err(e) => {
                        result.errors.push(format!("{}: {}", path_str, e));
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let localappdata = std::env::var("LOCALAPPDATA").unwrap_or_default();

        // WebView2 data directories
        let webview_paths = vec![
            format!("{}\\com.wuhen.app\\EBWebView\\Default\\Local Storage", localappdata),
            format!("{}\\com.wuhen.app\\EBWebView\\Default\\IndexedDB", localappdata),
            format!("{}\\com.wuhen.app\\EBWebView\\Default\\Service Worker", localappdata),
            format!("{}\\com.wuhen.app\\EBWebView\\Default\\Session Storage", localappdata),
            format!("{}\\com.wuhen.app\\EBWebView\\Default\\Network", localappdata),
        ];

        for path_str in webview_paths {
            let path = PathBuf::from(&path_str);
            if path.exists() {
                match clean_directory(&path) {
                    Ok((files, bytes)) => {
                        result.cleaned_files += files;
                        result.cleaned_bytes += bytes;
                    }
                    Err(e) => {
                        result.errors.push(format!("{}: {}", path_str, e));
                    }
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        // WebKitGTK data directories
        let webview_paths = vec![
            format!("{}/.local/share/com.wuhen.app/Local Storage", home_dir),
            format!("{}/.local/share/com.wuhen.app/IndexedDB", home_dir),
            format!("{}/.local/share/com.wuhen.app/Service Workers", home_dir),
            format!("{}/.local/share/com.wuhen.app/Session Storage", home_dir),
        ];

        for path_str in webview_paths {
            let path = PathBuf::from(&path_str);
            if path.exists() {
                match clean_directory(&path) {
                    Ok((files, bytes)) => {
                        result.cleaned_files += files;
                        result.cleaned_bytes += bytes;
                    }
                    Err(e) => {
                        result.errors.push(format!("{}: {}", path_str, e));
                    }
                }
            }
        }
    }

    result
}

/// Perform full self-cleaning
pub fn perform_self_cleaning() -> SelfCleanResult {
    let cache_result = clean_app_cache();
    let webview_result = clean_webview_data();

    SelfCleanResult {
        cleaned_files: cache_result.cleaned_files + webview_result.cleaned_files,
        cleaned_bytes: cache_result.cleaned_bytes + webview_result.cleaned_bytes,
        errors: cache_result
            .errors
            .into_iter()
            .chain(webview_result.errors)
            .collect(),
    }
}

/// Get estimated size of app traces
pub fn get_app_traces_size() -> (u64, u32) {
    let mut total_size: u64 = 0;
    let mut file_count: u32 = 0;

    let cache_paths = get_app_cache_paths();
    for path in cache_paths {
        let (size, count) = get_directory_stats(&path);
        total_size += size;
        file_count += count;
    }

    let temp_files = get_temp_files();
    for temp_file in temp_files {
        if let Ok(metadata) = fs::metadata(&temp_file) {
            total_size += metadata.len();
            file_count += 1;
        }
    }

    (total_size, file_count)
}

/// Get directory statistics
fn get_directory_stats(path: &Path) -> (u64, u32) {
    let mut total_size: u64 = 0;
    let mut file_count: u32 = 0;

    if !path.exists() {
        return (0, 0);
    }

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
                file_count += 1;
            }
        }
    }

    (total_size, file_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_app_cache_paths() {
        let paths = get_app_cache_paths();
        // May be empty if app hasn't created cache yet
        assert!(paths.iter().all(|p| p.exists()));
    }

    #[test]
    fn test_get_temp_files() {
        let files = get_temp_files();
        // May be empty
        assert!(files.iter().all(|p| p.exists()));
    }

    #[test]
    fn test_get_app_traces_size() {
        let (size, count) = get_app_traces_size();
        // Just verify it returns without error
        assert!(size >= 0);
        assert!(count >= 0);
    }
}
