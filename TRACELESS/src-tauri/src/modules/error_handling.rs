//! Error handling and logging module
//!
//! Provides structured error handling, logging, and operation tracking.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;

/// Operation error with context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationError {
    /// Error code for categorization
    pub code: ErrorCode,
    /// Human-readable message
    pub message: String,
    /// Additional context details
    pub context: Option<String>,
    /// Timestamp when error occurred
    pub timestamp: DateTime<Utc>,
    /// Operation that caused the error
    pub operation: String,
    /// Whether this error is recoverable
    pub recoverable: bool,
}

/// Error codes for categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    /// Permission denied
    PermissionDenied,
    /// File or path not found
    NotFound,
    /// Path validation failed
    PathValidation,
    /// I/O operation failed
    IoError,
    /// Operation timed out
    Timeout,
    /// Operation was cancelled
    Cancelled,
    /// Platform not supported
    PlatformNotSupported,
    /// Feature not available
    FeatureNotAvailable,
    /// Invalid input
    InvalidInput,
    /// Resource busy
    ResourceBusy,
    /// Network error
    NetworkError,
    /// Database error
    DatabaseError,
    /// Unknown error
    Unknown,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCode::PermissionDenied => write!(f, "PERMISSION_DENIED"),
            ErrorCode::NotFound => write!(f, "NOT_FOUND"),
            ErrorCode::PathValidation => write!(f, "PATH_VALIDATION"),
            ErrorCode::IoError => write!(f, "IO_ERROR"),
            ErrorCode::Timeout => write!(f, "TIMEOUT"),
            ErrorCode::Cancelled => write!(f, "CANCELLED"),
            ErrorCode::PlatformNotSupported => write!(f, "PLATFORM_NOT_SUPPORTED"),
            ErrorCode::FeatureNotAvailable => write!(f, "FEATURE_NOT_AVAILABLE"),
            ErrorCode::InvalidInput => write!(f, "INVALID_INPUT"),
            ErrorCode::ResourceBusy => write!(f, "RESOURCE_BUSY"),
            ErrorCode::NetworkError => write!(f, "NETWORK_ERROR"),
            ErrorCode::DatabaseError => write!(f, "DATABASE_ERROR"),
            ErrorCode::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

impl OperationError {
    /// Create a new operation error
    pub fn new(code: ErrorCode, message: impl Into<String>, operation: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            context: None,
            timestamp: Utc::now(),
            operation: operation.into(),
            recoverable: matches!(code, ErrorCode::Timeout | ErrorCode::ResourceBusy | ErrorCode::NetworkError),
        }
    }

    /// Add context to the error
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Mark as recoverable
    pub fn recoverable(mut self) -> Self {
        self.recoverable = true;
        self
    }

    /// Mark as non-recoverable
    pub fn non_recoverable(mut self) -> Self {
        self.recoverable = false;
        self
    }
}

impl fmt::Display for OperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.code, self.operation, self.message)?;
        if let Some(ref ctx) = self.context {
            write!(f, " ({})", ctx)?;
        }
        Ok(())
    }
}

impl std::error::Error for OperationError {}

impl From<OperationError> for String {
    fn from(err: OperationError) -> String {
        err.to_string()
    }
}

impl From<std::io::Error> for OperationError {
    fn from(err: std::io::Error) -> Self {
        let code = match err.kind() {
            std::io::ErrorKind::NotFound => ErrorCode::NotFound,
            std::io::ErrorKind::PermissionDenied => ErrorCode::PermissionDenied,
            std::io::ErrorKind::TimedOut => ErrorCode::Timeout,
            _ => ErrorCode::IoError,
        };
        OperationError::new(code, err.to_string(), "io_operation")
    }
}

/// Operation log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationLog {
    /// Unique operation ID
    pub id: u64,
    /// Operation type
    pub operation_type: OperationType,
    /// Operation status
    pub status: OperationStatus,
    /// Start time
    pub started_at: DateTime<Utc>,
    /// End time (if completed)
    pub ended_at: Option<DateTime<Utc>>,
    /// Target path or resource
    pub target: String,
    /// Additional details
    pub details: Option<String>,
    /// Error if failed
    pub error: Option<OperationError>,
    /// Items processed
    pub items_processed: u64,
    /// Items failed
    pub items_failed: u64,
}

/// Operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    SecureDelete,
    LogCleanup,
    MemoryCleanup,
    NetworkCleanup,
    RegistryCleanup,
    TimestampModify,
    SystemScan,
    ScheduledCleanup,
    CustomRule,
}

impl fmt::Display for OperationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperationType::SecureDelete => write!(f, "secure_delete"),
            OperationType::LogCleanup => write!(f, "log_cleanup"),
            OperationType::MemoryCleanup => write!(f, "memory_cleanup"),
            OperationType::NetworkCleanup => write!(f, "network_cleanup"),
            OperationType::RegistryCleanup => write!(f, "registry_cleanup"),
            OperationType::TimestampModify => write!(f, "timestamp_modify"),
            OperationType::SystemScan => write!(f, "system_scan"),
            OperationType::ScheduledCleanup => write!(f, "scheduled_cleanup"),
            OperationType::CustomRule => write!(f, "custom_rule"),
        }
    }
}

/// Operation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
}

/// Global operation ID counter
static OPERATION_ID: AtomicU64 = AtomicU64::new(1);

/// Operation log storage
static OPERATION_LOGS: Lazy<Mutex<Vec<OperationLog>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Maximum number of logs to keep
const MAX_LOG_ENTRIES: usize = 1000;

impl OperationLog {
    /// Create a new operation log entry
    pub fn new(operation_type: OperationType, target: impl Into<String>) -> Self {
        Self {
            id: OPERATION_ID.fetch_add(1, Ordering::SeqCst),
            operation_type,
            status: OperationStatus::Pending,
            started_at: Utc::now(),
            ended_at: None,
            target: target.into(),
            details: None,
            error: None,
            items_processed: 0,
            items_failed: 0,
        }
    }

    /// Start the operation
    pub fn start(&mut self) {
        self.status = OperationStatus::Running;
        self.started_at = Utc::now();
    }

    /// Complete the operation successfully
    pub fn complete(&mut self) {
        self.status = OperationStatus::Completed;
        self.ended_at = Some(Utc::now());
    }

    /// Mark operation as failed
    pub fn fail(&mut self, error: OperationError) {
        self.status = OperationStatus::Failed;
        self.ended_at = Some(Utc::now());
        self.error = Some(error);
    }

    /// Mark operation as cancelled
    pub fn cancel(&mut self) {
        self.status = OperationStatus::Cancelled;
        self.ended_at = Some(Utc::now());
    }

    /// Mark operation as timed out
    pub fn timeout(&mut self) {
        self.status = OperationStatus::TimedOut;
        self.ended_at = Some(Utc::now());
        self.error = Some(OperationError::new(
            ErrorCode::Timeout,
            "Operation timed out",
            self.operation_type.to_string(),
        ));
    }

    /// Add details
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Update progress
    pub fn update_progress(&mut self, processed: u64, failed: u64) {
        self.items_processed = processed;
        self.items_failed = failed;
    }

    /// Get duration in milliseconds
    pub fn duration_ms(&self) -> Option<i64> {
        self.ended_at.map(|end| {
            (end - self.started_at).num_milliseconds()
        })
    }

    /// Save this log entry
    pub fn save(&self) {
        if let Ok(mut logs) = OPERATION_LOGS.lock() {
            logs.push(self.clone());
            // Keep only the last MAX_LOG_ENTRIES
            if logs.len() > MAX_LOG_ENTRIES {
                let drain_count = logs.len() - MAX_LOG_ENTRIES;
                logs.drain(0..drain_count);
            }
        }
    }
}

/// Get all operation logs
pub fn get_operation_logs() -> Vec<OperationLog> {
    OPERATION_LOGS.lock().map(|logs| logs.clone()).unwrap_or_default()
}

/// Get recent operation logs (last N entries)
pub fn get_recent_logs(count: usize) -> Vec<OperationLog> {
    OPERATION_LOGS.lock().map(|logs| {
        let start = logs.len().saturating_sub(count);
        logs[start..].to_vec()
    }).unwrap_or_default()
}

