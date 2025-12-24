use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::path::PathBuf;
use std::sync::Mutex;
use once_cell::sync::Lazy;

/// 设置状态结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    // 外观和语言
    pub language: String,
    pub theme: String,

    // 通知设置
    pub notifications: bool,
    pub auto_cleanup: bool,

    // 文件粉碎设置
    pub secure_delete: bool,
    pub delete_method: String,
    pub confirm_before_delete: bool,

    // 系统日志设置
    pub auto_clean_logs: bool,
    pub log_retention_days: i32,

    // 内存清理设置
    pub auto_clean_memory: bool,
    pub memory_clean_interval: i32,
    pub default_memory_types: Vec<String>,

    // 网络清理设置
    pub auto_clear_dns_cache: bool,
    pub auto_clear_network_history: bool,

    // 注册表清理设置
    pub auto_clean_registry: bool,
    pub registry_clean_level: String,

    // 时间戳修改设置
    pub random_time_range_days: i32,

    // 反分析检测设置
    pub auto_anti_analysis_check: bool,
    pub anti_analysis_check_interval: i32,
    pub alert_on_threat_detected: bool,

    // 磁盘加密设置
    pub remind_disk_encryption: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "auto".to_string(),  // 默认自动检测系统语言
            theme: "auto".to_string(),     // 默认自动检测系统主题
            notifications: true,
            auto_cleanup: false,
            secure_delete: true,
            delete_method: "dod".to_string(),
            confirm_before_delete: true,
            auto_clean_logs: false,
            log_retention_days: 30,
            auto_clean_memory: false,
            memory_clean_interval: 30,
            default_memory_types: vec!["clipboard".to_string(), "working_set".to_string()],
            auto_clear_dns_cache: false,
            auto_clear_network_history: false,
            auto_clean_registry: false,
            registry_clean_level: "high".to_string(),
            random_time_range_days: 365,
            auto_anti_analysis_check: true,
            anti_analysis_check_interval: 60,
            alert_on_threat_detected: true,
            remind_disk_encryption: true,
        }
    }
}

/// 生成加密密钥 - 基于机器特征
fn generate_encryption_key() -> String {
    let mut hasher = Sha256::new();

    // 使用多种机器特征生成唯一密钥
    if let Ok(hostname) = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .or_else(|_| {
            #[cfg(unix)]
            {
                std::process::Command::new("hostname")
                    .output()
                    .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                    .map_err(|_| std::env::VarError::NotPresent)
            }
            #[cfg(not(unix))]
            {
                Err(std::env::VarError::NotPresent)
            }
        })
    {
        hasher.update(hostname.as_bytes());
    }

    // 添加用户名
    if let Ok(user) = std::env::var("USER").or_else(|_| std::env::var("USERNAME")) {
        hasher.update(user.as_bytes());
    }

    // 添加应用特定盐值
    hasher.update(b"anti-forensics-tool-settings-v1");

    // 添加一些系统信息
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("ioreg")
            .args(["-rd1", "-c", "IOPlatformExpertDevice"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if let Some(uuid_line) = output_str.lines()
                .find(|l| l.contains("IOPlatformUUID"))
            {
                hasher.update(uuid_line.as_bytes());
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(machine_id) = std::fs::read_to_string("/etc/machine-id") {
            hasher.update(machine_id.trim().as_bytes());
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = std::process::Command::new("wmic")
            .args(["csproduct", "get", "UUID"])
            .output()
        {
            hasher.update(&output.stdout);
        }
    }

    let result = hasher.finalize();
    hex::encode(result)
}

/// 获取数据库路径
fn get_db_path() -> PathBuf {
    let app_data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("anti-forensics-tool");

    // 确保目录存在
    let _ = std::fs::create_dir_all(&app_data_dir);

    app_data_dir.join("settings.db")
}

/// 全局数据库连接
static DB_CONNECTION: Lazy<Mutex<Option<Connection>>> = Lazy::new(|| Mutex::new(None));

/// 初始化数据库连接
fn init_connection() -> Result<Connection, String> {
    let db_path = get_db_path();
    let encryption_key = generate_encryption_key();

    // 尝试打开并验证数据库
    let try_open_db = || -> Result<Connection, String> {
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("无法打开数据库: {}", e))?;

        // 设置加密密钥
        conn.execute_batch(&format!("PRAGMA key = '{}';", encryption_key))
            .map_err(|e| format!("无法设置加密密钥: {}", e))?;

        // 验证密钥是否正确
        conn.execute_batch("SELECT count(*) FROM sqlite_master;")
            .map_err(|_| "数据库密钥验证失败".to_string())?;

        // 创建设置表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        ).map_err(|e| format!("无法创建设置表: {}", e))?;

        // 创建版本表用于迁移
        conn.execute(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY
            )",
            [],
        ).map_err(|e| format!("无法创建版本表: {}", e))?;

        Ok(conn)
    };

    // 第一次尝试打开
    match try_open_db() {
        Ok(conn) => Ok(conn),
        Err(e) => {
            // 如果是密钥验证失败，删除旧数据库并重新创建
            if e.contains("密钥验证失败") && db_path.exists() {
                // 删除旧数据库
                if let Err(del_err) = std::fs::remove_file(&db_path) {
                    return Err(format!("无法删除旧数据库: {}", del_err));
                }

                // 重新创建
                try_open_db()
            } else {
                Err(e)
            }
        }
    }
}

/// 获取数据库连接
fn get_connection() -> Result<std::sync::MutexGuard<'static, Option<Connection>>, String> {
    let mut guard = DB_CONNECTION.lock()
        .map_err(|_| "无法获取数据库锁".to_string())?;

    if guard.is_none() {
        *guard = Some(init_connection()?);
    }

    Ok(guard)
}

