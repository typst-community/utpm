/// Common test utilities and helpers
use std::env;
use std::fs::{self, create_dir_all};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Creates a temporary directory for testing
#[allow(dead_code)]
pub fn setup_temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp dir")
}

/// Creates a test typst.toml manifest in the given directory
#[allow(dead_code)]
pub fn create_test_manifest(dir: &Path, name: &str, version: &str) -> PathBuf {
    // Ensure directory exists
    create_dir_all(dir).expect("Failed to create directory");

    let manifest_path = dir.join("typst.toml");
    let content = format!(
        r#"[package]
name = "{}"
version = "{}"
entrypoint = "main.typ"
authors = ["Test Author"]
license = "MIT"
description = "Test package"

[tool.utpm]
exclude = []
"#,
        name, version
    );
    fs::write(&manifest_path, content).expect("Failed to write test manifest");
    manifest_path
}

/// Creates a test typst.toml manifest with custom fields
#[allow(dead_code)]
pub fn create_custom_manifest(dir: &Path, content: &str) -> PathBuf {
    // Ensure directory exists
    create_dir_all(dir).expect("Failed to create directory");

    let manifest_path = dir.join("typst.toml");
    fs::write(&manifest_path, content).expect("Failed to write test manifest");
    manifest_path
}

/// Creates a test main.typ file
#[allow(dead_code)]
pub fn create_test_entrypoint(dir: &Path) -> PathBuf {
    let main_path = dir.join("main.typ");
    fs::write(&main_path, "// Test entrypoint\n#let hello = \"world\"")
        .expect("Failed to write test entrypoint");
    main_path
}

/// Creates a complete test package structure
#[allow(dead_code)]
pub fn create_test_package(dir: &Path, name: &str, version: &str) -> PathBuf {
    create_test_manifest(dir, name, version);
    create_test_entrypoint(dir);

    // Create src directory
    let src_dir = dir.join("src");
    create_dir_all(&src_dir).expect("Failed to create src dir");

    // Create examples directory
    let examples_dir = dir.join("examples");
    create_dir_all(&examples_dir).expect("Failed to create examples dir");
    fs::write(examples_dir.join("example.typ"), "// Example file")
        .expect("Failed to write example file");

    dir.to_path_buf()
}

/// Sets up environment variables for testing
/// Note: Uses unsafe block as env::set_var is unsafe in Rust 2024 edition
#[allow(dead_code)]
pub fn setup_test_env(temp_dir: &Path) {
    unsafe {
        env::set_var("UTPM_DATA_DIR", temp_dir.join("data"));
        env::set_var("UTPM_CACHE_DIR", temp_dir.join("cache"));
        env::set_var("UTPM_CURRENT_DIR", temp_dir.join("current"));
    }

    // Create the directories
    create_dir_all(temp_dir.join("data/typst/packages")).ok();
    create_dir_all(temp_dir.join("cache/typst/packages")).ok();
    create_dir_all(temp_dir.join("current")).ok();
}

/// Cleans up environment variables after testing
/// Note: Uses unsafe block as env::remove_var is unsafe in Rust 2024 edition
#[allow(dead_code)]
pub fn cleanup_test_env() {
    unsafe {
        env::remove_var("UTPM_DATA_DIR");
        env::remove_var("UTPM_CACHE_DIR");
        env::remove_var("UTPM_CURRENT_DIR");
        env::remove_var("UTPM_DEBUG");
    }
}

/// Reads a file to string, panicking on error
#[allow(dead_code)]
pub fn read_file_string(path: &Path) -> String {
    fs::read_to_string(path).expect("Failed to read file")
}

/// Asserts that a file exists
#[allow(dead_code)]
pub fn assert_file_exists(path: &Path) {
    assert!(path.exists(), "File should exist: {:?}", path);
    assert!(path.is_file(), "Path should be a file: {:?}", path);
}

/// Asserts that a directory exists
#[allow(dead_code)]
pub fn assert_dir_exists(path: &Path) {
    assert!(path.exists(), "Directory should exist: {:?}", path);
    assert!(path.is_dir(), "Path should be a directory: {:?}", path);
}

/// Asserts that a path does not exist
#[allow(dead_code)]
pub fn assert_not_exists(path: &Path) {
    assert!(!path.exists(), "Path should not exist: {:?}", path);
}
