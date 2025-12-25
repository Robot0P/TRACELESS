//! Common test utilities and helpers

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

/// Create a temporary file with specified content
#[allow(dead_code)]
pub fn create_temp_file(dir: &std::path::Path, name: &str, content: &[u8]) -> PathBuf {
    let file_path = dir.join(name);
    let mut file = File::create(&file_path).expect("Failed to create temp file");
    file.write_all(content).expect("Failed to write to temp file");
    file_path
}

/// Create a nested directory structure
#[allow(dead_code)]
pub fn create_nested_dirs(base: &std::path::Path, levels: usize) -> PathBuf {
    let mut path = base.to_path_buf();
    for i in 0..levels {
        path = path.join(format!("level_{}", i));
    }
    fs::create_dir_all(&path).expect("Failed to create nested dirs");
    path
}

/// Create multiple test files in a directory
#[allow(dead_code)]
pub fn create_test_files(dir: &std::path::Path, count: usize) -> Vec<PathBuf> {
    let mut files = Vec::with_capacity(count);
    for i in 0..count {
        let file_path = create_temp_file(dir, &format!("test_file_{}.txt", i), b"test content");
        files.push(file_path);
    }
    files
}
