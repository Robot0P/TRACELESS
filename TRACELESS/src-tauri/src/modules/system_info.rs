use serde::{Deserialize, Serialize};
use sysinfo::{System, Disks, Networks};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

// 用于存储上次网络数据的全局状态
static NETWORK_STATS: Mutex<Option<NetworkStats>> = Mutex::new(None);

#[derive(Clone)]
struct NetworkStats {
    timestamp: u64,
    total_received: u64,
    total_transmitted: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemInfo {
    pub os: String,
    pub version: String,
    pub total_memory: u64, // MB
    pub used_memory: u64,  // MB
    pub cpu_usage: f32,    // percentage
    pub cpu_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkSpeed {
    pub download: f64, // MB/s
    pub upload: f64,   // MB/s
}

/// 获取系统信息
pub fn get_system_info() -> Result<SystemInfo, String> {
    let mut sys = System::new_all();
    sys.refresh_all();

    // 等待一小段时间以获取准确的CPU使用率
    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_cpu();

    let os = match std::env::consts::OS {
        "windows" => "Windows".to_string(),
        "macos" => "macOS".to_string(),
        "linux" => "Linux".to_string(),
        other => other.to_string(),
    };

    let version = System::os_version().unwrap_or_else(|| "Unknown".to_string());

    let total_memory = sys.total_memory() / (1024 * 1024); // 转换为 MB
    let used_memory = sys.used_memory() / (1024 * 1024);   // 转换为 MB

    // 获取全局 CPU 使用率
    let cpu_usage = sys.global_cpu_info().cpu_usage();
    let cpu_count = sys.cpus().len();

    Ok(SystemInfo {
        os,
        version,
        total_memory,
        used_memory,
        cpu_usage,
        cpu_count,
    })
}

/// 获取网络速度（基于差值计算实际速率）- 使用 sysinfo crate
pub fn get_network_speed() -> Result<NetworkSpeed, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();

    let mut stats_lock = NETWORK_STATS.lock().map_err(|e| e.to_string())?;

    // 使用 sysinfo Networks 获取网络统计
    let networks = Networks::new_with_refreshed_list();
    let mut total_received: u64 = 0;
    let mut total_transmitted: u64 = 0;

    for (interface_name, network) in &networks {
        // 跳过回环接口
        if interface_name.starts_with("lo") || interface_name == "lo0" {
            continue;
        }
        total_received += network.total_received();
        total_transmitted += network.total_transmitted();
    }

    let (download, upload) = if let Some(ref prev_stats) = *stats_lock {
        // 计算时间差（秒）
        let time_diff = (now - prev_stats.timestamp) as f64;

        if time_diff > 0.0 {
            // 计算字节差值
            let bytes_received_diff = total_received.saturating_sub(prev_stats.total_received) as f64;
            let bytes_transmitted_diff = total_transmitted.saturating_sub(prev_stats.total_transmitted) as f64;

            // 转换为 MB/s
            let download_speed = (bytes_received_diff / time_diff) / (1024.0 * 1024.0);
            let upload_speed = (bytes_transmitted_diff / time_diff) / (1024.0 * 1024.0);

            (download_speed, upload_speed)
        } else {
            (0.0, 0.0)
        }
    } else {
        // 第一次调用，返回0
        (0.0, 0.0)
    };

    // 更新存储的统计数据
    *stats_lock = Some(NetworkStats {
        timestamp: now,
        total_received,
        total_transmitted,
    });

    Ok(NetworkSpeed {
        download,
        upload,
    })
}

/// 获取磁盘信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,  // GB
    pub available_space: u64, // GB
    pub is_removable: bool,
}

pub fn get_disks_info() -> Result<Vec<DiskInfo>, String> {
    let disks = Disks::new_with_refreshed_list();
    let mut disk_list = Vec::new();

    for disk in &disks {
        let name = disk.name().to_string_lossy().to_string();
        let mount_point = disk.mount_point().to_string_lossy().to_string();
        let total_space = disk.total_space() / (1024 * 1024 * 1024); // 转换为 GB
        let available_space = disk.available_space() / (1024 * 1024 * 1024); // 转换为 GB
        let is_removable = disk.is_removable();

        disk_list.push(DiskInfo {
            name,
            mount_point,
            total_space,
            available_space,
            is_removable,
        });
    }

    Ok(disk_list)
}
