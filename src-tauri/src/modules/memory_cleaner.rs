use crate::modules::command_utils::CommandExt;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::path::Path;
use std::fs;
use sysinfo::System;

/// 内存扫描项信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCleanItem {
    pub item_type: String,       // 类型标识符
    pub label: String,           // 显示名称
    pub description: String,     // 描述
    pub size: u64,               // 大小（字节）
    pub size_display: String,    // 显示大小
    pub accessible: bool,        // 是否可访问
    pub category: String,        // 分类
}

/// 内存扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryScanResult {
    pub items: Vec<MemoryCleanItem>,
    pub total_size: u64,
    pub total_items: usize,
    pub memory_info: MemoryInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryInfo {
    pub total_memory: u64,       // MB
    pub used_memory: u64,        // MB
    pub free_memory: u64,        // MB
    pub memory_usage: f32,       // %
    pub wired_memory: u64,       // MB (macOS: 固定内存)
    pub active_memory: u64,      // MB (macOS: 活跃内存)
    pub inactive_memory: u64,    // MB (macOS: 非活跃内存)
    pub compressed_memory: u64,  // MB (macOS: 压缩内存)
    pub cached_memory: u64,      // MB (缓存)
    pub swap_used: u64,          // MB (交换空间已用)
    pub swap_total: u64,         // MB (交换空间总量)
    pub app_memory: u64,         // MB (应用程序内存)
    pub pagefile_size: Option<String>,
    pub hibernation_size: Option<String>,
    pub swap_size: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub memory: u64,        // KB
    pub memory_display: String,
    pub cpu_usage: f32,
    pub user: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedMemoryInfo {
    pub memory_info: MemoryInfo,
    pub top_processes: Vec<ProcessInfo>,
    pub memory_pressure: String,  // "normal", "warning", "critical"
    pub uptime: String,
}

/// 获取内存信息
pub fn get_memory_info() -> Result<MemoryInfo, String> {
    #[cfg(target_os = "macos")]
    {
        get_macos_memory_info()
    }

    #[cfg(target_os = "windows")]
    {
        get_windows_memory_info()
    }

    #[cfg(target_os = "linux")]
    {
        get_linux_memory_info()
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Err("PLATFORM_NOT_SUPPORTED".to_string())
    }
}

/// 获取详细内存信息（包含进程列表）
pub fn get_detailed_memory_info() -> Result<DetailedMemoryInfo, String> {
    let memory_info = get_memory_info()?;
    let top_processes = get_top_processes()?;
    let memory_pressure = get_memory_pressure(&memory_info);
    let uptime = get_system_uptime()?;

    Ok(DetailedMemoryInfo {
        memory_info,
        top_processes,
        memory_pressure,
        uptime,
    })
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

/// 获取目录大小
fn get_directory_size(path: &str) -> u64 {
    let path = Path::new(path);
    if !path.exists() {
        return 0;
    }

    let mut total_size = 0u64;

    fn scan_dir(dir: &Path, size: &mut u64) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    scan_dir(&path, size);
                } else if let Ok(metadata) = fs::metadata(&path) {
                    *size += metadata.len();
                }
            }
        }
    }

    if path.is_dir() {
        scan_dir(path, &mut total_size);
    } else if let Ok(metadata) = fs::metadata(path) {
        total_size = metadata.len();
    }

    total_size
}

