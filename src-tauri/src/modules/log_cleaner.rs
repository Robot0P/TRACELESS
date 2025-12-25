use std::process::Command;
use std::path::Path;
use std::fs;

/// 清理 Windows 事件日志
#[cfg(target_os = "windows")]
pub fn clear_windows_event_log(log_name: &str) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::core::{PCWSTR, PWSTR};
    use windows::Win32::System::EventLog::{ClearEventLogW, CloseEventLog, OpenEventLogW};

    unsafe {
        // 将字符串转换为宽字符
        let log_name_wide: Vec<u16> = OsStr::new(log_name)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        // 打开事件日志
        let handle = OpenEventLogW(
            PCWSTR::null(),
            PCWSTR::from_raw(log_name_wide.as_ptr()),
        )
        .map_err(|e| format!("打开日志 {} 失败: {}", log_name, e))?;

        // 清理事件日志
        ClearEventLogW(handle, PCWSTR::null())
            .map_err(|e| format!("清理日志 {} 失败: {}", log_name, e))?;

        // 关闭日志句柄
        CloseEventLog(handle)
            .map_err(|e| format!("关闭日志 {} 失败: {}", log_name, e))?;

        Ok(())
    }
}

/// 递归删除目录中的所有文件
fn clear_directory_contents(dir_path: &str) -> Result<usize, String> {
    let path = Path::new(dir_path);
    if !path.exists() {
        return Ok(0);
    }

    let mut count = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                if let Ok(n) = clear_directory_contents(&entry_path.to_string_lossy()) {
                    count += n;
                }
                let _ = fs::remove_dir(&entry_path);
            } else {
                if fs::remove_file(&entry_path).is_ok() {
                    count += 1;
                }
            }
        }
    }
    Ok(count)
}

/// 清空文件内容（而不是删除文件）
fn truncate_file(file_path: &str) -> Result<(), String> {
    let path = Path::new(file_path);
    if path.exists() && path.is_file() {
        fs::write(path, "").map_err(|e| format!("清空文件失败: {}", e))?;
    }
    Ok(())
}

/// 清理多个路径
fn clear_paths(paths: &[String]) -> Result<usize, String> {
    let mut total_count = 0;

    for path in paths {
        let p = Path::new(path);
        if p.exists() {
            if p.is_dir() {
                if let Ok(count) = clear_directory_contents(path) {
                    total_count += count;
                }
            } else if p.is_file() {
                if fs::remove_file(p).is_ok() {
                    total_count += 1;
                }
            }
        }
    }

    Ok(total_count)
}

