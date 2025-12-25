use crate::modules::log_cleaner;
use crate::commands::permission_ops::require_admin_for_operation;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogInfo {
    pub log_type: String,
    pub label: String,
    pub description: String,
    pub size: u64,           // 实际大小（字节）
    pub size_display: String, // 显示大小
    pub file_count: usize,    // 文件数量
    pub accessible: bool,     // 是否可访问
    pub category: String,     // 分类
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogScanResult {
    pub logs: Vec<LogInfo>,
    pub total_size: u64,
    pub total_files: usize,
    pub needs_permission: bool,
    pub permission_guide: String,
}

/// 获取目录大小和文件数量
fn get_directory_stats(path: &str) -> (u64, usize) {
    let path = Path::new(path);
    if !path.exists() {
        return (0, 0);
    }

    let mut total_size = 0u64;
    let mut file_count = 0usize;

    fn scan_dir(dir: &Path, size: &mut u64, count: &mut usize) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    scan_dir(&path, size, count);
                } else if let Ok(metadata) = fs::metadata(&path) {
                    *size += metadata.len();
                    *count += 1;
                }
            }
        }
    }

    if path.is_dir() {
        scan_dir(path, &mut total_size, &mut file_count);
    } else if let Ok(metadata) = fs::metadata(path) {
        total_size = metadata.len();
        file_count = 1;
    }

    (total_size, file_count)
}

