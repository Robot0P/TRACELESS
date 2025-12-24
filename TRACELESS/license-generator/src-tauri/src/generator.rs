//! License Key Generation Module
//!
//! Contains the core license key generation algorithm.

use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use rand::Rng;
use chrono::{DateTime, Utc, NaiveDate};
use serde::Serialize;

type HmacSha256 = Hmac<Sha256>;

/// HMAC signing key (must match the key in the main application)
const HMAC_SECRET: &[u8] = b"traceless-license-hmac-secret-key-v1";

/// Base32 alphabet (no I, O, 0, 1 to avoid confusion)
const BASE32_ALPHABET: &str = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

/// License key length (25 characters)
const KEY_LENGTH: usize = 25;

/// Group size for formatted key (5 characters per group)
const GROUP_SIZE: usize = 5;

/// License tiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LicenseTier {
    Monthly,
    Quarterly,
    Yearly,
}

impl LicenseTier {
    /// Get the duration in days for this tier
    fn duration_days(&self) -> u32 {
        match self {
            LicenseTier::Monthly => 30,
            LicenseTier::Quarterly => 90,
            LicenseTier::Yearly => 365,
        }
    }

    /// Convert to u8 for encoding
    fn as_u8(&self) -> u8 {
        match self {
            LicenseTier::Monthly => 1,
            LicenseTier::Quarterly => 2,
            LicenseTier::Yearly => 3,
        }
    }
}

/// License output structure
#[derive(Debug, Clone, Serialize)]
pub struct LicenseOutput {
    pub license_key: String,
    pub tier: String,
    pub machine_id: String,
    pub activation_date: String,
    pub expiration_date: String,
    pub days_valid: u32,
}

/// License key internal structure
#[derive(Debug, Clone)]
struct LicenseKeyData {
    tier: LicenseTier,
    timestamp_days: u32,
    machine_hash: u32,
    salt: u32,
    signature: u32,
    checksum: u16,
}

impl LicenseKeyData {
    /// Create new license key data
    fn new(tier: LicenseTier, machine_id: &str, activation_days: u32) -> Self {
        let machine_hash = hash_machine_id(machine_id);
        let salt: u32 = rand::thread_rng().gen::<u32>() & 0x00FFFFFF;

        let mut data = Self {
            tier,
            timestamp_days: activation_days,
            machine_hash,
            salt,
            signature: 0,
            checksum: 0,
        };

        data.signature = data.calculate_signature();
        data.checksum = data.calculate_checksum();

        data
    }

    /// Convert to bytes for encoding
    fn to_bytes(&self) -> Vec<u8> {
        let tier_byte = self.tier.as_u8();
        let mut bytes = vec![0u8; 16];

        bytes[0] = (tier_byte << 6) | ((self.timestamp_days >> 26) as u8 & 0x3F);
        bytes[1] = ((self.timestamp_days >> 18) & 0xFF) as u8;
        bytes[2] = ((self.timestamp_days >> 10) & 0xFF) as u8;
        bytes[3] = ((self.timestamp_days >> 2) & 0xFF) as u8;
        bytes[4] = ((self.timestamp_days << 6) as u8 & 0xC0) | ((self.machine_hash >> 26) as u8 & 0x3F);
        bytes[5] = ((self.machine_hash >> 18) & 0xFF) as u8;
        bytes[6] = ((self.machine_hash >> 10) & 0xFF) as u8;
        bytes[7] = ((self.machine_hash >> 2) & 0xFF) as u8;
        bytes[8] = ((self.machine_hash << 6) as u8 & 0xC0) | ((self.salt >> 18) as u8 & 0x3F);
        bytes[9] = ((self.salt >> 10) & 0xFF) as u8;
        bytes[10] = ((self.salt >> 2) & 0xFF) as u8;
        bytes[11] = ((self.salt << 6) as u8 & 0xC0) | ((self.signature >> 18) as u8 & 0x3F);
        bytes[12] = ((self.signature >> 10) & 0xFF) as u8;
        bytes[13] = ((self.signature >> 2) & 0xFF) as u8;
        bytes[14] = ((self.signature << 6) as u8 & 0xC0) | ((self.checksum >> 5) as u8 & 0x3F);
        bytes[15] = (self.checksum << 3) as u8 & 0xF8;

        bytes
    }

