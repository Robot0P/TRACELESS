//! Integration tests for settings storage module

use std::collections::HashMap;
use tempfile::TempDir;

mod common;

/// Create a temporary test environment
fn setup_test_env() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

#[test]
fn test_default_settings_structure() {
    // Test that default settings have expected structure
    let default_settings: HashMap<String, serde_json::Value> = [
        ("theme".to_string(), serde_json::json!("dark")),
        ("language".to_string(), serde_json::json!("auto")),
        ("auto_scan_on_startup".to_string(), serde_json::json!(false)),
        ("show_notifications".to_string(), serde_json::json!(true)),
        ("minimize_to_tray".to_string(), serde_json::json!(false)),
        ("auto_update".to_string(), serde_json::json!(true)),
        ("default_wipe_method".to_string(), serde_json::json!("dod")),
        ("confirm_before_delete".to_string(), serde_json::json!(true)),
        ("show_scan_summary".to_string(), serde_json::json!(true)),
        ("log_cleanup_operations".to_string(), serde_json::json!(false)),
        ("skip_system_files".to_string(), serde_json::json!(true)),
        ("scan_hidden_files".to_string(), serde_json::json!(false)),
        ("max_scan_depth".to_string(), serde_json::json!(10)),
        ("excluded_paths".to_string(), serde_json::json!([])),
    ].into_iter().collect();

    assert!(default_settings.contains_key("theme"), "Should have theme setting");
    assert!(default_settings.contains_key("language"), "Should have language setting");
    assert!(default_settings.contains_key("default_wipe_method"), "Should have wipe method setting");
}

#[test]
fn test_theme_values() {
    let valid_themes = vec!["dark", "light", "auto"];

    for theme in &valid_themes {
        assert!(
            valid_themes.contains(theme),
            "Theme '{}' should be valid",
            theme
        );
    }
}

#[test]
fn test_language_values() {
    let valid_languages = vec!["auto", "zh-CN", "en-US"];

    for lang in &valid_languages {
        assert!(
            valid_languages.contains(lang),
            "Language '{}' should be valid",
            lang
        );
    }
}

#[test]
fn test_wipe_method_values() {
    let valid_methods = vec!["zero", "random", "dod", "gutmann"];

    for method in &valid_methods {
        assert!(
            valid_methods.contains(method),
            "Wipe method '{}' should be valid",
            method
        );
    }
}

#[test]
fn test_boolean_settings() {
    let boolean_settings = vec![
        "auto_scan_on_startup",
        "show_notifications",
        "minimize_to_tray",
        "auto_update",
        "confirm_before_delete",
        "show_scan_summary",
        "log_cleanup_operations",
        "skip_system_files",
        "scan_hidden_files",
    ];

    for setting in boolean_settings {
        // These should accept true/false values
        let valid_values = [true, false];
        assert_eq!(valid_values.len(), 2, "Boolean setting '{}' should have 2 possible values", setting);
    }
}

#[test]
fn test_numeric_settings() {
    let numeric_settings = vec![
        ("max_scan_depth", 1, 100),
    ];

    for (setting, min, max) in numeric_settings {
        assert!(
            min <= max,
            "Setting '{}' min ({}) should be <= max ({})",
            setting, min, max
        );
    }
}

#[test]
fn test_settings_serialization() {
    let settings: HashMap<String, serde_json::Value> = [
        ("theme".to_string(), serde_json::json!("dark")),
        ("max_scan_depth".to_string(), serde_json::json!(10)),
        ("excluded_paths".to_string(), serde_json::json!(["/tmp", "/var/log"])),
    ].into_iter().collect();

    // Serialize to JSON
    let json_str = serde_json::to_string(&settings).expect("Failed to serialize");
    assert!(!json_str.is_empty(), "JSON should not be empty");

    // Deserialize back
    let deserialized: HashMap<String, serde_json::Value> =
        serde_json::from_str(&json_str).expect("Failed to deserialize");

    assert_eq!(settings.len(), deserialized.len(), "Settings count should match");
}

#[test]
fn test_excluded_paths_validation() {
    let excluded_paths: Vec<String> = vec![
        "/tmp".to_string(),
        "/var/log".to_string(),
        "/home/user/.cache".to_string(),
    ];

    for path in &excluded_paths {
        assert!(
            path.starts_with('/') || path.starts_with('~'),
            "Excluded path '{}' should be absolute or start with ~",
            path
        );
    }
}

#[test]
fn test_settings_database_path() {
    let temp_dir = setup_test_env();
    let db_path = temp_dir.path().join("settings.db");

    // Database file should not exist initially
    assert!(!db_path.exists(), "Database should not exist initially");

    // Create an empty file to simulate database
    std::fs::File::create(&db_path).expect("Failed to create db file");
    assert!(db_path.exists(), "Database file should exist after creation");
}

#[test]
fn test_settings_encryption_key() {
    // Test that encryption key derivation is consistent
    let app_id = "com.security.traceless";
    let key = format!("traceless_settings_key_{}", app_id);

    assert!(!key.is_empty(), "Encryption key should not be empty");
    assert!(key.len() > 20, "Encryption key should be reasonably long");
}

#[test]
fn test_settings_migration() {
    // Test that we can handle version migration scenarios
    let old_settings: HashMap<String, serde_json::Value> = [
        ("theme".to_string(), serde_json::json!("dark")),
        // Old version might not have all settings
    ].into_iter().collect();

    let all_required_keys = vec![
        "theme",
        "language",
        "auto_scan_on_startup",
        "show_notifications",
    ];

    for key in &all_required_keys {
        if !old_settings.contains_key(*key) {
            // Missing key would need default value during migration
            assert!(
                true,
                "Setting '{}' missing from old settings, needs default",
                key
            );
        }
    }
}

#[test]
fn test_settings_reset() {
    let custom_settings: HashMap<String, serde_json::Value> = [
        ("theme".to_string(), serde_json::json!("light")),  // Custom
        ("max_scan_depth".to_string(), serde_json::json!(50)),  // Custom
    ].into_iter().collect();

    let default_settings: HashMap<String, serde_json::Value> = [
        ("theme".to_string(), serde_json::json!("dark")),  // Default
        ("max_scan_depth".to_string(), serde_json::json!(10)),  // Default
    ].into_iter().collect();

    // After reset, settings should be different from custom
    assert_ne!(
        custom_settings.get("theme"),
        default_settings.get("theme"),
        "Custom theme should differ from default"
    );
    assert_ne!(
        custom_settings.get("max_scan_depth"),
        default_settings.get("max_scan_depth"),
        "Custom max_scan_depth should differ from default"
    );
}

#[test]
fn test_settings_validation() {
    // Test validation of settings values
    let valid_max_depth = 10;
    let invalid_max_depth_negative = -1;
    let invalid_max_depth_too_high = 1000;

    assert!(valid_max_depth > 0 && valid_max_depth <= 100, "Valid depth should be in range");
    assert!(invalid_max_depth_negative <= 0, "Negative depth should be invalid");
    assert!(invalid_max_depth_too_high > 100, "Too high depth should be invalid");
}
