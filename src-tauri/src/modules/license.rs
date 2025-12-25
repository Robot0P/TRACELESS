//! License system data structures and types
//!
//! This module defines the core types for the license management system,
//! including license tiers, status, and feature access rights.

use serde::{Deserialize, Serialize};

/// License tier types
///
/// - Free: Basic features only (scan + file shredder)
/// - Monthly: Pro features for 30 days
/// - Quarterly: Pro features for 90 days
/// - Yearly: Pro features for 365 days
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "u8", try_from = "u8")]
#[repr(u8)]
pub enum LicenseTier {
    Free = 0,
    Monthly = 1,
    Quarterly = 2,
    Yearly = 3,
}

impl From<LicenseTier> for u8 {
    fn from(tier: LicenseTier) -> u8 {
        tier as u8
    }
}

impl TryFrom<u8> for LicenseTier {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(LicenseTier::Free),
            1 => Ok(LicenseTier::Monthly),
            2 => Ok(LicenseTier::Quarterly),
            3 => Ok(LicenseTier::Yearly),
            _ => Err("Invalid license tier value"),
        }
    }
}

impl LicenseTier {
    /// Get duration in days for this license tier
    pub fn duration_days(&self) -> i32 {
        match self {
            LicenseTier::Free => 0,
            LicenseTier::Monthly => 30,
            LicenseTier::Quarterly => 90,
            LicenseTier::Yearly => 365,
        }
    }

    /// Check if this is a Pro tier
    pub fn is_pro(&self) -> bool {
        !matches!(self, LicenseTier::Free)
    }

    /// Convert from u8 value
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => LicenseTier::Monthly,
            2 => LicenseTier::Quarterly,
            3 => LicenseTier::Yearly,
            _ => LicenseTier::Free,
        }
    }

    /// Get display name for this tier
    pub fn display_name(&self) -> &'static str {
        match self {
            LicenseTier::Free => "Free",
            LicenseTier::Monthly => "Monthly",
            LicenseTier::Quarterly => "Quarterly",
            LicenseTier::Yearly => "Yearly",
        }
    }
}

/// License status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseStatus {
    /// Current license tier
    pub tier: LicenseTier,
    /// Whether this is a Pro license
    pub is_pro: bool,
    /// Whether a license is currently activated
    pub activated: bool,
    /// Expiration timestamp (Unix timestamp in seconds)
    pub expires_at: Option<i64>,
    /// Days remaining until expiration
    pub days_remaining: Option<i32>,
    /// Machine ID this license is bound to
    pub machine_id: String,
    /// Activation timestamp (Unix timestamp in seconds)
    pub activation_date: Option<i64>,
}

impl Default for LicenseStatus {
    fn default() -> Self {
        Self {
            tier: LicenseTier::Free,
            is_pro: false,
            activated: false,
            expires_at: None,
            days_remaining: None,
            machine_id: String::new(),
            activation_date: None,
        }
    }
}

/// Activated license stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivatedLicense {
    /// The license key
    pub license_key: String,
    /// License tier as u8
    pub tier: u8,
    /// Activation timestamp (Unix timestamp)
    pub activated_at: i64,
    /// Expiration timestamp (Unix timestamp)
    pub expires_at: i64,
    /// SHA256 hash of machine ID
    pub machine_id_hash: String,
}

/// License validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseValidationResult {
    /// Whether the license is valid
    pub valid: bool,
    /// Error code if validation failed
    pub error_code: Option<String>,
    /// Human-readable error message
    pub error_message: Option<String>,
    /// License info if validation succeeded
    pub license_info: Option<LicenseStatus>,
}

impl LicenseValidationResult {
    /// Create a successful validation result
    pub fn success(license_info: LicenseStatus) -> Self {
        Self {
            valid: true,
            error_code: None,
            error_message: None,
            license_info: Some(license_info),
        }
    }

