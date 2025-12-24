//! Dry-run/Preview mode for destructive operations
//!
//! Provides a way to preview what would be deleted/modified
//! before actually performing the operation.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Type of operation that would be performed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DryRunOperation {
    Delete,
    Shred,
    Modify,
    Clear,
    Flush,
}

/// A single item that would be affected by an operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewItem {
    pub path: String,
    pub operation: DryRunOperation,
    pub size: u64,
    pub item_type: String, // "file", "directory", "log", "cache", etc.
    pub description: String,
    pub risk_level: RiskLevel,
}

/// Risk level for the operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Result of a dry-run/preview operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewResult {
    pub items: Vec<PreviewItem>,
    pub total_files: u32,
    pub total_directories: u32,
    pub total_size: u64,
    pub estimated_time_seconds: u32,
    pub warnings: Vec<String>,
}

impl PreviewResult {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            total_files: 0,
            total_directories: 0,
            total_size: 0,
            estimated_time_seconds: 0,
            warnings: Vec::new(),
        }
    }

    pub fn add_item(&mut self, item: PreviewItem) {
        if item.item_type == "directory" {
            self.total_directories += 1;
        } else {
            self.total_files += 1;
        }
        self.total_size += item.size;
        self.items.push(item);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Estimate time based on file count and size
    pub fn calculate_estimated_time(&mut self) {
        // Rough estimate: 1 second per 100 files + 1 second per 100MB
        let file_time = self.total_files / 100;
        let size_time = (self.total_size / (100 * 1024 * 1024)) as u32;
        self.estimated_time_seconds = (file_time + size_time).max(1);
    }
}

impl Default for PreviewResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Preview file deletion operation
pub fn preview_file_deletion(paths: &[PathBuf], secure_delete: bool) -> PreviewResult {
    let mut result = PreviewResult::new();
    let operation = if secure_delete {
        DryRunOperation::Shred
    } else {
        DryRunOperation::Delete
    };

    for path in paths {
        if !path.exists() {
            result.add_warning(format!("Path does not exist: {}", path.display()));
            continue;
        }

        if path.is_file() {
            if let Ok(metadata) = fs::metadata(path) {
                result.add_item(PreviewItem {
                    path: path.to_string_lossy().to_string(),
                    operation: operation.clone(),
                    size: metadata.len(),
                    item_type: "file".to_string(),
                    description: format!("File: {}", path.file_name().unwrap_or_default().to_string_lossy()),
                    risk_level: determine_file_risk(path),
                });
            }
        } else if path.is_dir() {
            // Walk directory and add all files
            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                if let Ok(metadata) = entry.metadata() {
                    let entry_path = entry.path();
                    let item_type = if metadata.is_dir() {
                        "directory"
                    } else {
                        "file"
                    };

                    result.add_item(PreviewItem {
                        path: entry_path.to_string_lossy().to_string(),
                        operation: operation.clone(),
                        size: if metadata.is_file() { metadata.len() } else { 0 },
                        item_type: item_type.to_string(),
                        description: format!(
                            "{}: {}",
                            if metadata.is_dir() { "Directory" } else { "File" },
                            entry_path.file_name().unwrap_or_default().to_string_lossy()
                        ),
                        risk_level: determine_file_risk(entry_path),
                    });
                }
            }
        }
    }

    result.calculate_estimated_time();
    result
}

/// Preview log cleaning operation
pub fn preview_log_cleaning() -> PreviewResult {
    let mut result = PreviewResult::new();
    let home_dir = std::env::var("HOME").unwrap_or_default();

    #[cfg(target_os = "macos")]
    let log_dirs = vec![
        (format!("{}/Library/Logs", home_dir), "用户日志", RiskLevel::Medium),
        (format!("{}/Library/Logs/DiagnosticReports", home_dir), "诊断报告", RiskLevel::Low),
        ("/var/log".to_string(), "系统日志", RiskLevel::High),
        ("/Library/Logs".to_string(), "全局应用日志", RiskLevel::High),
    ];

    #[cfg(target_os = "windows")]
    let log_dirs = vec![
        ("C:\\Windows\\Logs".to_string(), "Windows 日志", RiskLevel::High),
        ("C:\\Windows\\Temp".to_string(), "Windows 临时文件", RiskLevel::Medium),
    ];

    #[cfg(target_os = "linux")]
    let log_dirs = vec![
        ("/var/log".to_string(), "系统日志", RiskLevel::High),
        (format!("{}/.local/share/systemd/user", home_dir), "用户日志", RiskLevel::Medium),
    ];

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    let log_dirs: Vec<(String, &str, RiskLevel)> = vec![];

    for (log_path, description, risk) in log_dirs {
        let path = Path::new(&log_path);
        if path.exists() {
            let (size, count) = get_directory_stats(path);
            if count > 0 {
                result.add_item(PreviewItem {
                    path: log_path.clone(),
                    operation: DryRunOperation::Clear,
                    size,
                    item_type: "log_directory".to_string(),
                    description: format!("{} ({} files)", description, count),
                    risk_level: risk,
                });
            }
        }
    }

    result.calculate_estimated_time();
    result
}

