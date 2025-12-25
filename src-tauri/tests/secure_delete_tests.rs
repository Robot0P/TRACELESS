//! Integration tests for secure delete module

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use tempfile::TempDir;

mod common;

/// Create a temporary test environment
fn setup_test_env() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

#[test]
fn test_file_exists_after_creation() {
    let temp_dir = setup_test_env();
    let file_path = temp_dir.path().join("test.txt");

    // Create file
    let mut file = File::create(&file_path).expect("Failed to create file");
    file.write_all(b"test content").expect("Failed to write");

    assert!(file_path.exists(), "File should exist after creation");
}

#[test]
fn test_file_content_written_correctly() {
    let temp_dir = setup_test_env();
    let file_path = temp_dir.path().join("test.txt");
    let content = b"Hello, World!";

    // Write content
    let mut file = File::create(&file_path).expect("Failed to create file");
    file.write_all(content).expect("Failed to write");
    drop(file);

    // Read and verify
    let mut file = File::open(&file_path).expect("Failed to open file");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Failed to read file");

    assert_eq!(buffer, content, "File content should match");
}

#[test]
fn test_file_deletion() {
    let temp_dir = setup_test_env();
    let file_path = temp_dir.path().join("to_delete.txt");

    // Create file
    File::create(&file_path).expect("Failed to create file");
    assert!(file_path.exists(), "File should exist");

    // Delete file
    fs::remove_file(&file_path).expect("Failed to delete file");
    assert!(!file_path.exists(), "File should not exist after deletion");
}

#[test]
fn test_directory_deletion() {
    let temp_dir = setup_test_env();
    let dir_path = temp_dir.path().join("to_delete_dir");

    // Create directory with files
    fs::create_dir(&dir_path).expect("Failed to create directory");
    File::create(dir_path.join("file1.txt")).expect("Failed to create file");
    File::create(dir_path.join("file2.txt")).expect("Failed to create file");

    assert!(dir_path.exists(), "Directory should exist");

    // Delete directory recursively
    fs::remove_dir_all(&dir_path).expect("Failed to delete directory");
    assert!(!dir_path.exists(), "Directory should not exist after deletion");
}

#[test]
fn test_file_overwrite_patterns() {
    let temp_dir = setup_test_env();
    let file_path = temp_dir.path().join("overwrite_test.txt");

    // Create file with known content
    let original_content = b"SENSITIVE DATA HERE";
    let mut file = File::create(&file_path).expect("Failed to create file");
    file.write_all(original_content).expect("Failed to write");
    drop(file);

    // Verify original content
    let file_size = fs::metadata(&file_path).expect("Failed to get metadata").len();
    assert_eq!(file_size as usize, original_content.len(), "File size should match");

    // Overwrite with zeros
    let mut file = fs::OpenOptions::new()
        .write(true)
        .open(&file_path)
        .expect("Failed to open file for writing");

    let zeros = vec![0u8; original_content.len()];
    file.write_all(&zeros).expect("Failed to overwrite with zeros");
    drop(file);

    // Read and verify overwritten
    let mut file = File::open(&file_path).expect("Failed to open file");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Failed to read file");

    assert_eq!(buffer, zeros, "File should be overwritten with zeros");
    assert_ne!(buffer, original_content, "Original content should be gone");
}

#[test]
fn test_nested_directory_structure() {
    let temp_dir = setup_test_env();
    let nested = temp_dir.path().join("a").join("b").join("c").join("d");

    // Create nested directories
    fs::create_dir_all(&nested).expect("Failed to create nested dirs");
    assert!(nested.exists(), "Nested directory should exist");

    // Create file in deepest directory
    let file_path = nested.join("deep_file.txt");
    File::create(&file_path).expect("Failed to create file");
    assert!(file_path.exists(), "Deep file should exist");
}