/// 清理 macOS 日志
#[cfg(target_os = "macos")]
pub fn clear_macos_log(log_type: &str) -> Result<(), String> {
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/Users".to_string());

    let paths: Vec<String> = match log_type {
        // ========== 系统类别 ==========
        "syslog" => vec![
            format!("{}/Library/Logs", home_dir),
            "/var/log".to_string(),
            "/Library/Logs".to_string(),
        ],
        "app_cache" => vec![
            format!("{}/Library/Caches", home_dir),
        ],
        "crash_logs" => vec![
            format!("{}/Library/Logs/DiagnosticReports", home_dir),
            format!("{}/Library/Logs/CrashReporter", home_dir),
            "/Library/Logs/DiagnosticReports".to_string(),
        ],
        "recent_items" => vec![
            format!("{}/Library/Application Support/com.apple.sharedfilelist", home_dir),
            format!("{}/Library/Preferences/com.apple.recentitems.plist", home_dir),
            format!("{}/Library/Application Support/com.apple.spotlight.Shortcuts", home_dir),
        ],
        "install_logs" => vec![
            "/var/log/install.log".to_string(),
            format!("{}/Library/Logs/Install.log", home_dir),
            "/Library/Receipts".to_string(),
            format!("{}/Library/Receipts", home_dir),
        ],

        // ========== 浏览器类别 ==========
        "safari" => vec![
            format!("{}/Library/Safari", home_dir),
            format!("{}/Library/Caches/com.apple.Safari", home_dir),
            format!("{}/Library/Cookies", home_dir),
            format!("{}/Library/WebKit", home_dir),
        ],
        "chrome" => vec![
            format!("{}/Library/Application Support/Google/Chrome", home_dir),
            format!("{}/Library/Caches/Google/Chrome", home_dir),
        ],
        "firefox" => vec![
            format!("{}/Library/Application Support/Firefox", home_dir),
            format!("{}/Library/Caches/Firefox", home_dir),
        ],
        "edge" => vec![
            format!("{}/Library/Application Support/Microsoft Edge", home_dir),
            format!("{}/Library/Caches/Microsoft Edge", home_dir),
        ],

        // ========== 开发工具类别 ==========
        "vscode" => vec![
            format!("{}/Library/Application Support/Code", home_dir),
            format!("{}/Library/Caches/com.microsoft.VSCode", home_dir),
            format!("{}/.vscode", home_dir),
        ],
        "xcode" => vec![
            format!("{}/Library/Developer", home_dir),
            format!("{}/Library/Caches/com.apple.dt.Xcode", home_dir),
            format!("{}/Library/Developer/Xcode/DerivedData", home_dir),
        ],
        "jetbrains" => vec![
            format!("{}/Library/Application Support/JetBrains", home_dir),
            format!("{}/Library/Caches/JetBrains", home_dir),
            format!("{}/Library/Logs/JetBrains", home_dir),
        ],
        "git" => vec![
            format!("{}/.gitconfig", home_dir),
            format!("{}/.git-credentials", home_dir),
        ],
        "npm" => vec![
            format!("{}/.npm", home_dir),
            format!("{}/.node_repl_history", home_dir),
            format!("{}/.npmrc", home_dir),
        ],
        "python" => vec![
            format!("{}/.python_history", home_dir),
            format!("{}/.ipython", home_dir),
            format!("{}/.jupyter", home_dir),
            format!("{}/Library/Caches/pip", home_dir),
        ],
        "docker" => vec![
            format!("{}/.docker", home_dir),
            format!("{}/Library/Containers/com.docker.docker", home_dir),
            format!("{}/Library/Group Containers/group.com.docker", home_dir),
        ],

        // ========== 办公软件类别 ==========
        "office" => vec![
            format!("{}/Library/Containers/com.microsoft.Word", home_dir),
            format!("{}/Library/Containers/com.microsoft.Excel", home_dir),
            format!("{}/Library/Containers/com.microsoft.Powerpoint", home_dir),
            format!("{}/Library/Group Containers/UBF8T346G9.Office", home_dir),
        ],

        // ========== 通讯软件类别 ==========
        "wechat" => vec![
            format!("{}/Library/Containers/com.tencent.xinWeChat", home_dir),
            format!("{}/Library/Application Support/com.tencent.xinWeChat", home_dir),
        ],
        "qq" => vec![
            format!("{}/Library/Containers/com.tencent.qq", home_dir),
            format!("{}/Library/Application Support/QQ", home_dir),
        ],
        "telegram" => vec![
            format!("{}/Library/Application Support/Telegram Desktop", home_dir),
            format!("{}/Library/Group Containers/6N38VWS5BX.ru.keepcoder.Telegram", home_dir),
        ],
        "discord" => vec![
            format!("{}/Library/Application Support/discord", home_dir),
            format!("{}/Library/Caches/discord", home_dir),
        ],
        "slack" => vec![
            format!("{}/Library/Application Support/Slack", home_dir),
            format!("{}/Library/Caches/com.tinyspeck.slackmacgap", home_dir),
        ],

        // ========== 多媒体类别 ==========
        "spotify" => vec![
            format!("{}/Library/Application Support/Spotify", home_dir),
            format!("{}/Library/Caches/com.spotify.client", home_dir),
        ],
        "vlc" => vec![
            format!("{}/Library/Application Support/VLC", home_dir),
            format!("{}/Library/Preferences/org.videolan.vlc", home_dir),
        ],

        // ========== 安全工具类别 ==========
        "burpsuite" => vec![
            format!("{}/.BurpSuite", home_dir),
            format!("{}/.java/.userPrefs/burp", home_dir),
        ],
        "wireshark" => vec![
            format!("{}/.config/wireshark", home_dir),
            format!("{}/Library/Application Support/Wireshark", home_dir),
        ],
        "metasploit" => vec![
            format!("{}/.msf4", home_dir),
        ],

        // ========== 终端类别 ==========
        "shell_history" => {
            // Shell 历史需要特殊处理 - 清空而不是删除
            let shell_files = vec![
                format!("{}/.bash_history", home_dir),
                format!("{}/.zsh_history", home_dir),
                format!("{}/.zhistory", home_dir),
                format!("{}/.fish_history", home_dir),
                format!("{}/.local/share/fish/fish_history", home_dir),
            ];
            for file in &shell_files {
                let _ = truncate_file(file);
            }
            return Ok(());
        },
        "ssh" => vec![
            format!("{}/.ssh/known_hosts", home_dir),
        ],
        "iterm" => vec![
            format!("{}/Library/Application Support/iTerm2", home_dir),
            format!("{}/Library/Caches/com.googlecode.iterm2", home_dir),
        ],

        // ========== 运维工具类别 ==========
        "homebrew" => vec![
            format!("{}/Library/Caches/Homebrew", home_dir),
            format!("{}/Library/Logs/Homebrew", home_dir),
        ],
        "kubernetes" => vec![
            format!("{}/.kube", home_dir),
            format!("{}/.minikube", home_dir),
        ],
        "aws" => vec![
            format!("{}/.aws", home_dir),
        ],

        // ========== 文件类别 ==========
        "trash" => vec![
            format!("{}/.Trash", home_dir),
        ],
        "downloads" => vec![
            format!("{}/Downloads", home_dir),
        ],

        // ========== 更多浏览器 ==========
        "arc" => vec![
            format!("{}/Library/Application Support/Arc", home_dir),
            format!("{}/Library/Caches/company.thebrowser.Browser", home_dir),
        ],
        "brave" => vec![
            format!("{}/Library/Application Support/BraveSoftware/Brave-Browser", home_dir),
            format!("{}/Library/Caches/BraveSoftware/Brave-Browser", home_dir),
        ],
        "opera" => vec![
            format!("{}/Library/Application Support/com.operasoftware.Opera", home_dir),
            format!("{}/Library/Caches/com.operasoftware.Opera", home_dir),
        ],
        "vivaldi" => vec![
            format!("{}/Library/Application Support/Vivaldi", home_dir),
            format!("{}/Library/Caches/Vivaldi", home_dir),
        ],

        // ========== 更多开发工具 ==========
        "android_studio" => vec![
            format!("{}/.android", home_dir),
            format!("{}/Library/Android/sdk", home_dir),
        ],
        "rust" => vec![
            format!("{}/.cargo", home_dir),
            format!("{}/.rustup", home_dir),
        ],
        "golang" => vec![
            format!("{}/go", home_dir),
            format!("{}/.cache/go-build", home_dir),
        ],
        "ruby" => vec![
            format!("{}/.gem", home_dir),
            format!("{}/.bundle", home_dir),
            format!("{}/.rbenv", home_dir),
            format!("{}/.rvm", home_dir),
        ],
        "java" => vec![
            format!("{}/.m2", home_dir),
            format!("{}/.gradle", home_dir),
            format!("{}/.java", home_dir),
        ],
        "sublime" => vec![
            format!("{}/Library/Application Support/Sublime Text", home_dir),
            format!("{}/Library/Caches/com.sublimetext.4", home_dir),
        ],
        "cursor" => vec![
            format!("{}/Library/Application Support/Cursor", home_dir),
            format!("{}/Library/Caches/Cursor", home_dir),
            format!("{}/.cursor", home_dir),
        ],
        "yarn_pnpm" => vec![
            format!("{}/.yarn", home_dir),
            format!("{}/.yarnrc", home_dir),
            format!("{}/Library/Caches/Yarn", home_dir),
            format!("{}/.pnpm-store", home_dir),
            format!("{}/Library/pnpm", home_dir),
        ],
        "cocoapods" => vec![
            format!("{}/.cocoapods", home_dir),
            format!("{}/Library/Caches/CocoaPods", home_dir),
        ],

        // ========== 更多通讯软件 ==========
        "feishu" => vec![
            format!("{}/Library/Containers/com.bytedance.lark", home_dir),
            format!("{}/Library/Application Support/Lark", home_dir),
            format!("{}/Library/Application Support/Feishu", home_dir),
        ],
        "dingtalk" => vec![
            format!("{}/Library/Containers/com.alibaba.DingTalkMac", home_dir),
            format!("{}/Library/Application Support/DingTalk", home_dir),
        ],
        "zoom" => vec![
            format!("{}/Library/Application Support/zoom.us", home_dir),
            format!("{}/Library/Caches/us.zoom.xos", home_dir),
            format!("{}/Library/Logs/zoom.us", home_dir),
        ],
        "teams" => vec![
            format!("{}/Library/Application Support/Microsoft/Teams", home_dir),
            format!("{}/Library/Caches/com.microsoft.teams", home_dir),
        ],
        "skype" => vec![
            format!("{}/Library/Application Support/Skype", home_dir),
            format!("{}/Library/Caches/com.skype.skype", home_dir),
        ],
        "whatsapp" => vec![
            format!("{}/Library/Application Support/WhatsApp", home_dir),
            format!("{}/Library/Containers/net.whatsapp.WhatsApp", home_dir),
        ],
        "signal" => vec![
            format!("{}/Library/Application Support/Signal", home_dir),
            format!("{}/Library/Caches/org.whispersystems.signal-desktop", home_dir),
        ],

        // ========== 更多多媒体 ==========
        "iina" => vec![
            format!("{}/Library/Application Support/com.colliderli.iina", home_dir),
            format!("{}/Library/Caches/com.colliderli.iina", home_dir),
        ],
        "netease_music" => vec![
            format!("{}/Library/Containers/com.netease.163music", home_dir),
            format!("{}/Library/Application Support/NeteaseMusic", home_dir),
        ],
        "qq_music" => vec![
            format!("{}/Library/Containers/com.tencent.QQMusicMac", home_dir),
            format!("{}/Library/Application Support/QQMusic", home_dir),
        ],
        "apple_music" => vec![
            format!("{}/Library/Caches/com.apple.Music", home_dir),
            format!("{}/Library/Application Support/Music", home_dir),
        ],
        "bilibili" => vec![
            format!("{}/Library/Application Support/com.bilibili.app.pgc", home_dir),
            format!("{}/Library/Containers/tv.danmaku.bilibili", home_dir),
        ],
        "tencent_video" => vec![
            format!("{}/Library/Containers/com.tencent.tenvideo", home_dir),
            format!("{}/Library/Application Support/TencentVideo", home_dir),
        ],

        // ========== 云存储 ==========
        "icloud" => vec![
            format!("{}/Library/Mobile Documents", home_dir),
            format!("{}/Library/Caches/com.apple.bird", home_dir),
        ],
        "dropbox" => vec![
            format!("{}/.dropbox", home_dir),
            format!("{}/Library/Dropbox", home_dir),
            format!("{}/Library/Caches/com.dropbox.client", home_dir),
        ],
        "google_drive" => vec![
            format!("{}/Library/Application Support/Google/DriveFS", home_dir),
            format!("{}/Library/Caches/com.google.drivefs", home_dir),
        ],
        "onedrive" => vec![
            format!("{}/Library/Application Support/OneDrive", home_dir),
            format!("{}/Library/Caches/com.microsoft.OneDrive", home_dir),
            format!("{}/Library/Logs/OneDrive", home_dir),
        ],
        "nutstore" => vec![
            format!("{}/Library/Application Support/Nutstore", home_dir),
            format!("{}/Library/Caches/com.jianguoyun.Nutstore", home_dir),
        ],
        "baidu_netdisk" => vec![
            format!("{}/Library/Application Support/BaiduNetdisk", home_dir),
            format!("{}/Library/Containers/com.baidu.BaiduNetdisk", home_dir),
        ],

        // ========== AI工具 ==========
        "chatgpt" => vec![
            format!("{}/Library/Application Support/com.openai.chat", home_dir),
            format!("{}/Library/Caches/com.openai.chat", home_dir),
        ],
        "claude" => vec![
            format!("{}/Library/Application Support/Claude", home_dir),
            format!("{}/Library/Caches/com.anthropic.claudefordesktop", home_dir),
            format!("{}/.claude", home_dir),
        ],
        "copilot" => vec![
            format!("{}/Library/Application Support/GitHub Copilot", home_dir),
            format!("{}/.config/github-copilot", home_dir),
        ],

        // ========== 数据库客户端 ==========
        "tableplus" => vec![
            format!("{}/Library/Application Support/com.tinyapp.TablePlus", home_dir),
            format!("{}/Library/Caches/com.tinyapp.TablePlus", home_dir),
        ],
        "dbeaver" => vec![
            format!("{}/Library/DBeaverData", home_dir),
            format!("{}/.dbeaver4", home_dir),
        ],
        "sequel" => vec![
            format!("{}/Library/Application Support/Sequel Pro", home_dir),
            format!("{}/Library/Application Support/Sequel Ace", home_dir),
        ],
        "mongodb_compass" => vec![
            format!("{}/Library/Application Support/MongoDB Compass", home_dir),
            format!("{}/Library/Caches/MongoDB Compass", home_dir),
        ],
        "redis_desktop" => vec![
            format!("{}/Library/Application Support/rdm", home_dir),
            format!("{}/.rdm", home_dir),
        ],

        // ========== 更多安全工具 ==========
        "charles" => vec![
            format!("{}/Library/Application Support/Charles", home_dir),
            format!("{}/Library/Caches/com.xk72.Charles", home_dir),
        ],
        "proxyman" => vec![
            format!("{}/Library/Application Support/com.proxyman.NSProxy", home_dir),
            format!("{}/Library/Caches/com.proxyman.NSProxy", home_dir),
        ],
        "postman" => vec![
            format!("{}/Library/Application Support/Postman", home_dir),
            format!("{}/Library/Caches/Postman", home_dir),
        ],
        "insomnia" => vec![
            format!("{}/Library/Application Support/Insomnia", home_dir),
            format!("{}/Library/Caches/Insomnia", home_dir),
        ],
        "onepassword" => vec![
            format!("{}/Library/Group Containers/2BUA8C4S2C.com.1password", home_dir),
            format!("{}/Library/Application Support/1Password", home_dir),
        ],
        "bitwarden" => vec![
            format!("{}/Library/Application Support/Bitwarden", home_dir),
            format!("{}/Library/Caches/com.bitwarden.desktop", home_dir),
        ],
        "keychain" => vec![
            format!("{}/Library/Keychains", home_dir),
        ],

        // ========== 虚拟化工具 ==========
        "vmware" => vec![
            format!("{}/Library/Application Support/VMware Fusion", home_dir),
            format!("{}/Library/Caches/com.vmware.fusion", home_dir),
            format!("{}/Library/Logs/VMware", home_dir),
        ],
        "parallels" => vec![
            format!("{}/Library/Parallels", home_dir),
            format!("{}/Library/Logs/Parallels", home_dir),
        ],
        "virtualbox" => vec![
            format!("{}/Library/VirtualBox", home_dir),
            format!("{}/.VirtualBox", home_dir),
        ],

        // ========== 设计工具 ==========
        "figma" => vec![
            format!("{}/Library/Application Support/Figma", home_dir),
            format!("{}/Library/Caches/com.figma.Desktop", home_dir),
        ],
        "sketch" => vec![
            format!("{}/Library/Application Support/com.bohemiancoding.sketch3", home_dir),
            format!("{}/Library/Caches/com.bohemiancoding.sketch3", home_dir),
        ],
        "adobe" => vec![
            format!("{}/Library/Application Support/Adobe", home_dir),
            format!("{}/Library/Caches/Adobe", home_dir),
            format!("{}/Library/Logs/Adobe", home_dir),
        ],

        // ========== 笔记工具 ==========
        "notion" => vec![
            format!("{}/Library/Application Support/Notion", home_dir),
            format!("{}/Library/Caches/notion.id", home_dir),
        ],
        "obsidian" => vec![
            format!("{}/Library/Application Support/obsidian", home_dir),
            format!("{}/Library/Caches/md.obsidian", home_dir),
        ],
        "evernote" => vec![
            format!("{}/Library/Application Support/Evernote", home_dir),
            format!("{}/Library/Containers/com.evernote.Evernote", home_dir),
        ],
        "bear" => vec![
            format!("{}/Library/Group Containers/9K33E3U3T4.net.shinyfrog.bear", home_dir),
            format!("{}/Library/Containers/net.shinyfrog.bear", home_dir),
        ],

        // ========== 下载工具 ==========
        "thunder" => vec![
            format!("{}/Library/Application Support/Thunder", home_dir),
            format!("{}/Library/Containers/com.xunlei.Thunder", home_dir),
        ],
        "motrix" => vec![
            format!("{}/Library/Application Support/Motrix", home_dir),
            format!("{}/Library/Caches/Motrix", home_dir),
        ],

        // ========== 系统工具 ==========
        "cleanmymac" => vec![
            format!("{}/Library/Application Support/CleanMyMac X", home_dir),
            format!("{}/Library/Caches/com.macpaw.CleanMyMac4", home_dir),
            format!("{}/Library/Logs/CleanMyMac X", home_dir),
        ],
        "alfred" => vec![
            format!("{}/Library/Application Support/Alfred", home_dir),
            format!("{}/Library/Caches/com.runningwithcrayons.Alfred", home_dir),
        ],
        "raycast" => vec![
            format!("{}/Library/Application Support/Raycast", home_dir),
            format!("{}/Library/Caches/com.raycast.macos", home_dir),
        ],

        // ========== 网络工具 ==========
        "vpn_tools" => vec![
            format!("{}/Library/Application Support/Surge", home_dir),
            format!("{}/Library/Application Support/com.west2online.ClashX", home_dir),
            format!("{}/Library/Application Support/com.west2online.ClashXPro", home_dir),
            format!("{}/.config/clash", home_dir),
            format!("{}/Library/Application Support/V2rayU", home_dir),
        ],

        // ========== 运维工具扩展 ==========
        "terraform" => vec![
            format!("{}/.terraform.d", home_dir),
            format!("{}/Library/Caches/terraform-plugin-cache", home_dir),
        ],
        "ansible" => vec![
            format!("{}/.ansible", home_dir),
        ],

        // ========== 游戏平台 ==========
        "steam" => vec![
            format!("{}/Library/Application Support/Steam", home_dir),
            format!("{}/Library/Caches/com.valvesoftware.steam", home_dir),
        ],
        "epic_games" => vec![
            format!("{}/Library/Application Support/Epic", home_dir),
            format!("{}/Library/Caches/com.epicgames.EpicGamesLauncher", home_dir),
            format!("{}/Library/Logs/EpicGamesLauncher", home_dir),
        ],
        "battlenet" => vec![
            format!("{}/Library/Application Support/Blizzard", home_dir),
            format!("{}/Library/Application Support/Battle.net", home_dir),
            format!("{}/Library/Caches/com.blizzard.bna", home_dir),
            "/Users/Shared/Blizzard".to_string(),
        ],
        "gog" => vec![
            format!("{}/Library/Application Support/GOG.com", home_dir),
            format!("{}/Library/Caches/com.gog.galaxy", home_dir),
        ],
        "origin" => vec![
            format!("{}/Library/Application Support/Origin", home_dir),
            format!("{}/Library/Application Support/Electronic Arts", home_dir),
            format!("{}/Library/Caches/com.ea.Origin", home_dir),
        ],

        // ========== 视频编辑 ==========
        "finalcut" => vec![
            format!("{}/Movies/Final Cut Backups", home_dir),
            format!("{}/Library/Caches/com.apple.FinalCut", home_dir),
            format!("{}/Library/Application Support/Final Cut Pro", home_dir),
        ],
        "davinci" => vec![
            format!("{}/Library/Application Support/Blackmagic Design", home_dir),
            format!("{}/Library/Caches/com.blackmagic-design.DaVinciResolve", home_dir),
            format!("{}/Movies/.gallery", home_dir),
            format!("{}/Movies/CacheClip", home_dir),
        ],
        "imovie" => vec![
            format!("{}/Library/Caches/com.apple.iMovieApp", home_dir),
            format!("{}/Library/Application Support/iMovie", home_dir),
        ],
        "screenflow" => vec![
            format!("{}/Library/Application Support/Telestream", home_dir),
            format!("{}/Library/Caches/net.telestream.screenflow10", home_dir),
        ],
        "obs" => vec![
            format!("{}/Library/Application Support/obs-studio", home_dir),
            format!("{}/Library/Caches/com.obsproject.obs-studio", home_dir),
        ],

        // ========== 音频编辑 ==========
        "logic" => vec![
            format!("{}/Library/Application Support/Logic", home_dir),
            format!("{}/Library/Caches/com.apple.logic10", home_dir),
            format!("{}/Music/Audio Music Apps", home_dir),
        ],
        "garageband" => vec![
            format!("{}/Library/Application Support/GarageBand", home_dir),
            format!("{}/Library/Caches/com.apple.garageband10", home_dir),
            format!("{}/Music/GarageBand", home_dir),
        ],
        "audacity" => vec![
            format!("{}/Library/Application Support/audacity", home_dir),
            format!("{}/Library/Caches/audacity", home_dir),
        ],

        // ========== 邮件客户端 ==========
        "apple_mail" => vec![
            format!("{}/Library/Mail", home_dir),
            format!("{}/Library/Caches/com.apple.mail", home_dir),
            format!("{}/Library/Containers/com.apple.mail", home_dir),
        ],
        "spark" => vec![
            format!("{}/Library/Application Support/Spark", home_dir),
            format!("{}/Library/Caches/com.readdle.SparkDesktop", home_dir),
            format!("{}/Library/Group Containers/group.com.readdle.smartemail", home_dir),
        ],
        "airmail" => vec![
            format!("{}/Library/Application Support/Airmail", home_dir),
            format!("{}/Library/Containers/it.bloop.airmail2", home_dir),
            format!("{}/Library/Group Containers/2E337YPCZY.airmail", home_dir),
        ],
        "mailspring" => vec![
            format!("{}/Library/Application Support/Mailspring", home_dir),
            format!("{}/Library/Caches/Mailspring", home_dir),
        ],
        "outlook" => vec![
            format!("{}/Library/Containers/com.microsoft.Outlook", home_dir),
            format!("{}/Library/Group Containers/UBF8T346G9.Office", home_dir),
            format!("{}/Library/Caches/com.microsoft.Outlook", home_dir),
        ],

        // ========== 任务管理/待办 ==========
        "things" => vec![
            format!("{}/Library/Containers/com.culturedcode.ThingsMac", home_dir),
            format!("{}/Library/Group Containers/JLMPQHK86H.com.culturedcode.ThingsMac", home_dir),
        ],
        "todoist" => vec![
            format!("{}/Library/Application Support/Todoist", home_dir),
            format!("{}/Library/Caches/com.todoist.mac.Todoist", home_dir),
        ],
        "ticktick" => vec![
            format!("{}/Library/Application Support/TickTick", home_dir),
            format!("{}/Library/Caches/com.TickTick.task.mac", home_dir),
        ],
        "reminders" => vec![
            format!("{}/Library/Containers/com.apple.reminders", home_dir),
            format!("{}/Library/Caches/com.apple.remindd", home_dir),
        ],

        // ========== 终端替代品 ==========
        "warp" => vec![
            format!("{}/Library/Application Support/dev.warp.Warp-Stable", home_dir),
            format!("{}/Library/Caches/dev.warp.Warp-Stable", home_dir),
            format!("{}/.warp", home_dir),
        ],
        "hyper" => vec![
            format!("{}/Library/Application Support/Hyper", home_dir),
            format!("{}/Library/Caches/co.zeit.hyper", home_dir),
            format!("{}/.hyper.js", home_dir),
            format!("{}/.hyper_plugins", home_dir),
        ],
        "alacritty" => vec![
            format!("{}/.config/alacritty", home_dir),
            format!("{}/Library/Caches/io.alacritty", home_dir),
        ],
        "kitty" => vec![
            format!("{}/.config/kitty", home_dir),
            format!("{}/Library/Caches/kitty", home_dir),
        ],

        // ========== 日历 ==========
        "calendar" => vec![
            format!("{}/Library/Calendars", home_dir),
            format!("{}/Library/Caches/com.apple.CalendarAgent", home_dir),
            format!("{}/Library/Containers/com.apple.iCal", home_dir),
        ],
        "fantastical" => vec![
            format!("{}/Library/Containers/com.flexibits.fantastical2.mac", home_dir),
            format!("{}/Library/Group Containers/85C27NK92C.com.flexibits.fantastical2.mac", home_dir),
        ],

        // ========== 照片/图片 ==========
        "photos" => vec![
            format!("{}/Library/Caches/com.apple.Photos", home_dir),
            format!("{}/Library/Containers/com.apple.Photos", home_dir),
        ],
        "lightroom" => vec![
            format!("{}/Library/Application Support/Adobe/Lightroom", home_dir),
            format!("{}/Library/Caches/Adobe/Lightroom", home_dir),
        ],
        "pixelmator" => vec![
            format!("{}/Library/Containers/com.pixelmatorteam.pixelmator.x", home_dir),
            format!("{}/Library/Caches/com.pixelmatorteam.pixelmator.x", home_dir),
        ],

        // ========== 压缩工具 ==========
        "unarchiver" => vec![
            format!("{}/Library/Containers/cx.c3.theunarchiver", home_dir),
            format!("{}/Library/Caches/cx.c3.theunarchiver", home_dir),
        ],
        "keka" => vec![
            format!("{}/Library/Application Support/Keka", home_dir),
            format!("{}/Library/Caches/com.aone.keka", home_dir),
        ],

        // ========== 截图工具 ==========
        "cleanshot" => vec![
            format!("{}/Library/Application Support/CleanShot", home_dir),
            format!("{}/Library/Caches/pl.maketheweb.cleanshotx", home_dir),
        ],
        "snagit" => vec![
            format!("{}/Library/Application Support/TechSmith/Snagit", home_dir),
            format!("{}/Library/Caches/com.TechSmith.Snagit", home_dir),
        ],

        // ========== 翻译工具 ==========
        "deepl" => vec![
            format!("{}/Library/Application Support/DeepL", home_dir),
            format!("{}/Library/Caches/com.linguee.DeepLCopyTranslator", home_dir),
        ],
        "eudic" => vec![
            format!("{}/Library/Application Support/Eudb", home_dir),
            format!("{}/Library/Caches/com.eusoft.eudic", home_dir),
        ],

        // ========== 阅读器 ==========
        "kindle" => vec![
            format!("{}/Library/Application Support/Kindle", home_dir),
            format!("{}/Library/Containers/com.amazon.Kindle", home_dir),
        ],
        "pdfexpert" => vec![
            format!("{}/Library/Containers/com.readdle.PDFExpert-Mac", home_dir),
            format!("{}/Library/Caches/com.readdle.PDFExpert-Mac", home_dir),
        ],
        "marginnote" => vec![
            format!("{}/Library/Containers/QReader.MarginStudyMac", home_dir),
            format!("{}/Library/Application Support/MarginNote 3", home_dir),
        ],

        // ========== 剪贴板管理 ==========
        "paste" => vec![
            format!("{}/Library/Containers/com.wiheads.paste", home_dir),
            format!("{}/Library/Application Support/Paste", home_dir),
        ],
        "maccy" => vec![
            format!("{}/Library/Containers/org.p0deje.Maccy", home_dir),
            format!("{}/Library/Caches/org.p0deje.Maccy", home_dir),
        ],

        // ========== 窗口管理 ==========
        "rectangle" => vec![
            format!("{}/Library/Application Support/Rectangle", home_dir),
            format!("{}/Library/Caches/com.knollsoft.Rectangle", home_dir),
        ],
        "magnet" => vec![
            format!("{}/Library/Containers/id.arunro.magnet", home_dir),
            format!("{}/Library/Caches/id.arunro.magnet", home_dir),
        ],

        // ========== 输入法 ==========
        "sogou" => vec![
            format!("{}/Library/Application Support/Sogou", home_dir),
            format!("{}/Library/Caches/com.sogou.inputmethod.sogou", home_dir),
        ],
        "baidu_input" => vec![
            format!("{}/Library/Application Support/com.baidu.inputmethod.BaiduIM", home_dir),
            format!("{}/Library/Caches/com.baidu.inputmethod.BaiduIM", home_dir),
        ],

        // ========== 社交媒体 ==========
        "twitter" => vec![
            format!("{}/Library/Containers/com.twitter.twitter-mac", home_dir),
            format!("{}/Library/Caches/com.twitter.twitter-mac", home_dir),
        ],
        "weibo" => vec![
            format!("{}/Library/Containers/com.sina.weibo", home_dir),
            format!("{}/Library/Application Support/Weibo", home_dir),
        ],
        "xiaohongshu" => vec![
            format!("{}/Library/Containers/com.xingin.xiaohongshu", home_dir),
            format!("{}/Library/Caches/com.xingin.xiaohongshu", home_dir),
        ],
        "douyin" => vec![
            format!("{}/Library/Containers/com.ss.mac.douyinlite", home_dir),
            format!("{}/Library/Application Support/Douyin", home_dir),
        ],

        // ========== 旧的日志类型兼容 ==========
        "auth" => vec![
            format!("{}/Library/Caches/com.apple.loginwindow", home_dir),
            format!("{}/Library/Application Support/com.apple.sharedfilelist", home_dir),
            format!("{}/Library/Preferences/com.apple.recentitems.plist", home_dir),
        ],
        "kern" => vec![
            format!("{}/Library/Logs/DiagnosticReports", home_dir),
            format!("{}/Library/Logs/CrashReporter", home_dir),
            "/Library/Logs/DiagnosticReports".to_string(),
        ],

        _ => {
            return Err(format!("未知的日志类型: {}", log_type));
        }
    };

    let _count = clear_paths(&paths)?;

    Ok(())
}

