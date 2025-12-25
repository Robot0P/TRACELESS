//! Operation timeout and cancellation module
//!
//! Provides timeout handling and cancellation support for long-running operations.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};

/// Cancellation token for operations
#[derive(Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
    operation_id: u64,
}

impl CancellationToken {
    /// Create a new cancellation token
    pub fn new(operation_id: u64) -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
            operation_id,
        }
    }

    /// Check if the operation is cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Cancel the operation
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Get the operation ID
    pub fn operation_id(&self) -> u64 {
        self.operation_id
    }

    /// Check cancellation and return error if cancelled
    pub fn check(&self) -> Result<(), OperationCancelledError> {
        if self.is_cancelled() {
            Err(OperationCancelledError::new(self.operation_id))
        } else {
            Ok(())
        }
    }
}

/// Error when operation is cancelled
#[derive(Debug, Clone)]
pub struct OperationCancelledError {
    operation_id: u64,
}

impl OperationCancelledError {
    pub fn new(operation_id: u64) -> Self {
        Self { operation_id }
    }
}

impl std::fmt::Display for OperationCancelledError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Operation {} was cancelled", self.operation_id)
    }
}

impl std::error::Error for OperationCancelledError {}

/// Operation ID counter
static OPERATION_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Active operations registry
static ACTIVE_OPERATIONS: Lazy<Mutex<HashMap<u64, OperationHandle>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Operation handle for tracking and cancellation
#[derive(Clone)]
pub struct OperationHandle {
    pub id: u64,
    pub name: String,
    pub started_at: Instant,
    pub timeout: Option<Duration>,
    token: CancellationToken,
}

impl OperationHandle {
    /// Create a new operation handle
    pub fn new(name: impl Into<String>) -> Self {
        let id = OPERATION_COUNTER.fetch_add(1, Ordering::SeqCst);
        let token = CancellationToken::new(id);

        let handle = Self {
            id,
            name: name.into(),
            started_at: Instant::now(),
            timeout: None,
            token,
        };

        // Register the operation
        if let Ok(mut ops) = ACTIVE_OPERATIONS.lock() {
            ops.insert(id, handle.clone());
        }

        handle
    }

    /// Create with timeout
    pub fn with_timeout(name: impl Into<String>, timeout: Duration) -> Self {
        let mut handle = Self::new(name);
        handle.timeout = Some(timeout);
        handle
    }

    /// Get the cancellation token
    pub fn token(&self) -> CancellationToken {
        self.token.clone()
    }

    /// Check if operation has timed out
    pub fn is_timed_out(&self) -> bool {
        if let Some(timeout) = self.timeout {
            self.started_at.elapsed() > timeout
        } else {
            false
        }
    }

    /// Check if operation is cancelled
    pub fn is_cancelled(&self) -> bool {
        self.token.is_cancelled()
    }

    /// Check if operation should continue (not cancelled and not timed out)
    pub fn should_continue(&self) -> bool {
        !self.is_cancelled() && !self.is_timed_out()
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Complete the operation (remove from active operations)
    pub fn complete(self) {
        if let Ok(mut ops) = ACTIVE_OPERATIONS.lock() {
            ops.remove(&self.id);
        }
    }

    /// Cancel this operation
    pub fn cancel(&self) {
        self.token.cancel();
    }
}

impl Drop for OperationHandle {
    fn drop(&mut self) {
        // Clean up from active operations when handle is dropped
        if let Ok(mut ops) = ACTIVE_OPERATIONS.lock() {
            ops.remove(&self.id);
        }
    }
}

/// Cancel an operation by ID
pub fn cancel_operation(operation_id: u64) -> bool {
    if let Ok(ops) = ACTIVE_OPERATIONS.lock() {
        if let Some(handle) = ops.get(&operation_id) {
            handle.cancel();
            return true;
        }
    }
    false
}

/// Cancel all active operations
pub fn cancel_all_operations() {
    if let Ok(ops) = ACTIVE_OPERATIONS.lock() {
        for handle in ops.values() {
            handle.cancel();
        }
    }
}

/// Get list of active operations
pub fn get_active_operations() -> Vec<OperationInfo> {
    ACTIVE_OPERATIONS.lock().map(|ops| {
        ops.values()
            .map(|h| OperationInfo {
                id: h.id,
                name: h.name.clone(),
                elapsed_ms: h.elapsed().as_millis() as u64,
                timeout_ms: h.timeout.map(|t| t.as_millis() as u64),
                is_cancelled: h.is_cancelled(),
                is_timed_out: h.is_timed_out(),
            })
            .collect()
    }).unwrap_or_default()
}

/// Operation information for external use
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationInfo {
    pub id: u64,
    pub name: String,
    pub elapsed_ms: u64,
    pub timeout_ms: Option<u64>,
    pub is_cancelled: bool,
    pub is_timed_out: bool,
}

/// Timeout configuration for different operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Default timeout for file operations (in seconds)
    pub file_operation: u64,
    /// Default timeout for directory scan (in seconds)
    pub directory_scan: u64,
    /// Default timeout for system operations (in seconds)
    pub system_operation: u64,
    /// Default timeout for network operations (in seconds)
    pub network_operation: u64,
    /// Maximum operation timeout (in seconds)
    pub max_timeout: u64,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            file_operation: 300,      // 5 minutes
            directory_scan: 600,      // 10 minutes
            system_operation: 120,    // 2 minutes
            network_operation: 60,    // 1 minute
            max_timeout: 3600,        // 1 hour
        }
    }
}

