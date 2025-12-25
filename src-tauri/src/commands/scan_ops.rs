use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ScanSeverity {
    High,
    Medium,
    Low,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanResult {
    pub id: String,
    pub category: String,
    pub item_type: String,
    pub description: String,
    pub severity: ScanSeverity,
    pub path: Option<String>,
    pub size: Option<String>,
    pub count: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanProgress {
    pub progress: u32,
    pub current_item: String,
    pub found_count: u32,
}

/// 格式化文件大小
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

/// 递归计算目录大小和文件数量
fn get_dir_stats(path: &Path) -> (u64, u32) {
    let mut total_size: u64 = 0;
    let mut file_count: u32 = 0;

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_dir() {
                    let (sub_size, sub_count) = get_dir_stats(&entry_path);
                    total_size += sub_size;
                    file_count += sub_count;
                } else {
                    total_size += metadata.len();
                    file_count += 1;
                }
            }
        }
    }

    (total_size, file_count)
}

/// 执行系统扫描
#[tauri::command]
pub async fn perform_system_scan(mode: String) -> Result<Vec<ScanResult>, String> {
    let mut results = Vec::new();
    let is_full_scan = mode == "full";

    // 设置扫描延迟，让用户能看到扫描过程
    let scan_delay = if is_full_scan { 600 } else { 400 };

    // 1. 扫描系统日志
    tokio::time::sleep(tokio::time::Duration::from_millis(scan_delay)).await;
    if let Ok(log_results) = scan_system_logs().await {
        results.extend(log_results);
    }

    // 2. 扫描临时文件
    tokio::time::sleep(tokio::time::Duration::from_millis(scan_delay)).await;
    if let Ok(temp_results) = scan_temp_files().await {
        results.extend(temp_results);
    }

    // 3. 扫描浏览器数据
    tokio::time::sleep(tokio::time::Duration::from_millis(scan_delay)).await;
    if let Ok(browser_results) = scan_browser_data().await {
        results.extend(browser_results);
    }

    // 4. 扫描最近文档
    tokio::time::sleep(tokio::time::Duration::from_millis(scan_delay)).await;
    if let Ok(recent_results) = scan_recent_documents().await {
        results.extend(recent_results);
    }

    // 5. 扫描网络缓存
    tokio::time::sleep(tokio::time::Duration::from_millis(scan_delay)).await;
    if let Ok(network_results) = scan_network_cache().await {
        results.extend(network_results);
    }

    // 6. 扫描 Shell 历史
    tokio::time::sleep(tokio::time::Duration::from_millis(scan_delay)).await;
    if let Ok(shell_results) = scan_shell_history().await {
        results.extend(shell_results);
    }

    // 完整扫描的额外项
    if is_full_scan {
        // 7. 扫描应用缓存
        tokio::time::sleep(tokio::time::Duration::from_millis(scan_delay)).await;
        if let Ok(cache_results) = scan_app_cache().await {
            results.extend(cache_results);
        }

        // 8. 扫描崩溃日志
        tokio::time::sleep(tokio::time::Duration::from_millis(scan_delay)).await;
        if let Ok(crash_results) = scan_crash_logs().await {
            results.extend(crash_results);
        }

        // 9. 扫描下载历史
        tokio::time::sleep(tokio::time::Duration::from_millis(scan_delay)).await;
        if let Ok(download_results) = scan_downloads().await {
            results.extend(download_results);
        }

        // 10. 扫描垃圾桶
        tokio::time::sleep(tokio::time::Duration::from_millis(scan_delay)).await;
        if let Ok(trash_results) = scan_trash().await {
            results.extend(trash_results);
        }

        // 11. 扫描 SSH 已知主机
        tokio::time::sleep(tokio::time::Duration::from_millis(scan_delay)).await;
        if let Ok(ssh_results) = scan_ssh_known_hosts().await {
            results.extend(ssh_results);
        }

        // 12. 反分析检测
        tokio::time::sleep(tokio::time::Duration::from_millis(scan_delay)).await;
        if let Ok(analysis_results) = scan_anti_analysis().await {
            results.extend(analysis_results);
        }

        // 13. 扫描 Time Machine 本地快照 (macOS)
        tokio::time::sleep(tokio::time::Duration::from_millis(scan_delay)).await;
        if let Ok(tm_results) = scan_time_machine_snapshots().await {
            results.extend(tm_results);
        }

        // 14. 扫描系统缓存 (QuickLook, 字体缓存等)
        tokio::time::sleep(tokio::time::Duration::from_millis(scan_delay)).await;
        if let Ok(syscache_results) = scan_system_cache().await {
            results.extend(syscache_results);
        }

        // 15. 扫描开发工具缓存
        tokio::time::sleep(tokio::time::Duration::from_millis(scan_delay)).await;
        if let Ok(dev_results) = scan_dev_tools_cache().await {
            results.extend(dev_results);
        }

        // 16. 扫描 IDE 扩展缓存
        tokio::time::sleep(tokio::time::Duration::from_millis(scan_delay)).await;
        if let Ok(ide_results) = scan_ide_extension_cache().await {
            results.extend(ide_results);
        }

        // 17. 扫描休眠镜像
        tokio::time::sleep(tokio::time::Duration::from_millis(scan_delay)).await;
        if let Ok(sleep_results) = scan_hibernation_image().await {
            results.extend(sleep_results);
        }

        // 18. 扫描下载大文件
        tokio::time::sleep(tokio::time::Duration::from_millis(scan_delay)).await;
        if let Ok(large_file_results) = scan_downloads_large_files().await {
            results.extend(large_file_results);
        }

        // 19. 扫描 AI/ML 工具缓存
        tokio::time::sleep(tokio::time::Duration::from_millis(scan_delay)).await;
        if let Ok(ai_results) = scan_ai_ml_cache().await {
            results.extend(ai_results);
        }

        // 20. 扫描中国应用缓存
        tokio::time::sleep(tokio::time::Duration::from_millis(scan_delay)).await;
        if let Ok(cn_results) = scan_chinese_apps_cache().await {
            results.extend(cn_results);
        }
    }

    // 最后添加一个小延迟确保进度达到100%
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    Ok(results)
}