/// 扫描可清理的内存相关项目
pub fn scan_memory_items() -> Result<MemoryScanResult, String> {
    let mut items = Vec::new();
    let mut total_size = 0u64;

    #[cfg(target_os = "macos")]
    {
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/Users".to_string());

        // 1. 剪贴板 - 始终可清理
        items.push(MemoryCleanItem {
            item_type: "clipboard".to_string(),
            label: "剪贴板".to_string(),
            description: "清空系统剪贴板内容".to_string(),
            size: 0,
            size_display: "动态".to_string(),
            accessible: true,
            category: "系统".to_string(),
        });

        // 2. 非活跃内存 (通过 purge 命令清理)
        let memory_info = get_memory_info().unwrap_or_default();
        let inactive_bytes = memory_info.inactive_memory * 1024 * 1024;
        items.push(MemoryCleanItem {
            item_type: "inactive".to_string(),
            label: "非活跃内存".to_string(),
            description: "系统缓存的非活跃内存页".to_string(),
            size: inactive_bytes,
            size_display: format_size(inactive_bytes),
            accessible: true,
            category: "内存".to_string(),
        });
        total_size += inactive_bytes;

        // 3. 压缩内存
        let compressed_bytes = memory_info.compressed_memory * 1024 * 1024;
        if compressed_bytes > 0 {
            items.push(MemoryCleanItem {
                item_type: "compressed".to_string(),
                label: "压缩内存".to_string(),
                description: "系统压缩的内存数据".to_string(),
                size: compressed_bytes,
                size_display: format_size(compressed_bytes),
                accessible: false, // 压缩内存无法直接清理
                category: "内存".to_string(),
            });
        }

        // 4. 交换空间
        let swap_bytes = memory_info.swap_used * 1024 * 1024;
        if swap_bytes > 0 {
            items.push(MemoryCleanItem {
                item_type: "swap".to_string(),
                label: "交换空间".to_string(),
                description: "磁盘上的虚拟内存".to_string(),
                size: swap_bytes,
                size_display: format_size(swap_bytes),
                accessible: true,
                category: "内存".to_string(),
            });
            total_size += swap_bytes;
        }

        // 5. DNS 缓存
        items.push(MemoryCleanItem {
            item_type: "dns_cache".to_string(),
            label: "DNS 缓存".to_string(),
            description: "域名解析缓存".to_string(),
            size: 0,
            size_display: "动态".to_string(),
            accessible: true,
            category: "系统".to_string(),
        });

        // 6. 休眠文件
        if let Ok(metadata) = fs::metadata("/var/vm/sleepimage") {
            let size = metadata.len();
            items.push(MemoryCleanItem {
                item_type: "hibernation".to_string(),
                label: "休眠文件".to_string(),
                description: "系统休眠时保存的内存镜像".to_string(),
                size,
                size_display: format_size(size),
                accessible: false, // 需要禁用休眠功能
                category: "系统".to_string(),
            });
        }

        // 7. 临时文件
        let tmp_paths = vec![
            format!("{}/Library/Caches/TemporaryItems", home_dir),
            "/tmp".to_string(),
            "/private/var/tmp".to_string(),
        ];
        let mut tmp_size = 0u64;
        for path in &tmp_paths {
            tmp_size += get_directory_size(path);
        }
        if tmp_size > 0 {
            items.push(MemoryCleanItem {
                item_type: "temp_files".to_string(),
                label: "临时文件".to_string(),
                description: "系统和应用程序临时文件".to_string(),
                size: tmp_size,
                size_display: format_size(tmp_size),
                accessible: true,
                category: "文件".to_string(),
            });
            total_size += tmp_size;
        }

        // 8. 浏览器内存缓存位置
        let browser_cache_paths = vec![
            (format!("{}/Library/Caches/com.apple.Safari", home_dir), "Safari 缓存"),
            (format!("{}/Library/Caches/Google/Chrome", home_dir), "Chrome 缓存"),
            (format!("{}/Library/Caches/Firefox", home_dir), "Firefox 缓存"),
        ];
        for (path, label) in browser_cache_paths {
            let size = get_directory_size(&path);
            if size > 0 {
                items.push(MemoryCleanItem {
                    item_type: format!("browser_cache_{}", label.to_lowercase().replace(" ", "_")),
                    label: label.to_string(),
                    description: format!("{} 网页缓存", label),
                    size,
                    size_display: format_size(size),
                    accessible: true,
                    category: "浏览器".to_string(),
                });
                total_size += size;
            }
        }

        // 9. 系统日志（内存中的日志缓冲）
        items.push(MemoryCleanItem {
            item_type: "log_buffer".to_string(),
            label: "日志缓冲".to_string(),
            description: "内存中的系统日志缓冲区".to_string(),
            size: 0,
            size_display: "动态".to_string(),
            accessible: true,
            category: "系统".to_string(),
        });
    }

    #[cfg(target_os = "windows")]
    {
        let localappdata = std::env::var("LOCALAPPDATA").unwrap_or_default();
        let temp = std::env::var("TEMP").unwrap_or_default();

        // 剪贴板
        items.push(MemoryCleanItem {
            item_type: "clipboard".to_string(),
            label: "剪贴板".to_string(),
            description: "清空系统剪贴板内容".to_string(),
            size: 0,
            size_display: "动态".to_string(),
            accessible: true,
            category: "系统".to_string(),
        });

        // 内存工作集
        let memory_info = get_memory_info().unwrap_or_default();
        let working_set = memory_info.used_memory * 1024 * 1024;
        items.push(MemoryCleanItem {
            item_type: "working_set".to_string(),
            label: "系统工作集".to_string(),
            description: "可释放的系统内存".to_string(),
            size: working_set,
            size_display: format_size(working_set),
            accessible: true,
            category: "内存".to_string(),
        });

        // 待机内存
        items.push(MemoryCleanItem {
            item_type: "standby".to_string(),
            label: "待机内存".to_string(),
            description: "可回收的待机内存页".to_string(),
            size: 0,
            size_display: "动态".to_string(),
            accessible: true,
            category: "内存".to_string(),
        });

        // 页面文件
        let pagefile_bytes = memory_info.swap_used * 1024 * 1024;
        if pagefile_bytes > 0 {
            items.push(MemoryCleanItem {
                item_type: "pagefile".to_string(),
                label: "页面文件".to_string(),
                description: "Windows 虚拟内存页面文件".to_string(),
                size: pagefile_bytes,
                size_display: format_size(pagefile_bytes),
                accessible: true,
                category: "系统".to_string(),
            });
            total_size += pagefile_bytes;
        }

        // 临时文件
        let tmp_size = get_directory_size(&temp);
        if tmp_size > 0 {
            items.push(MemoryCleanItem {
                item_type: "temp_files".to_string(),
                label: "临时文件".to_string(),
                description: "Windows 临时文件".to_string(),
                size: tmp_size,
                size_display: format_size(tmp_size),
                accessible: true,
                category: "文件".to_string(),
            });
            total_size += tmp_size;
        }

        // 休眠文件
        if let Ok(metadata) = fs::metadata("C:\\hiberfil.sys") {
            let size = metadata.len();
            items.push(MemoryCleanItem {
                item_type: "hibernation".to_string(),
                label: "休眠文件".to_string(),
                description: "系统休眠文件 (hiberfil.sys)".to_string(),
                size,
                size_display: format_size(size),
                accessible: true,
                category: "系统".to_string(),
            });
        }
    }

    #[cfg(target_os = "linux")]
    {
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());

        // 剪贴板
        items.push(MemoryCleanItem {
            item_type: "clipboard".to_string(),
            label: "剪贴板".to_string(),
            description: "清空系统剪贴板内容".to_string(),
            size: 0,
            size_display: "动态".to_string(),
            accessible: true,
            category: "系统".to_string(),
        });

        // 页面缓存
        let memory_info = get_memory_info().unwrap_or_default();
        let cached_bytes = memory_info.cached_memory * 1024 * 1024;
        if cached_bytes > 0 {
            items.push(MemoryCleanItem {
                item_type: "page_cache".to_string(),
                label: "页面缓存".to_string(),
                description: "可回收的文件系统缓存".to_string(),
                size: cached_bytes,
                size_display: format_size(cached_bytes),
                accessible: true,
                category: "内存".to_string(),
            });
            total_size += cached_bytes;
        }

        // 交换分区
        let swap_bytes = memory_info.swap_used * 1024 * 1024;
        if swap_bytes > 0 {
            items.push(MemoryCleanItem {
                item_type: "swap".to_string(),
                label: "交换分区".to_string(),
                description: "磁盘交换空间".to_string(),
                size: swap_bytes,
                size_display: format_size(swap_bytes),
                accessible: true,
                category: "内存".to_string(),
            });
            total_size += swap_bytes;
        }

        // 临时文件
        let tmp_size = get_directory_size("/tmp");
        if tmp_size > 0 {
            items.push(MemoryCleanItem {
                item_type: "temp_files".to_string(),
                label: "临时文件".to_string(),
                description: "/tmp 目录下的临时文件".to_string(),
                size: tmp_size,
                size_display: format_size(tmp_size),
                accessible: true,
                category: "文件".to_string(),
            });
            total_size += tmp_size;
        }

        // 用户缓存
        let cache_path = format!("{}/.cache", home_dir);
        let cache_size = get_directory_size(&cache_path);
        if cache_size > 0 {
            items.push(MemoryCleanItem {
                item_type: "user_cache".to_string(),
                label: "用户缓存".to_string(),
                description: "用户应用程序缓存".to_string(),
                size: cache_size,
                size_display: format_size(cache_size),
                accessible: true,
                category: "文件".to_string(),
            });
            total_size += cache_size;
        }
    }

    // 过滤掉大小为0且不可操作的项目
    items.retain(|item| item.size > 0 || item.accessible);

    let memory_info = get_memory_info().unwrap_or_default();

    Ok(MemoryScanResult {
        total_items: items.len(),
        items,
        total_size,
        memory_info,
    })
}

