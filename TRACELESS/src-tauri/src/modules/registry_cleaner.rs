use serde::{Deserialize, Serialize};
use std::process::Command;

#[cfg(target_os = "windows")]
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteKeyW, RegEnumKeyExW, RegEnumValueW, RegOpenKeyExW, RegQueryInfoKeyW,
    HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_ALL_ACCESS,
};
#[cfg(target_os = "windows")]
use windows::core::PCWSTR;
#[cfg(target_os = "windows")]
use std::ffi::OsStr;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;

/// 注册表/隐私痕迹项信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryItemInfo {
    pub key: String,
    pub name: String,
    pub description: String,
    pub path: String,
    pub entry_count: u32,
    pub size_estimate: String,
    pub risk_level: String,
    pub category: String,
    pub platform: String,
    pub requires_admin: bool,
    pub last_modified: Option<String>,
}

/// 注册表/隐私痕迹状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStatus {
    pub platform: String,
    pub supported: bool,
    pub items: Vec<RegistryItemInfo>,
    pub total_entries: u32,
    pub categories: Vec<String>,
}

/// 获取注册表/隐私痕迹信息
pub fn get_registry_info() -> Result<RegistryStatus, String> {
    #[cfg(target_os = "windows")]
    {
        get_windows_registry_info()
    }

    #[cfg(target_os = "macos")]
    {
        get_macos_privacy_info()
    }

    #[cfg(target_os = "linux")]
    {
        get_linux_privacy_info()
    }
}