/// 扫描多个路径并汇总
fn scan_paths(paths: &[String]) -> (u64, usize) {
    let mut total_size = 0u64;
    let mut total_count = 0usize;

    for path in paths {
        if Path::new(path).exists() {
            let (size, count) = get_directory_stats(path);
            total_size += size;
            total_count += count;
        }
    }

    (total_size, total_count)
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

/// 扫描系统日志信息
#[tauri::command]
pub async fn scan_system_logs() -> Result<LogScanResult, String> {
    let mut logs = Vec::new();
    let mut total_size = 0u64;
    let mut total_files = 0usize;

    #[cfg(target_os = "macos")]
    {
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/Users".to_string());

        // ========== 系统日志类别 ==========

        // 1. 系统日志
        let syslog_paths = vec![
            format!("{}/Library/Logs", home_dir),
            "/var/log".to_string(),
            "/Library/Logs".to_string(),
        ];
        let (syslog_size, syslog_count) = scan_paths(&syslog_paths);
        logs.push(LogInfo {
            log_type: "syslog".to_string(),
            label: "系统日志".to_string(),
            description: "系统和应用程序运行日志".to_string(),
            size: syslog_size,
            size_display: format_size(syslog_size),
            file_count: syslog_count,
            accessible: syslog_count > 0,
            category: "系统".to_string(),
        });
        total_size += syslog_size;
        total_files += syslog_count;

        // 2. 应用程序缓存
        let cache_paths = vec![
            format!("{}/Library/Caches", home_dir),
        ];
        let (cache_size, cache_count) = scan_paths(&cache_paths);
        logs.push(LogInfo {
            log_type: "app_cache".to_string(),
            label: "应用缓存".to_string(),
            description: "所有应用程序的缓存文件".to_string(),
            size: cache_size,
            size_display: format_size(cache_size),
            file_count: cache_count,
            accessible: cache_count > 0,
            category: "系统".to_string(),
        });
        total_size += cache_size;
        total_files += cache_count;

        // 3. 崩溃日志/诊断报告
        let crash_paths = vec![
            format!("{}/Library/Logs/DiagnosticReports", home_dir),
            format!("{}/Library/Logs/CrashReporter", home_dir),
            "/Library/Logs/DiagnosticReports".to_string(),
        ];
        let (crash_size, crash_count) = scan_paths(&crash_paths);
        logs.push(LogInfo {
            log_type: "crash_logs".to_string(),
            label: "崩溃日志".to_string(),
            description: "应用崩溃报告和诊断数据".to_string(),
            size: crash_size,
            size_display: format_size(crash_size),
            file_count: crash_count,
            accessible: crash_count > 0,
            category: "系统".to_string(),
        });
        total_size += crash_size;
        total_files += crash_count;

        // 4. 最近使用记录
        let recent_paths = vec![
            format!("{}/Library/Application Support/com.apple.sharedfilelist", home_dir),
            format!("{}/Library/Preferences/com.apple.recentitems.plist", home_dir),
            format!("{}/Library/Application Support/com.apple.spotlight.Shortcuts", home_dir),
        ];
        let (recent_size, recent_count) = scan_paths(&recent_paths);
        logs.push(LogInfo {
            log_type: "recent_items".to_string(),
            label: "最近使用".to_string(),
            description: "最近打开的文件和应用记录".to_string(),
            size: recent_size,
            size_display: format_size(recent_size),
            file_count: recent_count,
            accessible: recent_count > 0,
            category: "系统".to_string(),
        });
        total_size += recent_size;
        total_files += recent_count;

        // 5. 安装日志
        let install_paths = vec![
            "/var/log/install.log".to_string(),
            format!("{}/Library/Logs/Install.log", home_dir),
            "/Library/Receipts".to_string(),
            format!("{}/Library/Receipts", home_dir),
        ];
        let (install_size, install_count) = scan_paths(&install_paths);
        logs.push(LogInfo {
            log_type: "install_logs".to_string(),
            label: "安装日志".to_string(),
            description: "软件安装和更新记录".to_string(),
            size: install_size,
            size_display: format_size(install_size),
            file_count: install_count,
            accessible: install_count > 0,
            category: "系统".to_string(),
        });
        total_size += install_size;
        total_files += install_count;

        // ========== 浏览器类别 ==========

        // 6. Safari
        let safari_paths = vec![
            format!("{}/Library/Safari", home_dir),
            format!("{}/Library/Caches/com.apple.Safari", home_dir),
            format!("{}/Library/Cookies", home_dir),
            format!("{}/Library/WebKit", home_dir),
        ];
        let (safari_size, safari_count) = scan_paths(&safari_paths);
        logs.push(LogInfo {
            log_type: "safari".to_string(),
            label: "Safari".to_string(),
            description: "浏览历史、缓存、Cookie".to_string(),
            size: safari_size,
            size_display: format_size(safari_size),
            file_count: safari_count,
            accessible: safari_count > 0,
            category: "浏览器".to_string(),
        });
        total_size += safari_size;
        total_files += safari_count;

        // 7. Chrome
        let chrome_paths = vec![
            format!("{}/Library/Application Support/Google/Chrome", home_dir),
            format!("{}/Library/Caches/Google/Chrome", home_dir),
        ];
        let (chrome_size, chrome_count) = scan_paths(&chrome_paths);
        logs.push(LogInfo {
            log_type: "chrome".to_string(),
            label: "Chrome".to_string(),
            description: "浏览历史、缓存、Cookie、密码".to_string(),
            size: chrome_size,
            size_display: format_size(chrome_size),
            file_count: chrome_count,
            accessible: chrome_count > 0,
            category: "浏览器".to_string(),
        });
        total_size += chrome_size;
        total_files += chrome_count;

        // 8. Firefox
        let firefox_paths = vec![
            format!("{}/Library/Application Support/Firefox", home_dir),
            format!("{}/Library/Caches/Firefox", home_dir),
        ];
        let (firefox_size, firefox_count) = scan_paths(&firefox_paths);
        logs.push(LogInfo {
            log_type: "firefox".to_string(),
            label: "Firefox".to_string(),
            description: "浏览历史、缓存、Cookie".to_string(),
            size: firefox_size,
            size_display: format_size(firefox_size),
            file_count: firefox_count,
            accessible: firefox_count > 0,
            category: "浏览器".to_string(),
        });
        total_size += firefox_size;
        total_files += firefox_count;

        // 9. Edge
        let edge_paths = vec![
            format!("{}/Library/Application Support/Microsoft Edge", home_dir),
            format!("{}/Library/Caches/Microsoft Edge", home_dir),
        ];
        let (edge_size, edge_count) = scan_paths(&edge_paths);
        logs.push(LogInfo {
            log_type: "edge".to_string(),
            label: "Edge".to_string(),
            description: "浏览历史、缓存、Cookie".to_string(),
            size: edge_size,
            size_display: format_size(edge_size),
            file_count: edge_count,
            accessible: edge_count > 0,
            category: "浏览器".to_string(),
        });
        total_size += edge_size;
        total_files += edge_count;

        // ========== 开发工具类别 ==========

        // 10. VS Code
        let vscode_paths = vec![
            format!("{}/Library/Application Support/Code", home_dir),
            format!("{}/Library/Caches/com.microsoft.VSCode", home_dir),
            format!("{}/.vscode", home_dir),
        ];
        let (vscode_size, vscode_count) = scan_paths(&vscode_paths);
        logs.push(LogInfo {
            log_type: "vscode".to_string(),
            label: "VS Code".to_string(),
            description: "编辑器缓存、日志、工作区记录".to_string(),
            size: vscode_size,
            size_display: format_size(vscode_size),
            file_count: vscode_count,
            accessible: vscode_count > 0,
            category: "开发工具".to_string(),
        });
        total_size += vscode_size;
        total_files += vscode_count;

        // 11. Xcode
        let xcode_paths = vec![
            format!("{}/Library/Developer", home_dir),
            format!("{}/Library/Caches/com.apple.dt.Xcode", home_dir),
            "~/Library/Developer/Xcode/DerivedData".to_string(),
        ];
        let (xcode_size, xcode_count) = scan_paths(&xcode_paths);
        logs.push(LogInfo {
            log_type: "xcode".to_string(),
            label: "Xcode".to_string(),
            description: "构建缓存、模拟器数据、日志".to_string(),
            size: xcode_size,
            size_display: format_size(xcode_size),
            file_count: xcode_count,
            accessible: xcode_count > 0,
            category: "开发工具".to_string(),
        });
        total_size += xcode_size;
        total_files += xcode_count;

        // 12. JetBrains IDEs (IntelliJ, PyCharm, WebStorm等)
        let jetbrains_paths = vec![
            format!("{}/Library/Application Support/JetBrains", home_dir),
            format!("{}/Library/Caches/JetBrains", home_dir),
            format!("{}/Library/Logs/JetBrains", home_dir),
        ];
        let (jetbrains_size, jetbrains_count) = scan_paths(&jetbrains_paths);
        logs.push(LogInfo {
            log_type: "jetbrains".to_string(),
            label: "JetBrains".to_string(),
            description: "IntelliJ/PyCharm/WebStorm 等 IDE 数据".to_string(),
            size: jetbrains_size,
            size_display: format_size(jetbrains_size),
            file_count: jetbrains_count,
            accessible: jetbrains_count > 0,
            category: "开发工具".to_string(),
        });
        total_size += jetbrains_size;
        total_files += jetbrains_count;

        // 13. Git
        let git_paths = vec![
            format!("{}/.gitconfig", home_dir),
            format!("{}/.git-credentials", home_dir),
        ];
        let (git_size, git_count) = scan_paths(&git_paths);
        logs.push(LogInfo {
            log_type: "git".to_string(),
            label: "Git".to_string(),
            description: "Git 配置和凭据".to_string(),
            size: git_size,
            size_display: format_size(git_size),
            file_count: git_count,
            accessible: git_count > 0,
            category: "开发工具".to_string(),
        });
        total_size += git_size;
        total_files += git_count;

        // 14. Node.js/NPM
        let npm_paths = vec![
            format!("{}/.npm", home_dir),
            format!("{}/.node_repl_history", home_dir),
            format!("{}/.npmrc", home_dir),
        ];
        let (npm_size, npm_count) = scan_paths(&npm_paths);
        logs.push(LogInfo {
            log_type: "npm".to_string(),
            label: "NPM/Node".to_string(),
            description: "NPM 缓存和历史".to_string(),
            size: npm_size,
            size_display: format_size(npm_size),
            file_count: npm_count,
            accessible: npm_count > 0,
            category: "开发工具".to_string(),
        });
        total_size += npm_size;
        total_files += npm_count;

        // 15. Python
        let python_paths = vec![
            format!("{}/.python_history", home_dir),
            format!("{}/.ipython", home_dir),
            format!("{}/.jupyter", home_dir),
            format!("{}/Library/Caches/pip", home_dir),
        ];
        let (python_size, python_count) = scan_paths(&python_paths);
        logs.push(LogInfo {
            log_type: "python".to_string(),
            label: "Python".to_string(),
            description: "Python/pip/Jupyter 缓存和历史".to_string(),
            size: python_size,
            size_display: format_size(python_size),
            file_count: python_count,
            accessible: python_count > 0,
            category: "开发工具".to_string(),
        });
        total_size += python_size;
        total_files += python_count;

        // 16. Docker
        let docker_paths = vec![
            format!("{}/.docker", home_dir),
            format!("{}/Library/Containers/com.docker.docker", home_dir),
            format!("{}/Library/Group Containers/group.com.docker", home_dir),
        ];
        let (docker_size, docker_count) = scan_paths(&docker_paths);
        logs.push(LogInfo {
            log_type: "docker".to_string(),
            label: "Docker".to_string(),
            description: "Docker 配置、镜像缓存".to_string(),
            size: docker_size,
            size_display: format_size(docker_size),
            file_count: docker_count,
            accessible: docker_count > 0,
            category: "运维工具".to_string(),
        });
        total_size += docker_size;
        total_files += docker_count;

        // ========== 办公软件类别 ==========

        // 17. Microsoft Office
        let office_paths = vec![
            format!("{}/Library/Containers/com.microsoft.Word", home_dir),
            format!("{}/Library/Containers/com.microsoft.Excel", home_dir),
            format!("{}/Library/Containers/com.microsoft.Powerpoint", home_dir),
            format!("{}/Library/Group Containers/UBF8T346G9.Office", home_dir),
        ];
        let (office_size, office_count) = scan_paths(&office_paths);
        logs.push(LogInfo {
            log_type: "office".to_string(),
            label: "Microsoft Office".to_string(),
            description: "Word/Excel/PowerPoint 缓存和最近文档".to_string(),
            size: office_size,
            size_display: format_size(office_size),
            file_count: office_count,
            accessible: office_count > 0,
            category: "办公软件".to_string(),
        });
        total_size += office_size;
        total_files += office_count;

        // ========== 通讯软件类别 ==========

        // 18. 微信
        let wechat_paths = vec![
            format!("{}/Library/Containers/com.tencent.xinWeChat", home_dir),
            format!("{}/Library/Application Support/com.tencent.xinWeChat", home_dir),
        ];
        let (wechat_size, wechat_count) = scan_paths(&wechat_paths);
        logs.push(LogInfo {
            log_type: "wechat".to_string(),
            label: "微信".to_string(),
            description: "聊天缓存、图片、视频".to_string(),
            size: wechat_size,
            size_display: format_size(wechat_size),
            file_count: wechat_count,
            accessible: wechat_count > 0,
            category: "通讯软件".to_string(),
        });
        total_size += wechat_size;
        total_files += wechat_count;

        // 19. QQ
        let qq_paths = vec![
            format!("{}/Library/Containers/com.tencent.qq", home_dir),
            format!("{}/Library/Application Support/QQ", home_dir),
        ];
        let (qq_size, qq_count) = scan_paths(&qq_paths);
        logs.push(LogInfo {
            log_type: "qq".to_string(),
            label: "QQ".to_string(),
            description: "聊天缓存、图片、视频".to_string(),
            size: qq_size,
            size_display: format_size(qq_size),
            file_count: qq_count,
            accessible: qq_count > 0,
            category: "通讯软件".to_string(),
        });
        total_size += qq_size;
        total_files += qq_count;

        // 20. Telegram
        let telegram_paths = vec![
            format!("{}/Library/Application Support/Telegram Desktop", home_dir),
            format!("{}/Library/Group Containers/6N38VWS5BX.ru.keepcoder.Telegram", home_dir),
        ];
        let (telegram_size, telegram_count) = scan_paths(&telegram_paths);
        logs.push(LogInfo {
            log_type: "telegram".to_string(),
            label: "Telegram".to_string(),
            description: "聊天缓存、媒体文件".to_string(),
            size: telegram_size,
            size_display: format_size(telegram_size),
            file_count: telegram_count,
            accessible: telegram_count > 0,
            category: "通讯软件".to_string(),
        });
        total_size += telegram_size;
        total_files += telegram_count;

        // 21. Discord
        let discord_paths = vec![
            format!("{}/Library/Application Support/discord", home_dir),
            format!("{}/Library/Caches/discord", home_dir),
        ];
        let (discord_size, discord_count) = scan_paths(&discord_paths);
        logs.push(LogInfo {
            log_type: "discord".to_string(),
            label: "Discord".to_string(),
            description: "聊天缓存、媒体文件".to_string(),
            size: discord_size,
            size_display: format_size(discord_size),
            file_count: discord_count,
            accessible: discord_count > 0,
            category: "通讯软件".to_string(),
        });
        total_size += discord_size;
        total_files += discord_count;

        // 22. Slack
        let slack_paths = vec![
            format!("{}/Library/Application Support/Slack", home_dir),
            format!("{}/Library/Caches/com.tinyspeck.slackmacgap", home_dir),
        ];
        let (slack_size, slack_count) = scan_paths(&slack_paths);
        logs.push(LogInfo {
            log_type: "slack".to_string(),
            label: "Slack".to_string(),
            description: "聊天缓存、媒体文件".to_string(),
            size: slack_size,
            size_display: format_size(slack_size),
            file_count: slack_count,
            accessible: slack_count > 0,
            category: "通讯软件".to_string(),
        });
        total_size += slack_size;
        total_files += slack_count;

        // ========== 多媒体软件类别 ==========

        // 23. Spotify
        let spotify_paths = vec![
            format!("{}/Library/Application Support/Spotify", home_dir),
            format!("{}/Library/Caches/com.spotify.client", home_dir),
        ];
        let (spotify_size, spotify_count) = scan_paths(&spotify_paths);
        logs.push(LogInfo {
            log_type: "spotify".to_string(),
            label: "Spotify".to_string(),
            description: "音乐缓存、播放历史".to_string(),
            size: spotify_size,
            size_display: format_size(spotify_size),
            file_count: spotify_count,
            accessible: spotify_count > 0,
            category: "多媒体".to_string(),
        });
        total_size += spotify_size;
        total_files += spotify_count;

        // 24. VLC
        let vlc_paths = vec![
            format!("{}/Library/Application Support/VLC", home_dir),
            format!("{}/Library/Preferences/org.videolan.vlc", home_dir),
        ];
        let (vlc_size, vlc_count) = scan_paths(&vlc_paths);
        logs.push(LogInfo {
            log_type: "vlc".to_string(),
            label: "VLC".to_string(),
            description: "播放历史、配置".to_string(),
            size: vlc_size,
            size_display: format_size(vlc_size),
            file_count: vlc_count,
            accessible: vlc_count > 0,
            category: "多媒体".to_string(),
        });
        total_size += vlc_size;
        total_files += vlc_count;

        // ========== 安全/渗透工具类别 ==========

        // 25. Burp Suite
        let burp_paths = vec![
            format!("{}/.BurpSuite", home_dir),
            format!("{}/.java/.userPrefs/burp", home_dir),
        ];
        let (burp_size, burp_count) = scan_paths(&burp_paths);
        logs.push(LogInfo {
            log_type: "burpsuite".to_string(),
            label: "Burp Suite".to_string(),
            description: "项目文件、配置、历史".to_string(),
            size: burp_size,
            size_display: format_size(burp_size),
            file_count: burp_count,
            accessible: burp_count > 0,
            category: "安全工具".to_string(),
        });
        total_size += burp_size;
        total_files += burp_count;

        // 26. Wireshark
        let wireshark_paths = vec![
            format!("{}/.config/wireshark", home_dir),
            format!("{}/Library/Application Support/Wireshark", home_dir),
        ];
        let (wireshark_size, wireshark_count) = scan_paths(&wireshark_paths);
        logs.push(LogInfo {
            log_type: "wireshark".to_string(),
            label: "Wireshark".to_string(),
            description: "抓包文件、配置".to_string(),
            size: wireshark_size,
            size_display: format_size(wireshark_size),
            file_count: wireshark_count,
            accessible: wireshark_count > 0,
            category: "安全工具".to_string(),
        });
        total_size += wireshark_size;
        total_files += wireshark_count;

        // 27. Metasploit
        let msf_paths = vec![
            format!("{}/.msf4", home_dir),
        ];
        let (msf_size, msf_count) = scan_paths(&msf_paths);
        logs.push(LogInfo {
            log_type: "metasploit".to_string(),
            label: "Metasploit".to_string(),
            description: "数据库、日志、模块缓存".to_string(),
            size: msf_size,
            size_display: format_size(msf_size),
            file_count: msf_count,
            accessible: msf_count > 0,
            category: "安全工具".to_string(),
        });
        total_size += msf_size;
        total_files += msf_count;

        // ========== 终端/Shell 类别 ==========

        // 28. Shell历史
        let shell_paths = vec![
            format!("{}/.bash_history", home_dir),
            format!("{}/.zsh_history", home_dir),
            format!("{}/.zhistory", home_dir),
            format!("{}/.fish_history", home_dir),
            format!("{}/.local/share/fish/fish_history", home_dir),
        ];
        let (shell_size, shell_count) = scan_paths(&shell_paths);
        logs.push(LogInfo {
            log_type: "shell_history".to_string(),
            label: "Shell历史".to_string(),
            description: "Bash/Zsh/Fish 命令历史".to_string(),
            size: shell_size,
            size_display: format_size(shell_size),
            file_count: shell_count,
            accessible: shell_count > 0,
            category: "终端".to_string(),
        });
        total_size += shell_size;
        total_files += shell_count;

        // 29. SSH
        let ssh_paths = vec![
            format!("{}/.ssh/known_hosts", home_dir),
        ];
        let (ssh_size, ssh_count) = scan_paths(&ssh_paths);
        logs.push(LogInfo {
            log_type: "ssh".to_string(),
            label: "SSH".to_string(),
            description: "已知主机记录".to_string(),
            size: ssh_size,
            size_display: format_size(ssh_size),
            file_count: ssh_count,
            accessible: ssh_count > 0,
            category: "终端".to_string(),
        });
        total_size += ssh_size;
        total_files += ssh_count;

        // 30. iTerm2
        let iterm_paths = vec![
            format!("{}/Library/Application Support/iTerm2", home_dir),
            format!("{}/Library/Caches/com.googlecode.iterm2", home_dir),
        ];
        let (iterm_size, iterm_count) = scan_paths(&iterm_paths);
        logs.push(LogInfo {
            log_type: "iterm".to_string(),
            label: "iTerm2".to_string(),
            description: "终端配置、日志".to_string(),
            size: iterm_size,
            size_display: format_size(iterm_size),
            file_count: iterm_count,
            accessible: iterm_count > 0,
            category: "终端".to_string(),
        });
        total_size += iterm_size;
        total_files += iterm_count;

        // ========== 运维工具类别 ==========

        // 31. Homebrew
        let homebrew_paths = vec![
            format!("{}/Library/Caches/Homebrew", home_dir),
            format!("{}/Library/Logs/Homebrew", home_dir),
        ];
        let (homebrew_size, homebrew_count) = scan_paths(&homebrew_paths);
        logs.push(LogInfo {
            log_type: "homebrew".to_string(),
            label: "Homebrew".to_string(),
            description: "软件包缓存、安装日志".to_string(),
            size: homebrew_size,
            size_display: format_size(homebrew_size),
            file_count: homebrew_count,
            accessible: homebrew_count > 0,
            category: "运维工具".to_string(),
        });
        total_size += homebrew_size;
        total_files += homebrew_count;

        // 32. Kubernetes
        let k8s_paths = vec![
            format!("{}/.kube", home_dir),
            format!("{}/.minikube", home_dir),
        ];
        let (k8s_size, k8s_count) = scan_paths(&k8s_paths);
        logs.push(LogInfo {
            log_type: "kubernetes".to_string(),
            label: "Kubernetes".to_string(),
            description: "集群配置、缓存".to_string(),
            size: k8s_size,
            size_display: format_size(k8s_size),
            file_count: k8s_count,
            accessible: k8s_count > 0,
            category: "运维工具".to_string(),
        });
        total_size += k8s_size;
        total_files += k8s_count;

        // 33. AWS CLI
        let aws_paths = vec![
            format!("{}/.aws", home_dir),
        ];
        let (aws_size, aws_count) = scan_paths(&aws_paths);
        logs.push(LogInfo {
            log_type: "aws".to_string(),
            label: "AWS CLI".to_string(),
            description: "配置、凭据、缓存".to_string(),
            size: aws_size,
            size_display: format_size(aws_size),
            file_count: aws_count,
            accessible: aws_count > 0,
            category: "运维工具".to_string(),
        });
        total_size += aws_size;
        total_files += aws_count;

        // ========== 下载/文件类别 ==========

        // 34. 废纸篓
        let trash_paths = vec![
            format!("{}/.Trash", home_dir),
        ];
        let (trash_size, trash_count) = scan_paths(&trash_paths);
        logs.push(LogInfo {
            log_type: "trash".to_string(),
            label: "废纸篓".to_string(),
            description: "已删除的文件".to_string(),
            size: trash_size,
            size_display: format_size(trash_size),
            file_count: trash_count,
            accessible: trash_count > 0,
            category: "文件".to_string(),
        });
        total_size += trash_size;
        total_files += trash_count;

        // 35. Downloads
        let downloads_paths = vec![
            format!("{}/Downloads", home_dir),
        ];
        let (downloads_size, downloads_count) = scan_paths(&downloads_paths);
        logs.push(LogInfo {
            log_type: "downloads".to_string(),
            label: "下载文件夹".to_string(),
            description: "下载的文件".to_string(),
            size: downloads_size,
            size_display: format_size(downloads_size),
            file_count: downloads_count,
            accessible: downloads_count > 0,
            category: "文件".to_string(),
        });
        total_size += downloads_size;
        total_files += downloads_count;

        // ========== 更多浏览器 ==========

        // 36. Arc Browser
        let arc_paths = vec![
            format!("{}/Library/Application Support/Arc", home_dir),
            format!("{}/Library/Caches/company.thebrowser.Browser", home_dir),
        ];
        let (arc_size, arc_count) = scan_paths(&arc_paths);
        logs.push(LogInfo {
            log_type: "arc".to_string(),
            label: "Arc".to_string(),
            description: "浏览历史、缓存、Cookie".to_string(),
            size: arc_size,
            size_display: format_size(arc_size),
            file_count: arc_count,
            accessible: arc_count > 0,
            category: "浏览器".to_string(),
        });
        total_size += arc_size;
        total_files += arc_count;

        // 37. Brave Browser
        let brave_paths = vec![
            format!("{}/Library/Application Support/BraveSoftware/Brave-Browser", home_dir),
            format!("{}/Library/Caches/BraveSoftware/Brave-Browser", home_dir),
        ];
        let (brave_size, brave_count) = scan_paths(&brave_paths);
        logs.push(LogInfo {
            log_type: "brave".to_string(),
            label: "Brave".to_string(),
            description: "浏览历史、缓存、Cookie".to_string(),
            size: brave_size,
            size_display: format_size(brave_size),
            file_count: brave_count,
            accessible: brave_count > 0,
            category: "浏览器".to_string(),
        });
        total_size += brave_size;
        total_files += brave_count;

        // 38. Opera
        let opera_paths = vec![
            format!("{}/Library/Application Support/com.operasoftware.Opera", home_dir),
            format!("{}/Library/Caches/com.operasoftware.Opera", home_dir),
        ];
        let (opera_size, opera_count) = scan_paths(&opera_paths);
        logs.push(LogInfo {
            log_type: "opera".to_string(),
            label: "Opera".to_string(),
            description: "浏览历史、缓存、Cookie".to_string(),
            size: opera_size,
            size_display: format_size(opera_size),
            file_count: opera_count,
            accessible: opera_count > 0,
            category: "浏览器".to_string(),
        });
        total_size += opera_size;
        total_files += opera_count;

        // 39. Vivaldi
        let vivaldi_paths = vec![
            format!("{}/Library/Application Support/Vivaldi", home_dir),
            format!("{}/Library/Caches/Vivaldi", home_dir),
        ];
        let (vivaldi_size, vivaldi_count) = scan_paths(&vivaldi_paths);
        logs.push(LogInfo {
            log_type: "vivaldi".to_string(),
            label: "Vivaldi".to_string(),
            description: "浏览历史、缓存、Cookie".to_string(),
            size: vivaldi_size,
            size_display: format_size(vivaldi_size),
            file_count: vivaldi_count,
            accessible: vivaldi_count > 0,
            category: "浏览器".to_string(),
        });
        total_size += vivaldi_size;
        total_files += vivaldi_count;

        // ========== 更多开发工具 ==========

        // 40. Android Studio
        let android_studio_paths = vec![
            format!("{}/Library/Application Support/Google/AndroidStudio*", home_dir),
            format!("{}/Library/Caches/Google/AndroidStudio*", home_dir),
            format!("{}/.android", home_dir),
            format!("{}/Library/Android/sdk", home_dir),
        ];
        let (android_studio_size, android_studio_count) = scan_paths(&android_studio_paths);
        logs.push(LogInfo {
            log_type: "android_studio".to_string(),
            label: "Android Studio".to_string(),
            description: "Android 开发工具、SDK、模拟器".to_string(),
            size: android_studio_size,
            size_display: format_size(android_studio_size),
            file_count: android_studio_count,
            accessible: android_studio_count > 0,
            category: "开发工具".to_string(),
        });
        total_size += android_studio_size;
        total_files += android_studio_count;

        // 41. Rust/Cargo
        let rust_paths = vec![
            format!("{}/.cargo", home_dir),
            format!("{}/.rustup", home_dir),
        ];
        let (rust_size, rust_count) = scan_paths(&rust_paths);
        logs.push(LogInfo {
            log_type: "rust".to_string(),
            label: "Rust/Cargo".to_string(),
            description: "Rust 工具链、包缓存".to_string(),
            size: rust_size,
            size_display: format_size(rust_size),
            file_count: rust_count,
            accessible: rust_count > 0,
            category: "开发工具".to_string(),
        });
        total_size += rust_size;
        total_files += rust_count;

        // 42. Go
        let go_paths = vec![
            format!("{}/go", home_dir),
            format!("{}/.cache/go-build", home_dir),
        ];
        let (go_size, go_count) = scan_paths(&go_paths);
        logs.push(LogInfo {
            log_type: "golang".to_string(),
            label: "Go".to_string(),
            description: "Go 模块缓存、构建缓存".to_string(),
            size: go_size,
            size_display: format_size(go_size),
            file_count: go_count,
            accessible: go_count > 0,
            category: "开发工具".to_string(),
        });
        total_size += go_size;
        total_files += go_count;

        // 43. Ruby/Gem
        let ruby_paths = vec![
            format!("{}/.gem", home_dir),
            format!("{}/.bundle", home_dir),
            format!("{}/.rbenv", home_dir),
            format!("{}/.rvm", home_dir),
        ];
        let (ruby_size, ruby_count) = scan_paths(&ruby_paths);
        logs.push(LogInfo {
            log_type: "ruby".to_string(),
            label: "Ruby/Gem".to_string(),
            description: "Ruby 包管理器缓存".to_string(),
            size: ruby_size,
            size_display: format_size(ruby_size),
            file_count: ruby_count,
            accessible: ruby_count > 0,
            category: "开发工具".to_string(),
        });
        total_size += ruby_size;
        total_files += ruby_count;

        // 44. Java/Maven/Gradle
        let java_paths = vec![
            format!("{}/.m2", home_dir),
            format!("{}/.gradle", home_dir),
            format!("{}/.java", home_dir),
        ];
        let (java_size, java_count) = scan_paths(&java_paths);
        logs.push(LogInfo {
            log_type: "java".to_string(),
            label: "Java/Maven/Gradle".to_string(),
            description: "Java 构建工具缓存".to_string(),
            size: java_size,
            size_display: format_size(java_size),
            file_count: java_count,
            accessible: java_count > 0,
            category: "开发工具".to_string(),
        });
        total_size += java_size;
        total_files += java_count;

        // 45. Sublime Text
        let sublime_paths = vec![
            format!("{}/Library/Application Support/Sublime Text", home_dir),
            format!("{}/Library/Caches/com.sublimetext.4", home_dir),
        ];
        let (sublime_size, sublime_count) = scan_paths(&sublime_paths);
        logs.push(LogInfo {
            log_type: "sublime".to_string(),
            label: "Sublime Text".to_string(),
            description: "编辑器缓存、会话数据".to_string(),
            size: sublime_size,
            size_display: format_size(sublime_size),
            file_count: sublime_count,
            accessible: sublime_count > 0,
            category: "开发工具".to_string(),
        });
        total_size += sublime_size;
        total_files += sublime_count;

        // 46. Cursor (AI IDE)
        let cursor_paths = vec![
            format!("{}/Library/Application Support/Cursor", home_dir),
            format!("{}/Library/Caches/Cursor", home_dir),
            format!("{}/.cursor", home_dir),
        ];
        let (cursor_size, cursor_count) = scan_paths(&cursor_paths);
        logs.push(LogInfo {
            log_type: "cursor".to_string(),
            label: "Cursor".to_string(),
            description: "AI 编辑器缓存、配置".to_string(),
            size: cursor_size,
            size_display: format_size(cursor_size),
            file_count: cursor_count,
            accessible: cursor_count > 0,
            category: "开发工具".to_string(),
        });
        total_size += cursor_size;
        total_files += cursor_count;

        // 47. Yarn/pnpm
        let yarn_paths = vec![
            format!("{}/.yarn", home_dir),
            format!("{}/.yarnrc", home_dir),
            format!("{}/Library/Caches/Yarn", home_dir),
            format!("{}/.pnpm-store", home_dir),
            format!("{}/Library/pnpm", home_dir),
        ];
        let (yarn_size, yarn_count) = scan_paths(&yarn_paths);
        logs.push(LogInfo {
            log_type: "yarn_pnpm".to_string(),
            label: "Yarn/pnpm".to_string(),
            description: "JavaScript 包管理器缓存".to_string(),
            size: yarn_size,
            size_display: format_size(yarn_size),
            file_count: yarn_count,
            accessible: yarn_count > 0,
            category: "开发工具".to_string(),
        });
        total_size += yarn_size;
        total_files += yarn_count;

        // 48. CocoaPods
        let cocoapods_paths = vec![
            format!("{}/.cocoapods", home_dir),
            format!("{}/Library/Caches/CocoaPods", home_dir),
        ];
        let (cocoapods_size, cocoapods_count) = scan_paths(&cocoapods_paths);
        logs.push(LogInfo {
            log_type: "cocoapods".to_string(),
            label: "CocoaPods".to_string(),
            description: "iOS/macOS 依赖管理缓存".to_string(),
            size: cocoapods_size,
            size_display: format_size(cocoapods_size),
            file_count: cocoapods_count,
            accessible: cocoapods_count > 0,
            category: "开发工具".to_string(),
        });
        total_size += cocoapods_size;
        total_files += cocoapods_count;

        // ========== 更多通讯软件 ==========

        // 49. 飞书/Lark
        let feishu_paths = vec![
            format!("{}/Library/Containers/com.bytedance.lark", home_dir),
            format!("{}/Library/Application Support/Lark", home_dir),
            format!("{}/Library/Application Support/Feishu", home_dir),
        ];
        let (feishu_size, feishu_count) = scan_paths(&feishu_paths);
        logs.push(LogInfo {
            log_type: "feishu".to_string(),
            label: "飞书/Lark".to_string(),
            description: "聊天缓存、文件、日志".to_string(),
            size: feishu_size,
            size_display: format_size(feishu_size),
            file_count: feishu_count,
            accessible: feishu_count > 0,
            category: "通讯软件".to_string(),
        });
        total_size += feishu_size;
        total_files += feishu_count;

        // 50. 钉钉/DingTalk
        let dingtalk_paths = vec![
            format!("{}/Library/Containers/com.alibaba.DingTalkMac", home_dir),
            format!("{}/Library/Application Support/DingTalk", home_dir),
        ];
        let (dingtalk_size, dingtalk_count) = scan_paths(&dingtalk_paths);
        logs.push(LogInfo {
            log_type: "dingtalk".to_string(),
            label: "钉钉".to_string(),
            description: "聊天缓存、文件、日志".to_string(),
            size: dingtalk_size,
            size_display: format_size(dingtalk_size),
            file_count: dingtalk_count,
            accessible: dingtalk_count > 0,
            category: "通讯软件".to_string(),
        });
        total_size += dingtalk_size;
        total_files += dingtalk_count;

        // 51. Zoom
        let zoom_paths = vec![
            format!("{}/Library/Application Support/zoom.us", home_dir),
            format!("{}/Library/Caches/us.zoom.xos", home_dir),
            format!("{}/Library/Logs/zoom.us", home_dir),
        ];
        let (zoom_size, zoom_count) = scan_paths(&zoom_paths);
        logs.push(LogInfo {
            log_type: "zoom".to_string(),
            label: "Zoom".to_string(),
            description: "会议缓存、录制、日志".to_string(),
            size: zoom_size,
            size_display: format_size(zoom_size),
            file_count: zoom_count,
            accessible: zoom_count > 0,
            category: "通讯软件".to_string(),
        });
        total_size += zoom_size;
        total_files += zoom_count;

        // 52. Microsoft Teams
        let teams_paths = vec![
            format!("{}/Library/Application Support/Microsoft/Teams", home_dir),
            format!("{}/Library/Caches/com.microsoft.teams", home_dir),
        ];
        let (teams_size, teams_count) = scan_paths(&teams_paths);
        logs.push(LogInfo {
            log_type: "teams".to_string(),
            label: "Microsoft Teams".to_string(),
            description: "聊天缓存、会议数据".to_string(),
            size: teams_size,
            size_display: format_size(teams_size),
            file_count: teams_count,
            accessible: teams_count > 0,
            category: "通讯软件".to_string(),
        });
        total_size += teams_size;
        total_files += teams_count;

        // 53. Skype
        let skype_paths = vec![
            format!("{}/Library/Application Support/Skype", home_dir),
            format!("{}/Library/Caches/com.skype.skype", home_dir),
        ];
        let (skype_size, skype_count) = scan_paths(&skype_paths);
        logs.push(LogInfo {
            log_type: "skype".to_string(),
            label: "Skype".to_string(),
            description: "聊天缓存、通话记录".to_string(),
            size: skype_size,
            size_display: format_size(skype_size),
            file_count: skype_count,
            accessible: skype_count > 0,
            category: "通讯软件".to_string(),
        });
        total_size += skype_size;
        total_files += skype_count;

        // 54. WhatsApp
        let whatsapp_paths = vec![
            format!("{}/Library/Application Support/WhatsApp", home_dir),
            format!("{}/Library/Containers/net.whatsapp.WhatsApp", home_dir),
        ];
        let (whatsapp_size, whatsapp_count) = scan_paths(&whatsapp_paths);
        logs.push(LogInfo {
            log_type: "whatsapp".to_string(),
            label: "WhatsApp".to_string(),
            description: "聊天缓存、媒体文件".to_string(),
            size: whatsapp_size,
            size_display: format_size(whatsapp_size),
            file_count: whatsapp_count,
            accessible: whatsapp_count > 0,
            category: "通讯软件".to_string(),
        });
        total_size += whatsapp_size;
        total_files += whatsapp_count;

        // 55. Signal
        let signal_paths = vec![
            format!("{}/Library/Application Support/Signal", home_dir),
            format!("{}/Library/Caches/org.whispersystems.signal-desktop", home_dir),
        ];
        let (signal_size, signal_count) = scan_paths(&signal_paths);
        logs.push(LogInfo {
            log_type: "signal".to_string(),
            label: "Signal".to_string(),
            description: "聊天缓存、媒体文件".to_string(),
            size: signal_size,
            size_display: format_size(signal_size),
            file_count: signal_count,
            accessible: signal_count > 0,
            category: "通讯软件".to_string(),
        });
        total_size += signal_size;
        total_files += signal_count;

        // ========== 更多多媒体软件 ==========

        // 56. IINA
        let iina_paths = vec![
            format!("{}/Library/Application Support/com.colliderli.iina", home_dir),
            format!("{}/Library/Caches/com.colliderli.iina", home_dir),
        ];
        let (iina_size, iina_count) = scan_paths(&iina_paths);
        logs.push(LogInfo {
            log_type: "iina".to_string(),
            label: "IINA".to_string(),
            description: "播放历史、配置".to_string(),
            size: iina_size,
            size_display: format_size(iina_size),
            file_count: iina_count,
            accessible: iina_count > 0,
            category: "多媒体".to_string(),
        });
        total_size += iina_size;
        total_files += iina_count;

        // 57. 网易云音乐
        let netease_music_paths = vec![
            format!("{}/Library/Containers/com.netease.163music", home_dir),
            format!("{}/Library/Application Support/NeteaseMusic", home_dir),
        ];
        let (netease_music_size, netease_music_count) = scan_paths(&netease_music_paths);
        logs.push(LogInfo {
            log_type: "netease_music".to_string(),
            label: "网易云音乐".to_string(),
            description: "音乐缓存、播放历史".to_string(),
            size: netease_music_size,
            size_display: format_size(netease_music_size),
            file_count: netease_music_count,
            accessible: netease_music_count > 0,
            category: "多媒体".to_string(),
        });
        total_size += netease_music_size;
        total_files += netease_music_count;

        // 58. QQ音乐
        let qq_music_paths = vec![
            format!("{}/Library/Containers/com.tencent.QQMusicMac", home_dir),
            format!("{}/Library/Application Support/QQMusic", home_dir),
        ];
        let (qq_music_size, qq_music_count) = scan_paths(&qq_music_paths);
        logs.push(LogInfo {
            log_type: "qq_music".to_string(),
            label: "QQ音乐".to_string(),
            description: "音乐缓存、播放历史".to_string(),
            size: qq_music_size,
            size_display: format_size(qq_music_size),
            file_count: qq_music_count,
            accessible: qq_music_count > 0,
            category: "多媒体".to_string(),
        });
        total_size += qq_music_size;
        total_files += qq_music_count;

        // 59. Apple Music/iTunes
        let apple_music_paths = vec![
            format!("{}/Library/Caches/com.apple.Music", home_dir),
            format!("{}/Library/Application Support/Music", home_dir),
        ];
        let (apple_music_size, apple_music_count) = scan_paths(&apple_music_paths);
        logs.push(LogInfo {
            log_type: "apple_music".to_string(),
            label: "Apple Music".to_string(),
            description: "音乐缓存、播放数据".to_string(),
            size: apple_music_size,
            size_display: format_size(apple_music_size),
            file_count: apple_music_count,
            accessible: apple_music_count > 0,
            category: "多媒体".to_string(),
        });
        total_size += apple_music_size;
        total_files += apple_music_count;

        // 60. 哔哩哔哩
        let bilibili_paths = vec![
            format!("{}/Library/Application Support/com.bilibili.app.pgc", home_dir),
            format!("{}/Library/Containers/tv.danmaku.bilibili", home_dir),
        ];
        let (bilibili_size, bilibili_count) = scan_paths(&bilibili_paths);
        logs.push(LogInfo {
            log_type: "bilibili".to_string(),
            label: "哔哩哔哩".to_string(),
            description: "视频缓存、观看历史".to_string(),
            size: bilibili_size,
            size_display: format_size(bilibili_size),
            file_count: bilibili_count,
            accessible: bilibili_count > 0,
            category: "多媒体".to_string(),
        });
        total_size += bilibili_size;
        total_files += bilibili_count;

        // 61. 腾讯视频
        let tencent_video_paths = vec![
            format!("{}/Library/Containers/com.tencent.tenvideo", home_dir),
            format!("{}/Library/Application Support/TencentVideo", home_dir),
        ];
        let (tencent_video_size, tencent_video_count) = scan_paths(&tencent_video_paths);
        logs.push(LogInfo {
            log_type: "tencent_video".to_string(),
            label: "腾讯视频".to_string(),
            description: "视频缓存、观看历史".to_string(),
            size: tencent_video_size,
            size_display: format_size(tencent_video_size),
            file_count: tencent_video_count,
            accessible: tencent_video_count > 0,
            category: "多媒体".to_string(),
        });
        total_size += tencent_video_size;
        total_files += tencent_video_count;

        // ========== 云存储/同步工具 ==========

        // 62. iCloud
        let icloud_paths = vec![
            format!("{}/Library/Mobile Documents", home_dir),
            format!("{}/Library/Caches/com.apple.bird", home_dir),
        ];
        let (icloud_size, icloud_count) = scan_paths(&icloud_paths);
        logs.push(LogInfo {
            log_type: "icloud".to_string(),
            label: "iCloud".to_string(),
            description: "云同步缓存".to_string(),
            size: icloud_size,
            size_display: format_size(icloud_size),
            file_count: icloud_count,
            accessible: icloud_count > 0,
            category: "云存储".to_string(),
        });
        total_size += icloud_size;
        total_files += icloud_count;

        // 63. Dropbox
        let dropbox_paths = vec![
            format!("{}/.dropbox", home_dir),
            format!("{}/Library/Dropbox", home_dir),
            format!("{}/Library/Caches/com.dropbox.client", home_dir),
        ];
        let (dropbox_size, dropbox_count) = scan_paths(&dropbox_paths);
        logs.push(LogInfo {
            log_type: "dropbox".to_string(),
            label: "Dropbox".to_string(),
            description: "同步缓存、配置".to_string(),
            size: dropbox_size,
            size_display: format_size(dropbox_size),
            file_count: dropbox_count,
            accessible: dropbox_count > 0,
            category: "云存储".to_string(),
        });
        total_size += dropbox_size;
        total_files += dropbox_count;

        // 64. Google Drive
        let gdrive_paths = vec![
            format!("{}/Library/Application Support/Google/DriveFS", home_dir),
            format!("{}/Library/Caches/com.google.drivefs", home_dir),
        ];
        let (gdrive_size, gdrive_count) = scan_paths(&gdrive_paths);
        logs.push(LogInfo {
            log_type: "google_drive".to_string(),
            label: "Google Drive".to_string(),
            description: "同步缓存、配置".to_string(),
            size: gdrive_size,
            size_display: format_size(gdrive_size),
            file_count: gdrive_count,
            accessible: gdrive_count > 0,
            category: "云存储".to_string(),
        });
        total_size += gdrive_size;
        total_files += gdrive_count;

        // 65. OneDrive
        let onedrive_paths = vec![
            format!("{}/Library/Application Support/OneDrive", home_dir),
            format!("{}/Library/Caches/com.microsoft.OneDrive", home_dir),
            format!("{}/Library/Logs/OneDrive", home_dir),
        ];
        let (onedrive_size, onedrive_count) = scan_paths(&onedrive_paths);
        logs.push(LogInfo {
            log_type: "onedrive".to_string(),
            label: "OneDrive".to_string(),
            description: "同步缓存、日志".to_string(),
            size: onedrive_size,
            size_display: format_size(onedrive_size),
            file_count: onedrive_count,
            accessible: onedrive_count > 0,
            category: "云存储".to_string(),
        });
        total_size += onedrive_size;
        total_files += onedrive_count;

        // 66. 坚果云
        let nutstore_paths = vec![
            format!("{}/Library/Application Support/Nutstore", home_dir),
            format!("{}/Library/Caches/com.jianguoyun.Nutstore", home_dir),
        ];
        let (nutstore_size, nutstore_count) = scan_paths(&nutstore_paths);
        logs.push(LogInfo {
            log_type: "nutstore".to_string(),
            label: "坚果云".to_string(),
            description: "同步缓存、配置".to_string(),
            size: nutstore_size,
            size_display: format_size(nutstore_size),
            file_count: nutstore_count,
            accessible: nutstore_count > 0,
            category: "云存储".to_string(),
        });
        total_size += nutstore_size;
        total_files += nutstore_count;

        // 67. 百度网盘
        let baidu_netdisk_paths = vec![
            format!("{}/Library/Application Support/BaiduNetdisk", home_dir),
            format!("{}/Library/Containers/com.baidu.BaiduNetdisk", home_dir),
        ];
        let (baidu_netdisk_size, baidu_netdisk_count) = scan_paths(&baidu_netdisk_paths);
        logs.push(LogInfo {
            log_type: "baidu_netdisk".to_string(),
            label: "百度网盘".to_string(),
            description: "下载缓存、配置".to_string(),
            size: baidu_netdisk_size,
            size_display: format_size(baidu_netdisk_size),
            file_count: baidu_netdisk_count,
            accessible: baidu_netdisk_count > 0,
            category: "云存储".to_string(),
        });
        total_size += baidu_netdisk_size;
        total_files += baidu_netdisk_count;

        // ========== AI 工具 ==========

        // 68. ChatGPT
        let chatgpt_paths = vec![
            format!("{}/Library/Application Support/com.openai.chat", home_dir),
            format!("{}/Library/Caches/com.openai.chat", home_dir),
        ];
        let (chatgpt_size, chatgpt_count) = scan_paths(&chatgpt_paths);
        logs.push(LogInfo {
            log_type: "chatgpt".to_string(),
            label: "ChatGPT".to_string(),
            description: "对话缓存、配置".to_string(),
            size: chatgpt_size,
            size_display: format_size(chatgpt_size),
            file_count: chatgpt_count,
            accessible: chatgpt_count > 0,
            category: "AI工具".to_string(),
        });
        total_size += chatgpt_size;
        total_files += chatgpt_count;

        // 69. Claude
        let claude_paths = vec![
            format!("{}/Library/Application Support/Claude", home_dir),
            format!("{}/Library/Caches/com.anthropic.claudefordesktop", home_dir),
            format!("{}/.claude", home_dir),
        ];
        let (claude_size, claude_count) = scan_paths(&claude_paths);
        logs.push(LogInfo {
            log_type: "claude".to_string(),
            label: "Claude".to_string(),
            description: "对话缓存、配置".to_string(),
            size: claude_size,
            size_display: format_size(claude_size),
            file_count: claude_count,
            accessible: claude_count > 0,
            category: "AI工具".to_string(),
        });
        total_size += claude_size;
        total_files += claude_count;

        // 70. GitHub Copilot
        let copilot_paths = vec![
            format!("{}/Library/Application Support/GitHub Copilot", home_dir),
            format!("{}/.config/github-copilot", home_dir),
        ];
        let (copilot_size, copilot_count) = scan_paths(&copilot_paths);
        logs.push(LogInfo {
            log_type: "copilot".to_string(),
            label: "GitHub Copilot".to_string(),
            description: "AI 代码助手缓存".to_string(),
            size: copilot_size,
            size_display: format_size(copilot_size),
            file_count: copilot_count,
            accessible: copilot_count > 0,
            category: "AI工具".to_string(),
        });
        total_size += copilot_size;
        total_files += copilot_count;

        // ========== 数据库客户端 ==========

        // 71. TablePlus
        let tableplus_paths = vec![
            format!("{}/Library/Application Support/com.tinyapp.TablePlus", home_dir),
            format!("{}/Library/Caches/com.tinyapp.TablePlus", home_dir),
        ];
        let (tableplus_size, tableplus_count) = scan_paths(&tableplus_paths);
        logs.push(LogInfo {
            log_type: "tableplus".to_string(),
            label: "TablePlus".to_string(),
            description: "数据库连接、查询历史".to_string(),
            size: tableplus_size,
            size_display: format_size(tableplus_size),
            file_count: tableplus_count,
            accessible: tableplus_count > 0,
            category: "数据库".to_string(),
        });
        total_size += tableplus_size;
        total_files += tableplus_count;

        // 72. DBeaver
        let dbeaver_paths = vec![
            format!("{}/Library/DBeaverData", home_dir),
            format!("{}/.dbeaver4", home_dir),
        ];
        let (dbeaver_size, dbeaver_count) = scan_paths(&dbeaver_paths);
        logs.push(LogInfo {
            log_type: "dbeaver".to_string(),
            label: "DBeaver".to_string(),
            description: "数据库连接、查询历史".to_string(),
            size: dbeaver_size,
            size_display: format_size(dbeaver_size),
            file_count: dbeaver_count,
            accessible: dbeaver_count > 0,
            category: "数据库".to_string(),
        });
        total_size += dbeaver_size;
        total_files += dbeaver_count;

        // 73. Sequel Pro/Sequel Ace
        let sequel_paths = vec![
            format!("{}/Library/Application Support/Sequel Pro", home_dir),
            format!("{}/Library/Application Support/Sequel Ace", home_dir),
        ];
        let (sequel_size, sequel_count) = scan_paths(&sequel_paths);
        logs.push(LogInfo {
            log_type: "sequel".to_string(),
            label: "Sequel Pro/Ace".to_string(),
            description: "MySQL 客户端数据".to_string(),
            size: sequel_size,
            size_display: format_size(sequel_size),
            file_count: sequel_count,
            accessible: sequel_count > 0,
            category: "数据库".to_string(),
        });
        total_size += sequel_size;
        total_files += sequel_count;

        // 74. MongoDB Compass
        let mongodb_paths = vec![
            format!("{}/Library/Application Support/MongoDB Compass", home_dir),
            format!("{}/Library/Caches/MongoDB Compass", home_dir),
        ];
        let (mongodb_size, mongodb_count) = scan_paths(&mongodb_paths);
        logs.push(LogInfo {
            log_type: "mongodb_compass".to_string(),
            label: "MongoDB Compass".to_string(),
            description: "MongoDB 客户端数据".to_string(),
            size: mongodb_size,
            size_display: format_size(mongodb_size),
            file_count: mongodb_count,
            accessible: mongodb_count > 0,
            category: "数据库".to_string(),
        });
        total_size += mongodb_size;
        total_files += mongodb_count;

        // 75. Redis Desktop Manager
        let redis_paths = vec![
            format!("{}/Library/Application Support/rdm", home_dir),
            format!("{}/.rdm", home_dir),
        ];
        let (redis_size, redis_count) = scan_paths(&redis_paths);
        logs.push(LogInfo {
            log_type: "redis_desktop".to_string(),
            label: "Redis Desktop".to_string(),
            description: "Redis 客户端数据".to_string(),
            size: redis_size,
            size_display: format_size(redis_size),
            file_count: redis_count,
            accessible: redis_count > 0,
            category: "数据库".to_string(),
        });
        total_size += redis_size;
        total_files += redis_count;

        // ========== 更多安全工具 ==========

        // 76. Charles
        let charles_paths = vec![
            format!("{}/Library/Application Support/Charles", home_dir),
            format!("{}/Library/Caches/com.xk72.Charles", home_dir),
        ];
        let (charles_size, charles_count) = scan_paths(&charles_paths);
        logs.push(LogInfo {
            log_type: "charles".to_string(),
            label: "Charles".to_string(),
            description: "HTTP 代理、抓包数据".to_string(),
            size: charles_size,
            size_display: format_size(charles_size),
            file_count: charles_count,
            accessible: charles_count > 0,
            category: "安全工具".to_string(),
        });
        total_size += charles_size;
        total_files += charles_count;

        // 77. Proxyman
        let proxyman_paths = vec![
            format!("{}/Library/Application Support/com.proxyman.NSProxy", home_dir),
            format!("{}/Library/Caches/com.proxyman.NSProxy", home_dir),
        ];
        let (proxyman_size, proxyman_count) = scan_paths(&proxyman_paths);
        logs.push(LogInfo {
            log_type: "proxyman".to_string(),
            label: "Proxyman".to_string(),
            description: "HTTP 代理、抓包数据".to_string(),
            size: proxyman_size,
            size_display: format_size(proxyman_size),
            file_count: proxyman_count,
            accessible: proxyman_count > 0,
            category: "安全工具".to_string(),
        });
        total_size += proxyman_size;
        total_files += proxyman_count;

        // 78. Postman
        let postman_paths = vec![
            format!("{}/Library/Application Support/Postman", home_dir),
            format!("{}/Library/Caches/Postman", home_dir),
        ];
        let (postman_size, postman_count) = scan_paths(&postman_paths);
        logs.push(LogInfo {
            log_type: "postman".to_string(),
            label: "Postman".to_string(),
            description: "API 测试、历史记录".to_string(),
            size: postman_size,
            size_display: format_size(postman_size),
            file_count: postman_count,
            accessible: postman_count > 0,
            category: "安全工具".to_string(),
        });
        total_size += postman_size;
        total_files += postman_count;

        // 79. Insomnia
        let insomnia_paths = vec![
            format!("{}/Library/Application Support/Insomnia", home_dir),
            format!("{}/Library/Caches/Insomnia", home_dir),
        ];
        let (insomnia_size, insomnia_count) = scan_paths(&insomnia_paths);
        logs.push(LogInfo {
            log_type: "insomnia".to_string(),
            label: "Insomnia".to_string(),
            description: "API 测试、请求历史".to_string(),
            size: insomnia_size,
            size_display: format_size(insomnia_size),
            file_count: insomnia_count,
            accessible: insomnia_count > 0,
            category: "安全工具".to_string(),
        });
        total_size += insomnia_size;
        total_files += insomnia_count;

        // ========== 虚拟化工具 ==========

        // 80. VMware Fusion
        let vmware_paths = vec![
            format!("{}/Library/Application Support/VMware Fusion", home_dir),
            format!("{}/Library/Caches/com.vmware.fusion", home_dir),
            format!("{}/Library/Logs/VMware", home_dir),
        ];
        let (vmware_size, vmware_count) = scan_paths(&vmware_paths);
        logs.push(LogInfo {
            log_type: "vmware".to_string(),
            label: "VMware Fusion".to_string(),
            description: "虚拟机配置、日志".to_string(),
            size: vmware_size,
            size_display: format_size(vmware_size),
            file_count: vmware_count,
            accessible: vmware_count > 0,
            category: "虚拟化".to_string(),
        });
        total_size += vmware_size;
        total_files += vmware_count;

        // 81. Parallels
        let parallels_paths = vec![
            format!("{}/Library/Parallels", home_dir),
            format!("{}/Library/Logs/Parallels", home_dir),
        ];
        let (parallels_size, parallels_count) = scan_paths(&parallels_paths);
        logs.push(LogInfo {
            log_type: "parallels".to_string(),
            label: "Parallels".to_string(),
            description: "虚拟机配置、日志".to_string(),
            size: parallels_size,
            size_display: format_size(parallels_size),
            file_count: parallels_count,
            accessible: parallels_count > 0,
            category: "虚拟化".to_string(),
        });
        total_size += parallels_size;
        total_files += parallels_count;

        // 82. VirtualBox
        let virtualbox_paths = vec![
            format!("{}/Library/VirtualBox", home_dir),
            format!("{}/.VirtualBox", home_dir),
        ];
        let (virtualbox_size, virtualbox_count) = scan_paths(&virtualbox_paths);
        logs.push(LogInfo {
            log_type: "virtualbox".to_string(),
            label: "VirtualBox".to_string(),
            description: "虚拟机配置、日志".to_string(),
            size: virtualbox_size,
            size_display: format_size(virtualbox_size),
            file_count: virtualbox_count,
            accessible: virtualbox_count > 0,
            category: "虚拟化".to_string(),
        });
        total_size += virtualbox_size;
        total_files += virtualbox_count;

        // ========== 设计工具 ==========

        // 83. Figma
        let figma_paths = vec![
            format!("{}/Library/Application Support/Figma", home_dir),
            format!("{}/Library/Caches/com.figma.Desktop", home_dir),
        ];
        let (figma_size, figma_count) = scan_paths(&figma_paths);
        logs.push(LogInfo {
            log_type: "figma".to_string(),
            label: "Figma".to_string(),
            description: "设计缓存、本地文件".to_string(),
            size: figma_size,
            size_display: format_size(figma_size),
            file_count: figma_count,
            accessible: figma_count > 0,
            category: "设计工具".to_string(),
        });
        total_size += figma_size;
        total_files += figma_count;

        // 84. Sketch
        let sketch_paths = vec![
            format!("{}/Library/Application Support/com.bohemiancoding.sketch3", home_dir),
            format!("{}/Library/Caches/com.bohemiancoding.sketch3", home_dir),
        ];
        let (sketch_size, sketch_count) = scan_paths(&sketch_paths);
        logs.push(LogInfo {
            log_type: "sketch".to_string(),
            label: "Sketch".to_string(),
            description: "设计缓存、插件".to_string(),
            size: sketch_size,
            size_display: format_size(sketch_size),
            file_count: sketch_count,
            accessible: sketch_count > 0,
            category: "设计工具".to_string(),
        });
        total_size += sketch_size;
        total_files += sketch_count;

        // 85. Adobe Creative Cloud
        let adobe_paths = vec![
            format!("{}/Library/Application Support/Adobe", home_dir),
            format!("{}/Library/Caches/Adobe", home_dir),
            format!("{}/Library/Logs/Adobe", home_dir),
        ];
        let (adobe_size, adobe_count) = scan_paths(&adobe_paths);
        logs.push(LogInfo {
            log_type: "adobe".to_string(),
            label: "Adobe Creative Cloud".to_string(),
            description: "PS/AI/PR 缓存和配置".to_string(),
            size: adobe_size,
            size_display: format_size(adobe_size),
            file_count: adobe_count,
            accessible: adobe_count > 0,
            category: "设计工具".to_string(),
        });
        total_size += adobe_size;
        total_files += adobe_count;

        // ========== 笔记/文档工具 ==========

        // 86. Notion
        let notion_paths = vec![
            format!("{}/Library/Application Support/Notion", home_dir),
            format!("{}/Library/Caches/notion.id", home_dir),
        ];
        let (notion_size, notion_count) = scan_paths(&notion_paths);
        logs.push(LogInfo {
            log_type: "notion".to_string(),
            label: "Notion".to_string(),
            description: "笔记缓存、离线数据".to_string(),
            size: notion_size,
            size_display: format_size(notion_size),
            file_count: notion_count,
            accessible: notion_count > 0,
            category: "笔记工具".to_string(),
        });
        total_size += notion_size;
        total_files += notion_count;

        // 87. Obsidian
        let obsidian_paths = vec![
            format!("{}/Library/Application Support/obsidian", home_dir),
            format!("{}/Library/Caches/md.obsidian", home_dir),
        ];
        let (obsidian_size, obsidian_count) = scan_paths(&obsidian_paths);
        logs.push(LogInfo {
            log_type: "obsidian".to_string(),
            label: "Obsidian".to_string(),
            description: "笔记缓存、插件数据".to_string(),
            size: obsidian_size,
            size_display: format_size(obsidian_size),
            file_count: obsidian_count,
            accessible: obsidian_count > 0,
            category: "笔记工具".to_string(),
        });
        total_size += obsidian_size;
        total_files += obsidian_count;

        // 88. Evernote
        let evernote_paths = vec![
            format!("{}/Library/Application Support/Evernote", home_dir),
            format!("{}/Library/Containers/com.evernote.Evernote", home_dir),
        ];
        let (evernote_size, evernote_count) = scan_paths(&evernote_paths);
        logs.push(LogInfo {
            log_type: "evernote".to_string(),
            label: "Evernote".to_string(),
            description: "笔记缓存、离线数据".to_string(),
            size: evernote_size,
            size_display: format_size(evernote_size),
            file_count: evernote_count,
            accessible: evernote_count > 0,
            category: "笔记工具".to_string(),
        });
        total_size += evernote_size;
        total_files += evernote_count;

        // 89. Bear
        let bear_paths = vec![
            format!("{}/Library/Group Containers/9K33E3U3T4.net.shinyfrog.bear", home_dir),
            format!("{}/Library/Containers/net.shinyfrog.bear", home_dir),
        ];
        let (bear_size, bear_count) = scan_paths(&bear_paths);
        logs.push(LogInfo {
            log_type: "bear".to_string(),
            label: "Bear".to_string(),
            description: "笔记数据、附件".to_string(),
            size: bear_size,
            size_display: format_size(bear_size),
            file_count: bear_count,
            accessible: bear_count > 0,
            category: "笔记工具".to_string(),
        });
        total_size += bear_size;
        total_files += bear_count;

        // ========== 下载工具 ==========

        // 90. 迅雷
        let thunder_paths = vec![
            format!("{}/Library/Application Support/Thunder", home_dir),
            format!("{}/Library/Containers/com.xunlei.Thunder", home_dir),
        ];
        let (thunder_size, thunder_count) = scan_paths(&thunder_paths);
        logs.push(LogInfo {
            log_type: "thunder".to_string(),
            label: "迅雷".to_string(),
            description: "下载缓存、任务记录".to_string(),
            size: thunder_size,
            size_display: format_size(thunder_size),
            file_count: thunder_count,
            accessible: thunder_count > 0,
            category: "下载工具".to_string(),
        });
        total_size += thunder_size;
        total_files += thunder_count;

        // 91. Motrix
        let motrix_paths = vec![
            format!("{}/Library/Application Support/Motrix", home_dir),
            format!("{}/Library/Caches/Motrix", home_dir),
        ];
        let (motrix_size, motrix_count) = scan_paths(&motrix_paths);
        logs.push(LogInfo {
            log_type: "motrix".to_string(),
            label: "Motrix".to_string(),
            description: "下载缓存、任务记录".to_string(),
            size: motrix_size,
            size_display: format_size(motrix_size),
            file_count: motrix_count,
            accessible: motrix_count > 0,
            category: "下载工具".to_string(),
        });
        total_size += motrix_size;
        total_files += motrix_count;

        // ========== 其他常用软件 ==========

        // 92. CleanMyMac
        let cleanmymac_paths = vec![
            format!("{}/Library/Application Support/CleanMyMac X", home_dir),
            format!("{}/Library/Caches/com.macpaw.CleanMyMac4", home_dir),
            format!("{}/Library/Logs/CleanMyMac X", home_dir),
        ];
        let (cleanmymac_size, cleanmymac_count) = scan_paths(&cleanmymac_paths);
        logs.push(LogInfo {
            log_type: "cleanmymac".to_string(),
            label: "CleanMyMac".to_string(),
            description: "清理工具缓存、日志".to_string(),
            size: cleanmymac_size,
            size_display: format_size(cleanmymac_size),
            file_count: cleanmymac_count,
            accessible: cleanmymac_count > 0,
            category: "系统工具".to_string(),
        });
        total_size += cleanmymac_size;
        total_files += cleanmymac_count;

        // 93. Alfred
        let alfred_paths = vec![
            format!("{}/Library/Application Support/Alfred", home_dir),
            format!("{}/Library/Caches/com.runningwithcrayons.Alfred", home_dir),
        ];
        let (alfred_size, alfred_count) = scan_paths(&alfred_paths);
        logs.push(LogInfo {
            log_type: "alfred".to_string(),
            label: "Alfred".to_string(),
            description: "搜索缓存、工作流".to_string(),
            size: alfred_size,
            size_display: format_size(alfred_size),
            file_count: alfred_count,
            accessible: alfred_count > 0,
            category: "系统工具".to_string(),
        });
        total_size += alfred_size;
        total_files += alfred_count;

        // 94. Raycast
        let raycast_paths = vec![
            format!("{}/Library/Application Support/Raycast", home_dir),
            format!("{}/Library/Caches/com.raycast.macos", home_dir),
        ];
        let (raycast_size, raycast_count) = scan_paths(&raycast_paths);
        logs.push(LogInfo {
            log_type: "raycast".to_string(),
            label: "Raycast".to_string(),
            description: "搜索缓存、扩展数据".to_string(),
            size: raycast_size,
            size_display: format_size(raycast_size),
            file_count: raycast_count,
            accessible: raycast_count > 0,
            category: "系统工具".to_string(),
        });
        total_size += raycast_size;
        total_files += raycast_count;

        // 95. 1Password
        let onepassword_paths = vec![
            format!("{}/Library/Group Containers/2BUA8C4S2C.com.1password", home_dir),
            format!("{}/Library/Application Support/1Password", home_dir),
        ];
        let (onepassword_size, onepassword_count) = scan_paths(&onepassword_paths);
        logs.push(LogInfo {
            log_type: "onepassword".to_string(),
            label: "1Password".to_string(),
            description: "密码管理器缓存".to_string(),
            size: onepassword_size,
            size_display: format_size(onepassword_size),
            file_count: onepassword_count,
            accessible: onepassword_count > 0,
            category: "安全工具".to_string(),
        });
        total_size += onepassword_size;
        total_files += onepassword_count;

        // 96. Bitwarden
        let bitwarden_paths = vec![
            format!("{}/Library/Application Support/Bitwarden", home_dir),
            format!("{}/Library/Caches/com.bitwarden.desktop", home_dir),
        ];
        let (bitwarden_size, bitwarden_count) = scan_paths(&bitwarden_paths);
        logs.push(LogInfo {
            log_type: "bitwarden".to_string(),
            label: "Bitwarden".to_string(),
            description: "密码管理器缓存".to_string(),
            size: bitwarden_size,
            size_display: format_size(bitwarden_size),
            file_count: bitwarden_count,
            accessible: bitwarden_count > 0,
            category: "安全工具".to_string(),
        });
        total_size += bitwarden_size;
        total_files += bitwarden_count;

        // 97. Keychain (系统钥匙串缓存)
        let keychain_paths = vec![
            format!("{}/Library/Keychains", home_dir),
        ];
        let (keychain_size, keychain_count) = scan_paths(&keychain_paths);
        logs.push(LogInfo {
            log_type: "keychain".to_string(),
            label: "钥匙串".to_string(),
            description: "系统密码存储".to_string(),
            size: keychain_size,
            size_display: format_size(keychain_size),
            file_count: keychain_count,
            accessible: keychain_count > 0,
            category: "安全工具".to_string(),
        });
        total_size += keychain_size;
        total_files += keychain_count;

        // 98. VPN 工具 (Surge/ClashX等)
        let vpn_paths = vec![
            format!("{}/Library/Application Support/Surge", home_dir),
            format!("{}/Library/Application Support/com.west2online.ClashX", home_dir),
            format!("{}/Library/Application Support/com.west2online.ClashXPro", home_dir),
            format!("{}/.config/clash", home_dir),
            format!("{}/Library/Application Support/V2rayU", home_dir),
        ];
        let (vpn_size, vpn_count) = scan_paths(&vpn_paths);
        logs.push(LogInfo {
            log_type: "vpn_tools".to_string(),
            label: "VPN/代理工具".to_string(),
            description: "Surge/ClashX/V2Ray 配置".to_string(),
            size: vpn_size,
            size_display: format_size(vpn_size),
            file_count: vpn_count,
            accessible: vpn_count > 0,
            category: "网络工具".to_string(),
        });
        total_size += vpn_size;
        total_files += vpn_count;

        // 99. Terraform
        let terraform_paths = vec![
            format!("{}/.terraform.d", home_dir),
            format!("{}/Library/Caches/terraform-plugin-cache", home_dir),
        ];
        let (terraform_size, terraform_count) = scan_paths(&terraform_paths);
        logs.push(LogInfo {
            log_type: "terraform".to_string(),
            label: "Terraform".to_string(),
            description: "基础设施即代码工具缓存".to_string(),
            size: terraform_size,
            size_display: format_size(terraform_size),
            file_count: terraform_count,
            accessible: terraform_count > 0,
            category: "运维工具".to_string(),
        });
        total_size += terraform_size;
        total_files += terraform_count;

        // 100. Ansible
        let ansible_paths = vec![
            format!("{}/.ansible", home_dir),
        ];
        let (ansible_size, ansible_count) = scan_paths(&ansible_paths);
        logs.push(LogInfo {
            log_type: "ansible".to_string(),
            label: "Ansible".to_string(),
            description: "自动化运维工具缓存".to_string(),
            size: ansible_size,
            size_display: format_size(ansible_size),
            file_count: ansible_count,
            accessible: ansible_count > 0,
            category: "运维工具".to_string(),
        });
        total_size += ansible_size;
        total_files += ansible_count;

        // ========== 游戏平台 ==========

        // 101. Steam
        let steam_paths = vec![
            format!("{}/Library/Application Support/Steam", home_dir),
            format!("{}/Library/Caches/com.valvesoftware.steam", home_dir),
        ];
        let (steam_size, steam_count) = scan_paths(&steam_paths);
        logs.push(LogInfo {
            log_type: "steam".to_string(),
            label: "Steam".to_string(),
            description: "游戏平台缓存、下载".to_string(),
            size: steam_size,
            size_display: format_size(steam_size),
            file_count: steam_count,
            accessible: steam_count > 0,
            category: "游戏".to_string(),
        });
        total_size += steam_size;
        total_files += steam_count;

        // 102. Epic Games Launcher
        let epic_paths = vec![
            format!("{}/Library/Application Support/Epic", home_dir),
            format!("{}/Library/Caches/com.epicgames.EpicGamesLauncher", home_dir),
            format!("{}/Library/Logs/EpicGamesLauncher", home_dir),
        ];
        let (epic_size, epic_count) = scan_paths(&epic_paths);
        logs.push(LogInfo {
            log_type: "epic_games".to_string(),
            label: "Epic Games".to_string(),
            description: "Epic 游戏启动器缓存".to_string(),
            size: epic_size,
            size_display: format_size(epic_size),
            file_count: epic_count,
            accessible: epic_count > 0,
            category: "游戏".to_string(),
        });
        total_size += epic_size;
        total_files += epic_count;

        // 103. Battle.net
        let battlenet_paths = vec![
            format!("{}/Library/Application Support/Blizzard", home_dir),
            format!("{}/Library/Application Support/Battle.net", home_dir),
            format!("{}/Library/Caches/com.blizzard.bna", home_dir),
            "/Users/Shared/Blizzard".to_string(),
        ];
        let (battlenet_size, battlenet_count) = scan_paths(&battlenet_paths);
        logs.push(LogInfo {
            log_type: "battlenet".to_string(),
            label: "Battle.net".to_string(),
            description: "暴雪游戏平台缓存".to_string(),
            size: battlenet_size,
            size_display: format_size(battlenet_size),
            file_count: battlenet_count,
            accessible: battlenet_count > 0,
            category: "游戏".to_string(),
        });
        total_size += battlenet_size;
        total_files += battlenet_count;

        // 104. GOG Galaxy
        let gog_paths = vec![
            format!("{}/Library/Application Support/GOG.com", home_dir),
            format!("{}/Library/Caches/com.gog.galaxy", home_dir),
        ];
        let (gog_size, gog_count) = scan_paths(&gog_paths);
        logs.push(LogInfo {
            log_type: "gog".to_string(),
            label: "GOG Galaxy".to_string(),
            description: "GOG 游戏平台缓存".to_string(),
            size: gog_size,
            size_display: format_size(gog_size),
            file_count: gog_count,
            accessible: gog_count > 0,
            category: "游戏".to_string(),
        });
        total_size += gog_size;
        total_files += gog_count;

        // 105. Origin/EA App
        let origin_paths = vec![
            format!("{}/Library/Application Support/Origin", home_dir),
            format!("{}/Library/Application Support/Electronic Arts", home_dir),
            format!("{}/Library/Caches/com.ea.Origin", home_dir),
        ];
        let (origin_size, origin_count) = scan_paths(&origin_paths);
        logs.push(LogInfo {
            log_type: "origin".to_string(),
            label: "EA/Origin".to_string(),
            description: "EA 游戏平台缓存".to_string(),
            size: origin_size,
            size_display: format_size(origin_size),
            file_count: origin_count,
            accessible: origin_count > 0,
            category: "游戏".to_string(),
        });
        total_size += origin_size;
        total_files += origin_count;

        // ========== 视频编辑 ==========

        // 106. Final Cut Pro
        let finalcut_paths = vec![
            format!("{}/Movies/Final Cut Backups", home_dir),
            format!("{}/Library/Caches/com.apple.FinalCut", home_dir),
            format!("{}/Library/Application Support/Final Cut Pro", home_dir),
        ];
        let (finalcut_size, finalcut_count) = scan_paths(&finalcut_paths);
        logs.push(LogInfo {
            log_type: "finalcut".to_string(),
            label: "Final Cut Pro".to_string(),
            description: "视频编辑缓存、渲染文件".to_string(),
            size: finalcut_size,
            size_display: format_size(finalcut_size),
            file_count: finalcut_count,
            accessible: finalcut_count > 0,
            category: "视频编辑".to_string(),
        });
        total_size += finalcut_size;
        total_files += finalcut_count;

        // 107. DaVinci Resolve
        let davinci_paths = vec![
            format!("{}/Library/Application Support/Blackmagic Design", home_dir),
            format!("{}/Library/Caches/com.blackmagic-design.DaVinciResolve", home_dir),
            format!("{}/Movies/.gallery", home_dir),
            format!("{}/Movies/CacheClip", home_dir),
        ];
        let (davinci_size, davinci_count) = scan_paths(&davinci_paths);
        logs.push(LogInfo {
            log_type: "davinci".to_string(),
            label: "DaVinci Resolve".to_string(),
            description: "视频编辑缓存、项目数据".to_string(),
            size: davinci_size,
            size_display: format_size(davinci_size),
            file_count: davinci_count,
            accessible: davinci_count > 0,
            category: "视频编辑".to_string(),
        });
        total_size += davinci_size;
        total_files += davinci_count;

        // 108. iMovie
        let imovie_paths = vec![
            format!("{}/Library/Caches/com.apple.iMovieApp", home_dir),
            format!("{}/Library/Application Support/iMovie", home_dir),
            format!("{}/Movies/iMovie Library.imovielibrary", home_dir),
        ];
        let (imovie_size, imovie_count) = scan_paths(&imovie_paths);
        logs.push(LogInfo {
            log_type: "imovie".to_string(),
            label: "iMovie".to_string(),
            description: "视频编辑缓存".to_string(),
            size: imovie_size,
            size_display: format_size(imovie_size),
            file_count: imovie_count,
            accessible: imovie_count > 0,
            category: "视频编辑".to_string(),
        });
        total_size += imovie_size;
        total_files += imovie_count;

        // 109. ScreenFlow
        let screenflow_paths = vec![
            format!("{}/Library/Application Support/Telestream", home_dir),
            format!("{}/Library/Caches/net.telestream.screenflow10", home_dir),
        ];
        let (screenflow_size, screenflow_count) = scan_paths(&screenflow_paths);
        logs.push(LogInfo {
            log_type: "screenflow".to_string(),
            label: "ScreenFlow".to_string(),
            description: "屏幕录制软件缓存".to_string(),
            size: screenflow_size,
            size_display: format_size(screenflow_size),
            file_count: screenflow_count,
            accessible: screenflow_count > 0,
            category: "视频编辑".to_string(),
        });
        total_size += screenflow_size;
        total_files += screenflow_count;

        // 110. OBS Studio
        let obs_paths = vec![
            format!("{}/Library/Application Support/obs-studio", home_dir),
            format!("{}/Library/Caches/com.obsproject.obs-studio", home_dir),
        ];
        let (obs_size, obs_count) = scan_paths(&obs_paths);
        logs.push(LogInfo {
            log_type: "obs".to_string(),
            label: "OBS Studio".to_string(),
            description: "直播/录制软件缓存".to_string(),
            size: obs_size,
            size_display: format_size(obs_size),
            file_count: obs_count,
            accessible: obs_count > 0,
            category: "视频编辑".to_string(),
        });
        total_size += obs_size;
        total_files += obs_count;

        // ========== 音频编辑 ==========

        // 111. Logic Pro
        let logic_paths = vec![
            format!("{}/Library/Application Support/Logic", home_dir),
            format!("{}/Library/Caches/com.apple.logic10", home_dir),
            format!("{}/Music/Audio Music Apps", home_dir),
        ];
        let (logic_size, logic_count) = scan_paths(&logic_paths);
        logs.push(LogInfo {
            log_type: "logic".to_string(),
            label: "Logic Pro".to_string(),
            description: "音频编辑缓存、项目文件".to_string(),
            size: logic_size,
            size_display: format_size(logic_size),
            file_count: logic_count,
            accessible: logic_count > 0,
            category: "音频编辑".to_string(),
        });
        total_size += logic_size;
        total_files += logic_count;

        // 112. GarageBand
        let garageband_paths = vec![
            format!("{}/Library/Application Support/GarageBand", home_dir),
            format!("{}/Library/Caches/com.apple.garageband10", home_dir),
            format!("{}/Music/GarageBand", home_dir),
        ];
        let (garageband_size, garageband_count) = scan_paths(&garageband_paths);
        logs.push(LogInfo {
            log_type: "garageband".to_string(),
            label: "GarageBand".to_string(),
            description: "音乐创作软件缓存".to_string(),
            size: garageband_size,
            size_display: format_size(garageband_size),
            file_count: garageband_count,
            accessible: garageband_count > 0,
            category: "音频编辑".to_string(),
        });
        total_size += garageband_size;
        total_files += garageband_count;

        // 113. Audacity
        let audacity_paths = vec![
            format!("{}/Library/Application Support/audacity", home_dir),
            format!("{}/Library/Caches/audacity", home_dir),
        ];
        let (audacity_size, audacity_count) = scan_paths(&audacity_paths);
        logs.push(LogInfo {
            log_type: "audacity".to_string(),
            label: "Audacity".to_string(),
            description: "音频编辑软件缓存".to_string(),
            size: audacity_size,
            size_display: format_size(audacity_size),
            file_count: audacity_count,
            accessible: audacity_count > 0,
            category: "音频编辑".to_string(),
        });
        total_size += audacity_size;
        total_files += audacity_count;

        // ========== 邮件客户端 ==========

        // 114. Apple Mail
        let applemail_paths = vec![
            format!("{}/Library/Mail", home_dir),
            format!("{}/Library/Caches/com.apple.mail", home_dir),
            format!("{}/Library/Containers/com.apple.mail", home_dir),
        ];
        let (applemail_size, applemail_count) = scan_paths(&applemail_paths);
        logs.push(LogInfo {
            log_type: "apple_mail".to_string(),
            label: "Apple Mail".to_string(),
            description: "系统邮件缓存、附件".to_string(),
            size: applemail_size,
            size_display: format_size(applemail_size),
            file_count: applemail_count,
            accessible: applemail_count > 0,
            category: "邮件".to_string(),
        });
        total_size += applemail_size;
        total_files += applemail_count;

        // 115. Spark
        let spark_paths = vec![
            format!("{}/Library/Application Support/Spark", home_dir),
            format!("{}/Library/Caches/com.readdle.SparkDesktop", home_dir),
            format!("{}/Library/Group Containers/group.com.readdle.smartemail", home_dir),
        ];
        let (spark_size, spark_count) = scan_paths(&spark_paths);
        logs.push(LogInfo {
            log_type: "spark".to_string(),
            label: "Spark".to_string(),
            description: "邮件客户端缓存".to_string(),
            size: spark_size,
            size_display: format_size(spark_size),
            file_count: spark_count,
            accessible: spark_count > 0,
            category: "邮件".to_string(),
        });
        total_size += spark_size;
        total_files += spark_count;

        // 116. Airmail
        let airmail_paths = vec![
            format!("{}/Library/Application Support/Airmail", home_dir),
            format!("{}/Library/Containers/it.bloop.airmail2", home_dir),
            format!("{}/Library/Group Containers/2E337YPCZY.airmail", home_dir),
        ];
        let (airmail_size, airmail_count) = scan_paths(&airmail_paths);
        logs.push(LogInfo {
            log_type: "airmail".to_string(),
            label: "Airmail".to_string(),
            description: "邮件客户端缓存".to_string(),
            size: airmail_size,
            size_display: format_size(airmail_size),
            file_count: airmail_count,
            accessible: airmail_count > 0,
            category: "邮件".to_string(),
        });
        total_size += airmail_size;
        total_files += airmail_count;

        // 117. Mailspring
        let mailspring_paths = vec![
            format!("{}/Library/Application Support/Mailspring", home_dir),
            format!("{}/Library/Caches/Mailspring", home_dir),
        ];
        let (mailspring_size, mailspring_count) = scan_paths(&mailspring_paths);
        logs.push(LogInfo {
            log_type: "mailspring".to_string(),
            label: "Mailspring".to_string(),
            description: "邮件客户端缓存".to_string(),
            size: mailspring_size,
            size_display: format_size(mailspring_size),
            file_count: mailspring_count,
            accessible: mailspring_count > 0,
            category: "邮件".to_string(),
        });
        total_size += mailspring_size;
        total_files += mailspring_count;

        // 118. Outlook
        let outlook_paths = vec![
            format!("{}/Library/Containers/com.microsoft.Outlook", home_dir),
            format!("{}/Library/Group Containers/UBF8T346G9.Office", home_dir),
            format!("{}/Library/Caches/com.microsoft.Outlook", home_dir),
        ];
        let (outlook_size, outlook_count) = scan_paths(&outlook_paths);
        logs.push(LogInfo {
            log_type: "outlook".to_string(),
            label: "Microsoft Outlook".to_string(),
            description: "邮件客户端缓存".to_string(),
            size: outlook_size,
            size_display: format_size(outlook_size),
            file_count: outlook_count,
            accessible: outlook_count > 0,
            category: "邮件".to_string(),
        });
        total_size += outlook_size;
        total_files += outlook_count;

        // ========== 任务管理/待办 ==========

        // 119. Things
        let things_paths = vec![
            format!("{}/Library/Containers/com.culturedcode.ThingsMac", home_dir),
            format!("{}/Library/Group Containers/JLMPQHK86H.com.culturedcode.ThingsMac", home_dir),
        ];
        let (things_size, things_count) = scan_paths(&things_paths);
        logs.push(LogInfo {
            log_type: "things".to_string(),
            label: "Things".to_string(),
            description: "任务管理软件数据".to_string(),
            size: things_size,
            size_display: format_size(things_size),
            file_count: things_count,
            accessible: things_count > 0,
            category: "效率工具".to_string(),
        });
        total_size += things_size;
        total_files += things_count;

        // 120. Todoist
        let todoist_paths = vec![
            format!("{}/Library/Application Support/Todoist", home_dir),
            format!("{}/Library/Caches/com.todoist.mac.Todoist", home_dir),
        ];
        let (todoist_size, todoist_count) = scan_paths(&todoist_paths);
        logs.push(LogInfo {
            log_type: "todoist".to_string(),
            label: "Todoist".to_string(),
            description: "任务管理软件缓存".to_string(),
            size: todoist_size,
            size_display: format_size(todoist_size),
            file_count: todoist_count,
            accessible: todoist_count > 0,
            category: "效率工具".to_string(),
        });
        total_size += todoist_size;
        total_files += todoist_count;

        // 121. TickTick
        let ticktick_paths = vec![
            format!("{}/Library/Application Support/TickTick", home_dir),
            format!("{}/Library/Caches/com.TickTick.task.mac", home_dir),
        ];
        let (ticktick_size, ticktick_count) = scan_paths(&ticktick_paths);
        logs.push(LogInfo {
            log_type: "ticktick".to_string(),
            label: "TickTick".to_string(),
            description: "任务管理软件缓存".to_string(),
            size: ticktick_size,
            size_display: format_size(ticktick_size),
            file_count: ticktick_count,
            accessible: ticktick_count > 0,
            category: "效率工具".to_string(),
        });
        total_size += ticktick_size;
        total_files += ticktick_count;

        // 122. Apple Reminders
        let reminders_paths = vec![
            format!("{}/Library/Containers/com.apple.reminders", home_dir),
            format!("{}/Library/Caches/com.apple.remindd", home_dir),
        ];
        let (reminders_size, reminders_count) = scan_paths(&reminders_paths);
        logs.push(LogInfo {
            log_type: "reminders".to_string(),
            label: "提醒事项".to_string(),
            description: "Apple 提醒事项缓存".to_string(),
            size: reminders_size,
            size_display: format_size(reminders_size),
            file_count: reminders_count,
            accessible: reminders_count > 0,
            category: "效率工具".to_string(),
        });
        total_size += reminders_size;
        total_files += reminders_count;

        // ========== 终端替代品 ==========

        // 123. Warp
        let warp_paths = vec![
            format!("{}/Library/Application Support/dev.warp.Warp-Stable", home_dir),
            format!("{}/Library/Caches/dev.warp.Warp-Stable", home_dir),
            format!("{}/.warp", home_dir),
        ];
        let (warp_size, warp_count) = scan_paths(&warp_paths);
        logs.push(LogInfo {
            log_type: "warp".to_string(),
            label: "Warp".to_string(),
            description: "现代终端缓存".to_string(),
            size: warp_size,
            size_display: format_size(warp_size),
            file_count: warp_count,
            accessible: warp_count > 0,
            category: "终端".to_string(),
        });
        total_size += warp_size;
        total_files += warp_count;

        // 124. Hyper
        let hyper_paths = vec![
            format!("{}/Library/Application Support/Hyper", home_dir),
            format!("{}/Library/Caches/co.zeit.hyper", home_dir),
            format!("{}/.hyper.js", home_dir),
            format!("{}/.hyper_plugins", home_dir),
        ];
        let (hyper_size, hyper_count) = scan_paths(&hyper_paths);
        logs.push(LogInfo {
            log_type: "hyper".to_string(),
            label: "Hyper".to_string(),
            description: "Electron 终端缓存".to_string(),
            size: hyper_size,
            size_display: format_size(hyper_size),
            file_count: hyper_count,
            accessible: hyper_count > 0,
            category: "终端".to_string(),
        });
        total_size += hyper_size;
        total_files += hyper_count;

        // 125. Alacritty
        let alacritty_paths = vec![
            format!("{}/.config/alacritty", home_dir),
            format!("{}/Library/Caches/io.alacritty", home_dir),
        ];
        let (alacritty_size, alacritty_count) = scan_paths(&alacritty_paths);
        logs.push(LogInfo {
            log_type: "alacritty".to_string(),
            label: "Alacritty".to_string(),
            description: "GPU 终端配置".to_string(),
            size: alacritty_size,
            size_display: format_size(alacritty_size),
            file_count: alacritty_count,
            accessible: alacritty_count > 0,
            category: "终端".to_string(),
        });
        total_size += alacritty_size;
        total_files += alacritty_count;

        // 126. Kitty
        let kitty_paths = vec![
            format!("{}/.config/kitty", home_dir),
            format!("{}/Library/Caches/kitty", home_dir),
        ];
        let (kitty_size, kitty_count) = scan_paths(&kitty_paths);
        logs.push(LogInfo {
            log_type: "kitty".to_string(),
            label: "Kitty".to_string(),
            description: "GPU 终端配置".to_string(),
            size: kitty_size,
            size_display: format_size(kitty_size),
            file_count: kitty_count,
            accessible: kitty_count > 0,
            category: "终端".to_string(),
        });
        total_size += kitty_size;
        total_files += kitty_count;

        // ========== 日历 ==========

        // 127. Apple Calendar
        let calendar_paths = vec![
            format!("{}/Library/Calendars", home_dir),
            format!("{}/Library/Caches/com.apple.CalendarAgent", home_dir),
            format!("{}/Library/Containers/com.apple.iCal", home_dir),
        ];
        let (calendar_size, calendar_count) = scan_paths(&calendar_paths);
        logs.push(LogInfo {
            log_type: "calendar".to_string(),
            label: "日历".to_string(),
            description: "Apple 日历缓存".to_string(),
            size: calendar_size,
            size_display: format_size(calendar_size),
            file_count: calendar_count,
            accessible: calendar_count > 0,
            category: "效率工具".to_string(),
        });
        total_size += calendar_size;
        total_files += calendar_count;

        // 128. Fantastical
        let fantastical_paths = vec![
            format!("{}/Library/Containers/com.flexibits.fantastical2.mac", home_dir),
            format!("{}/Library/Group Containers/85C27NK92C.com.flexibits.fantastical2.mac", home_dir),
        ];
        let (fantastical_size, fantastical_count) = scan_paths(&fantastical_paths);
        logs.push(LogInfo {
            log_type: "fantastical".to_string(),
            label: "Fantastical".to_string(),
            description: "日历软件缓存".to_string(),
            size: fantastical_size,
            size_display: format_size(fantastical_size),
            file_count: fantastical_count,
            accessible: fantastical_count > 0,
            category: "效率工具".to_string(),
        });
        total_size += fantastical_size;
        total_files += fantastical_count;

        // ========== 照片/图片 ==========

        // 129. Apple Photos
        let photos_paths = vec![
            format!("{}/Library/Caches/com.apple.Photos", home_dir),
            format!("{}/Library/Containers/com.apple.Photos", home_dir),
        ];
        let (photos_size, photos_count) = scan_paths(&photos_paths);
        logs.push(LogInfo {
            log_type: "photos".to_string(),
            label: "照片".to_string(),
            description: "Apple 照片缓存".to_string(),
            size: photos_size,
            size_display: format_size(photos_size),
            file_count: photos_count,
            accessible: photos_count > 0,
            category: "多媒体".to_string(),
        });
        total_size += photos_size;
        total_files += photos_count;

        // 130. Lightroom
        let lightroom_paths = vec![
            format!("{}/Library/Application Support/Adobe/Lightroom", home_dir),
            format!("{}/Library/Caches/Adobe/Lightroom", home_dir),
        ];
        let (lightroom_size, lightroom_count) = scan_paths(&lightroom_paths);
        logs.push(LogInfo {
            log_type: "lightroom".to_string(),
            label: "Lightroom".to_string(),
            description: "照片编辑缓存".to_string(),
            size: lightroom_size,
            size_display: format_size(lightroom_size),
            file_count: lightroom_count,
            accessible: lightroom_count > 0,
            category: "设计工具".to_string(),
        });
        total_size += lightroom_size;
        total_files += lightroom_count;

        // 131. Pixelmator Pro
        let pixelmator_paths = vec![
            format!("{}/Library/Containers/com.pixelmatorteam.pixelmator.x", home_dir),
            format!("{}/Library/Caches/com.pixelmatorteam.pixelmator.x", home_dir),
        ];
        let (pixelmator_size, pixelmator_count) = scan_paths(&pixelmator_paths);
        logs.push(LogInfo {
            log_type: "pixelmator".to_string(),
            label: "Pixelmator Pro".to_string(),
            description: "图片编辑缓存".to_string(),
            size: pixelmator_size,
            size_display: format_size(pixelmator_size),
            file_count: pixelmator_count,
            accessible: pixelmator_count > 0,
            category: "设计工具".to_string(),
        });
        total_size += pixelmator_size;
        total_files += pixelmator_count;

        // ========== 压缩工具 ==========

        // 132. The Unarchiver
        let unarchiver_paths = vec![
            format!("{}/Library/Containers/cx.c3.theunarchiver", home_dir),
            format!("{}/Library/Caches/cx.c3.theunarchiver", home_dir),
        ];
        let (unarchiver_size, unarchiver_count) = scan_paths(&unarchiver_paths);
        logs.push(LogInfo {
            log_type: "unarchiver".to_string(),
            label: "The Unarchiver".to_string(),
            description: "解压工具缓存".to_string(),
            size: unarchiver_size,
            size_display: format_size(unarchiver_size),
            file_count: unarchiver_count,
            accessible: unarchiver_count > 0,
            category: "系统工具".to_string(),
        });
        total_size += unarchiver_size;
        total_files += unarchiver_count;

        // 133. Keka
        let keka_paths = vec![
            format!("{}/Library/Application Support/Keka", home_dir),
            format!("{}/Library/Caches/com.aone.keka", home_dir),
        ];
        let (keka_size, keka_count) = scan_paths(&keka_paths);
        logs.push(LogInfo {
            log_type: "keka".to_string(),
            label: "Keka".to_string(),
            description: "压缩工具缓存".to_string(),
            size: keka_size,
            size_display: format_size(keka_size),
            file_count: keka_count,
            accessible: keka_count > 0,
            category: "系统工具".to_string(),
        });
        total_size += keka_size;
        total_files += keka_count;

        // ========== 截图工具 ==========

        // 134. CleanShot X
        let cleanshot_paths = vec![
            format!("{}/Library/Application Support/CleanShot", home_dir),
            format!("{}/Library/Caches/pl.maketheweb.cleanshotx", home_dir),
        ];
        let (cleanshot_size, cleanshot_count) = scan_paths(&cleanshot_paths);
        logs.push(LogInfo {
            log_type: "cleanshot".to_string(),
            label: "CleanShot X".to_string(),
            description: "截图工具缓存".to_string(),
            size: cleanshot_size,
            size_display: format_size(cleanshot_size),
            file_count: cleanshot_count,
            accessible: cleanshot_count > 0,
            category: "效率工具".to_string(),
        });
        total_size += cleanshot_size;
        total_files += cleanshot_count;

        // 135. Snagit
        let snagit_paths = vec![
            format!("{}/Library/Application Support/TechSmith/Snagit", home_dir),
            format!("{}/Library/Caches/com.TechSmith.Snagit", home_dir),
        ];
        let (snagit_size, snagit_count) = scan_paths(&snagit_paths);
        logs.push(LogInfo {
            log_type: "snagit".to_string(),
            label: "Snagit".to_string(),
            description: "截图工具缓存".to_string(),
            size: snagit_size,
            size_display: format_size(snagit_size),
            file_count: snagit_count,
            accessible: snagit_count > 0,
            category: "效率工具".to_string(),
        });
        total_size += snagit_size;
        total_files += snagit_count;

        // ========== 翻译工具 ==========

        // 136. DeepL
        let deepl_paths = vec![
            format!("{}/Library/Application Support/DeepL", home_dir),
            format!("{}/Library/Caches/com.linguee.DeepLCopyTranslator", home_dir),
        ];
        let (deepl_size, deepl_count) = scan_paths(&deepl_paths);
        logs.push(LogInfo {
            log_type: "deepl".to_string(),
            label: "DeepL".to_string(),
            description: "翻译工具缓存".to_string(),
            size: deepl_size,
            size_display: format_size(deepl_size),
            file_count: deepl_count,
            accessible: deepl_count > 0,
            category: "效率工具".to_string(),
        });
        total_size += deepl_size;
        total_files += deepl_count;

        // 137. 欧路词典
        let eudic_paths = vec![
            format!("{}/Library/Application Support/Eudb", home_dir),
            format!("{}/Library/Caches/com.eusoft.eudic", home_dir),
        ];
        let (eudic_size, eudic_count) = scan_paths(&eudic_paths);
        logs.push(LogInfo {
            log_type: "eudic".to_string(),
            label: "欧路词典".to_string(),
            description: "词典软件缓存".to_string(),
            size: eudic_size,
            size_display: format_size(eudic_size),
            file_count: eudic_count,
            accessible: eudic_count > 0,
            category: "效率工具".to_string(),
        });
        total_size += eudic_size;
        total_files += eudic_count;

        // ========== 阅读器 ==========

        // 138. Kindle
        let kindle_paths = vec![
            format!("{}/Library/Application Support/Kindle", home_dir),
            format!("{}/Library/Containers/com.amazon.Kindle", home_dir),
        ];
        let (kindle_size, kindle_count) = scan_paths(&kindle_paths);
        logs.push(LogInfo {
            log_type: "kindle".to_string(),
            label: "Kindle".to_string(),
            description: "电子书缓存".to_string(),
            size: kindle_size,
            size_display: format_size(kindle_size),
            file_count: kindle_count,
            accessible: kindle_count > 0,
            category: "效率工具".to_string(),
        });
        total_size += kindle_size;
        total_files += kindle_count;

        // 139. PDF Expert
        let pdfexpert_paths = vec![
            format!("{}/Library/Containers/com.readdle.PDFExpert-Mac", home_dir),
            format!("{}/Library/Caches/com.readdle.PDFExpert-Mac", home_dir),
        ];
        let (pdfexpert_size, pdfexpert_count) = scan_paths(&pdfexpert_paths);
        logs.push(LogInfo {
            log_type: "pdfexpert".to_string(),
            label: "PDF Expert".to_string(),
            description: "PDF 阅读器缓存".to_string(),
            size: pdfexpert_size,
            size_display: format_size(pdfexpert_size),
            file_count: pdfexpert_count,
            accessible: pdfexpert_count > 0,
            category: "效率工具".to_string(),
        });
        total_size += pdfexpert_size;
        total_files += pdfexpert_count;

        // 140. MarginNote
        let marginnote_paths = vec![
            format!("{}/Library/Containers/QReader.MarginStudyMac", home_dir),
            format!("{}/Library/Application Support/MarginNote 3", home_dir),
        ];
        let (marginnote_size, marginnote_count) = scan_paths(&marginnote_paths);
        logs.push(LogInfo {
            log_type: "marginnote".to_string(),
            label: "MarginNote".to_string(),
            description: "阅读笔记软件缓存".to_string(),
            size: marginnote_size,
            size_display: format_size(marginnote_size),
            file_count: marginnote_count,
            accessible: marginnote_count > 0,
            category: "笔记工具".to_string(),
        });
        total_size += marginnote_size;
        total_files += marginnote_count;

        // ========== 剪贴板管理 ==========

        // 141. Paste
        let paste_paths = vec![
            format!("{}/Library/Containers/com.wiheads.paste", home_dir),
            format!("{}/Library/Application Support/Paste", home_dir),
        ];
        let (paste_size, paste_count) = scan_paths(&paste_paths);
        logs.push(LogInfo {
            log_type: "paste".to_string(),
            label: "Paste".to_string(),
            description: "剪贴板管理器缓存".to_string(),
            size: paste_size,
            size_display: format_size(paste_size),
            file_count: paste_count,
            accessible: paste_count > 0,
            category: "效率工具".to_string(),
        });
        total_size += paste_size;
        total_files += paste_count;

        // 142. Maccy
        let maccy_paths = vec![
            format!("{}/Library/Containers/org.p0deje.Maccy", home_dir),
            format!("{}/Library/Caches/org.p0deje.Maccy", home_dir),
        ];
        let (maccy_size, maccy_count) = scan_paths(&maccy_paths);
        logs.push(LogInfo {
            log_type: "maccy".to_string(),
            label: "Maccy".to_string(),
            description: "剪贴板管理器缓存".to_string(),
            size: maccy_size,
            size_display: format_size(maccy_size),
            file_count: maccy_count,
            accessible: maccy_count > 0,
            category: "效率工具".to_string(),
        });
        total_size += maccy_size;
        total_files += maccy_count;

        // ========== 窗口管理 ==========

        // 143. Rectangle
        let rectangle_paths = vec![
            format!("{}/Library/Application Support/Rectangle", home_dir),
            format!("{}/Library/Caches/com.knollsoft.Rectangle", home_dir),
        ];
        let (rectangle_size, rectangle_count) = scan_paths(&rectangle_paths);
        logs.push(LogInfo {
            log_type: "rectangle".to_string(),
            label: "Rectangle".to_string(),
            description: "窗口管理器缓存".to_string(),
            size: rectangle_size,
            size_display: format_size(rectangle_size),
            file_count: rectangle_count,
            accessible: rectangle_count > 0,
            category: "系统工具".to_string(),
        });
        total_size += rectangle_size;
        total_files += rectangle_count;

        // 144. Magnet
        let magnet_paths = vec![
            format!("{}/Library/Containers/id.arunro.magnet", home_dir),
            format!("{}/Library/Caches/id.arunro.magnet", home_dir),
        ];
        let (magnet_size, magnet_count) = scan_paths(&magnet_paths);
        logs.push(LogInfo {
            log_type: "magnet".to_string(),
            label: "Magnet".to_string(),
            description: "窗口管理器缓存".to_string(),
            size: magnet_size,
            size_display: format_size(magnet_size),
            file_count: magnet_count,
            accessible: magnet_count > 0,
            category: "系统工具".to_string(),
        });
        total_size += magnet_size;
        total_files += magnet_count;

        // ========== 输入法 ==========

        // 145. 搜狗输入法
        let sogou_paths = vec![
            format!("{}/Library/Application Support/Sogou", home_dir),
            format!("{}/Library/Caches/com.sogou.inputmethod.sogou", home_dir),
        ];
        let (sogou_size, sogou_count) = scan_paths(&sogou_paths);
        logs.push(LogInfo {
            log_type: "sogou".to_string(),
            label: "搜狗输入法".to_string(),
            description: "输入法缓存、词库".to_string(),
            size: sogou_size,
            size_display: format_size(sogou_size),
            file_count: sogou_count,
            accessible: sogou_count > 0,
            category: "系统工具".to_string(),
        });
        total_size += sogou_size;
        total_files += sogou_count;

        // 146. 百度输入法
        let baiduinput_paths = vec![
            format!("{}/Library/Application Support/com.baidu.inputmethod.BaiduIM", home_dir),
            format!("{}/Library/Caches/com.baidu.inputmethod.BaiduIM", home_dir),
        ];
        let (baiduinput_size, baiduinput_count) = scan_paths(&baiduinput_paths);
        logs.push(LogInfo {
            log_type: "baidu_input".to_string(),
            label: "百度输入法".to_string(),
            description: "输入法缓存、词库".to_string(),
            size: baiduinput_size,
            size_display: format_size(baiduinput_size),
            file_count: baiduinput_count,
            accessible: baiduinput_count > 0,
            category: "系统工具".to_string(),
        });
        total_size += baiduinput_size;
        total_files += baiduinput_count;

        // ========== 社交媒体 ==========

        // 147. Twitter/X
        let twitter_paths = vec![
            format!("{}/Library/Containers/com.twitter.twitter-mac", home_dir),
            format!("{}/Library/Caches/com.twitter.twitter-mac", home_dir),
        ];
        let (twitter_size, twitter_count) = scan_paths(&twitter_paths);
        logs.push(LogInfo {
            log_type: "twitter".to_string(),
            label: "Twitter/X".to_string(),
            description: "社交媒体缓存".to_string(),
            size: twitter_size,
            size_display: format_size(twitter_size),
            file_count: twitter_count,
            accessible: twitter_count > 0,
            category: "通讯软件".to_string(),
        });
        total_size += twitter_size;
        total_files += twitter_count;

        // 148. 微博
        let weibo_paths = vec![
            format!("{}/Library/Containers/com.sina.weibo", home_dir),
            format!("{}/Library/Application Support/Weibo", home_dir),
        ];
        let (weibo_size, weibo_count) = scan_paths(&weibo_paths);
        logs.push(LogInfo {
            log_type: "weibo".to_string(),
            label: "微博".to_string(),
            description: "社交媒体缓存".to_string(),
            size: weibo_size,
            size_display: format_size(weibo_size),
            file_count: weibo_count,
            accessible: weibo_count > 0,
            category: "通讯软件".to_string(),
        });
        total_size += weibo_size;
        total_files += weibo_count;

        // 149. 小红书
        let xiaohongshu_paths = vec![
            format!("{}/Library/Containers/com.xingin.xiaohongshu", home_dir),
            format!("{}/Library/Caches/com.xingin.xiaohongshu", home_dir),
        ];
        let (xiaohongshu_size, xiaohongshu_count) = scan_paths(&xiaohongshu_paths);
        logs.push(LogInfo {
            log_type: "xiaohongshu".to_string(),
            label: "小红书".to_string(),
            description: "社交媒体缓存".to_string(),
            size: xiaohongshu_size,
            size_display: format_size(xiaohongshu_size),
            file_count: xiaohongshu_count,
            accessible: xiaohongshu_count > 0,
            category: "通讯软件".to_string(),
        });
        total_size += xiaohongshu_size;
        total_files += xiaohongshu_count;

        // 150. 抖音
        let douyin_paths = vec![
            format!("{}/Library/Containers/com.ss.mac.douyinlite", home_dir),
            format!("{}/Library/Application Support/Douyin", home_dir),
        ];
        let (douyin_size, douyin_count) = scan_paths(&douyin_paths);
        logs.push(LogInfo {
            log_type: "douyin".to_string(),
            label: "抖音".to_string(),
            description: "短视频缓存".to_string(),
            size: douyin_size,
            size_display: format_size(douyin_size),
            file_count: douyin_count,
            accessible: douyin_count > 0,
            category: "多媒体".to_string(),
        });
        total_size += douyin_size;
        total_files += douyin_count;

        let permission_guide = r#"macOS 权限设置指南：

1. 完全磁盘访问权限（推荐）：
   • 打开「系统设置」→「隐私与安全性」→「完全磁盘访问权限」
   • 点击「+」添加本应用
   • 重启应用生效

2. 可清理内容类别：
   【系统】系统日志、应用缓存、崩溃报告、安装记录
   【浏览器】Safari、Chrome、Firefox、Edge 的历史和缓存
   【开发工具】VS Code、Xcode、JetBrains、Git、NPM、Python
   【办公软件】Microsoft Office 缓存
   【通讯软件】微信、QQ、Telegram、Discord、Slack
   【多媒体】Spotify、VLC 等播放器数据
   【安全工具】Burp Suite、Wireshark、Metasploit
   【终端】Shell 历史、SSH 记录、iTerm2
   【运维】Docker、Homebrew、Kubernetes、AWS

3. 需要 sudo 权限才能清理的内容：
   • /var/log - 系统日志（需要管理员密码）
   • 统一日志系统 - 需要终端执行 sudo log erase --all"#;

        // 过滤掉没有文件的条目
        logs.retain(|log| log.file_count > 0);

        Ok(LogScanResult {
            logs,
            total_size,
            total_files,
            needs_permission: false,
            permission_guide: permission_guide.to_string(),
        })
    }

    #[cfg(target_os = "windows")]
    {
        let home_dir = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users".to_string());
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| format!("{}\\AppData\\Roaming", home_dir));
        let localappdata = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| format!("{}\\AppData\\Local", home_dir));

        // Windows 事件日志
        logs.push(LogInfo {
            log_type: "Application".to_string(),
            label: "应用程序日志".to_string(),
            description: "Windows 应用程序事件日志".to_string(),
            size: 0,
            size_display: "需要管理员权限".to_string(),
            file_count: 0,
            accessible: false,
            category: "系统".to_string(),
        });

        logs.push(LogInfo {
            log_type: "System".to_string(),
            label: "系统日志".to_string(),
            description: "Windows 系统事件日志".to_string(),
            size: 0,
            size_display: "需要管理员权限".to_string(),
            file_count: 0,
            accessible: false,
            category: "系统".to_string(),
        });

        logs.push(LogInfo {
            log_type: "Security".to_string(),
            label: "安全日志".to_string(),
            description: "安全审计和身份验证日志".to_string(),
            size: 0,
            size_display: "需要管理员权限".to_string(),
            file_count: 0,
            accessible: false,
            category: "系统".to_string(),
        });

        // 临时文件
        let temp_paths = vec![
            format!("{}\\Temp", localappdata),
            std::env::var("TEMP").unwrap_or_else(|_| format!("{}\\Temp", localappdata)),
        ];
        let (temp_size, temp_count) = scan_paths(&temp_paths);
        logs.push(LogInfo {
            log_type: "temp".to_string(),
            label: "临时文件".to_string(),
            description: "系统和应用临时文件".to_string(),
            size: temp_size,
            size_display: format_size(temp_size),
            file_count: temp_count,
            accessible: temp_count > 0,
            category: "系统".to_string(),
        });
        total_size += temp_size;
        total_files += temp_count;

        // 最近使用
        let recent_paths = vec![
            format!("{}\\Microsoft\\Windows\\Recent", appdata),
        ];
        let (recent_size, recent_count) = scan_paths(&recent_paths);
        logs.push(LogInfo {
            log_type: "recent_items".to_string(),
            label: "最近使用".to_string(),
            description: "最近打开的文件记录".to_string(),
            size: recent_size,
            size_display: format_size(recent_size),
            file_count: recent_count,
            accessible: recent_count > 0,
            category: "系统".to_string(),
        });
        total_size += recent_size;
        total_files += recent_count;

        // Chrome
        let chrome_paths = vec![
            format!("{}\\Google\\Chrome\\User Data", localappdata),
        ];
        let (chrome_size, chrome_count) = scan_paths(&chrome_paths);
        logs.push(LogInfo {
            log_type: "chrome".to_string(),
            label: "Chrome".to_string(),
            description: "浏览历史、缓存、Cookie".to_string(),
            size: chrome_size,
            size_display: format_size(chrome_size),
            file_count: chrome_count,
            accessible: chrome_count > 0,
            category: "浏览器".to_string(),
        });
        total_size += chrome_size;
        total_files += chrome_count;

        // Firefox
        let firefox_paths = vec![
            format!("{}\\Mozilla\\Firefox\\Profiles", appdata),
        ];
        let (firefox_size, firefox_count) = scan_paths(&firefox_paths);
        logs.push(LogInfo {
            log_type: "firefox".to_string(),
            label: "Firefox".to_string(),
            description: "浏览历史、缓存、Cookie".to_string(),
            size: firefox_size,
            size_display: format_size(firefox_size),
            file_count: firefox_count,
            accessible: firefox_count > 0,
            category: "浏览器".to_string(),
        });
        total_size += firefox_size;
        total_files += firefox_count;

        // Edge
        let edge_paths = vec![
            format!("{}\\Microsoft\\Edge\\User Data", localappdata),
        ];
        let (edge_size, edge_count) = scan_paths(&edge_paths);
        logs.push(LogInfo {
            log_type: "edge".to_string(),
            label: "Edge".to_string(),
            description: "浏览历史、缓存、Cookie".to_string(),
            size: edge_size,
            size_display: format_size(edge_size),
            file_count: edge_count,
            accessible: edge_count > 0,
            category: "浏览器".to_string(),
        });
        total_size += edge_size;
        total_files += edge_count;

        // VS Code
        let vscode_paths = vec![
            format!("{}\\Code", appdata),
            format!("{}\\Code", localappdata),
        ];
        let (vscode_size, vscode_count) = scan_paths(&vscode_paths);
        logs.push(LogInfo {
            log_type: "vscode".to_string(),
            label: "VS Code".to_string(),
            description: "编辑器缓存、日志".to_string(),
            size: vscode_size,
            size_display: format_size(vscode_size),
            file_count: vscode_count,
            accessible: vscode_count > 0,
            category: "开发工具".to_string(),
        });
        total_size += vscode_size;
        total_files += vscode_count;

        // 回收站（需要管理员权限访问全部）
        logs.push(LogInfo {
            log_type: "trash".to_string(),
            label: "回收站".to_string(),
            description: "已删除的文件".to_string(),
            size: 0,
            size_display: "使用系统工具清理".to_string(),
            file_count: 0,
            accessible: false,
            category: "文件".to_string(),
        });

        // 过滤掉没有文件的条目
        logs.retain(|log| log.file_count > 0 || !log.accessible);

        let permission_guide = r#"Windows 权限设置指南：

1. 以管理员身份运行：
   • 右键点击应用程序
   • 选择「以管理员身份运行」

2. 可清理内容类别：
   【系统】事件日志、临时文件、最近使用记录
   【浏览器】Chrome、Firefox、Edge 的历史和缓存
   【开发工具】VS Code 等 IDE 数据

3. 清理系统日志需要管理员权限"#;

        Ok(LogScanResult {
            logs,
            total_size,
            total_files,
            needs_permission: true,
            permission_guide: permission_guide.to_string(),
        })
    }

    #[cfg(target_os = "linux")]
    {
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());

        // 系统日志
        let syslog_paths = vec![
            format!("{}/.local/share/systemd", home_dir),
            format!("{}/.xsession-errors", home_dir),
        ];
        let (syslog_size, syslog_count) = scan_paths(&syslog_paths);
        logs.push(LogInfo {
            log_type: "syslog".to_string(),
            label: "用户日志".to_string(),
            description: "用户会话和服务日志".to_string(),
            size: syslog_size,
            size_display: format_size(syslog_size),
            file_count: syslog_count,
            accessible: syslog_count > 0,
            category: "系统".to_string(),
        });
        total_size += syslog_size;
        total_files += syslog_count;

        // 缓存
        let cache_paths = vec![
            format!("{}/.cache", home_dir),
        ];
        let (cache_size, cache_count) = scan_paths(&cache_paths);
        logs.push(LogInfo {
            log_type: "app_cache".to_string(),
            label: "应用缓存".to_string(),
            description: "用户应用程序缓存".to_string(),
            size: cache_size,
            size_display: format_size(cache_size),
            file_count: cache_count,
            accessible: cache_count > 0,
            category: "系统".to_string(),
        });
        total_size += cache_size;
        total_files += cache_count;

        // 最近使用
        let recent_paths = vec![
            format!("{}/.local/share/recently-used.xbel", home_dir),
        ];
        let (recent_size, recent_count) = scan_paths(&recent_paths);
        logs.push(LogInfo {
            log_type: "recent_items".to_string(),
            label: "最近使用".to_string(),
            description: "最近打开的文件记录".to_string(),
            size: recent_size,
            size_display: format_size(recent_size),
            file_count: recent_count,
            accessible: recent_count > 0,
            category: "系统".to_string(),
        });
        total_size += recent_size;
        total_files += recent_count;

        // Chrome
        let chrome_paths = vec![
            format!("{}/.config/google-chrome", home_dir),
            format!("{}/.cache/google-chrome", home_dir),
        ];
        let (chrome_size, chrome_count) = scan_paths(&chrome_paths);
        logs.push(LogInfo {
            log_type: "chrome".to_string(),
            label: "Chrome".to_string(),
            description: "浏览历史、缓存、Cookie".to_string(),
            size: chrome_size,
            size_display: format_size(chrome_size),
            file_count: chrome_count,
            accessible: chrome_count > 0,
            category: "浏览器".to_string(),
        });
        total_size += chrome_size;
        total_files += chrome_count;

        // Firefox
        let firefox_paths = vec![
            format!("{}/.mozilla/firefox", home_dir),
            format!("{}/.cache/mozilla/firefox", home_dir),
        ];
        let (firefox_size, firefox_count) = scan_paths(&firefox_paths);
        logs.push(LogInfo {
            log_type: "firefox".to_string(),
            label: "Firefox".to_string(),
            description: "浏览历史、缓存、Cookie".to_string(),
            size: firefox_size,
            size_display: format_size(firefox_size),
            file_count: firefox_count,
            accessible: firefox_count > 0,
            category: "浏览器".to_string(),
        });
        total_size += firefox_size;
        total_files += firefox_count;

        // VS Code
        let vscode_paths = vec![
            format!("{}/.config/Code", home_dir),
            format!("{}/.cache/Code", home_dir),
        ];
        let (vscode_size, vscode_count) = scan_paths(&vscode_paths);
        logs.push(LogInfo {
            log_type: "vscode".to_string(),
            label: "VS Code".to_string(),
            description: "编辑器缓存、日志".to_string(),
            size: vscode_size,
            size_display: format_size(vscode_size),
            file_count: vscode_count,
            accessible: vscode_count > 0,
            category: "开发工具".to_string(),
        });
        total_size += vscode_size;
        total_files += vscode_count;

        // Shell 历史
        let shell_paths = vec![
            format!("{}/.bash_history", home_dir),
            format!("{}/.zsh_history", home_dir),
        ];
        let (shell_size, shell_count) = scan_paths(&shell_paths);
        logs.push(LogInfo {
            log_type: "shell_history".to_string(),
            label: "Shell历史".to_string(),
            description: "命令行历史记录".to_string(),
            size: shell_size,
            size_display: format_size(shell_size),
            file_count: shell_count,
            accessible: shell_count > 0,
            category: "终端".to_string(),
        });
        total_size += shell_size;
        total_files += shell_count;

        // 废纸篓
        let trash_paths = vec![
            format!("{}/.local/share/Trash", home_dir),
        ];
        let (trash_size, trash_count) = scan_paths(&trash_paths);
        logs.push(LogInfo {
            log_type: "trash".to_string(),
            label: "废纸篓".to_string(),
            description: "已删除的文件".to_string(),
            size: trash_size,
            size_display: format_size(trash_size),
            file_count: trash_count,
            accessible: trash_count > 0,
            category: "文件".to_string(),
        });
        total_size += trash_size;
        total_files += trash_count;

        // 过滤掉没有文件的条目
        logs.retain(|log| log.file_count > 0);

        let permission_guide = r#"Linux 权限设置指南：