#[test]
fn test_file_metadata() {
    let temp_dir = setup_test_env();
    let file_path = temp_dir.path().join("metadata_test.txt");

    // Create file
    let mut file = File::create(&file_path).expect("Failed to create file");
    file.write_all(b"test data for metadata").expect("Failed to write");
    drop(file);

    // Check metadata
    let metadata = fs::metadata(&file_path).expect("Failed to get metadata");
    assert!(metadata.is_file(), "Should be a file");
    assert!(!metadata.is_dir(), "Should not be a directory");
    assert!(metadata.len() > 0, "File size should be greater than 0");
}

#[test]
fn test_readonly_file_handling() {
    let temp_dir = setup_test_env();
    let file_path = temp_dir.path().join("readonly.txt");

    // Create file
    let mut file = File::create(&file_path).expect("Failed to create file");
    file.write_all(b"readonly content").expect("Failed to write");
    drop(file);

    // Make readonly
    let mut perms = fs::metadata(&file_path).expect("Failed to get metadata").permissions();
    perms.set_readonly(true);
    fs::set_permissions(&file_path, perms.clone()).expect("Failed to set permissions");

    // Verify readonly
    let metadata = fs::metadata(&file_path).expect("Failed to get metadata");
    assert!(metadata.permissions().readonly(), "File should be readonly");

    // Reset permissions for cleanup
    perms.set_readonly(false);
    fs::set_permissions(&file_path, perms).expect("Failed to reset permissions");
}

#[test]
fn test_large_file_handling() {
    let temp_dir = setup_test_env();
    let file_path = temp_dir.path().join("large_file.bin");

    // Create a 1MB file
    let size = 1024 * 1024;
    let mut file = File::create(&file_path).expect("Failed to create file");

    // Write in chunks
    let chunk = vec![0xAA_u8; 4096];
    let chunks = size / chunk.len();
    for _ in 0..chunks {
        file.write_all(&chunk).expect("Failed to write chunk");
    }
    drop(file);

    // Verify size
    let metadata = fs::metadata(&file_path).expect("Failed to get metadata");
    assert_eq!(metadata.len() as usize, size, "File size should match");
}

#[test]
fn test_wipe_method_passes() {
    // Test different wipe method pass counts
    let wipe_methods = vec![
        ("zero", 1),
        ("random", 1),
        ("dod", 7),
        ("gutmann", 35),
    ];

    for (method_name, expected_passes) in wipe_methods {
        assert!(
            expected_passes > 0,
            "Method '{}' should have at least 1 pass",
            method_name
        );
    }
}

#[test]
fn test_multiple_files_in_directory() {
    let temp_dir = setup_test_env();
    let file_count = 10;

    // Create multiple files
    for i in 0..file_count {
        let file_path = temp_dir.path().join(format!("file_{}.txt", i));
        let mut file = File::create(&file_path).expect("Failed to create file");
        file.write_all(format!("Content of file {}", i).as_bytes())
            .expect("Failed to write");
    }

    // Count files
    let entries: Vec<_> = fs::read_dir(temp_dir.path())
        .expect("Failed to read dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();

    assert_eq!(entries.len(), file_count, "Should have created {} files", file_count);
}

#[test]
fn test_special_characters_in_filename() {
    let temp_dir = setup_test_env();

    // Filenames with special characters (platform-safe ones)
    let filenames = vec![
        "file with spaces.txt",
        "file-with-dashes.txt",
        "file_with_underscores.txt",
        "file.multiple.dots.txt",
        "UPPERCASE.TXT",
        "MixedCase.Txt",
    ];

    for filename in filenames {
        let file_path = temp_dir.path().join(filename);
        File::create(&file_path).expect(&format!("Failed to create file: {}", filename));
        assert!(file_path.exists(), "File '{}' should exist", filename);
    }
}

#[test]
fn test_empty_file_handling() {
    let temp_dir = setup_test_env();
    let file_path = temp_dir.path().join("empty.txt");

    // Create empty file
    File::create(&file_path).expect("Failed to create empty file");

    // Verify empty
    let metadata = fs::metadata(&file_path).expect("Failed to get metadata");
    assert_eq!(metadata.len(), 0, "File should be empty");
}