/// Windows 注册表信息
#[cfg(target_os = "windows")]
fn get_windows_registry_info() -> Result<RegistryStatus, String> {
    let mut items = Vec::new();
    let mut total_entries = 0u32;

    // MRU 列表
    let mru_paths = vec![
        (r"Software\Microsoft\Windows\CurrentVersion\Explorer\ComDlg32\OpenSavePidlMRU", "打开/保存对话框历史"),
        (r"Software\Microsoft\Windows\CurrentVersion\Explorer\ComDlg32\LastVisitedPidlMRU", "最近访问目录"),
        (r"Software\Microsoft\Windows\CurrentVersion\Explorer\RunMRU", "运行对话框历史"),
        (r"Software\Microsoft\Windows\CurrentVersion\Explorer\TypedPaths", "地址栏输入历史"),
    ];

    let mut mru_count = 0u32;
    for (path, _desc) in &mru_paths {
        mru_count += count_registry_entries(path).unwrap_or(0);
    }
    total_entries += mru_count;

    items.push(RegistryItemInfo {
        key: "mru".to_string(),
        name: "MRU 列表".to_string(),
        description: "最近使用的文件、文件夹和命令历史".to_string(),
        path: "HKCU\\...\\Explorer\\*MRU".to_string(),
        entry_count: mru_count,
        size_estimate: format_entry_size(mru_count),
        risk_level: "high".to_string(),
        category: "使用记录".to_string(),
        platform: "windows".to_string(),
        requires_admin: false,
        last_modified: None,
    });

    // UserAssist
    let userassist_count = count_registry_entries(r"Software\Microsoft\Windows\CurrentVersion\Explorer\UserAssist").unwrap_or(0);
    total_entries += userassist_count;

    items.push(RegistryItemInfo {
        key: "userassist".to_string(),
        name: "UserAssist".to_string(),
        description: "程序执行历史和使用统计（ROT13 加密）".to_string(),
        path: "HKCU\\...\\Explorer\\UserAssist".to_string(),
        entry_count: userassist_count,
        size_estimate: format_entry_size(userassist_count),
        risk_level: "high".to_string(),
        category: "执行记录".to_string(),
        platform: "windows".to_string(),
        requires_admin: false,
        last_modified: None,
    });

    // ShellBags
    let shellbags_paths = vec![
        r"Software\Microsoft\Windows\Shell\Bags",
        r"Software\Microsoft\Windows\Shell\BagMRU",
        r"Software\Classes\Local Settings\Software\Microsoft\Windows\Shell\Bags",
        r"Software\Classes\Local Settings\Software\Microsoft\Windows\Shell\BagMRU",
    ];

    let mut shellbags_count = 0u32;
    for path in &shellbags_paths {
        shellbags_count += count_registry_entries(path).unwrap_or(0);
    }
    total_entries += shellbags_count;

    items.push(RegistryItemInfo {
        key: "shellbags".to_string(),
        name: "ShellBags".to_string(),
        description: "文件夹访问历史、窗口位置和视图设置".to_string(),
        path: "HKCU\\...\\Shell\\Bags".to_string(),
        entry_count: shellbags_count,
        size_estimate: format_entry_size(shellbags_count),
        risk_level: "high".to_string(),
        category: "访问记录".to_string(),
        platform: "windows".to_string(),
        requires_admin: false,
        last_modified: None,
    });

    // 最近文档
    let recentdocs_count = count_registry_entries(r"Software\Microsoft\Windows\CurrentVersion\Explorer\RecentDocs").unwrap_or(0);
    let recent_files_count = count_recent_files().unwrap_or(0);
    let total_recent = recentdocs_count + recent_files_count;
    total_entries += total_recent;

    items.push(RegistryItemInfo {
        key: "recentdocs".to_string(),
        name: "最近文档".to_string(),
        description: "最近打开的文档和快捷方式".to_string(),
        path: "HKCU\\...\\Explorer\\RecentDocs".to_string(),
        entry_count: total_recent,
        size_estimate: format_entry_size(total_recent),
        risk_level: "medium".to_string(),
        category: "文档记录".to_string(),
        platform: "windows".to_string(),
        requires_admin: false,
        last_modified: None,
    });

    // USB 设备历史
    let usb_count = count_registry_entries(r"Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2").unwrap_or(0);
    total_entries += usb_count;

    items.push(RegistryItemInfo {
        key: "usbhistory".to_string(),
        name: "USB 设备历史".to_string(),
        description: "USB 存储设备连接记录".to_string(),
        path: "HKCU\\...\\Explorer\\MountPoints2".to_string(),
        entry_count: usb_count,
        size_estimate: format_entry_size(usb_count),
        risk_level: "high".to_string(),
        category: "设备记录".to_string(),
        platform: "windows".to_string(),
        requires_admin: false,
        last_modified: None,
    });

    // 网络历史
    items.push(RegistryItemInfo {
        key: "network".to_string(),
        name: "网络连接历史".to_string(),
        description: "网络驱动器映射和网络位置历史".to_string(),
        path: "HKCU\\Network".to_string(),
        entry_count: count_registry_entries(r"Network").unwrap_or(0),
        size_estimate: "~2 KB".to_string(),
        risk_level: "medium".to_string(),
        category: "网络记录".to_string(),
        platform: "windows".to_string(),
        requires_admin: false,
        last_modified: None,
    });

    // 搜索历史
    let search_count = count_registry_entries(r"Software\Microsoft\Windows\CurrentVersion\Explorer\WordWheelQuery").unwrap_or(0);
    total_entries += search_count;

    items.push(RegistryItemInfo {
        key: "search".to_string(),
        name: "搜索历史".to_string(),
        description: "资源管理器搜索记录".to_string(),
        path: "HKCU\\...\\Explorer\\WordWheelQuery".to_string(),
        entry_count: search_count,
        size_estimate: format_entry_size(search_count),
        risk_level: "medium".to_string(),
        category: "搜索记录".to_string(),
        platform: "windows".to_string(),
        requires_admin: false,
        last_modified: None,
    });

    Ok(RegistryStatus {
        platform: "Windows".to_string(),
        supported: true,
        items,
        total_entries,
        categories: vec![
            "使用记录".to_string(),
            "执行记录".to_string(),
            "访问记录".to_string(),
            "文档记录".to_string(),
            "设备记录".to_string(),
            "网络记录".to_string(),
            "搜索记录".to_string(),
        ],
    })
}

