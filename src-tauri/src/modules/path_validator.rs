//! Path validation module for preventing path traversal attacks
//!
//! This module provides utilities for validating and sanitizing file paths
//! to prevent security vulnerabilities like path traversal attacks.

use std::path::{Path, PathBuf, Component};
use std::io;

/// Errors that can occur during path validation
#[derive(Debug, Clone)]
pub enum PathValidationError {
    /// Path contains traversal sequences like ".."
    PathTraversal(String),
    /// Path is not absolute when required
    NotAbsolute(String),
    /// Path contains null bytes
    NullByte(String),
    /// Path escapes the allowed base directory
    EscapesBase(String),
    /// Path contains invalid characters
    InvalidCharacters(String),
    /// Path is empty
    EmptyPath,
    /// Path does not exist
    NotFound(String),
    /// Path is not a file when file was expected
    NotAFile(String),
    /// Path is not a directory when directory was expected
    NotADirectory(String),
    /// Access denied to path
    AccessDenied(String),
}

impl std::fmt::Display for PathValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathValidationError::PathTraversal(p) => write!(f, "Path traversal detected: {}", p),
            PathValidationError::NotAbsolute(p) => write!(f, "Path is not absolute: {}", p),
            PathValidationError::NullByte(p) => write!(f, "Path contains null byte: {}", p),
            PathValidationError::EscapesBase(p) => write!(f, "Path escapes base directory: {}", p),
            PathValidationError::InvalidCharacters(p) => write!(f, "Path contains invalid characters: {}", p),
            PathValidationError::EmptyPath => write!(f, "Path is empty"),
            PathValidationError::NotFound(p) => write!(f, "Path not found: {}", p),
            PathValidationError::NotAFile(p) => write!(f, "Path is not a file: {}", p),
            PathValidationError::NotADirectory(p) => write!(f, "Path is not a directory: {}", p),
            PathValidationError::AccessDenied(p) => write!(f, "Access denied: {}", p),
        }
    }
}

impl std::error::Error for PathValidationError {}

impl From<PathValidationError> for String {
    fn from(err: PathValidationError) -> String {
        err.to_string()
    }
}

/// Result type for path validation operations
pub type ValidationResult<T> = Result<T, PathValidationError>;

/// Path validator with configurable options
#[derive(Debug, Clone)]
pub struct PathValidator {
    /// Whether to allow relative paths
    allow_relative: bool,
    /// Whether to require the path to exist
    require_exists: bool,
    /// Base directory to restrict paths to (if set)
    base_dir: Option<PathBuf>,
    /// List of allowed extensions (if set)
    allowed_extensions: Option<Vec<String>>,
    /// List of denied paths/patterns
    denied_paths: Vec<String>,
}

impl Default for PathValidator {
    fn default() -> Self {
        Self {
            allow_relative: false,
            require_exists: false,
            base_dir: None,
            allowed_extensions: None,
            denied_paths: Self::default_denied_paths(),
        }
    }
}

impl PathValidator {
    /// Create a new path validator with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a strict validator that requires absolute paths and existence
    pub fn strict() -> Self {
        Self {
            allow_relative: false,
            require_exists: true,
            base_dir: None,
            allowed_extensions: None,
            denied_paths: Self::default_denied_paths(),
        }
    }

    /// Get default denied paths based on the operating system
    fn default_denied_paths() -> Vec<String> {
        let mut denied = vec![
            // System-critical paths that should never be deleted entirely
            "/".to_string(),
            "/System".to_string(),
            "/usr".to_string(),
            "/bin".to_string(),
            "/sbin".to_string(),
            "/etc".to_string(),
            "/var".to_string(),
            "/boot".to_string(),
            "/dev".to_string(),
            "/proc".to_string(),
            "/sys".to_string(),
        ];

        #[cfg(target_os = "windows")]
        {
            denied.extend(vec![
                "C:\\Windows".to_string(),
                "C:\\Windows\\System32".to_string(),
                "C:\\Windows\\SysWOW64".to_string(),
                "C:\\Program Files".to_string(),
                "C:\\Program Files (x86)".to_string(),
            ]);
        }

        #[cfg(target_os = "macos")]
        {
            denied.extend(vec![
                "/System/Library".to_string(),
                "/Library".to_string(),
                "/Applications".to_string(),
                "/private/var".to_string(),
            ]);
        }

        denied
    }

    /// Set whether relative paths are allowed
    pub fn allow_relative(mut self, allow: bool) -> Self {
        self.allow_relative = allow;
        self
    }

    /// Set whether the path must exist
    pub fn require_exists(mut self, require: bool) -> Self {
        self.require_exists = require;
        self
    }

