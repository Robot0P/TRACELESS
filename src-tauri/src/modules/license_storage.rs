//! License storage using SQLCipher encrypted database
//!
//! This module handles persistent storage of activated licenses,
//! using the same encrypted database as settings.

use crate::modules::license::{LicenseTier, LicenseStatus, ActivatedLicense, FeatureAccess};
use crate::modules::license_validator::{get_machine_id, validate_license_key, create_activated_license};
use rusqlite::{Connection, params};
use sha2::{Sha256, Digest};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use once_cell::sync::Lazy;

/// Generate encryption key for license database
fn generate_encryption_key() -> String {
    let mut hasher = Sha256::new();

    // Use machine-specific features - environment variables (no CMD needed)
    if let Ok(hostname) = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
    {
        hasher.update(hostname.as_bytes());
    }

    if let Ok(user) = std::env::var("USER").or_else(|_| std::env::var("USERNAME")) {
        hasher.update(user.as_bytes());
    }

    // Add license-specific salt
    hasher.update(b"traceless-license-storage-v1");

    // Platform-specific UUID
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
        // Use Windows API instead of wmic command
        if let Some(uuid) = crate::modules::windows_utils::get_windows_uuid() {
            hasher.update(uuid.as_bytes());
        }
    }

    let result = hasher.finalize();
    hex::encode(result)
}

/// Get license database path
fn get_db_path() -> PathBuf {
    let app_data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("anti-forensics-tool");

    let _ = std::fs::create_dir_all(&app_data_dir);

    app_data_dir.join("license.db")
}

/// Global database connection
static LICENSE_DB: Lazy<Mutex<Option<Connection>>> = Lazy::new(|| Mutex::new(None));

/// Initialize database connection
fn init_connection() -> Result<Connection, String> {
    let db_path = get_db_path();
    let encryption_key = generate_encryption_key();

    let try_open_db = || -> Result<Connection, String> {
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("DATABASE_OPEN_FAILED:{}", e))?;

        // Set encryption key
        conn.execute_batch(&format!("PRAGMA key = '{}';", encryption_key))
            .map_err(|e| format!("DATABASE_KEY_FAILED:{}", e))?;

        // Verify key
        conn.execute_batch("SELECT count(*) FROM sqlite_master;")
            .map_err(|_| "DATABASE_KEY_INVALID".to_string())?;

        // Create licenses table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS licenses (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                license_key TEXT NOT NULL UNIQUE,
                tier INTEGER NOT NULL,
                activated_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL,
                machine_id_hash TEXT NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        ).map_err(|e| format!("DATABASE_TABLE_FAILED:{}", e))?;

        Ok(conn)
    };

    match try_open_db() {
        Ok(conn) => Ok(conn),
        Err(e) => {
            if e.contains("KEY_INVALID") && db_path.exists() {
                if let Err(del_err) = std::fs::remove_file(&db_path) {
                    return Err(format!("DATABASE_DELETE_FAILED:{}", del_err));
                }
                try_open_db()
            } else {
                Err(e)
            }
        }
    }
}

/// Get database connection
fn get_connection() -> Result<std::sync::MutexGuard<'static, Option<Connection>>, String> {
    let mut guard = LICENSE_DB.lock()
        .map_err(|_| "DATABASE_LOCK_FAILED".to_string())?;

    if guard.is_none() {
        *guard = Some(init_connection()?);
    }

    Ok(guard)
}

/// Save activated license to database
pub fn save_license(license: &ActivatedLicense) -> Result<(), String> {
    let guard = get_connection()?;
    let conn = guard.as_ref().ok_or("DATABASE_UNAVAILABLE")?;

    // Delete any existing license first (only one active license allowed)
    conn.execute("DELETE FROM licenses", [])
        .map_err(|e| format!("DATABASE_DELETE_FAILED:{}", e))?;

    // Insert new license
    conn.execute(
        "INSERT INTO licenses (license_key, tier, activated_at, expires_at, machine_id_hash)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            license.license_key,
            license.tier,
            license.activated_at,
            license.expires_at,
            license.machine_id_hash
        ],
    ).map_err(|e| format!("DATABASE_INSERT_FAILED:{}", e))?;

    Ok(())
}

/// Load activated license from database
pub fn load_license() -> Result<Option<ActivatedLicense>, String> {
    let guard = get_connection()?;
    let conn = guard.as_ref().ok_or("DATABASE_UNAVAILABLE")?;

    let result = conn.query_row(
        "SELECT license_key, tier, activated_at, expires_at, machine_id_hash
         FROM licenses ORDER BY id DESC LIMIT 1",
        [],
        |row| {
            Ok(ActivatedLicense {
                license_key: row.get(0)?,
                tier: row.get(1)?,
                activated_at: row.get(2)?,
                expires_at: row.get(3)?,
                machine_id_hash: row.get(4)?,
            })
        },
    );

    match result {
        Ok(license) => Ok(Some(license)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("DATABASE_QUERY_FAILED:{}", e)),
    }
}

