use crate::commands::permission_ops::require_admin_for_operation;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiskInfo {
    pub name: String,
    pub path: String,
    pub encrypted: bool,
    pub encryption_type: Option<String>,
    pub size: String,
    pub size_bytes: u64,
    pub used: String,
    pub used_bytes: u64,
    pub available: String,
    pub available_bytes: u64,
    pub usage_percent: f32,
    pub file_system: String,
    pub mount_point: String,
    pub encryption_progress: Option<u32>,
    pub is_system_disk: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptionStatus {
    pub enabled: bool,
    pub encryption_method: String,
    pub disks: Vec<DiskInfo>,
    pub platform: String,
    pub supported: bool,
    pub recovery_key_exists: bool,
    pub encryption_in_progress: bool,
}

/// 格式化字节大小
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// 检查磁盘加密状态
#[tauri::command]
pub async fn check_disk_encryption() -> Result<EncryptionStatus, String> {
    #[cfg(target_os = "windows")]
    {
        check_windows_encryption().await
    }

    #[cfg(target_os = "macos")]
    {
        check_macos_encryption().await
    }

    #[cfg(target_os = "linux")]
    {
        check_linux_encryption().await
    }
}

#[cfg(target_os = "macos")]
async fn check_macos_encryption() -> Result<EncryptionStatus, String> {
    let mut disks = Vec::new();
    let mut any_encrypted = false;
    let mut encryption_in_progress = false;

    // 检查 FileVault 状态
    let fv_output = Command::new("fdesetup")
        .arg("status")
        .output();

    let fv_enabled = if let Ok(output) = &fv_output {
        let status = String::from_utf8_lossy(&output.stdout);
        if status.contains("Encryption in progress") || status.contains("Decryption in progress") {
            encryption_in_progress = true;
        }
        status.contains("FileVault is On")
    } else {
        false
    };

    // 使用 df 获取磁盘使用情况
    let df_output = Command::new("df")
        .args(["-k"])
        .output()
        .map_err(|e| format!("IO_ERROR:{}", e))?;

    let df_str = String::from_utf8_lossy(&df_output.stdout);

    // 解析 df 输出获取挂载的磁盘
    for line in df_str.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 9 {
            let device = parts[0];
            let mount_point = parts[8];

            // 只处理真实的磁盘分区
            if !device.starts_with("/dev/disk") {
                continue;
            }

            // 获取更详细的磁盘信息
            let disk_info_output = Command::new("diskutil")
                .args(["info", device])
                .output();

            let mut disk_name = mount_point.split('/').last().unwrap_or("Unknown").to_string();
            let mut file_system = "Unknown".to_string();
            let mut is_encrypted = false;
            let mut encryption_type = None;

            if let Ok(output) = disk_info_output {
                let info_str = String::from_utf8_lossy(&output.stdout);

                for info_line in info_str.lines() {
                    let info_line = info_line.trim();
                    if info_line.starts_with("Volume Name:") {
                        disk_name = info_line.replace("Volume Name:", "").trim().to_string();
                        if disk_name.is_empty() || disk_name == "Not applicable" {
                            disk_name = mount_point.split('/').last().unwrap_or("Unknown").to_string();
                        }
                    } else if info_line.starts_with("File System Personality:") {
                        file_system = info_line.replace("File System Personality:", "").trim().to_string();
                    } else if info_line.starts_with("Type (Bundle):") {
                        let fs_type = info_line.replace("Type (Bundle):", "").trim().to_string();
                        if !fs_type.is_empty() {
                            file_system = fs_type;
                        }
                    } else if info_line.contains("Encrypted:") && info_line.contains("Yes") {
                        is_encrypted = true;
                        encryption_type = Some("FileVault 2".to_string());
                    } else if info_line.contains("FileVault:") && info_line.contains("Yes") {
                        is_encrypted = true;
                        encryption_type = Some("FileVault 2".to_string());
                    }
                }
            }

            // 解析容量
            let total_kb: u64 = parts[1].parse().unwrap_or(0);
            let used_kb: u64 = parts[2].parse().unwrap_or(0);
            let available_kb: u64 = parts[3].parse().unwrap_or(0);
            let capacity_str = parts[4];
            let usage_percent: f32 = capacity_str.trim_end_matches('%').parse().unwrap_or(0.0);

            let total_bytes = total_kb * 1024;
            let used_bytes = used_kb * 1024;
            let available_bytes = available_kb * 1024;

            // 对于根分区，使用 FileVault 状态
            let is_system_disk = mount_point == "/";
            if is_system_disk && fv_enabled {
                is_encrypted = true;
                encryption_type = Some("FileVault 2".to_string());
            }

            if is_encrypted {
                any_encrypted = true;
            }

            // 处理空名称
            if disk_name.is_empty() {
                disk_name = if is_system_disk {
                    "Macintosh HD".to_string()
                } else {
                    mount_point.to_string()
                };
            }

            disks.push(DiskInfo {
                name: disk_name,
                path: device.to_string(),
                encrypted: is_encrypted,
                encryption_type,
                size: format_size(total_bytes),
                size_bytes: total_bytes,
                used: format_size(used_bytes),
                used_bytes,
                available: format_size(available_bytes),
                available_bytes,
                usage_percent,
                file_system,
                mount_point: mount_point.to_string(),
                encryption_progress: if is_system_disk && encryption_in_progress { Some(50) } else { None },
                is_system_disk,
            });
        }
    }

    // 如果没有找到磁盘，添加默认信息
    if disks.is_empty() {
        disks.push(DiskInfo {
            name: "Macintosh HD".to_string(),
            path: "/dev/disk1s1".to_string(),
            encrypted: fv_enabled,
            encryption_type: if fv_enabled { Some("FileVault 2".to_string()) } else { None },
            size: "未知".to_string(),
            size_bytes: 0,
            used: "未知".to_string(),
            used_bytes: 0,
            available: "未知".to_string(),
            available_bytes: 0,
            usage_percent: 0.0,
            file_system: "APFS".to_string(),
            mount_point: "/".to_string(),
            encryption_progress: None,
            is_system_disk: true,
        });
        any_encrypted = fv_enabled;
    }

    // 检查是否有恢复密钥
    let recovery_key_exists = if let Ok(output) = Command::new("fdesetup")
        .arg("haspersonalrecoverykey")
        .output()
    {
        let status = String::from_utf8_lossy(&output.stdout);
        status.contains("true")
    } else {
        false
    };

    Ok(EncryptionStatus {
        enabled: any_encrypted,
        encryption_method: if any_encrypted {
            "FileVault 2".to_string()
        } else {
            "未加密".to_string()
        },
        disks,
        platform: "macOS".to_string(),
        supported: true,
        recovery_key_exists,
        encryption_in_progress,
    })
}