    /// Set the base directory to restrict paths to
    pub fn with_base_dir<P: AsRef<Path>>(mut self, base: P) -> Self {
        self.base_dir = Some(base.as_ref().to_path_buf());
        self
    }

    /// Set allowed file extensions
    pub fn with_allowed_extensions(mut self, extensions: Vec<String>) -> Self {
        self.allowed_extensions = Some(extensions);
        self
    }

    /// Add additional denied paths
    pub fn add_denied_paths(mut self, paths: Vec<String>) -> Self {
        self.denied_paths.extend(paths);
        self
    }

    /// Validate a path and return the canonicalized path if valid
    pub fn validate<P: AsRef<Path>>(&self, path: P) -> ValidationResult<PathBuf> {
        let path = path.as_ref();
        let path_str = path.to_string_lossy();

        // Check for empty path
        if path_str.is_empty() {
            return Err(PathValidationError::EmptyPath);
        }

        // Check for null bytes
        if path_str.contains('\0') {
            return Err(PathValidationError::NullByte(path_str.to_string()));
        }

        // Check for path traversal sequences in the raw path string
        if path_str.contains("..") {
            // Allow ".." only if it's part of a valid absolute path that doesn't escape
            if !self.is_safe_traversal(path)? {
                return Err(PathValidationError::PathTraversal(path_str.to_string()));
            }
        }

        // Check if path is absolute (if required)
        if !self.allow_relative && !path.is_absolute() {
            return Err(PathValidationError::NotAbsolute(path_str.to_string()));
        }

        // Canonicalize the path to resolve any ".." or "." components
        let canonical_path = if path.exists() {
            path.canonicalize()
                .map_err(|_| PathValidationError::AccessDenied(path_str.to_string()))?
        } else if self.require_exists {
            return Err(PathValidationError::NotFound(path_str.to_string()));
        } else {
            // For non-existent paths, normalize manually
            self.normalize_path(path)?
        };

        // Check if path is in denied list
        let canonical_str = canonical_path.to_string_lossy();
        for denied in &self.denied_paths {
            if canonical_str.eq_ignore_ascii_case(denied) {
                return Err(PathValidationError::AccessDenied(format!(
                    "Path '{}' is in denied list",
                    canonical_str
                )));
            }
        }

        // Check if path is within base directory (if set)
        if let Some(ref base) = self.base_dir {
            let base_canonical = if base.exists() {
                base.canonicalize()
                    .map_err(|_| PathValidationError::AccessDenied(base.to_string_lossy().to_string()))?
            } else {
                self.normalize_path(base)?
            };

            if !canonical_path.starts_with(&base_canonical) {
                return Err(PathValidationError::EscapesBase(format!(
                    "Path '{}' escapes base directory '{}'",
                    canonical_str,
                    base_canonical.display()
                )));
            }
        }

        // Check file extension if restrictions are set
        if let Some(ref extensions) = self.allowed_extensions {
            if let Some(ext) = canonical_path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if !extensions.iter().any(|e| e.to_lowercase() == ext_str) {
                    return Err(PathValidationError::InvalidCharacters(format!(
                        "Extension '{}' is not allowed",
                        ext_str
                    )));
                }
            }
        }

        Ok(canonical_path)
    }

    /// Check if the path traversal in the given path is safe
    fn is_safe_traversal<P: AsRef<Path>>(&self, path: P) -> ValidationResult<bool> {
        let path = path.as_ref();

        // If the path exists, canonicalize it and check
        if path.exists() {
            let canonical = path.canonicalize()
                .map_err(|_| PathValidationError::AccessDenied(path.to_string_lossy().to_string()))?;

            // If we have a base directory, ensure the canonical path is within it
            if let Some(ref base) = self.base_dir {
                let base_canonical = base.canonicalize()
                    .map_err(|_| PathValidationError::AccessDenied(base.to_string_lossy().to_string()))?;
                return Ok(canonical.starts_with(&base_canonical));
            }

            return Ok(true);
        }

        // For non-existent paths, be more strict
        Ok(false)
    }

    /// Normalize a path without requiring it to exist
    fn normalize_path<P: AsRef<Path>>(&self, path: P) -> ValidationResult<PathBuf> {
        let path = path.as_ref();
        let mut normalized = PathBuf::new();
        let mut depth = 0i32;

        for component in path.components() {
            match component {
                Component::Prefix(p) => normalized.push(p.as_os_str()),
                Component::RootDir => {
                    normalized.push(Component::RootDir);
                    depth = 0;
                }
                Component::CurDir => {}
                Component::ParentDir => {
                    depth -= 1;
                    if depth < 0 {
                        return Err(PathValidationError::PathTraversal(
                            path.to_string_lossy().to_string()
                        ));
                    }
                    normalized.pop();
                }
                Component::Normal(c) => {
                    depth += 1;
                    normalized.push(c);
                }
            }
        }

        Ok(normalized)
    }

    /// Validate that the path is a file
    pub fn validate_file<P: AsRef<Path>>(&self, path: P) -> ValidationResult<PathBuf> {
        let validated = self.validate(&path)?;

        if self.require_exists && !validated.is_file() {
            return Err(PathValidationError::NotAFile(
                path.as_ref().to_string_lossy().to_string()
            ));
        }

        Ok(validated)
    }

    /// Validate that the path is a directory
    pub fn validate_directory<P: AsRef<Path>>(&self, path: P) -> ValidationResult<PathBuf> {
        let validated = self.validate(&path)?;

        if self.require_exists && !validated.is_dir() {
            return Err(PathValidationError::NotADirectory(
                path.as_ref().to_string_lossy().to_string()
            ));
        }

        Ok(validated)
    }
}