/// Delete license from database
pub fn delete_license() -> Result<(), String> {
    let guard = get_connection()?;
    let conn = guard.as_ref().ok_or("DATABASE_UNAVAILABLE")?;

    conn.execute("DELETE FROM licenses", [])
        .map_err(|e| format!("DATABASE_DELETE_FAILED:{}", e))?;

    Ok(())
}

/// Get current license status
pub fn get_license_status() -> Result<LicenseStatus, String> {
    let machine_id = get_machine_id();

    // Try to load stored license
    let stored = load_license()?;

    match stored {
        Some(license) => {
            // Verify machine binding
            let mut hasher = Sha256::new();
            hasher.update(machine_id.as_bytes());
            let expected_hash = hex::encode(hasher.finalize());

            if license.machine_id_hash != expected_hash {
                // License is for different machine, delete it
                let _ = delete_license();
                return Ok(LicenseStatus::default());
            }

            // Check expiration
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            let tier = LicenseTier::from_u8(license.tier);

            if tier.duration_days() > 0 && current_time > license.expires_at {
                // License expired
                return Ok(LicenseStatus {
                    tier,
                    is_pro: false, // Expired = no pro features
                    activated: false,
                    expires_at: Some(license.expires_at),
                    days_remaining: Some(0),
                    machine_id,
                    activation_date: Some(license.activated_at),
                });
            }

            // Calculate remaining days
            let days_remaining = if tier.duration_days() > 0 {
                let remaining_seconds = license.expires_at - current_time;
                Some((remaining_seconds / 86400) as i32 + 1) // +1 to round up
            } else {
                None
            };

            Ok(LicenseStatus {
                tier,
                is_pro: tier.is_pro(),
                activated: true,
                expires_at: if tier.duration_days() > 0 { Some(license.expires_at) } else { None },
                days_remaining,
                machine_id,
                activation_date: Some(license.activated_at),
            })
        }
        None => {
            // No license stored
            Ok(LicenseStatus {
                tier: LicenseTier::Free,
                is_pro: false,
                activated: false,
                expires_at: None,
                days_remaining: None,
                machine_id,
                activation_date: None,
            })
        }
    }
}

/// Activate a license key
pub fn activate_license(license_key: &str) -> Result<LicenseStatus, String> {
    let machine_id = get_machine_id();

    // Validate the license key
    let result = validate_license_key(license_key, &machine_id);

    if !result.valid {
        let error_code = result.error_code.unwrap_or_else(|| "UNKNOWN".to_string());
        return Err(error_code);
    }

    // Create activated license record
    let activated = create_activated_license(license_key, &result, &machine_id)
        .ok_or("LICENSE_CREATE_FAILED")?;

    // Save to database
    save_license(&activated)?;

    // Return status
    result.license_info.ok_or_else(|| "LICENSE_INFO_MISSING".to_string())
}

/// Deactivate current license
pub fn deactivate_license() -> Result<(), String> {
    delete_license()
}

/// Get feature access based on current license
pub fn get_feature_access() -> Result<FeatureAccess, String> {
    let status = get_license_status()?;
    Ok(FeatureAccess::from_license_status(&status))
}

/// Check if a specific feature is accessible
pub fn can_access_feature(feature: &str) -> Result<bool, String> {
    let access = get_feature_access()?;

    Ok(match feature {
        "scan" => access.scan,
        "file_shredder" => access.file_shredder,
        "system_logs" => access.system_logs,
        "memory_cleanup" => access.memory_cleanup,
        "network_cleanup" => access.network_cleanup,
        "registry_cleanup" => access.registry_cleanup,
        "timestamp_modifier" => access.timestamp_modifier,
        "anti_analysis" => access.anti_analysis,
        "disk_encryption" => access.disk_encryption,
        "scheduled_tasks" => access.scheduled_tasks,
        _ => false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_license_status_default() {
        let status = get_license_status().unwrap();
        // Default should be Free tier
        assert_eq!(status.tier, LicenseTier::Free);
        assert!(!status.is_pro);
    }

    #[test]
    fn test_feature_access_free() {
        // Delete any existing license
        let _ = delete_license();

        let access = get_feature_access().unwrap();
        assert!(access.scan);
        assert!(access.file_shredder);
        assert!(!access.system_logs);
        assert!(!access.memory_cleanup);
    }

    #[test]
    fn test_can_access_feature() {
        let _ = delete_license();

        // Free features
        assert!(can_access_feature("scan").unwrap());
        assert!(can_access_feature("file_shredder").unwrap());

        // Pro features should be blocked
        assert!(!can_access_feature("system_logs").unwrap());
        assert!(!can_access_feature("memory_cleanup").unwrap());
    }
}
