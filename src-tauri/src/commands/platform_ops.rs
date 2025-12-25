//! Platform capabilities and feature availability commands

use crate::modules::{
    error_handling::{
        PlatformCapabilities, FeatureCapability, LinuxDistro,
        get_platform_capabilities, detect_linux_distro, is_feature_available,
        get_feature_unavailable_reason,
    },
    timeout_handler::{
        OperationInfo, TimeoutConfig,
        get_active_operations, get_timeout_config, set_timeout_config,
        cancel_operation, cancel_all_operations,
    },
};
use serde::{Deserialize, Serialize};

/// Get platform capabilities
#[tauri::command]
pub fn get_platform_info() -> Result<PlatformCapabilities, String> {
    Ok(get_platform_capabilities())
}

/// Get Linux distribution info (if on Linux)
#[tauri::command]
pub fn get_linux_distro_info() -> Result<Option<LinuxDistro>, String> {
    Ok(detect_linux_distro())
}

/// Check if a specific feature is available
#[tauri::command]
pub fn check_feature_available(feature: String) -> Result<bool, String> {
    Ok(is_feature_available(&feature))
}

/// Get reason why a feature is not available
#[tauri::command]
pub fn get_feature_reason(feature: String) -> Result<Option<String>, String> {
    Ok(get_feature_unavailable_reason(&feature))
}

/// Get all active operations
#[tauri::command]
pub fn get_running_operations() -> Result<Vec<OperationInfo>, String> {
    Ok(get_active_operations())
}

/// Cancel a specific operation
#[tauri::command]
pub fn cancel_running_operation(operation_id: u64) -> Result<bool, String> {
    Ok(cancel_operation(operation_id))
}

/// Cancel all running operations
#[tauri::command]
pub fn cancel_all_running_operations() -> Result<(), String> {
    cancel_all_operations();
    Ok(())
}

/// Get timeout configuration
#[tauri::command]
pub fn get_timeout_settings() -> Result<TimeoutConfig, String> {
    Ok(get_timeout_config())
}

/// Update timeout configuration
#[tauri::command]
pub fn set_timeout_settings(config: TimeoutConfig) -> Result<(), String> {
    set_timeout_config(config);
    Ok(())
}

/// Get CPU information for performance tuning
#[tauri::command]
pub fn get_cpu_info() -> Result<CpuInfo, String> {
    Ok(CpuInfo {
        logical_cores: crate::modules::get_cpu_count(),
        physical_cores: crate::modules::get_physical_cpu_count(),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub logical_cores: usize,
    pub physical_cores: usize,
}

/// Feature availability response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureStatus {
    pub feature: String,
    pub available: bool,
    pub reason: Option<String>,
    pub alternative: Option<String>,
}

/// Get status of multiple features at once
#[tauri::command]
pub fn check_features(features: Vec<String>) -> Result<Vec<FeatureStatus>, String> {
    let caps = get_platform_capabilities();

    let statuses: Vec<FeatureStatus> = features
        .into_iter()
        .map(|feature| {
            let cap = caps.features.iter().find(|f| f.name == feature);
            match cap {
                Some(c) => FeatureStatus {
                    feature: feature.clone(),
                    available: c.available,
                    reason: c.reason.clone(),
                    alternative: c.alternative.clone(),
                },
                None => FeatureStatus {
                    feature: feature.clone(),
                    available: false,
                    reason: Some("Feature not recognized".to_string()),
                    alternative: None,
                },
            }
        })
        .collect();

    Ok(statuses)
}
