//! License key validation and generation
//!
//! This module handles license key validation, including format checking,
//! signature verification, machine binding, and expiration checking.

use crate::modules::license::{LicenseTier, LicenseStatus, LicenseValidationResult, ActivatedLicense, constants};
use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

/// HMAC signing key (should be kept secret in production)
const HMAC_SECRET: &[u8] = b"traceless-license-hmac-secret-key-v1";

/// Get machine ID for license binding
pub fn get_machine_id() -> String {
    let mut hasher = Sha256::new();

    // Hostname
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

    // Username
    if let Ok(user) = std::env::var("USER").or_else(|_| std::env::var("USERNAME")) {
        hasher.update(user.as_bytes());
    }

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
        if let Ok(output) = std::process::Command::new("wmic")
            .args(["csproduct", "get", "UUID"])
            .output()
        {
            hasher.update(&output.stdout);
        }
    }

    // Add salt
    hasher.update(b"traceless-machine-id-v1");

    let result = hasher.finalize();
    hex::encode(result)
}

/// Get short machine ID for display (first 16 chars)
pub fn get_short_machine_id() -> String {
    let full_id = get_machine_id();
    full_id.chars().take(16).collect::<String>().to_uppercase()
}

/// Base32 encoding table
fn base32_alphabet() -> &'static [u8; 32] {
    static ALPHABET: [u8; 32] = *b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    &ALPHABET
}

/// Convert byte to base32 character
fn byte_to_base32(value: u8) -> char {
    let alphabet = base32_alphabet();
    alphabet[(value & 0x1F) as usize] as char
}

/// Convert base32 character to byte
fn base32_to_byte(c: char) -> Option<u8> {
    let c = c.to_ascii_uppercase();
    constants::BASE32_ALPHABET.chars().position(|x| x == c).map(|p| p as u8)
}

/// Format license key with dashes (XXXXX-XXXXX-XXXXX-XXXXX-XXXXX)
pub fn format_license_key(raw: &str) -> String {
    raw.chars()
        .collect::<Vec<_>>()
        .chunks(constants::GROUP_SIZE)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("-")
}

/// Remove dashes from license key
pub fn normalize_license_key(key: &str) -> String {
    key.chars()
        .filter(|c| *c != '-' && !c.is_whitespace())
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

/// Validate license key format
fn validate_format(key: &str) -> Result<(), String> {
    let normalized = normalize_license_key(key);

    if normalized.len() != constants::KEY_LENGTH {
        return Err("INVALID_FORMAT".to_string());
    }

    // Check all characters are valid base32
    for c in normalized.chars() {
        if base32_to_byte(c).is_none() {
            return Err("INVALID_FORMAT".to_string());
        }
    }

    Ok(())
}

/// Decode license key to bytes
fn decode_license_key(key: &str) -> Result<Vec<u8>, String> {
    let normalized = normalize_license_key(key);
    let mut bytes = Vec::new();
    let mut buffer: u64 = 0;
    let mut bits = 0;

    for c in normalized.chars() {
        let value = base32_to_byte(c).ok_or("INVALID_FORMAT")?;
        buffer = (buffer << 5) | (value as u64);
        bits += 5;

        while bits >= 8 {
            bits -= 8;
            bytes.push(((buffer >> bits) & 0xFF) as u8);
        }
    }

    // Handle remaining bits (important for checksum recovery)
    // If there are remaining bits, left-pad them to form a complete byte
    if bits > 0 {
        bytes.push(((buffer << (8 - bits)) & 0xFF) as u8);
    }

    Ok(bytes)
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

    // Handle remaining bits
    if bits > 0 {
        buffer <<= 5 - bits;
        result.push(byte_to_base32((buffer & 0x1F) as u8));
    }

    // Pad to exact length
    while result.len() < constants::KEY_LENGTH {
        result.push('A');
    }

    result.chars().take(constants::KEY_LENGTH).collect()
}

/// License key structure (125 bits total):
/// - Type: 2 bits (0-3 for Free/Monthly/Quarterly/Yearly)
/// - Timestamp: 32 bits (Unix timestamp / 86400 = days since epoch)
/// - Machine ID partial: 32 bits (first 8 chars of machine ID hash)
/// - Random salt: 24 bits
/// - HMAC signature: 24 bits (partial HMAC-SHA256)
/// - Checksum: 11 bits (CRC-like)
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
    /// Parse license key data from bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 15 {
            return Err("INVALID_FORMAT".to_string());
        }

        // Extract fields
        let tier_byte = (bytes[0] >> 6) & 0x03;
        let tier = LicenseTier::from_u8(tier_byte);

        let timestamp_days = ((bytes[0] as u32 & 0x3F) << 26)
            | ((bytes[1] as u32) << 18)
            | ((bytes[2] as u32) << 10)
            | ((bytes[3] as u32) << 2)
            | ((bytes[4] as u32) >> 6);

        let machine_hash = ((bytes[4] as u32 & 0x3F) << 26)
            | ((bytes[5] as u32) << 18)
            | ((bytes[6] as u32) << 10)
            | ((bytes[7] as u32) << 2)
            | ((bytes[8] as u32) >> 6);

        let salt = ((bytes[8] as u32 & 0x3F) << 18)
            | ((bytes[9] as u32) << 10)
            | ((bytes[10] as u32) << 2)
            | ((bytes[11] as u32) >> 6);

        let signature = ((bytes[11] as u32 & 0x3F) << 18)
            | ((bytes[12] as u32) << 10)
            | ((bytes[13] as u32) << 2)
            | ((bytes[14] as u32) >> 6);

        let checksum = ((bytes[14] as u16 & 0x3F) << 5)
            | if bytes.len() > 15 { (bytes[15] >> 3) as u16 } else { 0 };

        Ok(Self {
            tier,
            timestamp_days,
            machine_hash,
            salt,
            signature,
            checksum,
        })
    }

    /// Convert to bytes for encoding
    fn to_bytes(&self) -> Vec<u8> {
        let tier_byte = self.tier as u8;
        let mut bytes = vec![0u8; 16];

        // Pack fields into bytes
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
        bytes[15] = ((self.checksum << 3) as u8 & 0xF8);

        bytes
    }

    /// Calculate expected signature
    fn calculate_signature(&self) -> u32 {
        let mut mac = HmacSha256::new_from_slice(HMAC_SECRET)
            .expect("HMAC can take key of any size");

        mac.update(&[self.tier as u8]);
        mac.update(&self.timestamp_days.to_be_bytes());
        mac.update(&self.machine_hash.to_be_bytes());
        mac.update(&self.salt.to_be_bytes());

        let result = mac.finalize();
        let bytes = result.into_bytes();

        // Take first 3 bytes as signature (24 bits)
        ((bytes[0] as u32) << 16) | ((bytes[1] as u32) << 8) | (bytes[2] as u32)
    }

    /// Calculate checksum
    fn calculate_checksum(&self) -> u16 {
        let bytes = self.to_bytes();
        let mut checksum: u16 = 0;

        // Simple CRC-like checksum over first 14.5 bytes (before checksum field)
        for (i, &byte) in bytes.iter().take(15).enumerate() {
            checksum = checksum.wrapping_add((byte as u16).wrapping_mul((i as u16).wrapping_add(1)));
            checksum ^= checksum >> 3;
        }

        checksum & 0x07FF // 11 bits
    }

    /// Verify signature
    fn verify_signature(&self) -> bool {
        let expected = self.calculate_signature();
        self.signature == expected
    }

    /// Verify checksum
    fn verify_checksum(&self) -> bool {
        // Create a copy without the checksum to calculate expected
        let mut temp = self.clone();
        temp.checksum = 0;
        let expected = temp.calculate_checksum();
        self.checksum == expected
    }
}