#[cfg(target_os = "windows")]
async fn check_windows_encryption() -> Result<EncryptionStatus, String> {
    let mut disks = Vec::new();
    let mut any_encrypted = false;
    let mut encryption_in_progress = false;

    // 获取所有磁盘驱动器列表
    let drives = vec!["C:", "D:", "E:", "F:"];

    for drive in drives {
        // 检查驱动器是否存在
        let exists = Command::new("cmd")
            .args(["/C", &format!("if exist {}\\nul echo exists", drive)])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("exists"))
            .unwrap_or(false);

        if !exists {
            continue;
        }

        // 检查 BitLocker 状态
        let bitlocker_output = Command::new("manage-bde")
            .args(["-status", drive])
            .output();

        let mut is_encrypted = false;
        let mut encryption_type = None;
        let mut progress = None;

        if let Ok(bl_output) = bitlocker_output {
            let bl_status = String::from_utf8_lossy(&bl_output.stdout);

            if bl_status.contains("Protection On") || bl_status.contains("Fully Encrypted") {
                is_encrypted = true;
                encryption_type = Some("BitLocker".to_string());
                any_encrypted = true;
            }

            if bl_status.contains("Encryption in Progress") {
                encryption_in_progress = true;
                // 解析进度
                for line in bl_status.lines() {
                    if line.contains("Percentage Encrypted:") {
                        if let Some(pct) = line.split(':').nth(1) {
                            progress = pct.trim().trim_end_matches('%').trim().parse::<u32>().ok();
                        }
                    }
                }
            }
        }

        // 获取磁盘信息
        let wmic_output = Command::new("wmic")
            .args(["logicaldisk", "where", &format!("DeviceID='{}'", drive), "get", "Size,FreeSpace,FileSystem,VolumeName", "/format:list"])
            .output();

        let mut size_bytes: u64 = 0;
        let mut free_bytes: u64 = 0;
        let mut file_system = "NTFS".to_string();
        let mut volume_name = format!("本地磁盘 ({})", drive);

        if let Ok(output) = wmic_output {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.starts_with("Size=") {
                    size_bytes = line.replace("Size=", "").trim().parse().unwrap_or(0);
                } else if line.starts_with("FreeSpace=") {
                    free_bytes = line.replace("FreeSpace=", "").trim().parse().unwrap_or(0);
                } else if line.starts_with("FileSystem=") {
                    let fs = line.replace("FileSystem=", "").trim().to_string();
                    if !fs.is_empty() {
                        file_system = fs;
                    }
                } else if line.starts_with("VolumeName=") {
                    let name = line.replace("VolumeName=", "").trim().to_string();
                    if !name.is_empty() {
                        volume_name = name;
                    }
                }
            }
        }

        let used_bytes = size_bytes.saturating_sub(free_bytes);
        let usage_percent = if size_bytes > 0 {
            (used_bytes as f32 / size_bytes as f32) * 100.0
        } else {
            0.0
        };

        disks.push(DiskInfo {
            name: volume_name,
            path: drive.to_string(),
            encrypted: is_encrypted,
            encryption_type,
            size: format_size(size_bytes),
            size_bytes,
            used: format_size(used_bytes),
            used_bytes,
            available: format_size(free_bytes),
            available_bytes: free_bytes,
            usage_percent,
            file_system,
            mount_point: format!("{}\\", drive),
            encryption_progress: progress,
            is_system_disk: drive == "C:",
        });
    }

    // 如果没有找到磁盘，添加默认信息
    if disks.is_empty() {
        disks.push(DiskInfo {
            name: "系统盘 (C:)".to_string(),
            path: "C:".to_string(),
            encrypted: false,
            encryption_type: None,
            size: "未知".to_string(),
            size_bytes: 0,
            used: "未知".to_string(),
            used_bytes: 0,
            available: "未知".to_string(),
            available_bytes: 0,
            usage_percent: 0.0,
            file_system: "NTFS".to_string(),
            mount_point: "C:\\".to_string(),
            encryption_progress: None,
            is_system_disk: true,
        });
    }

    Ok(EncryptionStatus {
        enabled: any_encrypted,
        encryption_method: if any_encrypted {
            "BitLocker".to_string()
        } else {
            "未加密".to_string()
        },
        disks,
        platform: "Windows".to_string(),
        supported: true,
        recovery_key_exists: false,
        encryption_in_progress,
    })
}