/// Preview browser data cleaning
pub fn preview_browser_cleaning() -> PreviewResult {
    let mut result = PreviewResult::new();
    let home_dir = std::env::var("HOME").unwrap_or_default();

    #[cfg(target_os = "macos")]
    let browsers = vec![
        (format!("{}/Library/Safari", home_dir), "Safari", RiskLevel::High),
        (format!("{}/Library/Application Support/Google/Chrome", home_dir), "Chrome", RiskLevel::High),
        (format!("{}/Library/Application Support/Firefox", home_dir), "Firefox", RiskLevel::High),
        (format!("{}/Library/Application Support/Microsoft Edge", home_dir), "Edge", RiskLevel::High),
        (format!("{}/Library/Application Support/Arc", home_dir), "Arc", RiskLevel::High),
    ];

    #[cfg(target_os = "windows")]
    let browsers = {
        let localappdata = std::env::var("LOCALAPPDATA").unwrap_or_default();
        let appdata = std::env::var("APPDATA").unwrap_or_default();
        vec![
            (format!("{}\\Google\\Chrome\\User Data", localappdata), "Chrome", RiskLevel::High),
            (format!("{}\\Mozilla\\Firefox\\Profiles", appdata), "Firefox", RiskLevel::High),
            (format!("{}\\Microsoft\\Edge\\User Data", localappdata), "Edge", RiskLevel::High),
        ]
    };

    #[cfg(target_os = "linux")]
    let browsers = vec![
        (format!("{}/.config/google-chrome", home_dir), "Chrome", RiskLevel::High),
        (format!("{}/.mozilla/firefox", home_dir), "Firefox", RiskLevel::High),
    ];

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    let browsers: Vec<(String, &str, RiskLevel)> = vec![];

    for (browser_path, name, risk) in browsers {
        let path = Path::new(&browser_path);
        if path.exists() {
            let (size, count) = get_directory_stats(path);
            if count > 0 {
                result.add_item(PreviewItem {
                    path: browser_path.clone(),
                    operation: DryRunOperation::Clear,
                    size,
                    item_type: "browser_data".to_string(),
                    description: format!("{} 浏览器数据 ({} files, {})", name, count, format_size(size)),
                    risk_level: risk,
                });

                result.add_warning(format!(
                    "清理 {} 将删除所有浏览历史、Cookie 和缓存数据",
                    name
                ));
            }
        }
    }

    result.calculate_estimated_time();
    result
}

/// Preview shell history cleaning
pub fn preview_shell_history_cleaning() -> PreviewResult {
    let mut result = PreviewResult::new();
    let home_dir = std::env::var("HOME").unwrap_or_default();

    let shell_files = vec![
        (format!("{}/.bash_history", home_dir), "Bash 历史"),
        (format!("{}/.zsh_history", home_dir), "Zsh 历史"),
        (format!("{}/.zhistory", home_dir), "Zsh 历史"),
        (format!("{}/.fish_history", home_dir), "Fish 历史"),
        (format!("{}/.local/share/fish/fish_history", home_dir), "Fish 历史"),
    ];

    for (history_path, description) in shell_files {
        let path = Path::new(&history_path);
        if path.exists() {
            if let Ok(metadata) = fs::metadata(path) {
                let line_count = fs::read_to_string(path)
                    .map(|s| s.lines().count())
                    .unwrap_or(0);

                result.add_item(PreviewItem {
                    path: history_path.clone(),
                    operation: DryRunOperation::Clear,
                    size: metadata.len(),
                    item_type: "shell_history".to_string(),
                    description: format!("{} ({} 条命令)", description, line_count),
                    risk_level: RiskLevel::High,
                });
            }
        }
    }

    result.calculate_estimated_time();
    result
}

