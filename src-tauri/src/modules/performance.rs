//! Performance optimization module
//!
//! Provides parallel processing, chunked file operations, and optimized directory traversal.

use rayon::prelude::*;
use std::fs::{self, File};
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;
use walkdir::WalkDir;

use crate::modules::timeout_handler::CancellationToken;

/// Chunk size for large file operations (4MB)
const CHUNK_SIZE: usize = 4 * 1024 * 1024;

/// Minimum file size to use chunked processing (10MB)
const CHUNKED_THRESHOLD: u64 = 10 * 1024 * 1024;

/// Get the number of CPU cores
pub fn get_cpu_count() -> usize {
    num_cpus::get()
}

/// Get the number of physical CPU cores
pub fn get_physical_cpu_count() -> usize {
    num_cpus::get_physical()
}

/// Configure thread pool size for parallel operations
pub fn configure_thread_pool(threads: Option<usize>) {
    let num_threads = threads.unwrap_or_else(|| {
        // Use physical cores for I/O-bound operations
        let physical = get_physical_cpu_count();
        // Leave at least one core free for system responsiveness
        (physical.saturating_sub(1)).max(1)
    });

    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .ok();
}

/// File deletion progress
#[derive(Debug)]
pub struct DeletionProgress {
    pub total_files: u64,
    pub processed_files: AtomicU64,
    pub total_bytes: u64,
    pub processed_bytes: AtomicU64,
    pub current_file: std::sync::Mutex<String>,
    pub errors: std::sync::Mutex<Vec<String>>,
}

impl DeletionProgress {
    pub fn new(total_files: u64, total_bytes: u64) -> Arc<Self> {
        Arc::new(Self {
            total_files,
            processed_files: AtomicU64::new(0),
            total_bytes,
            processed_bytes: AtomicU64::new(0),
            current_file: std::sync::Mutex::new(String::new()),
            errors: std::sync::Mutex::new(Vec::new()),
        })
    }

    pub fn increment_files(&self) {
        self.processed_files.fetch_add(1, Ordering::SeqCst);
    }

    pub fn add_bytes(&self, bytes: u64) {
        self.processed_bytes.fetch_add(bytes, Ordering::SeqCst);
    }

    pub fn set_current_file(&self, file: &str) {
        if let Ok(mut current) = self.current_file.lock() {
            *current = file.to_string();
        }
    }

    pub fn add_error(&self, error: String) {
        if let Ok(mut errors) = self.errors.lock() {
            errors.push(error);
        }
    }

    pub fn get_progress(&self) -> (u64, u64, f64) {
        let processed = self.processed_files.load(Ordering::SeqCst);
        let percentage = if self.total_files > 0 {
            (processed as f64 / self.total_files as f64) * 100.0
        } else {
            0.0
        };
        (processed, self.total_files, percentage)
    }

    pub fn get_bytes_progress(&self) -> (u64, u64, f64) {
        let processed = self.processed_bytes.load(Ordering::SeqCst);
        let percentage = if self.total_bytes > 0 {
            (processed as f64 / self.total_bytes as f64) * 100.0
        } else {
            0.0
        };
        (processed, self.total_bytes, percentage)
    }

    pub fn get_errors(&self) -> Vec<String> {
        self.errors.lock().map(|e| e.clone()).unwrap_or_default()
    }
}

/// Scan directory and collect file information in parallel
pub fn scan_directory_parallel(
    path: &Path,
    max_depth: Option<usize>,
    cancel_token: Option<&CancellationToken>,
) -> Result<Vec<FileInfo>, String> {
    let mut walker = WalkDir::new(path).follow_links(false);

    if let Some(depth) = max_depth {
        walker = walker.max_depth(depth);
    }

    let entries: Vec<_> = walker
        .into_iter()
        .filter_entry(|e| {
            // Skip hidden files starting with '.'
            !e.file_name()
                .to_str()
                .map(|s| s.starts_with('.'))
                .unwrap_or(false)
        })
        .filter_map(|e| e.ok())
        .filter(|e| {
            // Check cancellation
            if let Some(token) = cancel_token {
                !token.is_cancelled()
            } else {
                true
            }
        })
        .collect();

    // Check final cancellation
    if let Some(token) = cancel_token {
        if token.is_cancelled() {
            return Err("Operation cancelled".to_string());
        }
    }

    // Process entries in parallel
    let files: Vec<FileInfo> = entries
        .par_iter()
        .filter_map(|entry| {
            let path = entry.path();
            let metadata = entry.metadata().ok()?;

            Some(FileInfo {
                path: path.to_path_buf(),
                size: metadata.len(),
                is_dir: metadata.is_dir(),
                modified: metadata.modified().ok(),
            })
        })
        .collect();

    Ok(files)
}