#[cfg(target_os = "linux")]
async fn check_linux_encryption() -> Result<EncryptionStatus, String> {
    let mut disks = Vec::new();
    let mut any_encrypted = false;

    // Get disk info using lsblk
    let output = Command::new("lsblk")
        .args(["-b", "-o", "NAME,SIZE,TYPE,FSTYPE,MOUNTPOINT"])
        .output()
        .map_err(|e| format!("IO_ERROR:{}", e))?;

    let output_str = String::from_utf8_lossy(&output.stdout);

    // 使用 df 获取使用情况
    let df_output = Command::new("df")
        .args(["-B1"])
        .output();

    let df_map: std::collections::HashMap<String, (u64, u64, u64)> = if let Ok(df_out) = df_output {
        let df_str = String::from_utf8_lossy(&df_out.stdout);
        df_str.lines().skip(1).filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                let mount = parts.last()?.to_string();
                let total: u64 = parts.get(1)?.parse().ok()?;
                let used: u64 = parts.get(2)?.parse().ok()?;
                let available: u64 = parts.get(3)?.parse().ok()?;
                Some((mount, (total, used, available)))
            } else {
                None
            }
        }).collect()
    } else {
        std::collections::HashMap::new()
    };

    for line in output_str.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let name = parts[0].trim_start_matches(['├', '└', '─', '│', ' ']);
            let size_bytes: u64 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            let device_type = parts.get(2).unwrap_or(&"");
            let fs_type = parts.get(3).unwrap_or(&"");
            let mount_point = parts.get(4).unwrap_or(&"");

            // 只处理分区
            if *device_type != "part" && *device_type != "crypt" {
                continue;
            }

            let is_encrypted = *fs_type == "crypto_LUKS" || *device_type == "crypt";
            if is_encrypted {
                any_encrypted = true;
            }

            let (total, used_bytes, available_bytes) = df_map.get(*mount_point)
                .copied()
                .unwrap_or((size_bytes, 0, size_bytes));

            let usage_percent = if total > 0 {
                (used_bytes as f32 / total as f32) * 100.0
            } else {
                0.0
            };

            disks.push(DiskInfo {
                name: name.to_string(),
                path: format!("/dev/{}", name),
                encrypted: is_encrypted,
                encryption_type: if is_encrypted { Some("LUKS".to_string()) } else { None },
                size: format_size(size_bytes),
                size_bytes,
                used: format_size(used_bytes),
                used_bytes,
                available: format_size(available_bytes),
                available_bytes,
                usage_percent,
                file_system: fs_type.to_string(),
                mount_point: mount_point.to_string(),
                encryption_progress: None,
                is_system_disk: *mount_point == "/",
            });
        }
    }

    if disks.is_empty() {
        disks.push(DiskInfo {
            name: "sda1".to_string(),
            path: "/dev/sda1".to_string(),
            encrypted: false,
            encryption_type: None,
            size: "未知".to_string(),
            size_bytes: 0,
            used: "未知".to_string(),
            used_bytes: 0,
            available: "未知".to_string(),
            available_bytes: 0,
            usage_percent: 0.0,
            file_system: "ext4".to_string(),
            mount_point: "/".to_string(),
            encryption_progress: None,
            is_system_disk: true,
        });
    }

    Ok(EncryptionStatus {
        enabled: any_encrypted,
        encryption_method: if any_encrypted {
            "LUKS".to_string()
        } else {
            "未加密".to_string()
        },
        disks,
        platform: "Linux".to_string(),
        supported: true,
        recovery_key_exists: false,
        encryption_in_progress: false,
    })
}