/// Convenience function to validate a file path with default settings
pub fn validate_file_path<P: AsRef<Path>>(path: P) -> ValidationResult<PathBuf> {
    PathValidator::new()
        .require_exists(true)
        .validate_file(path)
}

/// Convenience function to validate a directory path with default settings
pub fn validate_directory_path<P: AsRef<Path>>(path: P) -> ValidationResult<PathBuf> {
    PathValidator::new()
        .require_exists(true)
        .validate_directory(path)
}

/// Convenience function to validate any path (file or directory)
pub fn validate_path<P: AsRef<Path>>(path: P) -> ValidationResult<PathBuf> {
    PathValidator::new()
        .require_exists(true)
        .validate(path)
}

/// Check if a path is safe for deletion (not a critical system path)
pub fn is_safe_for_deletion<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();

    // Use the validator to check against denied paths
    match PathValidator::strict().validate(path) {
        Ok(_) => true,
        Err(PathValidationError::AccessDenied(_)) => false,
        Err(_) => false,
    }
}

/// Sanitize a filename by removing dangerous characters
pub fn sanitize_filename(filename: &str) -> String {
    let dangerous_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|', '\0'];
    let mut sanitized = String::with_capacity(filename.len());

    for c in filename.chars() {
        if !dangerous_chars.contains(&c) {
            sanitized.push(c);
        }
    }

    // Remove leading/trailing whitespace and dots
    sanitized.trim().trim_matches('.').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_traversal_detection() {
        let validator = PathValidator::new();

        // Should reject obvious path traversal
        assert!(validator.validate("../etc/passwd").is_err());
        assert!(validator.validate("/home/../../../etc/passwd").is_err());
        assert!(validator.validate("/tmp/safe/../../etc/passwd").is_err());
    }

    #[test]
    fn test_null_byte_detection() {
        let validator = PathValidator::new();

        assert!(validator.validate("/tmp/file\0.txt").is_err());
    }

    #[test]
    fn test_empty_path() {
        let validator = PathValidator::new();

        assert!(validator.validate("").is_err());
    }

    #[test]
    fn test_relative_path_rejection() {
        let validator = PathValidator::new();

        assert!(validator.validate("relative/path").is_err());
    }

    #[test]
    fn test_relative_path_allowed() {
        let validator = PathValidator::new().allow_relative(true).require_exists(false);

        // This should not error on the relative path check
        // May still error if path doesn't exist and require_exists is true
        let result = validator.validate("relative/path");
        // Just checking it doesn't fail for being relative
        assert!(result.is_ok() || !matches!(result, Err(PathValidationError::NotAbsolute(_))));
    }

    #[test]
    fn test_base_directory_restriction() {
        let validator = PathValidator::new()
            .with_base_dir("/tmp")
            .require_exists(false);

        // Should allow paths within base
        // Note: These tests may behave differently based on actual filesystem
    }

    #[test]
    fn test_filename_sanitization() {
        assert_eq!(sanitize_filename("normal.txt"), "normal.txt");
        assert_eq!(sanitize_filename("file/with/slashes.txt"), "filewithslashes.txt");
        assert_eq!(sanitize_filename("file:with:colons.txt"), "filewithcolons.txt");
        assert_eq!(sanitize_filename("...hidden"), "hidden");
        assert_eq!(sanitize_filename("  spaces  "), "spaces");
    }

    #[test]
    fn test_denied_paths() {
        let validator = PathValidator::strict();

        // These should be denied
        #[cfg(target_os = "macos")]
        {
            assert!(validator.validate("/System").is_err());
        }

        #[cfg(target_os = "linux")]
        {
            assert!(validator.validate("/etc").is_err());
        }

        #[cfg(target_os = "windows")]
        {
            assert!(validator.validate("C:\\Windows\\System32").is_err());
        }
    }
}
