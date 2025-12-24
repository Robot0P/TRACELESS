//! License-related Tauri commands
//!
//! This module exposes license management functions to the frontend.

use crate::modules::license::{LicenseStatus, FeatureAccess, LicenseValidationResult};
use crate::modules::license_validator::{
    get_machine_id as get_machine_id_internal,
    get_short_machine_id,
    validate_license_key,
};
use crate::modules::license_storage;

/// Get the machine ID for license binding
#[tauri::command]
pub fn get_machine_id() -> Result<String, String> {
    Ok(get_short_machine_id())
}

/// Get the full machine ID (for internal use)
#[tauri::command]
pub fn get_full_machine_id() -> Result<String, String> {
    Ok(get_machine_id_internal())
}

/// Get current license status
#[tauri::command]
pub fn get_license_status() -> Result<LicenseStatus, String> {
    license_storage::get_license_status()
}

/// Validate a license key without activating
#[tauri::command]
pub fn validate_license(license_key: String) -> Result<LicenseValidationResult, String> {
    let machine_id = get_machine_id_internal();
    Ok(validate_license_key(&license_key, &machine_id))
}

/// Activate a license key
#[tauri::command]
pub fn activate_license(license_key: String) -> Result<LicenseStatus, String> {
    license_storage::activate_license(&license_key)
}

/// Deactivate the current license
#[tauri::command]
pub fn deactivate_license() -> Result<(), String> {
    license_storage::deactivate_license()
}

/// Get feature access based on current license
#[tauri::command]
pub fn get_feature_access() -> Result<FeatureAccess, String> {
    license_storage::get_feature_access()
}

/// Check if a specific feature is accessible
#[tauri::command]
pub fn can_access_feature(feature: String) -> Result<bool, String> {
    license_storage::can_access_feature(&feature)
}

/// Check if user has Pro license
#[tauri::command]
pub fn is_pro_user() -> Result<bool, String> {
    let status = license_storage::get_license_status()?;
    Ok(status.is_pro && status.activated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_machine_id() {
        let id = get_machine_id().unwrap();
        assert_eq!(id.len(), 16);
    }

    #[test]
    fn test_get_full_machine_id() {
        let id = get_full_machine_id().unwrap();
        assert_eq!(id.len(), 64);
    }

    #[test]
    fn test_get_license_status() {
        let status = get_license_status().unwrap();
        // Should return some status (Free by default)
        assert!(!status.machine_id.is_empty());
    }

    #[test]
    fn test_get_feature_access() {
        let access = get_feature_access().unwrap();
        // Free features should always be true
        assert!(access.scan);
        assert!(access.file_shredder);
    }

    #[test]
    fn test_validate_invalid_license() {
        let result = validate_license("INVALID-LICENSE-KEY".to_string()).unwrap();
        assert!(!result.valid);
    }
}