/// Enable disk encryption
#[tauri::command]
pub async fn enable_disk_encryption(disk_path: String) -> Result<String, String> {
    // Check permission - disk encryption requires elevation
    require_admin_for_operation("disk_encryption")?;

    #[cfg(target_os = "windows")]
    {
        enable_bitlocker(disk_path).await
    }

    #[cfg(target_os = "macos")]
    {
        let _ = disk_path;
        enable_filevault().await
    }

    #[cfg(target_os = "linux")]
    {
        let _ = disk_path;
        Err("FEATURE_NOT_AVAILABLE:linuxEncryptionManual".to_string())
    }
}

#[cfg(target_os = "windows")]
async fn enable_bitlocker(drive: String) -> Result<String, String> {
    let output = Command::new("manage-bde")
        .args(["-on", &drive, "-RecoveryPassword"])
        .output()
        .map_err(|e| format!("IO_ERROR:{}", e))?;

    if output.status.success() {
        Ok(format!("OK:bitlocker_enabled:{}", drive))
    } else {
        Err(format!(
            "PERMISSION_DENIED:{}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

#[cfg(target_os = "macos")]
async fn enable_filevault() -> Result<String, String> {
    Err("FEATURE_NOT_AVAILABLE:filevaultEnableGuide".to_string())
}

/// Disable disk encryption
#[tauri::command]
pub async fn disable_disk_encryption(disk_path: String) -> Result<String, String> {
    // Check permission - disk encryption requires elevation
    require_admin_for_operation("disk_encryption")?;

    #[cfg(target_os = "windows")]
    {
        disable_bitlocker(disk_path).await
    }

    #[cfg(target_os = "macos")]
    {
        let _ = disk_path;
        disable_filevault().await
    }

    #[cfg(target_os = "linux")]
    {
        let _ = disk_path;
        Err("FEATURE_NOT_AVAILABLE:linuxLuksDisableNotSupported".to_string())
    }
}

#[cfg(target_os = "windows")]
async fn disable_bitlocker(drive: String) -> Result<String, String> {
    let output = Command::new("manage-bde")
        .args(["-off", &drive])
        .output()
        .map_err(|e| format!("IO_ERROR:{}", e))?;

    if output.status.success() {
        Ok(format!("OK:bitlocker_disabled:{}", drive))
    } else {
        Err(format!(
            "IO_ERROR:{}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

#[cfg(target_os = "macos")]
async fn disable_filevault() -> Result<String, String> {
    Err("FEATURE_NOT_AVAILABLE:filevaultDisableGuide".to_string())
}

/// 获取加密建议
#[tauri::command]
pub async fn get_encryption_recommendations() -> Result<Vec<String>, String> {
    let mut recommendations = Vec::new();

    #[cfg(target_os = "windows")]
    {
        recommendations.push("启用 BitLocker 全盘加密以保护数据安全".to_string());
        recommendations.push("确保您的设备支持 TPM 芯片以获得最佳安全性".to_string());
        recommendations.push("将恢复密钥保存到安全的位置（如 Microsoft 账户或 U 盘）".to_string());
        recommendations.push("定期检查 BitLocker 状态和恢复密钥有效性".to_string());
        recommendations.push("考虑加密所有数据驱动器，不仅仅是系统盘".to_string());
    }

    #[cfg(target_os = "macos")]
    {
        recommendations.push("启用 FileVault 2 全盘加密以保护 Mac 上的数据".to_string());
        recommendations.push("使用强密码保护您的用户账户".to_string());
        recommendations.push("将恢复密钥保存到 iCloud 或打印并安全保管".to_string());
        recommendations.push("启用「查找我的 Mac」以便在设备丢失时远程锁定或擦除".to_string());
        recommendations.push("确保固件密码已启用以防止未授权启动".to_string());
    }

    #[cfg(target_os = "linux")]
    {
        recommendations.push("使用 LUKS 进行全盘加密以保护数据".to_string());
        recommendations.push("在系统安装时选择加密选项以获得最佳体验".to_string());
        recommendations.push("使用强密码短语（建议 20 个字符以上）".to_string());
        recommendations.push("备份 LUKS 头部信息以防止数据丢失".to_string());
        recommendations.push("考虑使用 TPM 或密钥文件自动解锁".to_string());
    }

    Ok(recommendations)
}