/// 清理 Linux 系统日志
#[cfg(target_os = "linux")]
pub fn clear_linux_log(log_type: &str) -> Result<(), String> {
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());

    let paths: Vec<String> = match log_type {
        // ========== 系统类别 ==========
        "syslog" => vec![
            format!("{}/.local/share/systemd/user", home_dir),
            format!("{}/.xsession-errors", home_dir),
            format!("{}/.xsession-errors.old", home_dir),
        ],
        "app_cache" => vec![
            format!("{}/.cache", home_dir),
        ],
        "recent_items" => vec![
            format!("{}/.local/share/recently-used.xbel", home_dir),
        ],

        // ========== 浏览器类别 ==========
        "chrome" => vec![
            format!("{}/.config/google-chrome", home_dir),
            format!("{}/.cache/google-chrome", home_dir),
        ],
        "firefox" => vec![
            format!("{}/.mozilla/firefox", home_dir),
            format!("{}/.cache/mozilla/firefox", home_dir),
        ],

        // ========== 开发工具类别 ==========
        "vscode" => vec![
            format!("{}/.config/Code", home_dir),
            format!("{}/.cache/Code", home_dir),
        ],
        "git" => vec![
            format!("{}/.gitconfig", home_dir),
            format!("{}/.git-credentials", home_dir),
        ],
        "npm" => vec![
            format!("{}/.npm", home_dir),
            format!("{}/.node_repl_history", home_dir),
            format!("{}/.npmrc", home_dir),
        ],
        "python" => vec![
            format!("{}/.python_history", home_dir),
            format!("{}/.ipython", home_dir),
            format!("{}/.jupyter", home_dir),
            format!("{}/.cache/pip", home_dir),
        ],
        "docker" => vec![
            format!("{}/.docker", home_dir),
        ],

        // ========== 终端类别 ==========
        "shell_history" => {
            let shell_files = vec![
                format!("{}/.bash_history", home_dir),
                format!("{}/.zsh_history", home_dir),
                format!("{}/.zhistory", home_dir),
                format!("{}/.fish_history", home_dir),
                format!("{}/.local/share/fish/fish_history", home_dir),
            ];
            for file in &shell_files {
                let _ = truncate_file(file);
            }
            return Ok(());
        },
        "ssh" => vec![
            format!("{}/.ssh/known_hosts", home_dir),
        ],

        // ========== 文件类别 ==========
        "trash" => vec![
            format!("{}/.local/share/Trash", home_dir),
        ],

        // ========== 旧的日志类型兼容 ==========
        "auth" => vec![
            format!("{}/.cache/gnome-keyring", home_dir),
            format!("{}/.local/share/keyrings", home_dir),
        ],
        "kern" => {
            let kern_files = vec![
                format!("{}/.dmesg", home_dir),
            ];
            for file in &kern_files {
                let _ = truncate_file(file);
            }
            return Ok(());
        },

        _ => {
            return Err(format!("未知的日志类型: {}", log_type));
        }
    };

    let _count = clear_paths(&paths)?;

    Ok(())
}