/// 保存单个设置项
pub fn save_setting(key: &str, value: &str) -> Result<(), String> {
    let guard = get_connection()?;
    let conn = guard.as_ref().ok_or("数据库连接不可用")?;

    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?1, ?2, datetime('now'))",
        params![key, value],
    ).map_err(|e| format!("保存设置失败: {}", e))?;

    Ok(())
}

/// 读取单个设置项
pub fn get_setting(key: &str) -> Result<Option<String>, String> {
    let guard = get_connection()?;
    let conn = guard.as_ref().ok_or("数据库连接不可用")?;

    let result: Result<String, _> = conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    );

    match result {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("读取设置失败: {}", e)),
    }
}

/// 保存所有设置
pub fn save_all_settings(settings: &AppSettings) -> Result<(), String> {
    let guard = get_connection()?;
    let conn = guard.as_ref().ok_or("数据库连接不可用")?;

    let settings_json = serde_json::to_string(settings)
        .map_err(|e| format!("序列化设置失败: {}", e))?;

    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES ('app_settings', ?1, datetime('now'))",
        params![settings_json],
    ).map_err(|e| format!("保存设置失败: {}", e))?;

    Ok(())
}

/// 读取所有设置
pub fn load_all_settings() -> Result<AppSettings, String> {
    let guard = get_connection()?;
    let conn = guard.as_ref().ok_or("数据库连接不可用")?;

    let result: Result<String, _> = conn.query_row(
        "SELECT value FROM settings WHERE key = 'app_settings'",
        [],
        |row| row.get(0),
    );

    match result {
        Ok(json) => {
            serde_json::from_str(&json)
                .map_err(|e| format!("解析设置失败: {}", e))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            // 返回默认设置
            Ok(AppSettings::default())
        }
        Err(e) => Err(format!("读取设置失败: {}", e)),
    }
}

/// 重置所有设置为默认值
pub fn reset_all_settings() -> Result<AppSettings, String> {
    let default_settings = AppSettings::default();
    save_all_settings(&default_settings)?;
    Ok(default_settings)
}

/// 删除所有设置数据
pub fn clear_all_settings() -> Result<(), String> {
    let guard = get_connection()?;
    let conn = guard.as_ref().ok_or("数据库连接不可用")?;

    conn.execute("DELETE FROM settings", [])
        .map_err(|e| format!("清除设置失败: {}", e))?;

    Ok(())
}

/// 获取设置数据库信息
pub fn get_settings_db_info() -> Result<SettingsDbInfo, String> {
    let db_path = get_db_path();

    let size = std::fs::metadata(&db_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let exists = db_path.exists();

    Ok(SettingsDbInfo {
        path: db_path.to_string_lossy().to_string(),
        size_bytes: size,
        exists,
        encrypted: true,
    })
}

#[derive(Debug, Serialize)]
pub struct SettingsDbInfo {
    pub path: String,
    pub size_bytes: u64,
    pub exists: bool,
    pub encrypted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = AppSettings::default();
        assert_eq!(settings.language, "zh-CN");
        assert_eq!(settings.theme, "dark");
        assert!(settings.notifications);
    }

    #[test]
    fn test_encryption_key_generation() {
        let key1 = generate_encryption_key();
        let key2 = generate_encryption_key();
        // 同一机器上密钥应该相同
        assert_eq!(key1, key2);
        // 密钥应该是64字符的hex字符串 (SHA256)
        assert_eq!(key1.len(), 64);
    }
}