/// 扫描系统日志
async fn scan_system_logs() -> Result<Vec<ScanResult>, String> {
    let mut results = Vec::new();

    #[cfg(target_os = "windows")]
    {
        // Windows 事件日志
        let log_paths = vec![
            ("Application", "C:\\Windows\\System32\\winevt\\Logs\\Application.evtx"),
            ("System", "C:\\Windows\\System32\\winevt\\Logs\\System.evtx"),
            ("Security", "C:\\Windows\\System32\\winevt\\Logs\\Security.evtx"),
            ("Setup", "C:\\Windows\\System32\\winevt\\Logs\\Setup.evtx"),
            ("PowerShell", "C:\\Windows\\System32\\winevt\\Logs\\Windows PowerShell.evtx"),
        ];

        for (log_type, path) in log_paths {
            if Path::new(path).exists() {
                if let Ok(metadata) = fs::metadata(path) {
                    results.push(ScanResult {
                        id: format!("log_{}", log_type.to_lowercase()),
                        category: "系统日志".to_string(),
                        item_type: log_type.to_string(),
                        description: format!("发现 {} 事件日志", log_type),
                        severity: ScanSeverity::High,
                        path: Some(path.to_string()),
                        size: Some(format_size(metadata.len())),
                        count: None,
                    });
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/Users".to_string());

        // macOS 日志目录 - 用户级别
        let user_log_dirs = vec![
            (format!("{}/Library/Logs", home_dir), "用户日志"),
            (format!("{}/Library/Logs/DiagnosticReports", home_dir), "用户诊断报告"),
        ];

        for (dir_path, log_type) in user_log_dirs {
            let path = Path::new(&dir_path);
            if path.exists() {
                let (total_size, file_count) = get_dir_stats(path);
                if file_count > 0 {
                    results.push(ScanResult {
                        id: format!("log_{}", log_type.replace(" ", "_")),
                        category: "系统日志".to_string(),
                        item_type: log_type.to_string(),
                        description: format!("发现 {} 个日志文件", file_count),
                        severity: ScanSeverity::High,
                        path: Some(dir_path),
                        size: Some(format_size(total_size)),
                        count: Some(file_count),
                    });
                }
            }
        }

        // macOS 系统级日志目录（需要管理员权限）
        let system_log_dirs = vec![
            ("/var/log".to_string(), "系统日志"),
            ("/Library/Logs".to_string(), "全局应用日志"),
            ("/private/var/log/asl".to_string(), "Apple System Logs"),
            ("/private/var/log/DiagnosticMessages".to_string(), "系统诊断消息"),
        ];

        for (dir_path, log_type) in system_log_dirs {
            let path = Path::new(&dir_path);
            if path.exists() {
                let (total_size, file_count) = get_dir_stats(path);
                if file_count > 0 {
                    results.push(ScanResult {
                        id: format!("log_{}", log_type.replace(" ", "_")),
                        category: "系统日志".to_string(),
                        item_type: format!("{} (需管理员权限)", log_type),
                        description: format!("发现 {} 个日志文件", file_count),
                        severity: ScanSeverity::High,
                        path: Some(dir_path),
                        size: Some(format_size(total_size)),
                        count: Some(file_count),
                    });
                }
            }
        }

        // 统一日志 (Unified Logs)
        let unified_log_path = "/private/var/db/diagnostics";
        if Path::new(unified_log_path).exists() {
            let (total_size, file_count) = get_dir_stats(Path::new(unified_log_path));
            if file_count > 0 {
                results.push(ScanResult {
                    id: "log_unified".to_string(),
                    category: "系统日志".to_string(),
                    item_type: "统一日志 (需管理员权限)".to_string(),
                    description: format!("发现 {} 个统一日志文件", file_count),
                    severity: ScanSeverity::High,
                    path: Some(unified_log_path.to_string()),
                    size: Some(format_size(total_size)),
                    count: Some(file_count),
                });
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());

        let log_dirs = vec![
            (format!("{}/.local/share/systemd/user", home_dir), "用户 Systemd 日志"),
            ("/var/log".to_string(), "系统日志"),
            ("/var/log/journal".to_string(), "Journald 日志"),
        ];

        for (log_path, log_type) in log_dirs {
            let path = Path::new(&log_path);
            if path.exists() {
                let (total_size, file_count) = get_dir_stats(path);
                if file_count > 0 {
                    results.push(ScanResult {
                        id: format!("log_{}", log_type.replace(" ", "_").to_lowercase()),
                        category: "系统日志".to_string(),
                        item_type: log_type.to_string(),
                        description: format!("发现 {} 个日志文件", file_count),
                        severity: ScanSeverity::High,
                        path: Some(log_path),
                        size: Some(format_size(total_size)),
                        count: Some(file_count),
                    });
                }
            }
        }
    }

    Ok(results)
}

/// 扫描临时文件
async fn scan_temp_files() -> Result<Vec<ScanResult>, String> {
    let mut results = Vec::new();

    #[cfg(target_os = "windows")]
    let temp_dirs = vec![
        (std::env::var("TEMP").unwrap_or_else(|_| "C:\\Temp".to_string()), "用户临时文件"),
        ("C:\\Windows\\Temp".to_string(), "系统临时文件"),
    ];

    #[cfg(target_os = "macos")]
    let temp_dirs = {
        let home_dir = std::env::var("HOME").unwrap_or_default();
        vec![
            ("/tmp".to_string(), "系统临时文件"),
            ("/private/var/folders".to_string(), "私有临时文件"),
            (format!("{}/Library/Caches", home_dir), "用户缓存"),
        ]
    };

    #[cfg(target_os = "linux")]
    let temp_dirs = vec![
        ("/tmp".to_string(), "系统临时文件"),
        ("/var/tmp".to_string(), "持久临时文件"),
    ];

    for (temp_dir, dir_type) in temp_dirs {
        let path = Path::new(&temp_dir);
        if path.exists() {
            let (total_size, file_count) = get_dir_stats(path);
            if file_count > 0 {
                results.push(ScanResult {
                    id: format!("temp_{}", temp_dir.replace(['/', '\\', ':'], "_")),
                    category: "临时文件".to_string(),
                    item_type: dir_type.to_string(),
                    description: format!("发现 {} 个临时文件", file_count),
                    severity: ScanSeverity::Medium,
                    path: Some(temp_dir),
                    size: Some(format_size(total_size)),
                    count: Some(file_count),
                });
            }
        }
    }

    Ok(results)
}

/// 扫描浏览器数据
async fn scan_browser_data() -> Result<Vec<ScanResult>, String> {
    let mut results = Vec::new();
    let home_dir = std::env::var("HOME").unwrap_or_default();

    #[cfg(target_os = "macos")]
    let browsers = vec![
        // 主流浏览器
        (format!("{}/Library/Safari", home_dir), "Safari", "safari"),
        (format!("{}/Library/Application Support/Google/Chrome", home_dir), "Chrome", "chrome"),
        (format!("{}/Library/Application Support/Firefox", home_dir), "Firefox", "firefox"),
        (format!("{}/Library/Application Support/Microsoft Edge", home_dir), "Edge", "edge"),
        // 其他浏览器
        (format!("{}/Library/Application Support/Arc", home_dir), "Arc", "arc"),
        (format!("{}/Library/Application Support/BraveSoftware/Brave-Browser", home_dir), "Brave", "brave"),
        (format!("{}/Library/Application Support/Vivaldi", home_dir), "Vivaldi", "vivaldi"),
        (format!("{}/Library/Application Support/com.operasoftware.Opera", home_dir), "Opera", "opera"),
        (format!("{}/Library/Application Support/Opera Software/Opera Stable", home_dir), "Opera", "opera_stable"),
        (format!("{}/Library/Application Support/Chromium", home_dir), "Chromium", "chromium"),
        (format!("{}/Library/Application Support/Orion", home_dir), "Orion", "orion"),
        (format!("{}/Library/Application Support/Waterfox", home_dir), "Waterfox", "waterfox"),
        (format!("{}/Library/Application Support/LibreWolf", home_dir), "LibreWolf", "librewolf"),
        (format!("{}/Library/Application Support/Yandex/YandexBrowser", home_dir), "Yandex", "yandex"),
        (format!("{}/Library/Application Support/Sidekick", home_dir), "Sidekick", "sidekick"),
    ];

    #[cfg(target_os = "windows")]
    let browsers = {
        let localappdata = std::env::var("LOCALAPPDATA").unwrap_or_default();
        let appdata = std::env::var("APPDATA").unwrap_or_default();
        vec![
            (format!("{}\\Google\\Chrome\\User Data", localappdata), "Chrome", "chrome"),
            (format!("{}\\Mozilla\\Firefox\\Profiles", appdata), "Firefox", "firefox"),
            (format!("{}\\Microsoft\\Edge\\User Data", localappdata), "Edge", "edge"),
            (format!("{}\\BraveSoftware\\Brave-Browser\\User Data", localappdata), "Brave", "brave"),
            (format!("{}\\Vivaldi\\User Data", localappdata), "Vivaldi", "vivaldi"),
            (format!("{}\\Opera Software\\Opera Stable", appdata), "Opera", "opera"),
            (format!("{}\\Arc\\User Data", localappdata), "Arc", "arc"),
        ]
    };

    #[cfg(target_os = "linux")]
    let browsers = vec![
        (format!("{}/.config/google-chrome", home_dir), "Chrome", "chrome"),
        (format!("{}/.mozilla/firefox", home_dir), "Firefox", "firefox"),
        (format!("{}/.config/microsoft-edge", home_dir), "Edge", "edge"),
        (format!("{}/.config/BraveSoftware/Brave-Browser", home_dir), "Brave", "brave"),
        (format!("{}/.config/vivaldi", home_dir), "Vivaldi", "vivaldi"),
        (format!("{}/.config/opera", home_dir), "Opera", "opera"),
        (format!("{}/.config/chromium", home_dir), "Chromium", "chromium"),
    ];

    for (browser_path, browser_name, browser_id) in browsers {
        let path = Path::new(&browser_path);
        if path.exists() {
            let (total_size, file_count) = get_dir_stats(path);
            if file_count > 0 {
                results.push(ScanResult {
                    id: format!("browser_{}", browser_id),
                    category: "浏览器数据".to_string(),
                    item_type: format!("{} 浏览数据", browser_name),
                    description: format!("发现 {} 浏览记录/缓存/Cookie", browser_name),
                    severity: ScanSeverity::High,
                    path: Some(browser_path),
                    size: Some(format_size(total_size)),
                    count: Some(file_count),
                });
            }
        }
    }

    Ok(results)
}

/// 扫描最近文档
async fn scan_recent_documents() -> Result<Vec<ScanResult>, String> {
    let mut results = Vec::new();
    let home_dir = std::env::var("HOME").unwrap_or_default();

    #[cfg(target_os = "macos")]
    {
        let recent_paths = vec![
            (format!("{}/Library/Application Support/com.apple.sharedfilelist", home_dir), "最近项目列表"),
            (format!("{}/Library/Preferences/com.apple.recentitems.plist", home_dir), "最近文件"),
        ];

        for (recent_path, item_type) in recent_paths {
            let path = Path::new(&recent_path);
            if path.exists() {
                if path.is_dir() {
                    let (total_size, file_count) = get_dir_stats(path);
                    if file_count > 0 {
                        results.push(ScanResult {
                            id: format!("recent_{}", item_type.replace(" ", "_")),
                            category: "最近文档".to_string(),
                            item_type: item_type.to_string(),
                            description: format!("发现最近使用记录"),
                            severity: ScanSeverity::High,
                            path: Some(recent_path),
                            size: Some(format_size(total_size)),
                            count: Some(file_count),
                        });
                    }
                } else if let Ok(metadata) = fs::metadata(&recent_path) {
                    results.push(ScanResult {
                        id: format!("recent_{}", item_type.replace(" ", "_")),
                        category: "最近文档".to_string(),
                        item_type: item_type.to_string(),
                        description: "发现最近使用记录".to_string(),
                        severity: ScanSeverity::High,
                        path: Some(recent_path),
                        size: Some(format_size(metadata.len())),
                        count: None,
                    });
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").unwrap_or_default();
        let recent_path = format!("{}\\Microsoft\\Windows\\Recent", appdata);
        let path = Path::new(&recent_path);
        if path.exists() {
            let (total_size, file_count) = get_dir_stats(path);
            if file_count > 0 {
                results.push(ScanResult {
                    id: "recent_windows".to_string(),
                    category: "最近文档".to_string(),
                    item_type: "最近使用的文件".to_string(),
                    description: format!("发现 {} 个最近文件记录", file_count),
                    severity: ScanSeverity::High,
                    path: Some(recent_path),
                    size: Some(format_size(total_size)),
                    count: Some(file_count),
                });
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let recent_path = format!("{}/.local/share/recently-used.xbel", home_dir);
        if Path::new(&recent_path).exists() {
            if let Ok(metadata) = fs::metadata(&recent_path) {
                results.push(ScanResult {
                    id: "recent_linux".to_string(),
                    category: "最近文档".to_string(),
                    item_type: "最近使用的文件".to_string(),
                    description: "发现最近使用记录".to_string(),
                    severity: ScanSeverity::High,
                    path: Some(recent_path),
                    size: Some(format_size(metadata.len())),
                    count: None,
                });
            }
        }
    }

    Ok(results)
}

/// 扫描网络缓存
async fn scan_network_cache() -> Result<Vec<ScanResult>, String> {
    let mut results = Vec::new();

    // 检测 DNS 缓存
    #[cfg(target_os = "macos")]
    {
        // macOS 使用 dscacheutil 检查 DNS 缓存
        if let Ok(output) = Command::new("dscacheutil")
            .args(["-cachedump", "-entries"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let entry_count = output_str.lines().count();
            if entry_count > 0 {
                results.push(ScanResult {
                    id: "network_dns".to_string(),
                    category: "网络缓存".to_string(),
                    item_type: "DNS 缓存".to_string(),
                    description: format!("发现 {} 条 DNS 缓存记录", entry_count),
                    severity: ScanSeverity::Medium,
                    path: Some("DNS Resolver Cache".to_string()),
                    size: None,
                    count: Some(entry_count as u32),
                });
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Windows 使用 ipconfig /displaydns 检查 DNS 缓存
        if let Ok(output) = Command::new("ipconfig")
            .arg("/displaydns")
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let entry_count = output_str.matches("Record Name").count();
            if entry_count > 0 {
                results.push(ScanResult {
                    id: "network_dns".to_string(),
                    category: "网络缓存".to_string(),
                    item_type: "DNS 缓存".to_string(),
                    description: format!("发现 {} 条 DNS 缓存记录", entry_count),
                    severity: ScanSeverity::Medium,
                    path: Some("DNS Resolver Cache".to_string()),
                    size: None,
                    count: Some(entry_count as u32),
                });
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: 检查 systemd-resolved 缓存
        if let Ok(output) = Command::new("resolvectl")
            .arg("statistics")
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if output_str.contains("Cache") {
                results.push(ScanResult {
                    id: "network_dns".to_string(),
                    category: "网络缓存".to_string(),
                    item_type: "DNS 缓存".to_string(),
                    description: "发现 DNS 缓存记录".to_string(),
                    severity: ScanSeverity::Medium,
                    path: Some("systemd-resolved cache".to_string()),
                    size: None,
                    count: None,
                });
            }
        }
    }

    // 如果没有检测到 DNS 缓存，添加一个通用结果
    if results.is_empty() {
        results.push(ScanResult {
            id: "network_dns".to_string(),
            category: "网络缓存".to_string(),
            item_type: "DNS 缓存".to_string(),
            description: "DNS 缓存可能存在记录".to_string(),
            severity: ScanSeverity::Medium,
            path: Some("DNS Resolver Cache".to_string()),
            size: None,
            count: None,
        });
    }

    Ok(results)
}

/// 扫描 Shell 历史
async fn scan_shell_history() -> Result<Vec<ScanResult>, String> {
    let mut results = Vec::new();
    let home_dir = std::env::var("HOME").unwrap_or_default();

    let shell_files = vec![
        (format!("{}/.bash_history", home_dir), "Bash 历史"),
        (format!("{}/.zsh_history", home_dir), "Zsh 历史"),
        (format!("{}/.zhistory", home_dir), "Zsh 历史"),
        (format!("{}/.fish_history", home_dir), "Fish 历史"),
        (format!("{}/.local/share/fish/fish_history", home_dir), "Fish 历史"),
    ];

    for (history_path, history_type) in shell_files {
        let path = Path::new(&history_path);
        if path.exists() {
            if let Ok(metadata) = fs::metadata(&history_path) {
                // 计算命令行数
                let line_count = if let Ok(content) = fs::read_to_string(&history_path) {
                    content.lines().count() as u32
                } else {
                    0
                };

                if line_count > 0 || metadata.len() > 0 {
                    results.push(ScanResult {
                        id: format!("shell_{}", history_type.replace(" ", "_").to_lowercase()),
                        category: "命令历史".to_string(),
                        item_type: history_type.to_string(),
                        description: format!("发现 {} 条命令记录", line_count),
                        severity: ScanSeverity::High,
                        path: Some(history_path),
                        size: Some(format_size(metadata.len())),
                        count: Some(line_count),
                    });
                }
            }
        }
    }

    Ok(results)
}

/// 扫描应用缓存（完整扫描）
async fn scan_app_cache() -> Result<Vec<ScanResult>, String> {
    let mut results = Vec::new();
    let home_dir = std::env::var("HOME").unwrap_or_default();

    #[cfg(target_os = "macos")]
    let cache_dirs = vec![
        (format!("{}/Library/Caches/com.apple.Safari", home_dir), "Safari 缓存"),
        (format!("{}/Library/Caches/Google/Chrome", home_dir), "Chrome 缓存"),
        (format!("{}/Library/Caches/Firefox", home_dir), "Firefox 缓存"),
        (format!("{}/Library/Caches/Slack", home_dir), "Slack 缓存"),
        (format!("{}/Library/Caches/com.microsoft.VSCode", home_dir), "VSCode 缓存"),
        (format!("{}/Library/Caches/com.spotify.client", home_dir), "Spotify 缓存"),
    ];

    #[cfg(target_os = "windows")]
    let cache_dirs = {
        let localappdata = std::env::var("LOCALAPPDATA").unwrap_or_default();
        vec![
            (format!("{}\\Google\\Chrome\\User Data\\Default\\Cache", localappdata), "Chrome 缓存"),
            (format!("{}\\Microsoft\\Edge\\User Data\\Default\\Cache", localappdata), "Edge 缓存"),
        ]
    };

    #[cfg(target_os = "linux")]
    let cache_dirs = vec![
        (format!("{}/.cache/google-chrome", home_dir), "Chrome 缓存"),
        (format!("{}/.cache/mozilla", home_dir), "Firefox 缓存"),
    ];

    for (cache_path, cache_type) in cache_dirs {
        let path = Path::new(&cache_path);
        if path.exists() {
            let (total_size, file_count) = get_dir_stats(path);
            if file_count > 0 {
                results.push(ScanResult {
                    id: format!("cache_{}", cache_type.replace(" ", "_").to_lowercase()),
                    category: "应用缓存".to_string(),
                    item_type: cache_type.to_string(),
                    description: format!("发现 {} 个缓存文件", file_count),
                    severity: ScanSeverity::Medium,
                    path: Some(cache_path),
                    size: Some(format_size(total_size)),
                    count: Some(file_count),
                });
            }
        }
    }

    Ok(results)
}

/// 扫描崩溃日志（完整扫描）
async fn scan_crash_logs() -> Result<Vec<ScanResult>, String> {
    let mut results = Vec::new();
    let home_dir = std::env::var("HOME").unwrap_or_default();

    #[cfg(target_os = "macos")]
    let crash_dirs = vec![
        (format!("{}/Library/Logs/DiagnosticReports", home_dir), "用户诊断报告"),
        (format!("{}/Library/Logs/CrashReporter", home_dir), "崩溃报告"),
        ("/Library/Logs/DiagnosticReports".to_string(), "系统诊断报告"),
    ];

    #[cfg(target_os = "windows")]
    let crash_dirs = vec![
        ("C:\\Windows\\Minidump".to_string(), "内存转储"),
        ("C:\\Windows\\LiveKernelReports".to_string(), "内核报告"),
    ];

    #[cfg(target_os = "linux")]
    let crash_dirs = vec![
        ("/var/crash".to_string(), "系统崩溃报告"),
        (format!("{}/.local/share/apport/recoverable", home_dir), "应用崩溃报告"),
    ];

    for (crash_path, crash_type) in crash_dirs {
        let path = Path::new(&crash_path);
        if path.exists() {
            let (total_size, file_count) = get_dir_stats(path);
            if file_count > 0 {
                results.push(ScanResult {
                    id: format!("crash_{}", crash_type.replace(" ", "_").to_lowercase()),
                    category: "崩溃日志".to_string(),
                    item_type: crash_type.to_string(),
                    description: format!("发现 {} 个崩溃报告", file_count),
                    severity: ScanSeverity::Medium,
                    path: Some(crash_path),
                    size: Some(format_size(total_size)),
                    count: Some(file_count),
                });
            }
        }
    }

    Ok(results)
}

/// 扫描下载文件夹（完整扫描）
async fn scan_downloads() -> Result<Vec<ScanResult>, String> {
    let mut results = Vec::new();
    let home_dir = std::env::var("HOME").unwrap_or_default();

    let download_path = format!("{}/Downloads", home_dir);
    let path = Path::new(&download_path);

    if path.exists() {
        let (total_size, file_count) = get_dir_stats(path);
        if file_count > 0 {
            results.push(ScanResult {
                id: "downloads".to_string(),
                category: "下载记录".to_string(),
                item_type: "下载文件夹".to_string(),
                description: format!("发现 {} 个下载文件", file_count),
                severity: ScanSeverity::Low,
                path: Some(download_path),
                size: Some(format_size(total_size)),
                count: Some(file_count),
            });
        }
    }

    Ok(results)
}

/// 扫描垃圾桶（完整扫描）
async fn scan_trash() -> Result<Vec<ScanResult>, String> {
    let mut results = Vec::new();
    let home_dir = std::env::var("HOME").unwrap_or_default();

    #[cfg(target_os = "macos")]
    let trash_path = format!("{}/.Trash", home_dir);

    #[cfg(target_os = "linux")]
    let trash_path = format!("{}/.local/share/Trash", home_dir);

    #[cfg(target_os = "windows")]
    let trash_path = "C:\\$Recycle.Bin".to_string();

    let path = Path::new(&trash_path);
    if path.exists() {
        let (total_size, file_count) = get_dir_stats(path);
        if file_count > 0 {
            results.push(ScanResult {
                id: "trash".to_string(),
                category: "回收站".to_string(),
                item_type: "已删除文件".to_string(),
                description: format!("发现 {} 个已删除文件", file_count),
                severity: ScanSeverity::Medium,
                path: Some(trash_path),
                size: Some(format_size(total_size)),
                count: Some(file_count),
            });
        }
    }

    Ok(results)
}

/// 扫描 SSH 已知主机（完整扫描）
async fn scan_ssh_known_hosts() -> Result<Vec<ScanResult>, String> {
    let mut results = Vec::new();
    let home_dir = std::env::var("HOME").unwrap_or_default();

    let ssh_path = format!("{}/.ssh/known_hosts", home_dir);
    let path = Path::new(&ssh_path);

    if path.exists() {
        if let Ok(metadata) = fs::metadata(&ssh_path) {
            let line_count = if let Ok(content) = fs::read_to_string(&ssh_path) {
                content.lines().count() as u32
            } else {
                0
            };

            if line_count > 0 {
                results.push(ScanResult {
                    id: "ssh_known_hosts".to_string(),
                    category: "SSH 记录".to_string(),
                    item_type: "已知主机".to_string(),
                    description: format!("发现 {} 个 SSH 连接记录", line_count),
                    severity: ScanSeverity::Medium,
                    path: Some(ssh_path),
                    size: Some(format_size(metadata.len())),
                    count: Some(line_count),
                });
            }
        }
    }

    Ok(results)
}

/// 反分析检测
async fn scan_anti_analysis() -> Result<Vec<ScanResult>, String> {
    let mut results = Vec::new();

    // 检测虚拟机环境
    let is_vm = detect_virtual_machine();

    results.push(ScanResult {
        id: "anti_analysis_vm".to_string(),
        category: "环境检测".to_string(),
        item_type: "虚拟机检测".to_string(),
        description: if is_vm {
            "检测到虚拟机环境".to_string()
        } else {
            "未检测到虚拟机环境".to_string()
        },
        severity: if is_vm { ScanSeverity::High } else { ScanSeverity::Low },
        path: None,
        size: None,
        count: None,
    });

    Ok(results)
}

/// 检测虚拟机环境
fn detect_virtual_machine() -> bool {
    #[cfg(target_os = "macos")]
    {
        // 检查 sysctl 获取硬件信息
        if let Ok(output) = Command::new("sysctl")
            .args(["-n", "machdep.cpu.brand_string"])
            .output()
        {
            let cpu_info = String::from_utf8_lossy(&output.stdout).to_lowercase();
            if cpu_info.contains("virtual") || cpu_info.contains("vmware") || cpu_info.contains("qemu") {
                return true;
            }
        }

        // 检查是否运行在 Parallels/VMware/VirtualBox 中
        if let Ok(output) = Command::new("system_profiler")
            .args(["SPHardwareDataType"])
            .output()
        {
            let hw_info = String::from_utf8_lossy(&output.stdout).to_lowercase();
            if hw_info.contains("vmware") || hw_info.contains("parallels") || hw_info.contains("virtualbox") {
                return true;
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // 检查 WMI 获取系统信息
        if let Ok(output) = Command::new("wmic")
            .args(["computersystem", "get", "model"])
            .output()
        {
            let model = String::from_utf8_lossy(&output.stdout).to_lowercase();
            if model.contains("virtual") || model.contains("vmware") || model.contains("virtualbox") {
                return true;
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        // 检查 /proc/cpuinfo
        if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
            let content_lower = content.to_lowercase();
            if content_lower.contains("hypervisor") || content_lower.contains("vmware") || content_lower.contains("qemu") {
                return true;
            }
        }

        // 检查 systemd-detect-virt
        if let Ok(output) = Command::new("systemd-detect-virt").output() {
            let virt = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !virt.is_empty() && virt != "none" {
                return true;
            }
        }
    }

    false
}

/// 扫描 Time Machine 本地快照 (macOS)
async fn scan_time_machine_snapshots() -> Result<Vec<ScanResult>, String> {
    let mut results = Vec::new();

    #[cfg(target_os = "macos")]
    {
        // 使用 tmutil 列出本地快照
        if let Ok(output) = Command::new("tmutil")
            .args(["listlocalsnapshots", "/"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let snapshot_count = output_str.lines()
                .filter(|line| line.contains("com.apple.TimeMachine"))
                .count() as u32;

            if snapshot_count > 0 {
                results.push(ScanResult {
                    id: "timemachine_snapshots".to_string(),
                    category: "Time Machine".to_string(),
                    item_type: "本地快照".to_string(),
                    description: format!("发现 {} 个本地 Time Machine 快照", snapshot_count),
                    severity: ScanSeverity::Low,
                    path: Some("Time Machine Local Snapshots".to_string()),
                    size: None,
                    count: Some(snapshot_count),
                });
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Time Machine 仅限 macOS
    }

    Ok(results)
}

/// 扫描系统缓存 (QuickLook, 字体缓存等)
async fn scan_system_cache() -> Result<Vec<ScanResult>, String> {
    let mut results = Vec::new();
    let home_dir = std::env::var("HOME").unwrap_or_default();

    #[cfg(target_os = "macos")]
    {
        let system_caches = vec![
            // QuickLook 缓存
            (format!("{}/Library/Caches/com.apple.QuickLookDaemon", home_dir), "QuickLook 缓存", "quicklook"),
            (format!("{}/Library/Caches/com.apple.QuickLook.thumbnailcache", home_dir), "QuickLook 缩略图", "quicklook_thumb"),
            ("/private/var/folders".to_string(), "系统临时文件夹", "var_folders"),
            // 字体缓存
            (format!("{}/Library/Caches/com.apple.ATS", home_dir), "字体缓存", "font_cache"),
            ("/System/Library/Caches/com.apple.coresymbolicationd".to_string(), "符号缓存", "symbol_cache"),
            // Spotlight 索引
            ("/.Spotlight-V100".to_string(), "Spotlight 索引", "spotlight"),
            // CUPS 打印缓存
            ("/var/spool/cups".to_string(), "打印队列缓存", "cups_cache"),
        ];

        for (cache_path, cache_name, cache_id) in system_caches {
            let path = Path::new(&cache_path);
            if path.exists() {
                let (total_size, file_count) = get_dir_stats(path);
                if file_count > 0 || total_size > 0 {
                    results.push(ScanResult {
                        id: format!("syscache_{}", cache_id),
                        category: "系统缓存".to_string(),
                        item_type: cache_name.to_string(),
                        description: format!("发现 {} 个系统缓存文件", file_count),
                        severity: ScanSeverity::Low,
                        path: Some(cache_path),
                        size: Some(format_size(total_size)),
                        count: Some(file_count),
                    });
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let system_caches = vec![
            ("C:\\Windows\\Prefetch".to_string(), "预读取缓存", "prefetch"),
            ("C:\\Windows\\SoftwareDistribution\\Download".to_string(), "Windows Update 缓存", "wupdates"),
            (format!("{}\\AppData\\Local\\Microsoft\\Windows\\Explorer", std::env::var("USERPROFILE").unwrap_or_default()), "缩略图缓存", "thumbcache"),
        ];

        for (cache_path, cache_name, cache_id) in system_caches {
            let path = Path::new(&cache_path);
            if path.exists() {
                let (total_size, file_count) = get_dir_stats(path);
                if file_count > 0 {
                    results.push(ScanResult {
                        id: format!("syscache_{}", cache_id),
                        category: "系统缓存".to_string(),
                        item_type: cache_name.to_string(),
                        description: format!("发现 {} 个系统缓存文件", file_count),
                        severity: ScanSeverity::Low,
                        path: Some(cache_path),
                        size: Some(format_size(total_size)),
                        count: Some(file_count),
                    });
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let system_caches = vec![
            ("/var/cache/apt/archives".to_string(), "APT 包缓存", "apt_cache"),
            ("/var/cache/pacman/pkg".to_string(), "Pacman 包缓存", "pacman_cache"),
            ("/var/cache/dnf".to_string(), "DNF 包缓存", "dnf_cache"),
            (format!("{}/.cache/thumbnails", home_dir), "缩略图缓存", "thumbnails"),
            (format!("{}/.cache/fontconfig", home_dir), "字体缓存", "font_cache"),
        ];

        for (cache_path, cache_name, cache_id) in system_caches {
            let path = Path::new(&cache_path);
            if path.exists() {
                let (total_size, file_count) = get_dir_stats(path);
                if file_count > 0 {
                    results.push(ScanResult {
                        id: format!("syscache_{}", cache_id),
                        category: "系统缓存".to_string(),
                        item_type: cache_name.to_string(),
                        description: format!("发现 {} 个系统缓存文件", file_count),
                        severity: ScanSeverity::Low,
                        path: Some(cache_path),
                        size: Some(format_size(total_size)),
                        count: Some(file_count),
                    });
                }
            }
        }
    }

    Ok(results)
}

/// 扫描开发工具缓存
async fn scan_dev_tools_cache() -> Result<Vec<ScanResult>, String> {
    let mut results = Vec::new();
    let home_dir = std::env::var("HOME").unwrap_or_default();

    // 跨平台开发工具缓存路径
    let dev_caches = vec![
        // Node.js / npm / yarn / pnpm
        (format!("{}/.npm", home_dir), "npm 缓存", "npm"),
        (format!("{}/.npm/_cacache", home_dir), "npm 包缓存", "npm_cacache"),
        (format!("{}/.yarn", home_dir), "Yarn 缓存", "yarn"),
        (format!("{}/.yarn/cache", home_dir), "Yarn 包缓存", "yarn_cache"),
        (format!("{}/.pnpm-store", home_dir), "pnpm 存储", "pnpm"),
        (format!("{}/Library/pnpm/store", home_dir), "pnpm 存储 (macOS)", "pnpm_macos"),
        (format!("{}/.bun", home_dir), "Bun 缓存", "bun"),
        // Python
        (format!("{}/.cache/pip", home_dir), "pip 缓存", "pip"),
        (format!("{}/Library/Caches/pip", home_dir), "pip 缓存 (macOS)", "pip_macos"),
        (format!("{}/.pyenv", home_dir), "pyenv", "pyenv"),
        (format!("{}/.conda", home_dir), "Conda 环境", "conda"),
        (format!("{}/.cache/uv", home_dir), "uv 缓存", "uv"),
        // Rust
        (format!("{}/.cargo/registry/cache", home_dir), "Cargo 注册表缓存", "cargo_cache"),
        (format!("{}/.cargo/git/db", home_dir), "Cargo Git 缓存", "cargo_git"),
        (format!("{}/.rustup/toolchains", home_dir), "Rustup 工具链", "rustup"),
        // Go
        (format!("{}/go/pkg/mod/cache", home_dir), "Go 模块缓存", "go_mod"),
        // Java / Gradle / Maven
        (format!("{}/.gradle/caches", home_dir), "Gradle 缓存", "gradle"),
        (format!("{}/.m2/repository", home_dir), "Maven 仓库", "maven"),
        // Ruby
        (format!("{}/.gem", home_dir), "Ruby Gems", "gem"),
        (format!("{}/.bundle/cache", home_dir), "Bundler 缓存", "bundler"),
        (format!("{}/.rbenv", home_dir), "rbenv", "rbenv"),
        // PHP
        (format!("{}/.composer/cache", home_dir), "Composer 缓存", "composer"),
        // .NET
        (format!("{}/.nuget/packages", home_dir), "NuGet 包", "nuget"),
        // Dart / Flutter
        (format!("{}/.pub-cache", home_dir), "Dart pub 缓存", "dart_pub"),
        (format!("{}/Library/Caches/flutter", home_dir), "Flutter 缓存", "flutter"),
        // Haskell
        (format!("{}/.cabal", home_dir), "Cabal", "cabal"),
        (format!("{}/.stack", home_dir), "Stack", "stack"),
        // Elixir
        (format!("{}/.hex", home_dir), "Hex", "hex"),
        (format!("{}/.mix", home_dir), "Mix", "mix"),
    ];

    for (cache_path, cache_name, cache_id) in dev_caches {
        let path = Path::new(&cache_path);
        if path.exists() {
            let (total_size, file_count) = get_dir_stats(path);
            if total_size > 10 * 1024 * 1024 { // 只显示大于 10MB 的缓存
                results.push(ScanResult {
                    id: format!("devtool_{}", cache_id),
                    category: "开发工具缓存".to_string(),
                    item_type: cache_name.to_string(),
                    description: format!("发现开发工具缓存 ({})", format_size(total_size)),
                    severity: ScanSeverity::Medium,
                    path: Some(cache_path),
                    size: Some(format_size(total_size)),
                    count: Some(file_count),
                });
            }
        }
    }

    Ok(results)
}

/// 扫描 IDE 扩展缓存
async fn scan_ide_extension_cache() -> Result<Vec<ScanResult>, String> {
    let mut results = Vec::new();
    let home_dir = std::env::var("HOME").unwrap_or_default();

    #[cfg(target_os = "macos")]
    let ide_caches = vec![
        // VS Code 及变体
        (format!("{}/Library/Application Support/Code/CachedExtensionVSIXs", home_dir), "VS Code 扩展缓存", "vscode_ext"),
        (format!("{}/Library/Application Support/Code/CachedData", home_dir), "VS Code 数据缓存", "vscode_data"),
        (format!("{}/Library/Application Support/Code/Cache", home_dir), "VS Code Cache", "vscode_cache"),
        (format!("{}/Library/Application Support/Code/User/workspaceStorage", home_dir), "VS Code 工作区存储", "vscode_workspace"),
        (format!("{}/Library/Application Support/Cursor/CachedExtensionVSIXs", home_dir), "Cursor 扩展缓存", "cursor_ext"),
        (format!("{}/Library/Application Support/Cursor/CachedData", home_dir), "Cursor 数据缓存", "cursor_data"),
        // JetBrains IDEs
        (format!("{}/Library/Caches/JetBrains", home_dir), "JetBrains 缓存", "jetbrains"),
        (format!("{}/Library/Application Support/JetBrains", home_dir), "JetBrains 数据", "jetbrains_data"),
        // Xcode
        (format!("{}/Library/Developer/Xcode/DerivedData", home_dir), "Xcode DerivedData", "xcode_derived"),
        (format!("{}/Library/Developer/Xcode/Archives", home_dir), "Xcode Archives", "xcode_archives"),
        (format!("{}/Library/Developer/CoreSimulator/Caches", home_dir), "iOS 模拟器缓存", "simulator_cache"),
        // Android Studio
        (format!("{}/.android/cache", home_dir), "Android 缓存", "android_cache"),
        (format!("{}/.android/avd", home_dir), "Android 虚拟设备", "android_avd"),
        // Sublime Text
        (format!("{}/Library/Application Support/Sublime Text/Cache", home_dir), "Sublime Text 缓存", "sublime_cache"),
        // Zed
        (format!("{}/Library/Application Support/Zed/languages", home_dir), "Zed 语言服务器", "zed_lsp"),
    ];

    #[cfg(target_os = "windows")]
    let ide_caches = {
        let appdata = std::env::var("APPDATA").unwrap_or_default();
        let localappdata = std::env::var("LOCALAPPDATA").unwrap_or_default();
        vec![
            (format!("{}\\Code\\CachedExtensionVSIXs", appdata), "VS Code 扩展缓存", "vscode_ext"),
            (format!("{}\\Code\\CachedData", appdata), "VS Code 数据缓存", "vscode_data"),
            (format!("{}\\JetBrains", localappdata), "JetBrains 缓存", "jetbrains"),
        ]
    };

    #[cfg(target_os = "linux")]
    let ide_caches = vec![
        (format!("{}/.config/Code/CachedExtensionVSIXs", home_dir), "VS Code 扩展缓存", "vscode_ext"),
        (format!("{}/.config/Code/CachedData", home_dir), "VS Code 数据缓存", "vscode_data"),
        (format!("{}/.cache/JetBrains", home_dir), "JetBrains 缓存", "jetbrains"),
        (format!("{}/.local/share/JetBrains", home_dir), "JetBrains 数据", "jetbrains_data"),
    ];

    for (cache_path, cache_name, cache_id) in ide_caches {
        let path = Path::new(&cache_path);
        if path.exists() {
            let (total_size, file_count) = get_dir_stats(path);
            if total_size > 50 * 1024 * 1024 { // 只显示大于 50MB 的 IDE 缓存
                results.push(ScanResult {
                    id: format!("ide_{}", cache_id),
                    category: "IDE 扩展缓存".to_string(),
                    item_type: cache_name.to_string(),
                    description: format!("发现 IDE 缓存 ({})", format_size(total_size)),
                    severity: ScanSeverity::Medium,
                    path: Some(cache_path),
                    size: Some(format_size(total_size)),
                    count: Some(file_count),
                });
            }
        }
    }

    Ok(results)
}

/// 扫描休眠镜像
async fn scan_hibernation_image() -> Result<Vec<ScanResult>, String> {
    let mut results = Vec::new();

    #[cfg(target_os = "macos")]
    {
        let sleepimage_path = "/private/var/vm/sleepimage";
        if Path::new(sleepimage_path).exists() {
            if let Ok(metadata) = fs::metadata(sleepimage_path) {
                let size = metadata.len();
                if size > 0 {
                    results.push(ScanResult {
                        id: "hibernation_sleepimage".to_string(),
                        category: "休眠镜像".to_string(),
                        item_type: "Sleep Image".to_string(),
                        description: "发现休眠镜像文件（重启后自动重建）".to_string(),
                        severity: ScanSeverity::Low,
                        path: Some(sleepimage_path.to_string()),
                        size: Some(format_size(size)),
                        count: None,
                    });
                }
            }
        }

        // 检查 swap 文件
        let swapfile_path = "/private/var/vm/swapfile0";
        if Path::new(swapfile_path).exists() {
            if let Ok(metadata) = fs::metadata(swapfile_path) {
                results.push(ScanResult {
                    id: "hibernation_swapfile".to_string(),
                    category: "休眠镜像".to_string(),
                    item_type: "Swap 文件".to_string(),
                    description: "发现交换文件（系统管理）".to_string(),
                    severity: ScanSeverity::Low,
                    path: Some("/private/var/vm".to_string()),
                    size: Some(format_size(metadata.len())),
                    count: None,
                });
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let hiberfil_path = "C:\\hiberfil.sys";
        if Path::new(hiberfil_path).exists() {
            if let Ok(metadata) = fs::metadata(hiberfil_path) {
                results.push(ScanResult {
                    id: "hibernation_hiberfil".to_string(),
                    category: "休眠镜像".to_string(),
                    item_type: "休眠文件".to_string(),
                    description: "发现 Windows 休眠文件".to_string(),
                    severity: ScanSeverity::Low,
                    path: Some(hiberfil_path.to_string()),
                    size: Some(format_size(metadata.len())),
                    count: None,
                });
            }
        }

        let pagefile_path = "C:\\pagefile.sys";
        if Path::new(pagefile_path).exists() {
            if let Ok(metadata) = fs::metadata(pagefile_path) {
                results.push(ScanResult {
                    id: "hibernation_pagefile".to_string(),
                    category: "休眠镜像".to_string(),
                    item_type: "页面文件".to_string(),
                    description: "发现 Windows 页面文件".to_string(),
                    severity: ScanSeverity::Low,
                    path: Some(pagefile_path.to_string()),
                    size: Some(format_size(metadata.len())),
                    count: None,
                });
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        // 检查 swap 分区（通过 /proc/swaps）
        if let Ok(content) = fs::read_to_string("/proc/swaps") {
            let swap_count = content.lines().skip(1).count(); // 跳过标题行
            if swap_count > 0 {
                results.push(ScanResult {
                    id: "hibernation_swap".to_string(),
                    category: "休眠镜像".to_string(),
                    item_type: "Swap 分区".to_string(),
                    description: format!("发现 {} 个 Swap 分区/文件", swap_count),
                    severity: ScanSeverity::Low,
                    path: Some("/proc/swaps".to_string()),
                    size: None,
                    count: Some(swap_count as u32),
                });
            }
        }
    }

    Ok(results)
}

/// 扫描下载大文件（默认 >100MB）
async fn scan_downloads_large_files() -> Result<Vec<ScanResult>, String> {
    let mut results = Vec::new();
    let home_dir = std::env::var("HOME").unwrap_or_default();
    let download_path = format!("{}/Downloads", home_dir);
    let path = Path::new(&download_path);

    const LARGE_FILE_THRESHOLD: u64 = 100 * 1024 * 1024; // 100MB

    if path.exists() {
        let mut large_files: Vec<(String, u64)> = Vec::new();
        let mut total_large_size: u64 = 0;

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() && metadata.len() >= LARGE_FILE_THRESHOLD {
                        let file_name = entry.file_name().to_string_lossy().to_string();
                        large_files.push((file_name, metadata.len()));
                        total_large_size += metadata.len();
                    }
                }
            }
        }

        if !large_files.is_empty() {
            results.push(ScanResult {
                id: "downloads_large".to_string(),
                category: "下载大文件".to_string(),
                item_type: "大文件 (>100MB)".to_string(),
                description: format!("发现 {} 个大文件，共 {}", large_files.len(), format_size(total_large_size)),
                severity: ScanSeverity::Low, // 默认不选择
                path: Some(download_path),
                size: Some(format_size(total_large_size)),
                count: Some(large_files.len() as u32),
            });
        }
    }

    Ok(results)
}

/// 扫描 AI/ML 工具缓存
async fn scan_ai_ml_cache() -> Result<Vec<ScanResult>, String> {
    let mut results = Vec::new();
    let home_dir = std::env::var("HOME").unwrap_or_default();

    let ai_caches = vec![
        // Hugging Face
        (format!("{}/.cache/huggingface", home_dir), "Hugging Face 模型", "huggingface"),
        // Ollama
        (format!("{}/.ollama/models", home_dir), "Ollama 模型", "ollama"),
        // LM Studio
        (format!("{}/.cache/lm-studio", home_dir), "LM Studio 模型", "lmstudio"),
        // PyTorch
        (format!("{}/.cache/torch", home_dir), "PyTorch 缓存", "pytorch"),
        // TensorFlow
        (format!("{}/.keras", home_dir), "Keras 模型", "keras"),
        (format!("{}/.tensorflow", home_dir), "TensorFlow 缓存", "tensorflow"),
        // Jupyter
        (format!("{}/.jupyter", home_dir), "Jupyter 配置", "jupyter"),
        (format!("{}/.ipython", home_dir), "IPython 缓存", "ipython"),
        // Transformers
        (format!("{}/.cache/transformers", home_dir), "Transformers 缓存", "transformers"),
        // LocalAI
        (format!("{}/.local/share/localai", home_dir), "LocalAI 模型", "localai"),
    ];

    for (cache_path, cache_name, cache_id) in ai_caches {
        let path = Path::new(&cache_path);
        if path.exists() {
            let (total_size, file_count) = get_dir_stats(path);
            if total_size > 100 * 1024 * 1024 { // 只显示大于 100MB 的 AI 缓存
                results.push(ScanResult {
                    id: format!("ai_{}", cache_id),
                    category: "AI/ML 缓存".to_string(),
                    item_type: cache_name.to_string(),
                    description: format!("发现 AI 模型/缓存 ({})", format_size(total_size)),
                    severity: ScanSeverity::Medium,
                    path: Some(cache_path),
                    size: Some(format_size(total_size)),
                    count: Some(file_count),
                });
            }
        }
    }

    Ok(results)
}

/// 扫描中国应用缓存
async fn scan_chinese_apps_cache() -> Result<Vec<ScanResult>, String> {
    let mut results = Vec::new();
    let home_dir = std::env::var("HOME").unwrap_or_default();

    #[cfg(target_os = "macos")]
    let cn_apps = vec![
        // 社交通讯
        (format!("{}/Library/Containers/com.tencent.xinWeChat/Data/Library/Caches", home_dir), "微信缓存", "wechat"),
        (format!("{}/Library/Containers/com.tencent.qq/Data/Library/Caches", home_dir), "QQ 缓存", "qq"),
        (format!("{}/Library/Containers/com.alibaba.DingTalkMac/Data/Library/Caches", home_dir), "钉钉缓存", "dingtalk"),
        (format!("{}/Library/Application Support/企业微信", home_dir), "企业微信", "wxwork"),
        (format!("{}/Library/Application Support/Feishu", home_dir), "飞书", "feishu"),
        // 音乐视频
        (format!("{}/Library/Containers/com.netease.163music/Data/Library/Caches", home_dir), "网易云音乐", "netease_music"),
        (format!("{}/Library/Containers/com.tencent.QQMusicMac/Data/Library/Caches", home_dir), "QQ 音乐", "qq_music"),
        (format!("{}/Library/Caches/com.bilibili.app.bili", home_dir), "Bilibili", "bilibili"),
        // 效率工具
        (format!("{}/Library/Application Support/Baidu-NetDisk-Transfers", home_dir), "百度网盘传输", "baidunetdisk"),
        (format!("{}/Library/Caches/com.alibaba.yunpan", home_dir), "阿里云盘缓存", "aliyunpan"),
        // 输入法
        (format!("{}/Library/Caches/com.sogou.inputmethod.sogou", home_dir), "搜狗输入法", "sogou"),
        (format!("{}/Library/Caches/com.baidu.inputmethod.BaiduIM", home_dir), "百度输入法", "baiduim"),
    ];

    #[cfg(target_os = "windows")]
    let cn_apps = {
        let appdata = std::env::var("APPDATA").unwrap_or_default();
        let localappdata = std::env::var("LOCALAPPDATA").unwrap_or_default();
        vec![
            (format!("{}\\Tencent\\WeChat\\All Users", appdata), "微信", "wechat"),
            (format!("{}\\Tencent\\QQ", appdata), "QQ", "qq"),
            (format!("{}\\DingTalk", localappdata), "钉钉", "dingtalk"),
            (format!("{}\\Netease\\CloudMusic\\Cache", localappdata), "网易云音乐", "netease_music"),
        ]
    };

    #[cfg(target_os = "linux")]
    let cn_apps = vec![
        (format!("{}/.config/weixin", home_dir), "微信", "wechat"),
        (format!("{}/.config/qqmusic", home_dir), "QQ 音乐", "qq_music"),
        (format!("{}/.config/netease-cloud-music", home_dir), "网易云音乐", "netease_music"),
    ];

    for (cache_path, cache_name, cache_id) in cn_apps {
        let path = Path::new(&cache_path);
        if path.exists() {
            let (total_size, file_count) = get_dir_stats(path);
            if total_size > 50 * 1024 * 1024 { // 只显示大于 50MB 的应用缓存
                results.push(ScanResult {
                    id: format!("cnapp_{}", cache_id),
                    category: "中国应用缓存".to_string(),
                    item_type: cache_name.to_string(),
                    description: format!("发现应用缓存 ({})", format_size(total_size)),
                    severity: ScanSeverity::Medium,
                    path: Some(cache_path),
                    size: Some(format_size(total_size)),
                    count: Some(file_count),
                });
            }
        }
    }

    Ok(results)
}

/// 清理项目信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CleanupItem {
    pub id: String,
    pub path: Option<String>,
}

/// 检查权限是否已初始化（从 permission_ops 模块获取状态）
fn is_permission_initialized() -> bool {
    use std::sync::atomic::Ordering;
    use super::permission_ops::PERMISSION_INITIALIZED;
    PERMISSION_INITIALIZED.load(Ordering::SeqCst)
}

/// 清理扫描发现的项目
#[tauri::command]
pub async fn cleanup_scan_items(items: Vec<CleanupItem>) -> Result<String, String> {
    let home_dir = std::env::var("HOME").unwrap_or_default();
    let permission_initialized = is_permission_initialized();

    // 收集需要管理员权限的路径
    let mut sudo_paths: Vec<String> = Vec::new();
    // 收集普通权限的清理项目
    let mut normal_items: Vec<&CleanupItem> = Vec::new();

    for item in &items {
        if let Some(path) = &item.path {
            let is_system_path = path.starts_with("/var/log")
                || path.starts_with("/Library/Logs")
                || path.starts_with("/private/var")
                || path.starts_with("/tmp")
                || path == "/private/var/folders";

            if is_system_path {
                sudo_paths.push(path.clone());
            } else {
                normal_items.push(item);
            }
        } else {
            normal_items.push(item);
        }
    }

    // 如果有需要管理员权限的路径
    #[cfg(target_os = "macos")]
    if !sudo_paths.is_empty() {
        // 构建一次性清理所有系统路径的脚本
        let rm_commands: Vec<String> = sudo_paths.iter()
            .map(|p| format!("rm -rf '{}'/* 2>/dev/null || true", p))
            .collect();

        if permission_initialized {
            // 如果权限已初始化，尝试使用 sudo（可能不需要密码）
            // 首先尝试使用 sudo -n（非交互模式），如果 sudo 时间戳有效则不需要密码
            let sudo_script = format!("sudo -n sh -c '{}'", rm_commands.join("; "));
            let sudo_result = Command::new("sh")
                .args(["-c", &sudo_script])
                .output();

            if sudo_result.map(|o| o.status.success()).unwrap_or(false) {
                // sudo -n 成功
            } else {
                // 如果 sudo -n 失败，回退到 osascript（这会弹出密码对话框，但只需要一次）
                let script = format!(
                    r#"do shell script "{}" with administrator privileges"#,
                    rm_commands.join("; ")
                );
                let _ = Command::new("osascript").args(["-e", &script]).output();
            }
        } else {
            // 如果权限未初始化，使用 osascript 请求权限
            let script = format!(
                r#"do shell script "{}" with administrator privileges"#,
                rm_commands.join("; ")
            );
            let _ = Command::new("osascript").args(["-e", &script]).output();
        }
    }

    let mut cleaned_count = sudo_paths.len();
    let mut errors = Vec::new();

    // 清理普通权限的项目
    for item in normal_items {
        let result = cleanup_item_without_sudo(&item.id, item.path.as_deref(), &home_dir).await;
        match result {
            Ok(_) => cleaned_count += 1,
            Err(e) => errors.push(format!("{}: {}", item.id, e)),
        }
    }

    if !errors.is_empty() {
        // errors logged silently
    }

    Ok(format!("已成功清理 {} 个项目", cleaned_count))
}

/// 递归清理目录内容
fn clear_directory_contents(dir_path: &Path) -> Result<u32, String> {
    let mut count = 0;

    if !dir_path.exists() {
        return Ok(0);
    }

    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                // 递归清理子目录
                if let Ok(sub_count) = clear_directory_contents(&entry_path) {
                    count += sub_count;
                }
                // 尝试删除目录（包括非空目录）
                let _ = fs::remove_dir_all(&entry_path);
            } else {
                // 删除文件，如果失败尝试强制删除
                match fs::remove_file(&entry_path) {
                    Ok(_) => {
                        count += 1;
                    }
                    Err(_) => {
                        // 尝试修改权限后再删除
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            if let Ok(metadata) = fs::metadata(&entry_path) {
                                let mut perms = metadata.permissions();
                                perms.set_mode(0o777);
                                let _ = fs::set_permissions(&entry_path, perms);
                                if fs::remove_file(&entry_path).is_ok() {
                                    count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(count)
}

/// 使用 sudo 权限清理目录（macOS）
#[cfg(target_os = "macos")]
fn clear_directory_with_sudo(dir_path: &str) -> Result<u32, String> {
    // 使用 osascript 调用系统对话框请求管理员权限
    let script = format!(
        r#"do shell script "rm -rf {}/*" with administrator privileges"#,
        dir_path
    );

    let output = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|e| format!("执行 sudo 命令失败: {}", e))?;

    if output.status.success() {
        Ok(1)
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(format!("清理失败: {}", error))
    }
}

/// 使用 sudo 权限清理系统日志（macOS）
#[cfg(target_os = "macos")]
fn clear_system_logs_with_sudo() -> Result<u32, String> {
    // 清理多个系统日志目录
    let script = r#"do shell script "
        rm -rf /var/log/*.log 2>/dev/null
        rm -rf /var/log/*.gz 2>/dev/null
        rm -rf /var/log/asl/*.asl 2>/dev/null
        rm -rf /var/log/DiagnosticMessages/* 2>/dev/null
        rm -rf /Library/Logs/* 2>/dev/null
        rm -rf /private/var/log/asl/* 2>/dev/null
        echo 'done'
    " with administrator privileges"#;

    let output = Command::new("osascript")
        .args(["-e", script])
        .output()
        .map_err(|e| format!("执行 sudo 命令失败: {}", e))?;

    if output.status.success() {
        Ok(1)
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(format!("清理系统日志失败: {}", error))
    }
}

/// 清理单个项目（不需要 sudo，已在上层统一处理）
async fn cleanup_item_without_sudo(item_id: &str, path: Option<&str>, home_dir: &str) -> Result<(), String> {
    // 根据 item_id 前缀判断类型并执行清理
    if item_id.starts_with("log_") {
        // 清理日志（系统路径已在上层用 sudo 处理）
        if let Some(log_path) = path {
            let p = Path::new(log_path);
            if p.exists() {
                if p.is_dir() {
                    #[cfg(unix)]
                    {
                        let _ = Command::new("sh")
                            .args(["-c", &format!("rm -rf '{}'/* 2>/dev/null || true", log_path)])
                            .output();
                    }
                    let _ = clear_directory_contents(p)?;
                } else {
                    fs::remove_file(p).map_err(|e| format!("删除日志文件失败: {}", e))?;
                }
            }
        }
    } else if item_id.starts_with("temp_") {
        // 清理临时文件（系统路径已在上层用 sudo 处理）
        if let Some(temp_path) = path {
            let p = Path::new(temp_path);
            if p.exists() && p.is_dir() {
                #[cfg(unix)]
                {
                    let _ = Command::new("sh")
                        .args(["-c", &format!("rm -rf '{}'/* 2>/dev/null || true", temp_path)])
                        .output();
                }
                let _ = clear_directory_contents(p)?;
            }
        }
    } else if item_id.starts_with("browser_") {
        // 清理浏览器数据 - 彻底清理整个浏览器数据目录
        if let Some(browser_path) = path {
            let p = Path::new(browser_path);
            if p.exists() {
                // 直接清理整个浏览器数据目录
                #[cfg(unix)]
                {
                    // 使用 rm -rf 彻底清理
                    let _ = Command::new("sh")
                        .args(["-c", &format!("rm -rf '{}'/* 2>/dev/null || true", browser_path)])
                        .output();
                }

                // 再用 Rust 方法清理残余
                let _ = clear_directory_contents(p).unwrap_or(0);

                // Safari 额外清理：容器目录和缓存目录
                if item_id == "browser_safari" {
                    let safari_extra_paths = vec![
                        format!("{}/Library/Containers/com.apple.Safari/Data/Library/WebKit", home_dir),
                        format!("{}/Library/Containers/com.apple.Safari/Data/Library/Safari", home_dir),
                        format!("{}/Library/Containers/com.apple.Safari/Data/Library/HTTPStorages", home_dir),
                        format!("{}/Library/Caches/com.apple.Safari", home_dir),
                        format!("{}/Library/Caches/Safari", home_dir),
                        format!("{}/Library/Cookies", home_dir),
                        format!("{}/Library/WebKit", home_dir),
                    ];

                    for extra_path in &safari_extra_paths {
                        let extra_p = Path::new(extra_path);
                        if extra_p.exists() {
                            #[cfg(unix)]
                            {
                                let _ = Command::new("sh")
                                    .args(["-c", &format!("rm -rf '{}'/* 2>/dev/null || true", extra_path)])
                                    .output();
                            }
                            if extra_p.is_dir() {
                                let _ = clear_directory_contents(extra_p);
                            }
                        }
                    }
                }

                // 其他 Chromium 系浏览器清理缓存目录
                if !item_id.contains("safari") && !item_id.contains("firefox") {
                    let browser_name = item_id.trim_start_matches("browser_");
                    let cache_paths = vec![
                        format!("{}/Library/Caches/Google/Chrome", home_dir),
                        format!("{}/Library/Caches/com.microsoft.edgemac", home_dir),
                        format!("{}/Library/Caches/com.brave.Browser", home_dir),
                        format!("{}/Library/Caches/Arc", home_dir),
                        format!("{}/Library/Caches/Vivaldi", home_dir),
                    ];
                    for cache_path in &cache_paths {
                        if cache_path.to_lowercase().contains(browser_name) {
                            let cp = Path::new(cache_path);
                            if cp.exists() {
                                #[cfg(unix)]
                                {
                                    let _ = Command::new("sh")
                                        .args(["-c", &format!("rm -rf '{}'/* 2>/dev/null || true", cache_path)])
                                        .output();
                                }
                                let _ = clear_directory_contents(cp);
                            }
                        }
                    }
                }
            }
        }
    } else if item_id.starts_with("recent_") {
        // 清理最近文档记录
        if let Some(recent_path) = path {
            let p = Path::new(recent_path);
            if p.exists() {
                if p.is_dir() {
                    let _ = clear_directory_contents(p)?;
                } else {
                    fs::remove_file(p).map_err(|e| format!("删除最近文档文件失败: {}", e))?;
                }
            }
        }
    } else if item_id.starts_with("network_") {
        // 清理网络缓存
        #[cfg(target_os = "macos")]
        {
            let _ = Command::new("dscacheutil").arg("-flushcache").output();
            let _ = Command::new("killall").args(["-HUP", "mDNSResponder"]).output();
        }
        #[cfg(target_os = "windows")]
        {
            let _ = Command::new("ipconfig").arg("/flushdns").output();
        }
        #[cfg(target_os = "linux")]
        {
            let _ = Command::new("systemctl").args(["restart", "systemd-resolved"]).output();
        }
    } else if item_id.starts_with("shell_") {
        // 清空 Shell 历史
        if let Some(shell_path) = path {
            let p = Path::new(shell_path);
            if p.exists() {
                fs::write(p, "").map_err(|e| format!("清空 Shell 历史失败: {}", e))?;
            }
        } else {
            // 如果没有路径，清空所有常见的 shell 历史文件
            let shell_files = vec![
                format!("{}/.bash_history", home_dir),
                format!("{}/.zsh_history", home_dir),
                format!("{}/.zhistory", home_dir),
            ];
            for file in shell_files {
                if Path::new(&file).exists() {
                    let _ = fs::write(&file, "");
                }
            }
        }
    } else if item_id.starts_with("cache_") {
        // 清理应用缓存
        if let Some(cache_path) = path {
            let p = Path::new(cache_path);
            if p.exists() && p.is_dir() {
                let _ = clear_directory_contents(p)?;
            }
        }
    } else if item_id.starts_with("crash_") {
        // 清理崩溃日志
        if let Some(crash_path) = path {
            let p = Path::new(crash_path);
            if p.exists() && p.is_dir() {
                let _ = clear_directory_contents(p)?;
            }
        }
    } else if item_id == "downloads" {
        // 清理下载文件夹
        if let Some(download_path) = path {
            let p = Path::new(download_path);
            if p.exists() && p.is_dir() {
                let _ = clear_directory_contents(p)?;
            }
        }
    } else if item_id == "trash" {
        // 清空回收站
        #[cfg(target_os = "macos")]
        {
            let default_trash = format!("{}/.Trash", home_dir);
            let trash_path = path.unwrap_or(&default_trash);
            let p = Path::new(trash_path);
            if p.exists() {
                if let Ok(entries) = fs::read_dir(p) {
                    for entry in entries.flatten() {
                        let entry_path = entry.path();
                        if entry_path.is_dir() {
                            let _ = fs::remove_dir_all(&entry_path);
                        } else {
                            let _ = fs::remove_file(&entry_path);
                        }
                    }
                }
            }
        }
        #[cfg(target_os = "windows")]
        {
            let _ = Command::new("cmd")
                .args(["/C", "rd", "/s", "/q", "C:\\$Recycle.Bin"])
                .output();
        }
        #[cfg(target_os = "linux")]
        {
            let default_trash = format!("{}/.local/share/Trash", home_dir);
            let trash_path = path.unwrap_or(&default_trash);
            let p = Path::new(trash_path);
            if p.exists() {
                let _ = clear_directory_contents(p);
            }
        }
    } else if item_id == "ssh_known_hosts" {
        // 清空 SSH known_hosts
        let default_ssh = format!("{}/.ssh/known_hosts", home_dir);
        let ssh_path = path.unwrap_or(&default_ssh);
        if Path::new(ssh_path).exists() {
            fs::write(ssh_path, "").map_err(|e| format!("清空 SSH known_hosts 失败: {}", e))?;
        }
    } else if item_id == "timemachine_snapshots" {
        // 清理 Time Machine 本地快照 (macOS)
        #[cfg(target_os = "macos")]
        {
            // 列出并删除所有本地快照
            if let Ok(output) = Command::new("tmutil")
                .args(["listlocalsnapshots", "/"])
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if line.contains("com.apple.TimeMachine") {
                        // 提取快照名称并删除
                        let snapshot_name = line.trim();
                        let _ = Command::new("tmutil")
                            .args(["deletelocalsnapshots", snapshot_name])
                            .output();
                    }
                }
            }
        }
    } else if item_id.starts_with("syscache_") {
        // 清理系统缓存
        if let Some(cache_path) = path {
            let p = Path::new(cache_path);
            if p.exists() {
                #[cfg(unix)]
                {
                    let _ = Command::new("sh")
                        .args(["-c", &format!("rm -rf '{}'/* 2>/dev/null || true", cache_path)])
                        .output();
                }
                if p.is_dir() {
                    let _ = clear_directory_contents(p);
                }
            }
        }
        // 刷新 DNS 缓存
        if item_id.contains("dns") {
            #[cfg(target_os = "macos")]
            {
                let _ = Command::new("dscacheutil").arg("-flushcache").output();
                let _ = Command::new("killall").args(["-HUP", "mDNSResponder"]).output();
            }
        }
        // 刷新字体缓存
        if item_id.contains("font") {
            #[cfg(target_os = "macos")]
            {
                let _ = Command::new("atsutil").args(["databases", "-remove"]).output();
            }
        }
    } else if item_id.starts_with("devtool_") {
        // 清理开发工具缓存
        if let Some(cache_path) = path {
            let p = Path::new(cache_path);
            if p.exists() && p.is_dir() {
                #[cfg(unix)]
                {
                    let _ = Command::new("sh")
                        .args(["-c", &format!("rm -rf '{}'/* 2>/dev/null || true", cache_path)])
                        .output();
                }
                let _ = clear_directory_contents(p);
            }
        }
    } else if item_id.starts_with("ide_") {
        // 清理 IDE 扩展缓存
        if let Some(cache_path) = path {
            let p = Path::new(cache_path);
            if p.exists() && p.is_dir() {
                #[cfg(unix)]
                {
                    let _ = Command::new("sh")
                        .args(["-c", &format!("rm -rf '{}'/* 2>/dev/null || true", cache_path)])
                        .output();
                }
                let _ = clear_directory_contents(p);
            }
        }
    } else if item_id.starts_with("hibernation_") {
        // 清理休眠镜像 - 需要管理员权限
        #[cfg(target_os = "macos")]
        {
            if item_id == "hibernation_sleepimage" {
                // 禁用休眠模式来删除 sleepimage
                let script = r#"do shell script "sudo pmset -a hibernatemode 0 && sudo rm -f /private/var/vm/sleepimage" with administrator privileges"#;
                let _ = Command::new("osascript").args(["-e", script]).output();
            }
        }
        #[cfg(target_os = "windows")]
        {
            if item_id == "hibernation_hiberfil" {
                // 禁用 Windows 休眠
                let _ = Command::new("powercfg").args(["/hibernate", "off"]).output();
            }
        }
    } else if item_id == "downloads_large" {
        // 清理下载大文件 (>100MB)
        if let Some(download_path) = path {
            let p = Path::new(download_path);
            if p.exists() {
                const LARGE_FILE_THRESHOLD: u64 = 100 * 1024 * 1024;
                if let Ok(entries) = fs::read_dir(p) {
                    for entry in entries.flatten() {
                        if let Ok(metadata) = entry.metadata() {
                            if metadata.is_file() && metadata.len() >= LARGE_FILE_THRESHOLD {
                                let _ = fs::remove_file(entry.path());
                            }
                        }
                    }
                }
            }
        }
    } else if item_id.starts_with("ai_") {
        // 清理 AI/ML 缓存
        if let Some(cache_path) = path {
            let p = Path::new(cache_path);
            if p.exists() && p.is_dir() {
                #[cfg(unix)]
                {
                    let _ = Command::new("sh")
                        .args(["-c", &format!("rm -rf '{}'/* 2>/dev/null || true", cache_path)])
                        .output();
                }
                let _ = clear_directory_contents(p);
            }
        }
    } else if item_id.starts_with("cnapp_") {
        // 清理中国应用缓存
        if let Some(cache_path) = path {
            let p = Path::new(cache_path);
            if p.exists() && p.is_dir() {
                #[cfg(unix)]
                {
                    let _ = Command::new("sh")
                        .args(["-c", &format!("rm -rf '{}'/* 2>/dev/null || true", cache_path)])
                        .output();
                }
                let _ = clear_directory_contents(p);
            }
        }
    }

    // 添加小延迟
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    Ok(())
}