/// File information
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
    pub modified: Option<std::time::SystemTime>,
}

/// Securely wipe a large file using chunked processing
pub fn wipe_file_chunked(
    path: &Path,
    pattern: &[u8],
    passes: u32,
    progress: Option<Arc<DeletionProgress>>,
    cancel_token: Option<&CancellationToken>,
) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .open(path)
        .map_err(|e| format!("Failed to open file: {}", e))?;

    let file_size = file
        .metadata()
        .map_err(|e| format!("Failed to get file metadata: {}", e))?
        .len();

    // Create chunk buffer
    let chunk_size = CHUNK_SIZE.min(file_size as usize);
    let mut chunk = vec![0u8; chunk_size];

    for pass in 0..passes {
        // Check cancellation
        if let Some(token) = cancel_token {
            if token.is_cancelled() {
                return Err("Operation cancelled".to_string());
            }
        }

        // Seek to beginning
        file.seek(SeekFrom::Start(0))
            .map_err(|e| format!("Failed to seek: {}", e))?;

        let mut written: u64 = 0;

        while written < file_size {
            // Check cancellation
            if let Some(token) = cancel_token {
                if token.is_cancelled() {
                    return Err("Operation cancelled".to_string());
                }
            }

            let remaining = (file_size - written) as usize;
            let write_size = chunk_size.min(remaining);

            // Fill chunk with pattern
            for i in 0..write_size {
                chunk[i] = pattern[i % pattern.len()];
            }

            file.write_all(&chunk[..write_size])
                .map_err(|e| format!("Failed to write chunk: {}", e))?;

            written += write_size as u64;

            // Update progress
            if let Some(ref prog) = progress {
                prog.add_bytes(write_size as u64 / passes as u64);
            }
        }

        // Sync to disk
        file.sync_all()
            .map_err(|e| format!("Failed to sync: {}", e))?;
    }

    Ok(())
}

/// Delete files in parallel with progress reporting
pub fn delete_files_parallel(
    files: Vec<PathBuf>,
    wipe_method: WipeMethod,
    progress: Arc<DeletionProgress>,
    cancel_token: Option<CancellationToken>,
) -> Result<u64, String> {
    let cancelled = Arc::new(AtomicBool::new(false));
    let cancelled_clone = cancelled.clone();

    // Set up cancellation check
    if let Some(ref token) = cancel_token {
        let token_clone = token.clone();
        std::thread::spawn(move || {
            while !cancelled_clone.load(Ordering::SeqCst) {
                if token_clone.is_cancelled() {
                    cancelled_clone.store(true, Ordering::SeqCst);
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        });
    }

    let success_count = AtomicU64::new(0);
    let success_count_ref = &success_count;

    // Process files in parallel
    files.par_iter().for_each(|path| {
        // Check cancellation
        if cancelled.load(Ordering::SeqCst) {
            return;
        }

        progress.set_current_file(&path.to_string_lossy());

        // Get file size for progress
        let file_size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);

        // Perform secure deletion
        let result = if file_size > CHUNKED_THRESHOLD {
            // Use chunked processing for large files
            let pattern = wipe_method.get_pattern();
            wipe_file_chunked(path, &pattern, wipe_method.passes(), Some(progress.clone()), None)
                .and_then(|_| fs::remove_file(path).map_err(|e| e.to_string()))
        } else {
            // Standard deletion for smaller files
            secure_delete_file(path, &wipe_method)
        };

        match result {
            Ok(_) => {
                success_count_ref.fetch_add(1, Ordering::SeqCst);
                progress.increment_files();
                progress.add_bytes(file_size);
            }
            Err(e) => {
                progress.add_error(format!("{}: {}", path.display(), e));
            }
        }
    });

    // Mark cancellation complete
    cancelled.store(true, Ordering::SeqCst);

    Ok(success_count.load(Ordering::SeqCst))
}

/// Secure delete a single file
fn secure_delete_file(path: &Path, method: &WipeMethod) -> Result<(), String> {
    let metadata = fs::metadata(path).map_err(|e| format!("Failed to get metadata: {}", e))?;

    if metadata.is_dir() {
        return Err("Path is a directory".to_string());
    }

    let file_size = metadata.len();

    // Open file for writing
    let mut file = fs::OpenOptions::new()
        .write(true)
        .open(path)
        .map_err(|e| format!("Failed to open file: {}", e))?;

    // Wipe with pattern
    let pattern = method.get_pattern();
    let mut buffer = vec![0u8; 4096];

    for _ in 0..method.passes() {
        file.seek(SeekFrom::Start(0))
            .map_err(|e| format!("Failed to seek: {}", e))?;

        let mut written: u64 = 0;
        while written < file_size {
            let write_size = buffer.len().min((file_size - written) as usize);

            // Fill buffer with pattern
            for i in 0..write_size {
                buffer[i] = pattern[i % pattern.len()];
            }

            file.write_all(&buffer[..write_size])
                .map_err(|e| format!("Failed to write: {}", e))?;

            written += write_size as u64;
        }

        file.sync_all()
            .map_err(|e| format!("Failed to sync: {}", e))?;
    }

    drop(file);

    // Remove the file
    fs::remove_file(path).map_err(|e| format!("Failed to remove file: {}", e))?;

    Ok(())
}

/// Wipe method configuration
#[derive(Debug, Clone, Copy)]
pub enum WipeMethod {
    Zero,
    Random,
    DoD,
    Gutmann,
}

impl WipeMethod {
    pub fn passes(&self) -> u32 {
        match self {
            WipeMethod::Zero => 1,
            WipeMethod::Random => 1,
            WipeMethod::DoD => 7,
            WipeMethod::Gutmann => 35,
        }
    }

    pub fn get_pattern(&self) -> Vec<u8> {
        match self {
            WipeMethod::Zero => vec![0x00],
            WipeMethod::Random => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                (0..256).map(|_| rng.gen()).collect()
            }
            WipeMethod::DoD => vec![0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00],
            WipeMethod::Gutmann => {
                // Gutmann patterns (simplified)
                vec![0x55, 0xAA, 0x92, 0x49, 0x24, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
                     0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]
            }
        }
    }
}