/// Hash machine ID to 32-bit value
fn hash_machine_id(machine_id: &str) -> u32 {
    let bytes = hex::decode(machine_id).unwrap_or_default();
    if bytes.len() >= 4 {
        ((bytes[0] as u32) << 24) | ((bytes[1] as u32) << 16) | ((bytes[2] as u32) << 8) | (bytes[3] as u32)
    } else {
        0
    }
}

/// Get current timestamp in days since epoch
fn current_days() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| (d.as_secs() / 86400) as u32)
        .unwrap_or(0)
}

/// Generate a license key
pub fn generate_license_key(tier: LicenseTier, machine_id: &str) -> String {
    use rand::Rng;

    let machine_hash = hash_machine_id(machine_id);
    let timestamp_days = current_days();
    let salt: u32 = rand::thread_rng().gen::<u32>() & 0x00FFFFFF; // 24 bits

    let mut data = LicenseKeyData {
        tier,
        timestamp_days,
        machine_hash,
        salt,
        signature: 0,
        checksum: 0,
    };

    // Calculate signature
    data.signature = data.calculate_signature();

    // Calculate checksum (need to set signature first)
    data.checksum = data.calculate_checksum();

    let bytes = data.to_bytes();
    let raw_key = encode_license_key(&bytes);
    format_license_key(&raw_key)
}

