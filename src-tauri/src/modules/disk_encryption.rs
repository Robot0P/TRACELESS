use crate::modules::command_utils::CommandExt;
use std::process::Command;

/// 磁盘加密状态
#[derive(Debug, Clone)]
pub struct DiskEncryptionStatus {
    pub enabled: bool,
    pub encryption_method: String,
    pub disks: Vec<DiskInfo>,
}

#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub name: String,
    pub path: String,
    pub encrypted: bool,
    pub encryption_type: Option<String>,
    pub size: u64,
}

/// 检查 BitLocker 状态 (Windows)
#[cfg(target_os = "windows")]
pub fn check_bitlocker_status() -> Result<DiskEncryptionStatus, String> {
    // 使用 manage-bde 命令检查 BitLocker 状态
    let output = Command::new("manage-bde")
        .arg("-status")
        .hide_window()
        .output()
        .map_err(|e| format!("无法执行 manage-bde: {}", e))?;

    let status_text = String::from_utf8_lossy(&output.stdout);

    // 解析每个磁盘的 BitLocker 状态
    let mut disks = Vec::new();
    let mut any_enabled = false;
    let mut current_drive: Option<String> = None;
    let mut current_encrypted = false;
    let mut current_protection = false;

    for line in status_text.lines() {
        let line = line.trim();

        // 检测新磁盘卷 (格式: "Volume C: [Label]" 或 "卷 C: [标签]")
        if (line.starts_with("Volume ") || line.starts_with("卷 ")) && line.contains(":") {
            // 保存之前的磁盘信息
            if let Some(drive) = current_drive.take() {
                let encrypted = current_encrypted || current_protection;
                if encrypted {
                    any_enabled = true;
                }

                // 获取磁盘大小
                let disk_size = get_windows_disk_size(&drive);

                disks.push(DiskInfo {
                    name: drive.clone(),
                    path: format!("{}\\", drive),
                    encrypted,
                    encryption_type: if encrypted {
                        Some("BitLocker".to_string())
                    } else {
                        None
                    },
                    size: disk_size,
                });
            }

            // 提取驱动器号
            if let Some(colon_pos) = line.find(':') {
                let start = if line.starts_with("Volume ") { 7 } else { 2 };
                current_drive = Some(line[start..=colon_pos].trim().to_string());
                current_encrypted = false;
                current_protection = false;
            }
        }
        // 检测加密状态
        else if line.contains("Conversion Status") || line.contains("转换状态") {
            current_encrypted = line.contains("Fully Encrypted") ||
                               line.contains("Used Space Only Encrypted") ||
                               line.contains("已完全加密") ||
                               line.contains("仅加密已用空间");
        }
        // 检测保护状态
        else if line.contains("Protection Status") || line.contains("保护状态") {
            current_protection = line.contains("Protection On") || line.contains("保护已启用");
        }
    }

    // 保存最后一个磁盘
    if let Some(drive) = current_drive {
        let encrypted = current_encrypted || current_protection;
        if encrypted {
            any_enabled = true;
        }

        let disk_size = get_windows_disk_size(&drive);

        disks.push(DiskInfo {
            name: drive.clone(),
            path: format!("{}\\", drive),
            encrypted,
            encryption_type: if encrypted {
                Some("BitLocker".to_string())
            } else {
                None
            },
            size: disk_size,
        });
    }

    // 如果没有解析到任何磁盘，至少添加 C: 驱动器
    if disks.is_empty() {
        let c_size = get_windows_disk_size("C:");
        let c_encrypted = status_text.contains("Protection On") || status_text.contains("保护已启用");
        any_enabled = c_encrypted;

        disks.push(DiskInfo {
            name: "C:".to_string(),
            path: "C:\\".to_string(),
            encrypted: c_encrypted,
            encryption_type: if c_encrypted {
                Some("BitLocker".to_string())
            } else {
                None
            },
            size: c_size,
        });
    }

    Ok(DiskEncryptionStatus {
        enabled: any_enabled,
        encryption_method: if any_enabled {
            "BitLocker".to_string()
        } else {
            "未加密".to_string()
        },
        disks,
    })
}

/// 获取 Windows 磁盘大小
#[cfg(target_os = "windows")]
fn get_windows_disk_size(drive: &str) -> u64 {
    // Use Windows API instead of wmic command
    crate::modules::windows_utils::get_disk_size(drive)
}