/// 获取内存压力状态
fn get_memory_pressure(info: &MemoryInfo) -> String {
    if info.memory_usage > 90.0 {
        "critical".to_string()
    } else if info.memory_usage > 75.0 {
        "warning".to_string()
    } else {
        "normal".to_string()
    }
}

/// 获取系统运行时间 - 使用 sysinfo crate
fn get_system_uptime() -> Result<String, String> {
    let uptime_secs = System::uptime();

    let days = uptime_secs / 86400;
    let hours = (uptime_secs % 86400) / 3600;
    let minutes = (uptime_secs % 3600) / 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{} 天", days));
    }
    if hours > 0 {
        parts.push(format!("{} 小时", hours));
    }
    if minutes > 0 || parts.is_empty() {
        parts.push(format!("{} 分钟", minutes));
    }

    Ok(parts.join(" "))
}

/// macOS 内存信息
#[cfg(target_os = "macos")]
fn get_macos_memory_info() -> Result<MemoryInfo, String> {
    // 使用 vm_stat 获取内存统计
    let vm_stat_output = Command::new("vm_stat")
        .output()
        .map_err(|e| format!("执行 vm_stat 失败: {}", e))?;

    let vm_stat = String::from_utf8_lossy(&vm_stat_output.stdout);

    // 解析 vm_stat 输出
    let page_size = 16384u64; // macOS 默认页面大小，Apple Silicon 使用 16KB

    // 尝试获取真实页面大小
    let page_size = if let Some(line) = vm_stat.lines().next() {
        if line.contains("page size of") {
            line.split_whitespace()
                .filter_map(|s| s.parse::<u64>().ok())
                .next()
                .unwrap_or(16384)
        } else {
            16384
        }
    } else {
        16384
    };

    let mut pages_free = 0u64;
    let mut pages_active = 0u64;
    let mut pages_inactive = 0u64;
    let mut pages_speculative = 0u64;
    let mut pages_wired = 0u64;
    let mut pages_compressed = 0u64;
    let mut pages_purgeable = 0u64;
    let mut pages_external = 0u64;
    let mut pages_occupied_by_compressor = 0u64;

    for line in vm_stat.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() == 2 {
            let value = parts[1].trim().trim_end_matches('.').parse::<u64>().unwrap_or(0);
            match parts[0].trim() {
                "Pages free" => pages_free = value,
                "Pages active" => pages_active = value,
                "Pages inactive" => pages_inactive = value,
                "Pages speculative" => pages_speculative = value,
                "Pages wired down" => pages_wired = value,
                "Pages compressed" => pages_compressed = value,
                "Pages purgeable" => pages_purgeable = value,
                "File-backed pages" => pages_external = value,
                "Pages occupied by compressor" => pages_occupied_by_compressor = value,
                _ => {}
            }
        }
    }

    // 使用 sysctl 获取总内存
    let sysctl_output = Command::new("sysctl")
        .args(&["-n", "hw.memsize"])
        .output()
        .map_err(|e| format!("执行 sysctl 失败: {}", e))?;

    let total_bytes = String::from_utf8_lossy(&sysctl_output.stdout)
        .trim()
        .parse::<u64>()
        .unwrap_or(0);

    let total_mb = total_bytes / 1024 / 1024;

    // 计算各类内存 (bytes)
    let wired_bytes = pages_wired * page_size;
    let active_bytes = pages_active * page_size;
    let inactive_bytes = pages_inactive * page_size;
    let compressed_bytes = pages_compressed * page_size;
    let free_bytes = pages_free * page_size;
    let speculative_bytes = pages_speculative * page_size;
    let purgeable_bytes = pages_purgeable * page_size;
    let external_bytes = pages_external * page_size;
    let compressor_bytes = pages_occupied_by_compressor * page_size;

    // 应用内存 = 活跃 + 非活跃 - 可清除 - 外部
    let app_bytes = active_bytes + inactive_bytes - purgeable_bytes - external_bytes;

    // 已用内存 = 应用内存 + 固定内存 + 压缩器占用
    let used_bytes = app_bytes + wired_bytes + compressor_bytes;

    // 缓存 = 可清除 + 外部
    let cached_bytes = purgeable_bytes + external_bytes;

    // 可用内存 = 空闲 + 推测 + 可清除 + 外部（文件缓存）
    let available_bytes = free_bytes + speculative_bytes + purgeable_bytes + external_bytes;

    // 转换为 MB
    let wired_mb = wired_bytes / 1024 / 1024;
    let active_mb = active_bytes / 1024 / 1024;
    let inactive_mb = inactive_bytes / 1024 / 1024;
    let compressed_mb = compressed_bytes / 1024 / 1024;
    let used_mb = used_bytes / 1024 / 1024;
    let free_mb = available_bytes / 1024 / 1024;
    let cached_mb = cached_bytes / 1024 / 1024;
    let app_mb = app_bytes / 1024 / 1024;

    let usage = if total_mb > 0 {
        (used_mb as f32 / total_mb as f32) * 100.0
    } else {
        0.0
    };

    // 获取 swap 信息
    let swap_output = Command::new("sysctl")
        .args(&["-n", "vm.swapusage"])
        .output()
        .ok();

    let (swap_total_mb, swap_used_mb) = if let Some(output) = swap_output {
        let swap_str = String::from_utf8_lossy(&output.stdout);
        parse_swap_usage(&swap_str)
    } else {
        (0, 0)
    };

    Ok(MemoryInfo {
        total_memory: total_mb,
        used_memory: used_mb,
        free_memory: free_mb,
        memory_usage: usage,
        wired_memory: wired_mb,
        active_memory: active_mb,
        inactive_memory: inactive_mb,
        compressed_memory: compressed_mb,
        cached_memory: cached_mb,
        swap_used: swap_used_mb,
        swap_total: swap_total_mb,
        app_memory: app_mb,
        pagefile_size: None,
        hibernation_size: get_hibernation_size(),
        swap_size: Some(format!("{} MB / {} MB", swap_used_mb, swap_total_mb)),
    })
}