/// macOS 隐私痕迹信息
#[cfg(target_os = "macos")]
fn get_macos_privacy_info() -> Result<RegistryStatus, String> {
    let mut items = Vec::new();
    let mut total_entries = 0u32;
    let home = std::env::var("HOME").unwrap_or_default();

    // 最近使用的项目
    let recent_items_count = count_plist_entries(&format!("{}/.LSApplications", home)).unwrap_or(0)
        + count_directory_entries(&format!("{}/Library/Application Support/com.apple.sharedfilelist", home)).unwrap_or(0);
    total_entries += recent_items_count;

    items.push(RegistryItemInfo {
        key: "recent_items".to_string(),
        name: "最近项目".to_string(),
        description: "最近使用的应用程序、文档和服务器".to_string(),
        path: "~/Library/Application Support/com.apple.sharedfilelist".to_string(),
        entry_count: recent_items_count,
        size_estimate: format_entry_size(recent_items_count),
        risk_level: "high".to_string(),
        category: "使用记录".to_string(),
        platform: "macos".to_string(),
        requires_admin: false,
        last_modified: None,
    });

    // Finder 最近文件夹
    let finder_recents = count_finder_recents().unwrap_or(0);
    total_entries += finder_recents;

    items.push(RegistryItemInfo {
        key: "finder_recents".to_string(),
        name: "Finder 最近文件夹".to_string(),
        description: "Finder 侧边栏最近访问的文件夹".to_string(),
        path: "~/Library/Application Support/com.apple.sharedfilelist/com.apple.LSSharedFileList.RecentDocuments.sfl2".to_string(),
        entry_count: finder_recents,
        size_estimate: format_entry_size(finder_recents),
        risk_level: "medium".to_string(),
        category: "访问记录".to_string(),
        platform: "macos".to_string(),
        requires_admin: false,
        last_modified: None,
    });

    // 应用程序使用历史
    let app_usage = count_directory_entries(&format!("{}/Library/Application Support/com.apple.TCC", home)).unwrap_or(0);
    total_entries += app_usage;

    items.push(RegistryItemInfo {
        key: "app_usage".to_string(),
        name: "应用程序权限记录".to_string(),
        description: "应用程序权限请求和授权历史".to_string(),
        path: "~/Library/Application Support/com.apple.TCC".to_string(),
        entry_count: app_usage,
        size_estimate: format_entry_size(app_usage),
        risk_level: "medium".to_string(),
        category: "权限记录".to_string(),
        platform: "macos".to_string(),
        requires_admin: false,
        last_modified: None,
    });

    // QuickLook 缓存
    let quicklook_count = count_directory_entries(&format!("{}/Library/Caches/com.apple.QuickLook.thumbnailcache", home)).unwrap_or(0);
    total_entries += quicklook_count;

    items.push(RegistryItemInfo {
        key: "quicklook".to_string(),
        name: "QuickLook 缓存".to_string(),
        description: "文件预览缩略图缓存".to_string(),
        path: "~/Library/Caches/com.apple.QuickLook.thumbnailcache".to_string(),
        entry_count: quicklook_count,
        size_estimate: calculate_directory_size(&format!("{}/Library/Caches/com.apple.QuickLook.thumbnailcache", home)),
        risk_level: "high".to_string(),
        category: "预览缓存".to_string(),
        platform: "macos".to_string(),
        requires_admin: false,
        last_modified: None,
    });

    // Spotlight 历史
    items.push(RegistryItemInfo {
        key: "spotlight".to_string(),
        name: "Spotlight 搜索历史".to_string(),
        description: "Spotlight 搜索查询历史".to_string(),
        path: "~/Library/Application Support/com.apple.spotlight".to_string(),
        entry_count: 0,
        size_estimate: "~1 KB".to_string(),
        risk_level: "medium".to_string(),
        category: "搜索记录".to_string(),
        platform: "macos".to_string(),
        requires_admin: false,
        last_modified: None,
    });

    // 终端历史
    let bash_history = count_file_lines(&format!("{}/.bash_history", home)).unwrap_or(0);
    let zsh_history = count_file_lines(&format!("{}/.zsh_history", home)).unwrap_or(0);
    let shell_history = bash_history + zsh_history;
    total_entries += shell_history;

    items.push(RegistryItemInfo {
        key: "shell_history".to_string(),
        name: "终端命令历史".to_string(),
        description: "Bash/Zsh 命令行历史记录".to_string(),
        path: "~/.bash_history, ~/.zsh_history".to_string(),
        entry_count: shell_history,
        size_estimate: format_entry_size(shell_history),
        risk_level: "high".to_string(),
        category: "命令历史".to_string(),
        platform: "macos".to_string(),
        requires_admin: false,
        last_modified: None,
    });

    // 下载历史
    let downloads_plist = count_plist_entries(&format!("{}/Library/Preferences/com.apple.LaunchServices.QuarantineEventsV2", home)).unwrap_or(0);
    total_entries += downloads_plist;

    items.push(RegistryItemInfo {
        key: "quarantine".to_string(),
        name: "下载隔离记录".to_string(),
        description: "从网络下载的文件隔离记录".to_string(),
        path: "~/Library/Preferences/com.apple.LaunchServices.QuarantineEventsV2".to_string(),
        entry_count: downloads_plist,
        size_estimate: format_entry_size(downloads_plist),
        risk_level: "high".to_string(),
        category: "下载记录".to_string(),
        platform: "macos".to_string(),
        requires_admin: false,
        last_modified: None,
    });

    // Safari 历史（如果存在）
    let safari_history = count_directory_entries(&format!("{}/Library/Safari", home)).unwrap_or(0);
    if safari_history > 0 {
        total_entries += safari_history;
        items.push(RegistryItemInfo {
            key: "safari".to_string(),
            name: "Safari 浏览历史".to_string(),
            description: "Safari 浏览器历史、书签和缓存".to_string(),
            path: "~/Library/Safari".to_string(),
            entry_count: safari_history,
            size_estimate: calculate_directory_size(&format!("{}/Library/Safari", home)),
            risk_level: "high".to_string(),
            category: "浏览器记录".to_string(),
            platform: "macos".to_string(),
            requires_admin: false,
            last_modified: None,
        });
    }

    Ok(RegistryStatus {
        platform: "macOS".to_string(),
        supported: true,
        items,
        total_entries,
        categories: vec![
            "使用记录".to_string(),
            "访问记录".to_string(),
            "权限记录".to_string(),
            "预览缓存".to_string(),
            "搜索记录".to_string(),
            "命令历史".to_string(),
            "下载记录".to_string(),
            "浏览器记录".to_string(),
        ],
    })
}