/// Global timeout configuration
static TIMEOUT_CONFIG: Lazy<Mutex<TimeoutConfig>> = Lazy::new(|| Mutex::new(TimeoutConfig::default()));

/// Get timeout configuration
pub fn get_timeout_config() -> TimeoutConfig {
    TIMEOUT_CONFIG.lock().map(|c| c.clone()).unwrap_or_default()
}

/// Update timeout configuration
pub fn set_timeout_config(config: TimeoutConfig) {
    if let Ok(mut c) = TIMEOUT_CONFIG.lock() {
        *c = config;
    }
}

/// Progress reporter for long operations
pub struct ProgressReporter<F>
where
    F: Fn(ProgressUpdate) + Send + Sync,
{
    callback: F,
    total_items: u64,
    processed_items: AtomicU64,
    last_update: Mutex<Instant>,
    update_interval: Duration,
    operation_name: String,
}

impl<F> ProgressReporter<F>
where
    F: Fn(ProgressUpdate) + Send + Sync,
{
    /// Create a new progress reporter
    pub fn new(operation_name: impl Into<String>, total_items: u64, callback: F) -> Self {
        Self {
            callback,
            total_items,
            processed_items: AtomicU64::new(0),
            last_update: Mutex::new(Instant::now()),
            update_interval: Duration::from_millis(100), // Update at most 10 times per second
            operation_name: operation_name.into(),
        }
    }

    /// Increment processed items
    pub fn increment(&self) {
        self.add(1);
    }

    /// Add to processed items
    pub fn add(&self, count: u64) {
        let new_count = self.processed_items.fetch_add(count, Ordering::SeqCst) + count;
        self.maybe_report(new_count);
    }

    /// Set processed items directly
    pub fn set(&self, count: u64) {
        self.processed_items.store(count, Ordering::SeqCst);
        self.maybe_report(count);
    }

    /// Maybe report progress (if enough time has passed)
    fn maybe_report(&self, processed: u64) {
        let should_report = {
            if let Ok(mut last) = self.last_update.lock() {
                if last.elapsed() >= self.update_interval {
                    *last = Instant::now();
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };

        if should_report || processed >= self.total_items {
            let update = ProgressUpdate {
                operation: self.operation_name.clone(),
                processed,
                total: self.total_items,
                percentage: if self.total_items > 0 {
                    (processed as f64 / self.total_items as f64) * 100.0
                } else {
                    0.0
                },
            };
            (self.callback)(update);
        }
    }

    /// Force report current progress
    pub fn report(&self) {
        let processed = self.processed_items.load(Ordering::SeqCst);
        let update = ProgressUpdate {
            operation: self.operation_name.clone(),
            processed,
            total: self.total_items,
            percentage: if self.total_items > 0 {
                (processed as f64 / self.total_items as f64) * 100.0
            } else {
                0.0
            },
        };
        (self.callback)(update);
    }
}

/// Progress update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub operation: String,
    pub processed: u64,
    pub total: u64,
    pub percentage: f64,
}

/// Run an operation with timeout
pub async fn with_timeout<T, F, Fut>(
    timeout: Duration,
    operation_name: &str,
    operation: F,
) -> Result<T, TimeoutError>
where
    F: FnOnce(CancellationToken) -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let handle = OperationHandle::with_timeout(operation_name, timeout);
    let token = handle.token();

    let result = tokio::select! {
        result = operation(token.clone()) => {
            handle.complete();
            Ok(result)
        }
        _ = tokio::time::sleep(timeout) => {
            token.cancel();
            Err(TimeoutError::new(operation_name, timeout))
        }
    };

    result
}

/// Timeout error
#[derive(Debug, Clone)]
pub struct TimeoutError {
    operation: String,
    timeout: Duration,
}

impl TimeoutError {
    pub fn new(operation: impl Into<String>, timeout: Duration) -> Self {
        Self {
            operation: operation.into(),
            timeout,
        }
    }
}

impl std::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Operation '{}' timed out after {:?}",
            self.operation, self.timeout
        )
    }
}

impl std::error::Error for TimeoutError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cancellation_token() {
        let token = CancellationToken::new(1);
        assert!(!token.is_cancelled());

        token.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn test_operation_handle() {
        let handle = OperationHandle::new("test_operation");
        assert!(!handle.is_cancelled());
        assert!(handle.should_continue());

        handle.cancel();
        assert!(handle.is_cancelled());
        assert!(!handle.should_continue());
    }

    #[test]
    fn test_timeout_detection() {
        let handle = OperationHandle::with_timeout("fast_operation", Duration::from_millis(1));
        std::thread::sleep(Duration::from_millis(10));
        assert!(handle.is_timed_out());
    }

    #[test]
    fn test_progress_reporter() {
        use std::sync::atomic::AtomicU64;
        use std::sync::Arc;

        let updates = Arc::new(AtomicU64::new(0));
        let updates_clone = updates.clone();

        let reporter = ProgressReporter::new("test", 100, move |_update| {
            updates_clone.fetch_add(1, Ordering::SeqCst);
        });

        for _ in 0..100 {
            reporter.increment();
        }

        reporter.report();
        assert!(updates.load(Ordering::SeqCst) > 0);
    }
}