/// Get logs by operation type
pub fn get_logs_by_type(op_type: OperationType) -> Vec<OperationLog> {
    OPERATION_LOGS.lock().map(|logs| {
        logs.iter()
            .filter(|log| log.operation_type == op_type)
            .cloned()
            .collect()
    }).unwrap_or_default()
}

/// Get failed operations
pub fn get_failed_operations() -> Vec<OperationLog> {
    OPERATION_LOGS.lock().map(|logs| {
        logs.iter()
            .filter(|log| log.status == OperationStatus::Failed)
            .cloned()
            .collect()
    }).unwrap_or_default()
}

/// Clear all logs
pub fn clear_logs() {
    if let Ok(mut logs) = OPERATION_LOGS.lock() {
        logs.clear();
    }
}

/// Platform capability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformCapabilities {
    /// Platform name
    pub platform: String,
    /// Platform version
    pub version: String,
    /// Linux distribution info (if applicable)
    pub linux_distro: Option<LinuxDistro>,
    /// Available features
    pub features: Vec<FeatureCapability>,
}

/// Linux distribution information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinuxDistro {
    /// Distribution name (e.g., Ubuntu, Fedora, Arch)
    pub name: String,
    /// Distribution ID (lowercase, e.g., ubuntu, fedora, arch)
    pub id: String,
    /// Version string
    pub version: String,
    /// Version codename (if available)
    pub codename: Option<String>,
    /// Package manager type
    pub package_manager: PackageManager,
}

/// Package manager types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageManager {
    Apt,      // Debian, Ubuntu
    Dnf,      // Fedora, RHEL
    Yum,      // CentOS, older RHEL
    Pacman,   // Arch
    Zypper,   // openSUSE
    Apk,      // Alpine
    Unknown,
}

impl fmt::Display for PackageManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackageManager::Apt => write!(f, "apt"),
            PackageManager::Dnf => write!(f, "dnf"),
            PackageManager::Yum => write!(f, "yum"),
            PackageManager::Pacman => write!(f, "pacman"),
            PackageManager::Zypper => write!(f, "zypper"),
            PackageManager::Apk => write!(f, "apk"),
            PackageManager::Unknown => write!(f, "unknown"),
        }
    }
}

/// Feature capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureCapability {
    /// Feature name
    pub name: String,
    /// Whether feature is available
    pub available: bool,
    /// Reason if not available
    pub reason: Option<String>,
    /// Alternative suggestion if not available
    pub alternative: Option<String>,
}

impl FeatureCapability {
    /// Create an available feature
    pub fn available(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            available: true,
            reason: None,
            alternative: None,
        }
    }

    /// Create an unavailable feature
    pub fn unavailable(name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            available: false,
            reason: Some(reason.into()),
            alternative: None,
        }
    }

    /// Add alternative suggestion
    pub fn with_alternative(mut self, alt: impl Into<String>) -> Self {
        self.alternative = Some(alt.into());
        self
    }
}