/// Recursively delete a directory with parallel processing
pub fn delete_directory_parallel(
    path: &Path,
    wipe_method: WipeMethod,
    progress: Arc<DeletionProgress>,
    cancel_token: Option<CancellationToken>,
) -> Result<u64, String> {
    // First, collect all files
    let files: Vec<PathBuf> = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();

    // Delete files in parallel
    let deleted = delete_files_parallel(files, wipe_method, progress, cancel_token)?;

    // Now remove empty directories (bottom-up)
    let mut dirs: Vec<PathBuf> = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .map(|e| e.path().to_path_buf())
        .collect();

    // Sort by depth (deepest first)
    dirs.sort_by(|a, b| b.components().count().cmp(&a.components().count()));

    for dir in dirs {
        let _ = fs::remove_dir(&dir);
    }

    Ok(deleted)
}

/// Batch file operation result
#[derive(Debug)]
pub struct BatchResult {
    pub total: u64,
    pub success: u64,
    pub failed: u64,
    pub errors: Vec<String>,
    pub duration_ms: u64,
}

/// Execute a batch file operation with parallel processing
pub fn batch_operation<F>(
    files: Vec<PathBuf>,
    operation: F,
    max_threads: Option<usize>,
) -> BatchResult
where
    F: Fn(&Path) -> Result<(), String> + Sync + Send,
{
    let start = std::time::Instant::now();

    // Configure thread pool if specified
    if let Some(threads) = max_threads {
        configure_thread_pool(Some(threads));
    }

    let total = files.len() as u64;
    let success = AtomicU64::new(0);
    let errors: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

    files.par_iter().for_each(|path| {
        match operation(path) {
            Ok(_) => {
                success.fetch_add(1, Ordering::SeqCst);
            }
            Err(e) => {
                if let Ok(mut errs) = errors.lock() {
                    errs.push(format!("{}: {}", path.display(), e));
                }
            }
        }
    });

    let success_count = success.load(Ordering::SeqCst);
    let error_list = errors.into_inner().unwrap_or_default();

    BatchResult {
        total,
        success: success_count,
        failed: total - success_count,
        errors: error_list,
        duration_ms: start.elapsed().as_millis() as u64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_get_cpu_count() {
        let count = get_cpu_count();
        assert!(count > 0);
    }

    #[test]
    fn test_wipe_method_patterns() {
        let zero = WipeMethod::Zero;
        assert_eq!(zero.passes(), 1);
        assert_eq!(zero.get_pattern(), vec![0x00]);

        let dod = WipeMethod::DoD;
        assert_eq!(dod.passes(), 7);
    }

    #[test]
    fn test_deletion_progress() {
        let progress = DeletionProgress::new(10, 1000);

        progress.increment_files();
        progress.add_bytes(100);

        let (processed, total, percentage) = progress.get_progress();
        assert_eq!(processed, 1);
        assert_eq!(total, 10);
        assert!((percentage - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_batch_result() {
        let result = BatchResult {
            total: 10,
            success: 8,
            failed: 2,
            errors: vec!["error1".to_string(), "error2".to_string()],
            duration_ms: 100,
        };

        assert_eq!(result.success + result.failed, result.total);
    }
}