    /// Calculate HMAC signature
    fn calculate_signature(&self) -> u32 {
        let mut mac = HmacSha256::new_from_slice(HMAC_SECRET)
            .expect("HMAC can take key of any size");

        mac.update(&[self.tier.as_u8()]);
        mac.update(&self.timestamp_days.to_be_bytes());
        mac.update(&self.machine_hash.to_be_bytes());
        mac.update(&self.salt.to_be_bytes());

        let result = mac.finalize();
        let bytes = result.into_bytes();

        ((bytes[0] as u32) << 16) | ((bytes[1] as u32) << 8) | (bytes[2] as u32)
    }

    /// Calculate checksum
    fn calculate_checksum(&self) -> u16 {
        let bytes = self.to_bytes();
        let mut checksum: u16 = 0;

        for (i, &byte) in bytes.iter().take(15).enumerate() {
            checksum = checksum.wrapping_add((byte as u16).wrapping_mul((i as u16).wrapping_add(1)));
            checksum ^= checksum >> 3;
        }

        checksum & 0x07FF
    }
}

/// Hash machine ID to 32-bit value
/// The machine_id is already a hex string from the main app (64 chars SHA256 hash)
/// We decode it and take first 4 bytes
fn hash_machine_id(machine_id: &str) -> u32 {
    // Machine ID from main app is already a 64-char hex SHA256 hash
    // We decode it and take first 4 bytes
    let bytes = hex::decode(machine_id).unwrap_or_default();
    if bytes.len() >= 4 {
        ((bytes[0] as u32) << 24) | ((bytes[1] as u32) << 16) | ((bytes[2] as u32) << 8) | (bytes[3] as u32)
    } else {
        0
    }
}

/// Convert byte to base32 character
fn byte_to_base32(value: u8) -> char {
    BASE32_ALPHABET.chars().nth((value & 0x1F) as usize).unwrap_or('A')
}

/// Encode bytes to base32 license key
fn encode_license_key(bytes: &[u8]) -> String {
    let mut result = String::new();
    let mut buffer: u64 = 0;
    let mut bits = 0;

    for &byte in bytes {
        buffer = (buffer << 8) | (byte as u64);
        bits += 8;

        while bits >= 5 {
            bits -= 5;
            result.push(byte_to_base32(((buffer >> bits) & 0x1F) as u8));
        }
    }

    if bits > 0 {
        buffer <<= 5 - bits;
        result.push(byte_to_base32((buffer & 0x1F) as u8));
    }

    while result.len() < KEY_LENGTH {
        result.push('A');
    }

    result.chars().take(KEY_LENGTH).collect()
}

/// Format license key with dashes
fn format_license_key(raw: &str) -> String {
    raw.chars()
        .collect::<Vec<_>>()
        .chunks(GROUP_SIZE)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("-")
}

/// Get current days since epoch
fn current_days() -> u32 {
    let now = Utc::now();
    (now.timestamp() / 86400) as u32
}

/// Parse date string to days since epoch
fn parse_date_to_days(date_str: &str) -> Result<u32, String> {
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date format: {}. Use YYYY-MM-DD", e))?;

    let datetime = date.and_hms_opt(0, 0, 0).unwrap();
    let timestamp = datetime.and_utc().timestamp();
    Ok((timestamp / 86400) as u32)
}

/// Days to date string
fn days_to_date_string(days: u32) -> String {
    let timestamp = (days as i64) * 86400;
    let datetime = DateTime::from_timestamp(timestamp, 0)
        .unwrap_or_else(Utc::now);
    datetime.format("%Y-%m-%d").to_string()
}

/// Generate a license key
pub fn generate_license_key(
    tier: LicenseTier,
    machine_id: &str,
    activation_date: Option<&str>,
) -> Result<LicenseOutput, String> {
    let activation_days = match activation_date {
        Some(date) => parse_date_to_days(date)?,
        None => current_days(),
    };

    let expiration_days = activation_days + tier.duration_days();

    let data = LicenseKeyData::new(tier, machine_id, activation_days);
    let bytes = data.to_bytes();
    let raw_key = encode_license_key(&bytes);
    let license_key = format_license_key(&raw_key);

    let tier_name = match tier {
        LicenseTier::Monthly => "monthly",
        LicenseTier::Quarterly => "quarterly",
        LicenseTier::Yearly => "yearly",
    };

    Ok(LicenseOutput {
        license_key,
        tier: tier_name.to_string(),
        machine_id: machine_id.to_string(),
        activation_date: days_to_date_string(activation_days),
        expiration_date: days_to_date_string(expiration_days),
        days_valid: tier.duration_days(),
    })
}