/// Detect Linux distribution
#[cfg(target_os = "linux")]
pub fn detect_linux_distro() -> Option<LinuxDistro> {
    use std::fs;

    // Try to read /etc/os-release first (standard on most modern distros)
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        let mut name = String::new();
        let mut id = String::new();
        let mut version = String::new();
        let mut codename = None;

        for line in content.lines() {
            if let Some((key, value)) = line.split_once('=') {
                let value = value.trim_matches('"');
                match key {
                    "NAME" => name = value.to_string(),
                    "ID" => id = value.to_lowercase(),
                    "VERSION_ID" => version = value.to_string(),
                    "VERSION_CODENAME" => codename = Some(value.to_string()),
                    _ => {}
                }
            }
        }

        if !id.is_empty() {
            let package_manager = detect_package_manager(&id);
            return Some(LinuxDistro {
                name,
                id,
                version,
                codename,
                package_manager,
            });
        }
    }

    // Fallback: try /etc/lsb-release (Ubuntu and derivatives)
    if let Ok(content) = fs::read_to_string("/etc/lsb-release") {
        let mut name = String::new();
        let mut id = String::new();
        let mut version = String::new();
        let mut codename = None;

        for line in content.lines() {
            if let Some((key, value)) = line.split_once('=') {
                let value = value.trim_matches('"');
                match key {
                    "DISTRIB_ID" => {
                        name = value.to_string();
                        id = value.to_lowercase();
                    }
                    "DISTRIB_RELEASE" => version = value.to_string(),
                    "DISTRIB_CODENAME" => codename = Some(value.to_string()),
                    _ => {}
                }
            }
        }

        if !id.is_empty() {
            let package_manager = detect_package_manager(&id);
            return Some(LinuxDistro {
                name,
                id,
                version,
                codename,
                package_manager,
            });
        }
    }

    // Try individual distro files
    let distro_files = [
        ("/etc/debian_version", "debian", "Debian", PackageManager::Apt),
        ("/etc/fedora-release", "fedora", "Fedora", PackageManager::Dnf),
        ("/etc/centos-release", "centos", "CentOS", PackageManager::Yum),
        ("/etc/redhat-release", "rhel", "Red Hat Enterprise Linux", PackageManager::Yum),
        ("/etc/arch-release", "arch", "Arch Linux", PackageManager::Pacman),
        ("/etc/gentoo-release", "gentoo", "Gentoo", PackageManager::Unknown),
        ("/etc/SuSE-release", "suse", "SUSE Linux", PackageManager::Zypper),
        ("/etc/alpine-release", "alpine", "Alpine Linux", PackageManager::Apk),
    ];

    for (path, id, name, pm) in distro_files {
        if std::path::Path::new(path).exists() {
            let version = fs::read_to_string(path)
                .unwrap_or_default()
                .trim()
                .to_string();
            return Some(LinuxDistro {
                name: name.to_string(),
                id: id.to_string(),
                version,
                codename: None,
                package_manager: pm,
            });
        }
    }

    None
}

#[cfg(not(target_os = "linux"))]
pub fn detect_linux_distro() -> Option<LinuxDistro> {
    None
}

/// Detect package manager based on distro ID
fn detect_package_manager(distro_id: &str) -> PackageManager {
    match distro_id {
        "ubuntu" | "debian" | "linuxmint" | "pop" | "elementary" | "zorin" | "kali" => {
            PackageManager::Apt
        }
        "fedora" | "rhel" | "rocky" | "almalinux" => PackageManager::Dnf,
        "centos" => PackageManager::Yum,
        "arch" | "manjaro" | "endeavouros" | "garuda" => PackageManager::Pacman,
        "opensuse" | "opensuse-leap" | "opensuse-tumbleweed" | "sles" => PackageManager::Zypper,
        "alpine" => PackageManager::Apk,
        _ => PackageManager::Unknown,
    }
}

/// Get platform capabilities
pub fn get_platform_capabilities() -> PlatformCapabilities {
    let (platform, version) = get_platform_info();
    let linux_distro = detect_linux_distro();
    let features = get_feature_capabilities(&platform, linux_distro.as_ref());

    PlatformCapabilities {
        platform,
        version,
        linux_distro,
        features,
    }
}

/// Get platform info
fn get_platform_info() -> (String, String) {
    #[cfg(target_os = "macos")]
    {
        let version = std::process::Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "Unknown".to_string());
        ("macOS".to_string(), version)
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let version = Command::new("cmd")
            .args(["/C", "ver"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "Unknown".to_string());
        ("Windows".to_string(), version)
    }

    #[cfg(target_os = "linux")]
    {
        let version = std::fs::read_to_string("/proc/version")
            .unwrap_or_else(|_| "Unknown".to_string())
            .lines()
            .next()
            .unwrap_or("Unknown")
            .to_string();
        ("Linux".to_string(), version)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        ("Unknown".to_string(), "Unknown".to_string())
    }
}