/// 清理 Windows 日志
#[cfg(target_os = "windows")]
pub fn clear_windows_log(log_type: &str) -> Result<(), String> {
    let home_dir = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users".to_string());
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| format!("{}\\AppData\\Roaming", home_dir));
    let localappdata = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| format!("{}\\AppData\\Local", home_dir));

    // Windows 事件日志需要特殊处理
    match log_type {
        "Application" | "System" | "Security" | "Setup" => {
            return clear_windows_event_log(log_type);
        }
        _ => {}
    }

    let paths: Vec<String> = match log_type {
        // ========== 系统类别 ==========
        "temp" => vec![
            format!("{}\\Temp", localappdata),
            std::env::var("TEMP").unwrap_or_else(|_| format!("{}\\Temp", localappdata)),
        ],
        "recent_items" => vec![
            format!("{}\\Microsoft\\Windows\\Recent", appdata),
        ],

        // ========== 浏览器类别 ==========
        "chrome" => vec![
            format!("{}\\Google\\Chrome\\User Data", localappdata),
        ],
        "firefox" => vec![
            format!("{}\\Mozilla\\Firefox\\Profiles", appdata),
        ],
        "edge" => vec![
            format!("{}\\Microsoft\\Edge\\User Data", localappdata),
        ],

        // ========== 开发工具类别 ==========
        "vscode" => vec![
            format!("{}\\Code", appdata),
            format!("{}\\Code", localappdata),
        ],

        // ========== 文件类别 ==========
        "trash" => {
            // Use Rust std to clear recycle bin contents
            // Note: Full recycle bin clearing requires Shell API or admin rights
            // This clears accessible parts
            let recycle_path = std::path::Path::new("C:\\$Recycle.Bin");
            if recycle_path.exists() {
                if let Ok(entries) = std::fs::read_dir(recycle_path) {
                    for entry in entries.flatten() {
                        let _ = std::fs::remove_dir_all(entry.path());
                    }
                }
            }
            return Ok(());
        },

        _ => {
            return Err(format!("未知的日志类型: {}", log_type));
        }
    };

    let _count = clear_paths(&paths)?;

    Ok(())
}