/// Linux 隐私痕迹信息
#[cfg(target_os = "linux")]
fn get_linux_privacy_info() -> Result<RegistryStatus, String> {
    let mut items = Vec::new();
    let mut total_entries = 0u32;
    let home = std::env::var("HOME").unwrap_or_default();

    // 最近使用的文件
    let recently_used = count_file_lines(&format!("{}/.local/share/recently-used.xbel", home)).unwrap_or(0);
    total_entries += recently_used;

    items.push(RegistryItemInfo {
        key: "recently_used".to_string(),
        name: "最近使用的文件".to_string(),
        description: "最近打开的文件和文档记录".to_string(),
        path: "~/.local/share/recently-used.xbel".to_string(),
        entry_count: recently_used,
        size_estimate: format_entry_size(recently_used),
        risk_level: "high".to_string(),
        category: "使用记录".to_string(),
        platform: "linux".to_string(),
        requires_admin: false,
        last_modified: None,
    });

    // Bash 历史
    let bash_history = count_file_lines(&format!("{}/.bash_history", home)).unwrap_or(0);
    total_entries += bash_history;

    items.push(RegistryItemInfo {
        key: "bash_history".to_string(),
        name: "Bash 命令历史".to_string(),
        description: "Bash shell 命令行历史".to_string(),
        path: "~/.bash_history".to_string(),
        entry_count: bash_history,
        size_estimate: format_entry_size(bash_history),
        risk_level: "high".to_string(),
        category: "命令历史".to_string(),
        platform: "linux".to_string(),
        requires_admin: false,
        last_modified: None,
    });

    // Zsh 历史
    let zsh_history = count_file_lines(&format!("{}/.zsh_history", home)).unwrap_or(0);
    if zsh_history > 0 {
        total_entries += zsh_history;
        items.push(RegistryItemInfo {
            key: "zsh_history".to_string(),
            name: "Zsh 命令历史".to_string(),
            description: "Zsh shell 命令行历史".to_string(),
            path: "~/.zsh_history".to_string(),
            entry_count: zsh_history,
            size_estimate: format_entry_size(zsh_history),
            risk_level: "high".to_string(),
            category: "命令历史".to_string(),
            platform: "linux".to_string(),
            requires_admin: false,
            last_modified: None,
        });
    }

    // 缩略图缓存
    let thumbnails = count_directory_entries(&format!("{}/.cache/thumbnails", home)).unwrap_or(0);
    total_entries += thumbnails;

    items.push(RegistryItemInfo {
        key: "thumbnails".to_string(),
        name: "缩略图缓存".to_string(),
        description: "文件管理器生成的缩略图".to_string(),
        path: "~/.cache/thumbnails".to_string(),
        entry_count: thumbnails,
        size_estimate: calculate_directory_size(&format!("{}/.cache/thumbnails", home)),
        risk_level: "medium".to_string(),
        category: "预览缓存".to_string(),
        platform: "linux".to_string(),
        requires_admin: false,
        last_modified: None,
    });

    // Trash
    let trash_count = count_directory_entries(&format!("{}/.local/share/Trash/files", home)).unwrap_or(0);
    total_entries += trash_count;

    items.push(RegistryItemInfo {
        key: "trash".to_string(),
        name: "回收站".to_string(),
        description: "已删除但未清空的文件".to_string(),
        path: "~/.local/share/Trash".to_string(),
        entry_count: trash_count,
        size_estimate: calculate_directory_size(&format!("{}/.local/share/Trash", home)),
        risk_level: "high".to_string(),
        category: "回收站".to_string(),
        platform: "linux".to_string(),
        requires_admin: false,
        last_modified: None,
    });

    // Vim 历史
    let vim_history = count_file_lines(&format!("{}/.viminfo", home)).unwrap_or(0);
    if vim_history > 0 {
        total_entries += vim_history;
        items.push(RegistryItemInfo {
            key: "viminfo".to_string(),
            name: "Vim 历史".to_string(),
            description: "Vim 编辑器命令和文件历史".to_string(),
            path: "~/.viminfo".to_string(),
            entry_count: vim_history,
            size_estimate: format_entry_size(vim_history),
            risk_level: "medium".to_string(),
            category: "编辑器历史".to_string(),
            platform: "linux".to_string(),
            requires_admin: false,
            last_modified: None,
        });
    }

    Ok(RegistryStatus {
        platform: "Linux".to_string(),
        supported: true,
        items,
        total_entries,
        categories: vec![
            "使用记录".to_string(),
            "命令历史".to_string(),
            "预览缓存".to_string(),
            "回收站".to_string(),
            "编辑器历史".to_string(),
        ],
    })
}