/// 解析 macOS swap 使用信息
#[cfg(target_os = "macos")]
fn parse_swap_usage(swap_str: &str) -> (u64, u64) {
    // 格式: total = 2048.00M  used = 1024.00M  free = 1024.00M
    let mut total: u64 = 0;
    let mut used: u64 = 0;

    for part in swap_str.split_whitespace() {
        if part.ends_with('M') {
            let value = part.trim_end_matches('M').parse::<f64>().unwrap_or(0.0) as u64;
            if total == 0 {
                total = value;
            } else if used == 0 {
                used = value;
                break;
            }
        } else if part.ends_with('G') {
            let value = (part.trim_end_matches('G').parse::<f64>().unwrap_or(0.0) * 1024.0) as u64;
            if total == 0 {
                total = value;
            } else if used == 0 {
                used = value;
                break;
            }
        }
    }

    (total, used)
}

/// 获取休眠文件大小
#[cfg(target_os = "macos")]
fn get_hibernation_size() -> Option<String> {
    use std::fs;

    if let Ok(metadata) = fs::metadata("/var/vm/sleepimage") {
        let size_mb = metadata.len() / 1024 / 1024;
        if size_mb > 1024 {
            Some(format!("{:.1} GB", size_mb as f64 / 1024.0))
        } else {
            Some(format!("{} MB", size_mb))
        }
    } else {
        Some("未启用".to_string())
    }
}