/// 清理系统日志（跨平台入口）
pub fn clear_system_logs(logs: Vec<String>) -> Result<Vec<String>, String> {
    let mut cleared = Vec::new();
    let mut errors = Vec::new();

    for log in logs {
        #[cfg(target_os = "windows")]
        {
            match clear_windows_log(&log) {
                Ok(_) => cleared.push(log.clone()),
                Err(e) => errors.push(format!("{}: {}", log, e)),
            }
        }

        #[cfg(target_os = "macos")]
        {
            match clear_macos_log(&log) {
                Ok(_) => cleared.push(log.clone()),
                Err(e) => errors.push(format!("{}: {}", log, e)),
            }
        }

        #[cfg(target_os = "linux")]
        {
            match clear_linux_log(&log) {
                Ok(_) => cleared.push(log.clone()),
                Err(e) => errors.push(format!("{}: {}", log, e)),
            }
        }
    }

    if !errors.is_empty() && cleared.is_empty() {
        return Err(format!(
            "日志清理失败:\n{}",
            errors.join("\n")
        ));
    }

    // errors logged silently

    Ok(cleared)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    fn test_clear_windows_log() {
        // 注意：此测试需要管理员权限
        // let result = clear_windows_event_log("Application");
        // assert!(result.is_ok());
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_clear_macos_log() {
        // 注意：此测试会实际清理文件，谨慎运行
        // let result = clear_macos_log("syslog");
        // assert!(result.is_ok());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_clear_linux_log() {
        // 注意：此测试会实际清理文件，谨慎运行
        // let result = clear_linux_log("syslog");
        // assert!(result.is_ok());
    }
}