/// 检查 FileVault 状态 (macOS)
#[cfg(target_os = "macos")]
pub fn check_filevault_status() -> Result<DiskEncryptionStatus, String> {
    // 使用 fdesetup 命令检查 FileVault 状态
    let output = Command::new("fdesetup")
        .arg("status")
        .output()
        .map_err(|e| format!("无法执行 fdesetup: {}", e))?;

    let status_text = String::from_utf8_lossy(&output.stdout);
    let enabled = status_text.contains("FileVault is On");

    let mut disks = Vec::new();
    disks.push(DiskInfo {
        name: "Macintosh HD".to_string(),
        path: "/".to_string(),
        encrypted: enabled,
        encryption_type: if enabled {
            Some("FileVault".to_string())
        } else {
            None
        },
        size: 0,
    });

    Ok(DiskEncryptionStatus {
        enabled,
        encryption_method: if enabled {
            "FileVault".to_string()
        } else {
            "未加密".to_string()
        },
        disks,
    })
}

/// 检查 LUKS 状态 (Linux)
#[cfg(target_os = "linux")]
pub fn check_luks_status() -> Result<DiskEncryptionStatus, String> {
    // 检查是否有加密的分区
    let output = Command::new("lsblk")
        .arg("-o")
        .arg("NAME,TYPE,FSTYPE")
        .output()
        .map_err(|e| format!("无法执行 lsblk: {}", e))?;

    let status_text = String::from_utf8_lossy(&output.stdout);
    let has_luks = status_text.contains("crypto_LUKS");

    let mut disks = Vec::new();
    disks.push(DiskInfo {
        name: "sda1".to_string(),
        path: "/dev/sda1".to_string(),
        encrypted: has_luks,
        encryption_type: if has_luks {
            Some("LUKS".to_string())
        } else {
            None
        },
        size: 0,
    });

    Ok(DiskEncryptionStatus {
        enabled: has_luks,
        encryption_method: if has_luks {
            "LUKS".to_string()
        } else {
            "未加密".to_string()
        },
        disks,
    })
}

/// 启用 BitLocker (Windows)
#[cfg(target_os = "windows")]
pub fn enable_bitlocker(drive: &str) -> Result<String, String> {
    // 注意：实际启用 BitLocker 需要管理员权限
    let output = Command::new("manage-bde")
        .arg("-on")
        .arg(drive)
        .arg("-RecoveryPassword")
        .hide_window()
        .output()
        .map_err(|e| format!("无法启用 BitLocker: {}", e))?;

    if output.status.success() {
        Ok(format!("成功启用 {} 的 BitLocker 加密", drive))
    } else {
        Err(format!(
            "启用 BitLocker 失败: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

/// 启用 FileVault (macOS)
#[cfg(target_os = "macos")]
pub fn enable_filevault() -> Result<String, String> {
    // 注意：实际启用 FileVault 需要管理员权限和用户交互
    let output = Command::new("fdesetup")
        .arg("enable")
        .output()
        .map_err(|e| format!("无法启用 FileVault: {}", e))?;

    if output.status.success() {
        Ok("成功启用 FileVault 加密".to_string())
    } else {
        Err(format!(
            "启用 FileVault 失败: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

/// 禁用 BitLocker (Windows)
#[cfg(target_os = "windows")]
pub fn disable_bitlocker(drive: &str) -> Result<String, String> {
    let output = Command::new("manage-bde")
        .arg("-off")
        .arg(drive)
        .hide_window()
        .output()
        .map_err(|e| format!("无法禁用 BitLocker: {}", e))?;

    if output.status.success() {
        Ok(format!("成功禁用 {} 的 BitLocker 加密", drive))
    } else {
        Err(format!(
            "禁用 BitLocker 失败: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

/// 禁用 FileVault (macOS)
#[cfg(target_os = "macos")]
pub fn disable_filevault() -> Result<String, String> {
    let output = Command::new("fdesetup")
        .arg("disable")
        .output()
        .map_err(|e| format!("无法禁用 FileVault: {}", e))?;

    if output.status.success() {
        Ok("成功禁用 FileVault 加密".to_string())
    } else {
        Err(format!(
            "禁用 FileVault 失败: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}