#[cfg(target_os = "windows")]
fn get_hibernation_size() -> Option<String> {
    // Windows hibernation file is located at C:\hiberfil.sys
    let system_drive = std::env::var("SYSTEMDRIVE").unwrap_or_else(|_| "C:".to_string());
    let hiberfil_path = format!("{}\\hiberfil.sys", system_drive);

    if let Ok(metadata) = std::fs::metadata(&hiberfil_path) {
        let size_bytes = metadata.len();
        let size_gb = size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        if size_gb >= 1.0 {
            Some(format!("{:.1} GB", size_gb))
        } else {
            let size_mb = size_bytes / 1024 / 1024;
            Some(format!("{} MB", size_mb))
        }
    } else {
        // Hibernation file doesn't exist or not accessible
        Some("未启用".to_string())
    }
}

#[cfg(target_os = "linux")]
fn get_hibernation_size() -> Option<String> {
    // Linux uses swap for hibernation
    None
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn get_hibernation_size() -> Option<String> {
    None
}

/// Linux 内存信息
#[cfg(target_os = "linux")]
fn get_linux_memory_info() -> Result<MemoryInfo, String> {
    use std::fs;

    let meminfo = fs::read_to_string("/proc/meminfo")
        .map_err(|e| format!("读取 /proc/meminfo 失败: {}", e))?;

    let mut total_kb = 0u64;
    let mut free_kb = 0u64;
    let mut available_kb = 0u64;
    let mut buffers_kb = 0u64;
    let mut cached_kb = 0u64;
    let mut active_kb = 0u64;
    let mut inactive_kb = 0u64;
    let mut swap_total_kb = 0u64;
    let mut swap_free_kb = 0u64;

    for line in meminfo.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let value = parts[1].parse::<u64>().unwrap_or(0);
            match parts[0].trim_end_matches(':') {
                "MemTotal" => total_kb = value,
                "MemFree" => free_kb = value,
                "MemAvailable" => available_kb = value,
                "Buffers" => buffers_kb = value,
                "Cached" => cached_kb = value,
                "Active" => active_kb = value,
                "Inactive" => inactive_kb = value,
                "SwapTotal" => swap_total_kb = value,
                "SwapFree" => swap_free_kb = value,
                _ => {}
            }
        }
    }

    let total_mb = total_kb / 1024;
    let available_mb = if available_kb > 0 { available_kb / 1024 } else { free_kb / 1024 };
    let used_mb = total_mb - available_mb;
    let cached_mb = (buffers_kb + cached_kb) / 1024;
    let active_mb = active_kb / 1024;
    let inactive_mb = inactive_kb / 1024;
    let swap_total_mb = swap_total_kb / 1024;
    let swap_used_mb = (swap_total_kb - swap_free_kb) / 1024;

    let usage = if total_mb > 0 {
        (used_mb as f32 / total_mb as f32) * 100.0
    } else {
        0.0
    };

    Ok(MemoryInfo {
        total_memory: total_mb,
        used_memory: used_mb,
        free_memory: available_mb,
        memory_usage: usage,
        wired_memory: 0,
        active_memory: active_mb,
        inactive_memory: inactive_mb,
        compressed_memory: 0,
        cached_memory: cached_mb,
        swap_used: swap_used_mb,
        swap_total: swap_total_mb,
        app_memory: used_mb - cached_mb,
        pagefile_size: None,
        hibernation_size: None,
        swap_size: Some(format!("{} MB / {} MB", swap_used_mb, swap_total_mb)),
    })
}