// ============ 辅助函数 ============

fn format_entry_size(count: u32) -> String {
    if count == 0 {
        return "0 B".to_string();
    }
    // 估算每条记录约 100 字节
    let bytes = count as u64 * 100;
    if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(target_os = "windows")]
fn count_registry_entries(key_path: &str) -> Result<u32, String> {
    unsafe {
        let key_path_wide: Vec<u16> = OsStr::new(key_path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut hkey: HKEY = HKEY::default();

        let result = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR::from_raw(key_path_wide.as_ptr()),
            0,
            KEY_READ,
            &mut hkey,
        );

        if result.is_err() {
            return Ok(0);
        }

        let mut sub_keys: u32 = 0;
        let mut values: u32 = 0;

        let _ = RegQueryInfoKeyW(
            hkey,
            windows::core::PWSTR::null(),
            None,
            None,
            Some(&mut sub_keys),
            None,
            None,
            Some(&mut values),
            None,
            None,
            None,
            None,
        );

        let _ = RegCloseKey(hkey);

        Ok(sub_keys + values)
    }
}

#[cfg(target_os = "windows")]
fn count_recent_files() -> Result<u32, String> {
    let recent_folder = std::env::var("APPDATA")
        .map(|appdata| format!("{}\\Microsoft\\Windows\\Recent", appdata))
        .map_err(|e| e.to_string())?;

    count_directory_entries(&recent_folder)
}

#[cfg(not(target_os = "windows"))]
fn count_registry_entries(_key_path: &str) -> Result<u32, String> {
    Ok(0)
}

#[cfg(not(target_os = "windows"))]
fn count_recent_files() -> Result<u32, String> {
    Ok(0)
}

fn count_directory_entries(path: &str) -> Result<u32, String> {
    let path = std::path::Path::new(path);
    if !path.exists() {
        return Ok(0);
    }

    let count = std::fs::read_dir(path)
        .map(|entries| entries.count() as u32)
        .unwrap_or(0);

    Ok(count)
}

fn count_file_lines(path: &str) -> Result<u32, String> {
    let path = std::path::Path::new(path);
    if !path.exists() {
        return Ok(0);
    }

    let content = std::fs::read_to_string(path).unwrap_or_default();
    Ok(content.lines().count() as u32)
}

#[cfg(target_os = "macos")]
fn count_plist_entries(path: &str) -> Result<u32, String> {
    // 使用 plutil 获取 plist 条目数
    let output = Command::new("plutil")
        .args(["-p", path])
        .output();

    if let Ok(output) = output {
        let content = String::from_utf8_lossy(&output.stdout);
        // 简单计算行数作为条目数估计
        return Ok(content.lines().count() as u32);
    }

    Ok(0)
}

#[cfg(not(target_os = "macos"))]
fn count_plist_entries(_path: &str) -> Result<u32, String> {
    Ok(0)
}

#[cfg(target_os = "macos")]
fn count_finder_recents() -> Result<u32, String> {
    let home = std::env::var("HOME").unwrap_or_default();
    let shared_file_list = format!("{}/Library/Application Support/com.apple.sharedfilelist", home);

    count_directory_entries(&shared_file_list)
}

#[cfg(not(target_os = "macos"))]
fn count_finder_recents() -> Result<u32, String> {
    Ok(0)
}

fn calculate_directory_size(path: &str) -> String {
    let path = std::path::Path::new(path);
    if !path.exists() {
        return "0 B".to_string();
    }

    fn dir_size(path: &std::path::Path) -> u64 {
        let mut size = 0;
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    size += entry.metadata().map(|m| m.len()).unwrap_or(0);
                } else if path.is_dir() {
                    size += dir_size(&path);
                }
            }
        }
        size
    }

    let bytes = dir_size(path);
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