/// Get feature capabilities for the current platform
fn get_feature_capabilities(platform: &str, linux_distro: Option<&LinuxDistro>) -> Vec<FeatureCapability> {
    let mut features = Vec::new();

    // Secure delete
    features.push(FeatureCapability::available("secure_delete"));

    // System log cleanup
    features.push(FeatureCapability::available("log_cleanup"));

    // Memory cleanup
    features.push(FeatureCapability::available("memory_cleanup"));

    // Network cleanup
    features.push(FeatureCapability::available("network_cleanup"));

    // Registry cleanup (Windows only)
    if platform == "Windows" {
        features.push(FeatureCapability::available("registry_cleanup"));
    } else {
        features.push(
            FeatureCapability::unavailable(
                "registry_cleanup",
                "Registry is a Windows-specific feature",
            )
            .with_alternative("Use privacy trace cleanup instead"),
        );
    }

    // File creation time modification
    if platform == "Linux" {
        features.push(
            FeatureCapability::unavailable(
                "creation_time_modify",
                "Linux ext4 filesystem does not support creation time modification",
            )
            .with_alternative("Modify access and modification times instead"),
        );
    } else {
        features.push(FeatureCapability::available("creation_time_modify"));
    }

    // Full disk access (macOS only)
    if platform == "macOS" {
        features.push(FeatureCapability::available("full_disk_access"));
    }

    // SYSTEM/TrustedInstaller execution (Windows only)
    if platform == "Windows" {
        features.push(FeatureCapability::available("system_execution"));
        features.push(FeatureCapability::available("trustedinstaller_execution"));
    }

    // Journal cleanup (Linux only)
    if platform == "Linux" {
        if let Some(distro) = linux_distro {
            let has_journald = matches!(
                distro.package_manager,
                PackageManager::Apt | PackageManager::Dnf | PackageManager::Pacman | PackageManager::Zypper
            );
            if has_journald {
                features.push(FeatureCapability::available("journald_cleanup"));
            }
        }
    }

    // Package cache cleanup
    if platform == "Linux" {
        if let Some(distro) = linux_distro {
            let cache_feature = match distro.package_manager {
                PackageManager::Apt => FeatureCapability::available("apt_cache_cleanup"),
                PackageManager::Dnf => FeatureCapability::available("dnf_cache_cleanup"),
                PackageManager::Pacman => FeatureCapability::available("pacman_cache_cleanup"),
                _ => FeatureCapability::unavailable(
                    "package_cache_cleanup",
                    "Unsupported package manager",
                ),
            };
            features.push(cache_feature);
        }
    }

    features
}

/// Check if a feature is available
pub fn is_feature_available(feature_name: &str) -> bool {
    let caps = get_platform_capabilities();
    caps.features
        .iter()
        .find(|f| f.name == feature_name)
        .map(|f| f.available)
        .unwrap_or(false)
}

/// Get feature unavailability reason
pub fn get_feature_unavailable_reason(feature_name: &str) -> Option<String> {
    let caps = get_platform_capabilities();
    caps.features
        .iter()
        .find(|f| f.name == feature_name && !f.available)
        .and_then(|f| f.reason.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_error_creation() {
        let error = OperationError::new(
            ErrorCode::NotFound,
            "File not found",
            "secure_delete",
        );
        assert_eq!(error.code, ErrorCode::NotFound);
        assert!(!error.recoverable);
    }

    #[test]
    fn test_operation_error_with_context() {
        let error = OperationError::new(
            ErrorCode::IoError,
            "Read failed",
            "file_read",
        )
        .with_context("/path/to/file");

        assert!(error.context.is_some());
        assert_eq!(error.context.unwrap(), "/path/to/file");
    }

    #[test]
    fn test_operation_log_lifecycle() {
        let mut log = OperationLog::new(OperationType::SecureDelete, "/tmp/test");
        assert_eq!(log.status, OperationStatus::Pending);

        log.start();
        assert_eq!(log.status, OperationStatus::Running);

        log.complete();
        assert_eq!(log.status, OperationStatus::Completed);
        assert!(log.ended_at.is_some());
    }

    #[test]
    fn test_feature_capability() {
        let available = FeatureCapability::available("test_feature");
        assert!(available.available);

        let unavailable = FeatureCapability::unavailable("missing", "Not supported")
            .with_alternative("Use alternative");
        assert!(!unavailable.available);
        assert!(unavailable.alternative.is_some());
    }
}