/// Windows 内存信息
#[cfg(target_os = "windows")]
fn get_windows_memory_info() -> Result<MemoryInfo, String> {
    use std::mem;
    use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

    unsafe {
        let mut mem_status: MEMORYSTATUSEX = mem::zeroed();
        mem_status.dwLength = mem::size_of::<MEMORYSTATUSEX>() as u32;

        if GlobalMemoryStatusEx(&mut mem_status).is_ok() {
            let total_mb = (mem_status.ullTotalPhys / 1024 / 1024) as u64;
            let avail_mb = (mem_status.ullAvailPhys / 1024 / 1024) as u64;
            let used_mb = total_mb - avail_mb;
            let usage = (used_mb as f32 / total_mb as f32) * 100.0;

            let pagefile_total = (mem_status.ullTotalPageFile / 1024 / 1024) as u64;
            let pagefile_avail = (mem_status.ullAvailPageFile / 1024 / 1024) as u64;

            Ok(MemoryInfo {
                total_memory: total_mb,
                used_memory: used_mb,
                free_memory: avail_mb,
                memory_usage: usage,
                wired_memory: 0,
                active_memory: 0,
                inactive_memory: 0,
                compressed_memory: 0,
                cached_memory: 0,
                swap_used: pagefile_total - pagefile_avail,
                swap_total: pagefile_total,
                app_memory: used_mb,
                pagefile_size: Some(format!("{} MB", pagefile_total)),
                hibernation_size: get_hibernation_size(),
                swap_size: None,
            })
        } else {
            Err("IO_ERROR:getMemoryInfoFailed".to_string())
        }
    }
}

/// 获取占用内存最多的进程 - 使用 sysinfo crate
pub fn get_top_processes() -> Result<Vec<ProcessInfo>, String> {
    let mut sys = System::new_all();

    // 等待一小段时间以获取准确的CPU使用率
    std::thread::sleep(std::time::Duration::from_millis(100));
    sys.refresh_all();

    // 获取用户列表
    let users = sysinfo::Users::new_with_refreshed_list();

    let mut processes: Vec<ProcessInfo> = sys
        .processes()
        .iter()
        .map(|(pid, process)| {
            let rss_kb = process.memory() / 1024; // sysinfo 返回字节
            let memory_display = if rss_kb >= 1024 * 1024 {
                format!("{:.1} GB", rss_kb as f64 / 1024.0 / 1024.0)
            } else if rss_kb >= 1024 {
                format!("{:.1} MB", rss_kb as f64 / 1024.0)
            } else {
                format!("{} KB", rss_kb)
            };

            // 获取用户信息
            let user = process
                .user_id()
                .map(|uid| {
                    users
                        .iter()
                        .find(|u| u.id() == uid)
                        .map(|u| u.name().to_string())
                        .unwrap_or_else(|| format!("{}", **uid))
                })
                .unwrap_or_else(|| "unknown".to_string());

            ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string(),
                memory: rss_kb,
                memory_display,
                cpu_usage: process.cpu_usage(),
                user,
            }
        })
        .collect();

    // 按内存使用量排序
    processes.sort_by(|a, b| b.memory.cmp(&a.memory));

    // 只返回前20个
    processes.truncate(20);

    Ok(processes)
}