1. 用户级日志（无需额外权限）：
   • ~/.cache - 应用缓存
   • ~/.local/share - 应用数据
   • ~/.config - 应用配置

2. 系统级日志（需要 root 权限）：
   sudo truncate -s 0 /var/log/syslog
   sudo truncate -s 0 /var/log/auth.log
   sudo journalctl --vacuum-time=1d"#;

        Ok(LogScanResult {
            logs,
            total_size,
            total_files,
            needs_permission: false,
            permission_guide: permission_guide.to_string(),
        })
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Err("PLATFORM_NOT_SUPPORTED".to_string())
    }
}

#[tauri::command]
pub async fn clear_system_logs(log_types: Vec<String>) -> Result<String, String> {
    // Check if any log types were selected
    if log_types.is_empty() {
        return Err("INVALID_INPUT:selectLogTypes".to_string());
    }

    // Check permission - system logs cleanup requires elevation
    require_admin_for_operation("system_logs")?;

    // Clear logs
    match log_cleaner::clear_system_logs(log_types.clone()) {
        Ok(cleared) => {
            Ok(format!(
                "OK:{}:{}",
                cleared.len(),
                cleared.join(",")
            ))
        }
        Err(e) => {
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn get_platform() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    return Ok("windows".to_string());

    #[cfg(target_os = "linux")]
    return Ok("linux".to_string());

    #[cfg(target_os = "macos")]
    return Ok("macos".to_string());

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    return Ok("unknown".to_string());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_platform() {
        let result = get_platform().await;
        assert!(result.is_ok());
        #[cfg(target_os = "windows")]
        assert_eq!(result.unwrap(), "windows");
        #[cfg(target_os = "linux")]
        assert_eq!(result.unwrap(), "linux");
        #[cfg(target_os = "macos")]
        assert_eq!(result.unwrap(), "macos");
    }

    #[tokio::test]
    async fn test_clear_system_logs_empty() {
        let result = clear_system_logs(vec![]).await;
        assert!(result.is_err());
    }
}