// ============ 清理函数 ============

/// 清理 MRU (Most Recently Used) 列表
#[cfg(target_os = "windows")]
pub fn clear_mru() -> Result<(), String> {
    let mru_keys = vec![
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\ComDlg32\OpenSavePidlMRU",
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\ComDlg32\LastVisitedPidlMRU",
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\RunMRU",
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\TypedPaths",
    ];

    for key_path in mru_keys {
        let _ = delete_registry_key(key_path);
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn clear_mru() -> Result<(), String> {
    Err("MRU clearing is only available on Windows".to_string())
}

/// 清理 UserAssist
#[cfg(target_os = "windows")]
pub fn clear_userassist() -> Result<(), String> {
    let userassist_path = r"Software\Microsoft\Windows\CurrentVersion\Explorer\UserAssist";
    let _ = delete_registry_key(userassist_path);
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn clear_userassist() -> Result<(), String> {
    Err("UserAssist clearing is only available on Windows".to_string())
}

/// 清理 ShellBags
#[cfg(target_os = "windows")]
pub fn clear_shellbags() -> Result<(), String> {
    let shellbags_keys = vec![
        r"Software\Microsoft\Windows\Shell\Bags",
        r"Software\Microsoft\Windows\Shell\BagMRU",
        r"Software\Microsoft\Windows\ShellNoRoam\Bags",
        r"Software\Microsoft\Windows\ShellNoRoam\BagMRU",
        r"Software\Classes\Local Settings\Software\Microsoft\Windows\Shell\Bags",
        r"Software\Classes\Local Settings\Software\Microsoft\Windows\Shell\BagMRU",
    ];

    for key_path in shellbags_keys {
        let _ = delete_registry_key(key_path);
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn clear_shellbags() -> Result<(), String> {
    Err("ShellBags clearing is only available on Windows".to_string())
}

/// 清理最近文档
#[cfg(target_os = "windows")]
pub fn clear_recentdocs() -> Result<(), String> {
    let recentdocs_keys = vec![
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\RecentDocs",
        r"Software\Microsoft\Office\16.0\Common\Open Find",
        r"Software\Microsoft\Office\15.0\Common\Open Find",
    ];

    for key_path in recentdocs_keys {
        let _ = delete_registry_key(key_path);
    }

    // 清理物理文件夹
    let recent_folder = std::env::var("APPDATA")
        .map(|appdata| format!("{}\\Microsoft\\Windows\\Recent", appdata))
        .ok();

    if let Some(folder) = recent_folder {
        let _ = Command::new("cmd")
            .args(["/C", "del", "/F", "/Q", &format!("{}\\*", folder)])
            .output();
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn clear_recentdocs() -> Result<(), String> {
    Err("Recent documents clearing is only available on Windows".to_string())
}

/// 清理 USB 设备历史
#[cfg(target_os = "windows")]
pub fn clear_usbhistory() -> Result<(), String> {
    let usb_keys = vec![
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2",
    ];

    for key_path in usb_keys {
        let _ = delete_registry_key(key_path);
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn clear_usbhistory() -> Result<(), String> {
    Err("USB history clearing is only available on Windows".to_string())
}

/// 清理搜索历史
#[cfg(target_os = "windows")]
pub fn clear_search() -> Result<(), String> {
    let search_keys = vec![
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\WordWheelQuery",
    ];

    for key_path in search_keys {
        let _ = delete_registry_key(key_path);
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn clear_search() -> Result<(), String> {
    Err("Search history clearing is only available on Windows".to_string())
}

/// 清理网络历史
#[cfg(target_os = "windows")]
pub fn clear_network_history() -> Result<(), String> {
    let network_keys = vec![
        r"Network",
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\Map Network Drive MRU",
    ];

    for key_path in network_keys {
        let _ = delete_registry_key(key_path);
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn clear_network_history() -> Result<(), String> {
    Err("Network history clearing is only available on Windows".to_string())
}

/// 删除注册表键
#[cfg(target_os = "windows")]
fn delete_registry_key(key_path: &str) -> Result<(), String> {
    unsafe {
        let key_path_wide: Vec<u16> = OsStr::new(key_path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut hkey: HKEY = HKEY::default();

        let result = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR::from_raw(key_path_wide.as_ptr()),
            0,
            KEY_ALL_ACCESS,
            &mut hkey,
        );

        if result.is_err() {
            return Ok(());
        }

        let _ = RegDeleteKeyW(HKEY_CURRENT_USER, PCWSTR::from_raw(key_path_wide.as_ptr()));
        let _ = RegCloseKey(hkey);
    }

    Ok(())
}

// ============ macOS 清理函数 ============

#[cfg(target_os = "macos")]
pub fn clear_recent_items() -> Result<(), String> {
    let home = std::env::var("HOME").unwrap_or_default();

    // 清理最近项目
    let shared_file_list = format!("{}/Library/Application Support/com.apple.sharedfilelist", home);
    if std::path::Path::new(&shared_file_list).exists() {
        let _ = std::fs::remove_dir_all(&shared_file_list);
    }

    // 使用 osascript 清除最近项目
    let _ = Command::new("osascript")
        .args(["-e", "tell application \"System Events\" to delete every recent item"])
        .output();

    Ok(())
}

#[cfg(target_os = "macos")]
pub fn clear_finder_recents() -> Result<(), String> {
    let home = std::env::var("HOME").unwrap_or_default();

    // 清理 Finder 偏好设置
    let _ = Command::new("defaults")
        .args(["delete", "com.apple.finder", "FXRecentFolders"])
        .output();

    let _ = Command::new("defaults")
        .args(["delete", "com.apple.finder", "RecentMoveAndCopyDestinations"])
        .output();

    // 清理 GoToField 历史
    let _ = Command::new("defaults")
        .args(["delete", "com.apple.finder", "GoToField"])
        .output();

    // 重启 Finder
    let _ = Command::new("killall")
        .args(["Finder"])
        .output();

    Ok(())
}

#[cfg(target_os = "macos")]
pub fn clear_quicklook_cache() -> Result<(), String> {
    let home = std::env::var("HOME").unwrap_or_default();

    // 清理 QuickLook 缓存
    let quicklook_cache = format!("{}/Library/Caches/com.apple.QuickLook.thumbnailcache", home);
    if std::path::Path::new(&quicklook_cache).exists() {
        let _ = std::fs::remove_dir_all(&quicklook_cache);
    }

    // 使用 qlmanage 清理
    let _ = Command::new("qlmanage")
        .args(["-r", "cache"])
        .output();

    Ok(())
}

#[cfg(target_os = "macos")]
pub fn clear_shell_history() -> Result<(), String> {
    let home = std::env::var("HOME").unwrap_or_default();

    // 清理 bash 历史
    let bash_history = format!("{}/.bash_history", home);
    if std::path::Path::new(&bash_history).exists() {
        let _ = std::fs::write(&bash_history, "");
    }

    // 清理 zsh 历史
    let zsh_history = format!("{}/.zsh_history", home);
    if std::path::Path::new(&zsh_history).exists() {
        let _ = std::fs::write(&zsh_history, "");
    }

    Ok(())
}

#[cfg(target_os = "macos")]
pub fn clear_quarantine() -> Result<(), String> {
    let home = std::env::var("HOME").unwrap_or_default();

    // 清理下载隔离数据库
    let quarantine_db = format!("{}/Library/Preferences/com.apple.LaunchServices.QuarantineEventsV2", home);
    if std::path::Path::new(&quarantine_db).exists() {
        let _ = std::fs::remove_file(&quarantine_db);
    }

    Ok(())
}

#[cfg(target_os = "macos")]
pub fn clear_safari_history() -> Result<(), String> {
    let home = std::env::var("HOME").unwrap_or_default();

    // 清理 Safari 历史
    let history_db = format!("{}/Library/Safari/History.db", home);
    if std::path::Path::new(&history_db).exists() {
        let _ = std::fs::remove_file(&history_db);
    }

    // 清理 Safari 下载列表
    let downloads_plist = format!("{}/Library/Safari/Downloads.plist", home);
    if std::path::Path::new(&downloads_plist).exists() {
        let _ = std::fs::remove_file(&downloads_plist);
    }

    Ok(())
}

// ============ Linux 清理函数 ============

#[cfg(target_os = "linux")]
pub fn clear_recently_used() -> Result<(), String> {
    let home = std::env::var("HOME").unwrap_or_default();

    let recently_used = format!("{}/.local/share/recently-used.xbel", home);
    if std::path::Path::new(&recently_used).exists() {
        let _ = std::fs::write(&recently_used, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<xbel version=\"1.0\">\n</xbel>");
    }

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn clear_bash_history() -> Result<(), String> {
    let home = std::env::var("HOME").unwrap_or_default();

    let bash_history = format!("{}/.bash_history", home);
    if std::path::Path::new(&bash_history).exists() {
        let _ = std::fs::write(&bash_history, "");
    }

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn clear_zsh_history() -> Result<(), String> {
    let home = std::env::var("HOME").unwrap_or_default();

    let zsh_history = format!("{}/.zsh_history", home);
    if std::path::Path::new(&zsh_history).exists() {
        let _ = std::fs::write(&zsh_history, "");
    }

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn clear_thumbnails() -> Result<(), String> {
    let home = std::env::var("HOME").unwrap_or_default();

    let thumbnails = format!("{}/.cache/thumbnails", home);
    if std::path::Path::new(&thumbnails).exists() {
        let _ = std::fs::remove_dir_all(&thumbnails);
        let _ = std::fs::create_dir_all(&thumbnails);
    }

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn clear_trash() -> Result<(), String> {
    let home = std::env::var("HOME").unwrap_or_default();

    let trash_files = format!("{}/.local/share/Trash/files", home);
    let trash_info = format!("{}/.local/share/Trash/info", home);

    if std::path::Path::new(&trash_files).exists() {
        let _ = std::fs::remove_dir_all(&trash_files);
        let _ = std::fs::create_dir_all(&trash_files);
    }

    if std::path::Path::new(&trash_info).exists() {
        let _ = std::fs::remove_dir_all(&trash_info);
        let _ = std::fs::create_dir_all(&trash_info);
    }

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn clear_viminfo() -> Result<(), String> {
    let home = std::env::var("HOME").unwrap_or_default();

    let viminfo = format!("{}/.viminfo", home);
    if std::path::Path::new(&viminfo).exists() {
        let _ = std::fs::remove_file(&viminfo);
    }

    Ok(())
}

/// 清理注册表/隐私痕迹
pub fn clean_registry(types: Vec<String>) -> Result<Vec<String>, String> {
    let mut cleaned = Vec::new();
    let mut errors = Vec::new();

    for registry_type in types {
        let result = match registry_type.as_str() {
            // Windows
            "mru" | "mru_lists" => clear_mru(),
            "userassist" | "user_assist" => clear_userassist(),
            "shellbags" => clear_shellbags(),
            "recentdocs" | "recent_docs" => clear_recentdocs(),
            "usbhistory" | "usb_history" => clear_usbhistory(),
            "search" | "run_history" => clear_search(),
            "network" | "typed_paths" => clear_network_history(),

            // macOS
            #[cfg(target_os = "macos")]
            "recent_items" => clear_recent_items(),
            #[cfg(target_os = "macos")]
            "finder_recents" => clear_finder_recents(),
            #[cfg(target_os = "macos")]
            "quicklook" => clear_quicklook_cache(),
            #[cfg(target_os = "macos")]
            "shell_history" => clear_shell_history(),
            #[cfg(target_os = "macos")]
            "quarantine" => clear_quarantine(),
            #[cfg(target_os = "macos")]
            "safari" => clear_safari_history(),
            #[cfg(target_os = "macos")]
            "spotlight" => {
                // Spotlight 需要 sudo 权限，这里只清理用户历史
                let home = std::env::var("HOME").unwrap_or_default();
                let _ = Command::new("defaults")
                    .args(["delete", "com.apple.Spotlight", "UserShortcuts"])
                    .output();
                Ok(())
            },
            #[cfg(target_os = "macos")]
            "app_usage" => {
                // TCC database requires SIP disabled, skip here
                Err("FEATURE_NOT_AVAILABLE:tccRequiresSIP".to_string())
            },

            // Linux
            #[cfg(target_os = "linux")]
            "recently_used" => clear_recently_used(),
            #[cfg(target_os = "linux")]
            "bash_history" => clear_bash_history(),
            #[cfg(target_os = "linux")]
            "zsh_history" => clear_zsh_history(),
            #[cfg(target_os = "linux")]
            "thumbnails" => clear_thumbnails(),
            #[cfg(target_os = "linux")]
            "trash" => clear_trash(),
            #[cfg(target_os = "linux")]
            "viminfo" => clear_viminfo(),

            _ => Err(format!("未知的类型: {}", registry_type)),
        };

        match result {
            Ok(_) => cleaned.push(registry_type),
            Err(e) => errors.push(format!("{}: {}", registry_type, e)),
        }
    }

    if !errors.is_empty() && cleaned.is_empty() {
        return Err(format!(
            "清理失败:\n{}",
            errors.join("\n")
        ));
    }

    // errors logged silently

    Ok(cleaned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_registry_info() {
        let result = get_registry_info();
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_entry_size() {
        assert_eq!(format_entry_size(0), "0 B");
        assert_eq!(format_entry_size(10), "1000 B");
        assert_eq!(format_entry_size(100), "9.8 KB");
    }
}