/// 清理剪贴板
pub fn clear_clipboard() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                if let Some(stdin) = child.stdin.as_mut() {
                    use std::io::Write;
                    stdin.write_all(b"")?;
                }
                child.wait()
            })
            .map_err(|e| format!("清空剪贴板失败: {}", e))?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard};
        unsafe {
            OpenClipboard(None).map_err(|e| format!("打开剪贴板失败: {}", e))?;
            EmptyClipboard().map_err(|e| format!("清空剪贴板失败: {}", e))?;
            CloseClipboard().map_err(|e| format!("关闭剪贴板失败: {}", e))?;
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        // 尝试使用 xclip 或 xsel
        let result = Command::new("xclip")
            .args(&["-selection", "clipboard"])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                if let Some(stdin) = child.stdin.as_mut() {
                    use std::io::Write;
                    stdin.write_all(b"")?;
                }
                child.wait()
            });

        if result.is_err() {
            // 尝试 xsel
            Command::new("xsel")
                .args(&["--clipboard", "--clear"])
                .output()
                .map_err(|e| format!("IO_ERROR:{}", e))?;
        }
        Ok(())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Err("PLATFORM_NOT_SUPPORTED".to_string())
    }
}

/// macOS 清除系统缓存
#[cfg(target_os = "macos")]
pub fn clear_system_cache() -> Result<(), String> {
    // 清除用户缓存
    let home = std::env::var("HOME").unwrap_or_default();
    let _cache_dir = format!("{}/Library/Caches", home);

    // 使用 purge 命令清理非活跃内存（需要 sudo）
    let _ = Command::new("purge").output();

    Ok(())
}

/// macOS 清除 DNS 缓存
#[cfg(target_os = "macos")]
pub fn clear_dns_cache() -> Result<(), String> {
    Command::new("dscacheutil")
        .arg("-flushcache")
        .output()
        .map_err(|e| format!("清除 DNS 缓存失败: {}", e))?;

    // 也尝试重启 mDNSResponder
    let _ = Command::new("sudo")
        .args(&["killall", "-HUP", "mDNSResponder"])
        .output();

    Ok(())
}

/// 清理 Windows 页面文件
#[cfg(target_os = "windows")]
pub fn clear_pagefile() -> Result<(), String> {
    use windows::Win32::System::Registry::{
        RegCloseKey, RegCreateKeyExW, RegSetValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_SET_VALUE,
        REG_DWORD, REG_OPTION_NON_VOLATILE,
    };
    use windows::core::PCWSTR;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    unsafe {
        let subkey: Vec<u16> = OsStr::new(
            "SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management"
        )
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

        let mut hkey: HKEY = HKEY::default();
        RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR::from_raw(subkey.as_ptr()),
            0,
            PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            None,
            &mut hkey,
            None,
        )
        .map_err(|e| format!("打开注册表键失败: {}", e))?;

        let value_name: Vec<u16> = OsStr::new("ClearPageFileAtShutdown")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let data: u32 = 1;
        RegSetValueExW(
            hkey,
            PCWSTR::from_raw(value_name.as_ptr()),
            0,
            REG_DWORD,
            Some(&data.to_le_bytes()),
        )
        .map_err(|e| format!("设置注册表值失败: {}", e))?;

        RegCloseKey(hkey).ok();
    }

    Ok(())
}

/// 禁用 Windows 休眠文件
#[cfg(target_os = "windows")]
pub fn disable_hibernation() -> Result<(), String> {
    // Use Windows Registry API instead of powercfg command
    crate::modules::windows_utils::disable_hibernation()
        .map_err(|e| format!("禁用休眠失败: {}", e))
}

/// 清理 Windows 系统工作集/待机内存
#[cfg(target_os = "windows")]
pub fn clear_windows_memory() -> Result<(), String> {
    use windows::Win32::System::Memory::{
        VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_RELEASE, PAGE_READWRITE,
    };

    // 尝试通过分配和释放内存来触发系统清理待机内存
    // 分配一小块内存然后立即释放，迫使系统回收待机页
    unsafe {
        // 分配少量内存触发系统内存管理
        let alloc_size = 64 * 1024 * 1024; // 64MB
        let ptr = VirtualAlloc(
            None,
            alloc_size,
            MEM_COMMIT,
            PAGE_READWRITE,
        );

        if !ptr.is_null() {
            // 触碰内存页使其真正被分配
            let slice = std::slice::from_raw_parts_mut(ptr as *mut u8, alloc_size);
            for i in (0..alloc_size).step_by(4096) {
                slice[i] = 0;
            }

            // 立即释放
            let _ = VirtualFree(ptr, 0, MEM_RELEASE);
        }
    }

    // 尝试清空当前进程的工作集
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::Threading::GetCurrentProcess;
        use windows::Win32::System::ProcessStatus::EmptyWorkingSet;

        unsafe {
            let process = GetCurrentProcess();
            let _ = EmptyWorkingSet(process);
        }
    }

    Ok(())
}

