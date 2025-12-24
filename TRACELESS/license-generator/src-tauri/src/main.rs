//! License Key Generator - Tauri Backend
//!
//! Generates valid license keys for the Traceless application.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod generator;

use generator::{generate_license_key, LicenseTier, LicenseOutput};

#[tauri::command]
fn generate_license(
    tier: String,
    machine_id: String,
    activation_date: Option<String>,
    count: Option<usize>,
) -> Result<Vec<LicenseOutput>, String> {
    let tier = match tier.to_lowercase().as_str() {
        "monthly" => LicenseTier::Monthly,
        "quarterly" => LicenseTier::Quarterly,
        "yearly" => LicenseTier::Yearly,
        _ => return Err("Invalid tier. Use: monthly, quarterly, yearly".to_string()),
    };

    if machine_id.len() < 8 {
        return Err("Machine ID must be at least 8 characters".to_string());
    }

    let count = count.unwrap_or(1).min(100); // Max 100 licenses at once
    let mut licenses = Vec::new();

    for _ in 0..count {
        let license = generate_license_key(tier, &machine_id.to_uppercase(), activation_date.as_deref())?;
        licenses.push(license);
    }

    Ok(licenses)
}

#[tauri::command]
fn get_tiers() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "value": "monthly",
            "label": "月度版 (Monthly)",
            "days": 30,
            "description": "30 days validity"
        }),
        serde_json::json!({
            "value": "quarterly",
            "label": "季度版 (Quarterly)",
            "days": 90,
            "description": "90 days validity"
        }),
        serde_json::json!({
            "value": "yearly",
            "label": "年度版 (Yearly)",
            "days": 365,
            "description": "365 days validity"
        }),
    ]
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            generate_license,
            get_tiers,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