/// Preview DNS cache flushing
pub fn preview_dns_flush() -> PreviewResult {
    let mut result = PreviewResult::new();

    result.add_item(PreviewItem {
        path: "DNS Resolver Cache".to_string(),
        operation: DryRunOperation::Flush,
        size: 0,
        item_type: "network_cache".to_string(),
        description: "刷新 DNS 缓存，清除所有已解析的域名记录".to_string(),
        risk_level: RiskLevel::Low,
    });

    result.calculate_estimated_time();
    result
}

/// Preview trash emptying
pub fn preview_trash_cleaning() -> PreviewResult {
    let mut result = PreviewResult::new();
    let home_dir = std::env::var("HOME").unwrap_or_default();

    #[cfg(target_os = "macos")]
    let trash_path = format!("{}/.Trash", home_dir);

    #[cfg(target_os = "linux")]
    let trash_path = format!("{}/.local/share/Trash", home_dir);

    #[cfg(target_os = "windows")]
    let trash_path = "C:\\$Recycle.Bin".to_string();

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    let trash_path = String::new();

    if !trash_path.is_empty() {
        let path = Path::new(&trash_path);
        if path.exists() {
            let (size, count) = get_directory_stats(path);
            if count > 0 {
                result.add_item(PreviewItem {
                    path: trash_path,
                    operation: DryRunOperation::Delete,
                    size,
                    item_type: "trash".to_string(),
                    description: format!("回收站 ({} 项, {})", count, format_size(size)),
                    risk_level: RiskLevel::Medium,
                });

                result.add_warning("清空回收站后，文件将无法恢复".to_string());
            }
        }
    }

    result.calculate_estimated_time();
    result
}

/// Combined preview for all cleanup operations
pub fn preview_full_cleanup() -> PreviewResult {
    let mut result = PreviewResult::new();

    // Combine all preview results
    let previews = vec![
        preview_log_cleaning(),
        preview_browser_cleaning(),
        preview_shell_history_cleaning(),
        preview_dns_flush(),
        preview_trash_cleaning(),
    ];

    for preview in previews {
        for item in preview.items {
            result.add_item(item);
        }
        for warning in preview.warnings {
            result.add_warning(warning);
        }
    }

    result.calculate_estimated_time();
    result
}

/// Determine risk level for a file based on its path
fn determine_file_risk(path: &Path) -> RiskLevel {
    let path_str = path.to_string_lossy().to_lowercase();

    // Critical system paths
    if path_str.contains("/system/") || path_str.contains("\\windows\\system32") {
        return RiskLevel::Critical;
    }

    // High risk paths
    if path_str.contains("/etc/")
        || path_str.contains("/var/log")
        || path_str.contains("\\windows\\")
        || path_str.contains("application support")
        || path_str.contains("appdata")
    {
        return RiskLevel::High;
    }

    // Medium risk paths
    if path_str.contains("/library/")
        || path_str.contains("cache")
        || path_str.contains("temp")
        || path_str.contains("tmp")
    {
        return RiskLevel::Medium;
    }

    RiskLevel::Low
}

/// Get directory statistics
fn get_directory_stats(path: &Path) -> (u64, u32) {
    let mut total_size: u64 = 0;
    let mut file_count: u32 = 0;

    if !path.exists() {
        return (0, 0);
    }

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        if let Ok(metadata) = entry.metadata() {
            total_size += metadata.len();
            file_count += 1;
        }
    }

    (total_size, file_count)
}

/// Format size for display
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preview_result() {
        let mut result = PreviewResult::new();
        result.add_item(PreviewItem {
            path: "/tmp/test.txt".to_string(),
            operation: DryRunOperation::Delete,
            size: 1024,
            item_type: "file".to_string(),
            description: "Test file".to_string(),
            risk_level: RiskLevel::Low,
        });

        assert_eq!(result.total_files, 1);
        assert_eq!(result.total_size, 1024);
    }

    #[test]
    fn test_determine_file_risk() {
        assert_eq!(
            determine_file_risk(Path::new("/System/Library/test")),
            RiskLevel::Critical
        );
        assert_eq!(
            determine_file_risk(Path::new("/var/log/test.log")),
            RiskLevel::High
        );
        assert_eq!(
            determine_file_risk(Path::new("/tmp/cache/test")),
            RiskLevel::Medium
        );
        assert_eq!(
            determine_file_risk(Path::new("/home/user/documents/test.txt")),
            RiskLevel::Low
        );
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
    }
}