/// Validate a license key
pub fn validate_license_key(key: &str, machine_id: &str) -> LicenseValidationResult {
    // Step 1: Validate format
    if let Err(e) = validate_format(key) {
        return LicenseValidationResult::failure(&e, "Invalid license key format");
    }

    // Step 2: Decode key
    let bytes = match decode_license_key(key) {
        Ok(b) => b,
        Err(e) => return LicenseValidationResult::failure(&e, "Failed to decode license key"),
    };

    // Step 3: Parse key data
    let data = match LicenseKeyData::from_bytes(&bytes) {
        Ok(d) => d,
        Err(e) => return LicenseValidationResult::failure(&e, "Failed to parse license key"),
    };

    // Step 4: Verify checksum
    if !data.verify_checksum() {
        return LicenseValidationResult::failure("INVALID_CHECKSUM", "License key checksum verification failed");
    }

    // Step 5: Verify signature
    if !data.verify_signature() {
        return LicenseValidationResult::failure("INVALID_SIGNATURE", "License key signature verification failed");
    }

    // Step 6: Verify machine binding
    let expected_hash = hash_machine_id(machine_id);
    if data.machine_hash != expected_hash {
        return LicenseValidationResult::failure("MACHINE_MISMATCH", "This license is bound to a different machine");
    }

    // Step 7: Check expiration
    let activation_days = data.timestamp_days;
    let duration_days = data.tier.duration_days();
    let expiration_days = activation_days + duration_days as u32;
    let current = current_days();

    if duration_days > 0 && current > expiration_days {
        return LicenseValidationResult::failure("EXPIRED", "License has expired");
    }

    // Calculate remaining days
    let days_remaining = if duration_days > 0 {
        Some((expiration_days as i32) - (current as i32))
    } else {
        None
    };

    // Calculate timestamps
    let activation_timestamp = (activation_days as i64) * 86400;
    let expiration_timestamp = if duration_days > 0 {
        Some((expiration_days as i64) * 86400)
    } else {
        None
    };

    // Create license status
    let status = LicenseStatus {
        tier: data.tier,
        is_pro: data.tier.is_pro(),
        activated: true,
        expires_at: expiration_timestamp,
        days_remaining,
        machine_id: machine_id.to_string(),
        activation_date: Some(activation_timestamp),
    };

    LicenseValidationResult::success(status)
}

/// Create ActivatedLicense from validation result
pub fn create_activated_license(key: &str, result: &LicenseValidationResult, machine_id: &str) -> Option<ActivatedLicense> {
    if !result.valid {
        return None;
    }

    let info = result.license_info.as_ref()?;

    Some(ActivatedLicense {
        license_key: normalize_license_key(key),
        tier: info.tier as u8,
        activated_at: info.activation_date.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0)
        }),
        expires_at: info.expires_at.unwrap_or(0),
        machine_id_hash: {
            let mut hasher = Sha256::new();
            hasher.update(machine_id.as_bytes());
            hex::encode(hasher.finalize())
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_machine_id() {
        let id = get_machine_id();
        assert_eq!(id.len(), 64); // SHA256 hex is 64 chars

        // Should be consistent
        let id2 = get_machine_id();
        assert_eq!(id, id2);
    }

    #[test]
    fn test_short_machine_id() {
        let short_id = get_short_machine_id();
        assert_eq!(short_id.len(), 16);
    }

    #[test]
    fn test_normalize_license_key() {
        let key = "ABCDE-FGHIJ-KLMNP-QRSTU-VWXYZ";
        let normalized = normalize_license_key(key);
        assert_eq!(normalized, "ABCDEFGHIJKLMNPQRSTUVWXYZ");
    }

    #[test]
    fn test_validate_format() {
        // Valid format - using only valid Base32 chars (no I, O, 0, 1)
        assert!(validate_format("ABCDE-FGHJK-LMNPQ-RSTUV-WXYZ2").is_ok());

        // Too short
        assert!(validate_format("ABCDE-FGHJK").is_err());

        // Invalid characters (O, I, 0, 1 not in alphabet)
        assert!(validate_format("ABCDE-FGHIJ-KLMNO-QRSTU-VWXYZ").is_err());
    }

    #[test]
    fn test_generate_and_validate() {
        let machine_id = get_machine_id();

        // Generate monthly license
        let key = generate_license_key(LicenseTier::Monthly, &machine_id);
        assert_eq!(key.len(), 29); // 25 chars + 4 dashes

        // Validate
        let result = validate_license_key(&key, &machine_id);
        assert!(result.valid);
        assert!(result.license_info.is_some());

        let info = result.license_info.unwrap();
        assert_eq!(info.tier, LicenseTier::Monthly);
        assert!(info.is_pro);
        assert!(info.activated);
        assert!(info.days_remaining.unwrap() > 0);
    }

    #[test]
    fn test_machine_mismatch() {
        let machine_id = get_machine_id();
        let key = generate_license_key(LicenseTier::Yearly, &machine_id);

        // Try with different machine ID
        let result = validate_license_key(&key, "different_machine_id");
        assert!(!result.valid);
        assert_eq!(result.error_code, Some("MACHINE_MISMATCH".to_string()));
    }

    #[test]
    fn test_all_tiers() {
        let machine_id = get_machine_id();

        for tier in [LicenseTier::Monthly, LicenseTier::Quarterly, LicenseTier::Yearly] {
            let key = generate_license_key(tier, &machine_id);
            let result = validate_license_key(&key, &machine_id);
            assert!(result.valid, "Failed for tier {:?}", tier);
            assert_eq!(result.license_info.as_ref().unwrap().tier, tier);
        }
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let original = vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
        let encoded = encode_license_key(&original);
        let decoded = decode_license_key(&encoded).unwrap();

        // All 16 bytes should be recoverable (125 bits + 3 padding bits)
        assert_eq!(&original[..], &decoded[..16]);
    }
}