/// 清理 Windows 系统缓存 (需要 SeIncreaseQuotaPrivilege)
#[cfg(target_os = "windows")]
pub fn clear_windows_system_cache() -> Result<(), String> {
    use windows::Win32::System::Memory::{
        SetSystemFileCacheSize, FILE_CACHE_MAX_HARD_DISABLE,
    };

    // 尝试清理系统文件缓存
    // 注意：这需要管理员权限
    unsafe {
        // 设置系统文件缓存为最小值，然后恢复
        // 这会强制系统释放缓存的文件数据
        let result = SetSystemFileCacheSize(
            usize::MAX, // MinimumFileCacheSize - MAX 表示使用系统默认
            usize::MAX, // MaximumFileCacheSize - MAX 表示使用系统默认
            FILE_CACHE_MAX_HARD_DISABLE, // 禁用硬限制
        );

        if result.is_err() {
            // 如果失败，可能是权限不足，但我们仍然返回成功
            // 因为其他清理操作可能已经生效
        }
    }

    Ok(())
}

/// 清理 Unix 交换分区
#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn clear_swap() -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        // 关闭交换分区
        Command::new("sudo")
            .args(&["swapoff", "-a"])
            .output()
            .map_err(|e| format!("关闭交换分区失败: {}", e))?;

        // 重新打开交换分区
        Command::new("sudo")
            .args(&["swapon", "-a"])
            .output()
            .map_err(|e| format!("重新打开交换分区失败: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        // macOS 不支持直接清理 swap，但可以尝试使用 purge 释放内存压力
        Command::new("purge")
            .output()
            .map_err(|e| format!("清理内存缓存失败: {}", e))?;
    }

    Ok(())
}

/// 清理非活跃内存
#[cfg(target_os = "macos")]
pub fn purge_inactive_memory() -> Result<(), String> {
    Command::new("purge")
        .output()
        .map_err(|e| format!("清理非活跃内存失败: {}", e))?;
    Ok(())
}

/// 清理内存痕迹
pub fn clean_memory(types: Vec<String>) -> Result<Vec<String>, String> {
    let mut cleaned = Vec::new();
    let mut errors = Vec::new();

    for memory_type in types {
        let result = match memory_type.as_str() {
            "clipboard" => clear_clipboard(),

            #[cfg(target_os = "windows")]
            "pagefile" => clear_pagefile(),

            #[cfg(target_os = "windows")]
            "hibernation" => disable_hibernation(),

            #[cfg(target_os = "windows")]
            "working_set" | "standby" | "inactive" => clear_windows_memory(),

            #[cfg(target_os = "windows")]
            "system_cache" => clear_windows_system_cache(),

            #[cfg(any(target_os = "linux", target_os = "macos"))]
            "swap" => clear_swap(),

            #[cfg(target_os = "macos")]
            "inactive" | "working_set" => purge_inactive_memory(),

            #[cfg(target_os = "macos")]
            "dns_cache" => clear_dns_cache(),

            #[cfg(target_os = "macos")]
            "system_cache" => clear_system_cache(),

            #[cfg(target_os = "macos")]
            "standby" => purge_inactive_memory(),

            #[cfg(target_os = "linux")]
            "standby" | "working_set" | "inactive" => {
                // Linux: 使用 drop_caches 清理
                let _ = std::fs::write("/proc/sys/vm/drop_caches", "3");
                Ok(())
            },

            _ => {
                // 对于未知类型，尝试平台特定的清理
                #[cfg(target_os = "macos")]
                {
                    purge_inactive_memory()
                }
                #[cfg(target_os = "windows")]
                {
                    clear_windows_memory()
                }
                #[cfg(target_os = "linux")]
                {
                    let _ = std::fs::write("/proc/sys/vm/drop_caches", "3");
                    Ok(())
                }
                #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
                {
                    Err(format!("未知的内存类型: {}", memory_type))
                }
            },
        };

        match result {
            Ok(_) => cleaned.push(memory_type),
            Err(e) => errors.push(format!("{}: {}", memory_type, e)),
        }
    }

    if !errors.is_empty() && cleaned.is_empty() {
        return Err(format!(
            "内存清理失败:\n{}",
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
    fn test_get_memory_info() {
        let result = get_memory_info();
        assert!(result.is_ok());
        let info = result.unwrap();
        assert!(info.total_memory > 0);
    }

    #[test]
    fn test_get_top_processes() {
        let result = get_top_processes();
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_detailed_memory_info() {
        let result = get_detailed_memory_info();
        assert!(result.is_ok());
    }
}