    /// Create a failed validation result
    pub fn failure(error_code: &str, error_message: &str) -> Self {
        Self {
            valid: false,
            error_code: Some(error_code.to_string()),
            error_message: Some(error_message.to_string()),
            license_info: None,
        }
    }
}

/// Feature access rights based on license tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureAccess {
    /// Scan functionality - Free
    pub scan: bool,
    /// File shredder - Free
    pub file_shredder: bool,
    /// System logs cleanup - Pro
    pub system_logs: bool,
    /// Memory cleanup - Pro
    pub memory_cleanup: bool,
    /// Network cleanup - Pro
    pub network_cleanup: bool,
    /// Registry/privacy cleanup - Pro
    pub registry_cleanup: bool,
    /// Timestamp modifier - Pro
    pub timestamp_modifier: bool,
    /// Anti-analysis detection - Pro
    pub anti_analysis: bool,
    /// Disk encryption management - Pro
    pub disk_encryption: bool,
    /// Scheduled tasks - Pro
    pub scheduled_tasks: bool,
}

impl FeatureAccess {
    /// Create feature access for Free tier
    pub fn free() -> Self {
        Self {
            scan: true,
            file_shredder: true,
            system_logs: false,
            memory_cleanup: false,
            network_cleanup: false,
            registry_cleanup: false,
            timestamp_modifier: false,
            anti_analysis: false,
            disk_encryption: false,
            scheduled_tasks: false,
        }
    }

    /// Create feature access for Pro tier
    pub fn pro() -> Self {
        Self {
            scan: true,
            file_shredder: true,
            system_logs: true,
            memory_cleanup: true,
            network_cleanup: true,
            registry_cleanup: true,
            timestamp_modifier: true,
            anti_analysis: true,
            disk_encryption: true,
            scheduled_tasks: true,
        }
    }

    /// Get feature access based on license status
    pub fn from_license_status(status: &LicenseStatus) -> Self {
        if status.is_pro && status.activated {
            Self::pro()
        } else {
            Self::free()
        }
    }
}

/// License key constants
pub mod constants {
    /// Base32 alphabet for license key encoding (no ambiguous chars)
    pub const BASE32_ALPHABET: &str = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

    /// License key length (without dashes)
    pub const KEY_LENGTH: usize = 25;

    /// License key group size
    pub const GROUP_SIZE: usize = 5;

    /// Number of groups in license key
    pub const NUM_GROUPS: usize = 5;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_license_tier_duration() {
        assert_eq!(LicenseTier::Free.duration_days(), 0);
        assert_eq!(LicenseTier::Monthly.duration_days(), 30);
        assert_eq!(LicenseTier::Quarterly.duration_days(), 90);
        assert_eq!(LicenseTier::Yearly.duration_days(), 365);
    }

    #[test]
    fn test_license_tier_is_pro() {
        assert!(!LicenseTier::Free.is_pro());
        assert!(LicenseTier::Monthly.is_pro());
        assert!(LicenseTier::Quarterly.is_pro());
        assert!(LicenseTier::Yearly.is_pro());
    }

    #[test]
    fn test_license_tier_from_u8() {
        assert_eq!(LicenseTier::from_u8(0), LicenseTier::Free);
        assert_eq!(LicenseTier::from_u8(1), LicenseTier::Monthly);
        assert_eq!(LicenseTier::from_u8(2), LicenseTier::Quarterly);
        assert_eq!(LicenseTier::from_u8(3), LicenseTier::Yearly);
        assert_eq!(LicenseTier::from_u8(99), LicenseTier::Free);
    }

    #[test]
    fn test_feature_access_free() {
        let access = FeatureAccess::free();
        assert!(access.scan);
        assert!(access.file_shredder);
        assert!(!access.system_logs);
        assert!(!access.memory_cleanup);
    }

    #[test]
    fn test_feature_access_pro() {
        let access = FeatureAccess::pro();
        assert!(access.scan);
        assert!(access.file_shredder);
        assert!(access.system_logs);
        assert!(access.memory_cleanup);
        assert!(access.network_cleanup);
    }
}
