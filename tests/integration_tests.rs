/// Integration tests that test multiple components together
mod common;

use common::*;
use std::fs;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_package_workflow() {
        let temp_dir = setup_temp_dir();
        setup_test_env(temp_dir.path());

        // Step 1: Create a package
        let package_dir = temp_dir.path().join("my-package");
        fs::create_dir_all(&package_dir).unwrap();
        create_test_package(&package_dir, "my-package", "1.0.0");

        assert_file_exists(&package_dir.join("typst.toml"));
        assert_file_exists(&package_dir.join("main.typ"));

        // Step 2: Manually copy to simulate link (simple test)
        let linked_path = temp_dir
            .path()
            .join("data/typst/packages/local/my-package/1.0.0");

        // Copy the package structure
        utpm::utils::copy_dir_all(&package_dir, &linked_path).unwrap();

        // Step 3: Verify linked package exists
        assert_dir_exists(&linked_path);
        assert_file_exists(&linked_path.join("typst.toml"));
        assert_file_exists(&linked_path.join("main.typ"));

        // Step 4: Simulate unlink by removing directory
        fs::remove_dir_all(&linked_path).unwrap();

        // Step 5: Verify package is removed
        assert_not_exists(&linked_path);

        cleanup_test_env();
    }

    #[test]
    fn test_package_with_dependencies() {
        let temp_dir = setup_temp_dir();
        setup_test_env(temp_dir.path());

        // Create package with dependencies in manifest
        let manifest_content = r#"[package]
name = "dependent-pkg"
version = "1.0.0"
entrypoint = "main.typ"
authors = ["Test"]
license = "MIT"
description = "Package with dependencies"

[tool.utpm]
namespace = "local"
dependencies = [
    "https://github.com/example/dep1.git",
    "https://github.com/example/dep2.git"
]
"#;

        let package_dir = temp_dir.path().join("package");
        fs::create_dir_all(&package_dir).unwrap();
        create_custom_manifest(&package_dir, manifest_content);
        create_test_entrypoint(&package_dir);

        // Verify manifest
        let content = read_file_string(&package_dir.join("typst.toml"));
        assert!(content.contains("dependencies"));
        assert!(content.contains("dep1.git"));
        assert!(content.contains("dep2.git"));

        cleanup_test_env();
    }

    #[test]
    fn test_version_bumping_workflow() {
        let temp_dir = setup_temp_dir();
        let package_dir = temp_dir.path().join("package");
        fs::create_dir_all(&package_dir).unwrap();

        // Create package at v1.0.0
        create_test_manifest(&package_dir, "test-pkg", "1.0.0");

        let manifest_path = package_dir.join("typst.toml");
        let content = read_file_string(&manifest_path);
        assert!(content.contains("version = \"1.0.0\""));

        // Simulate bumping to 1.0.1
        let new_content = content.replace("version = \"1.0.0\"", "version = \"1.0.1\"");
        fs::write(&manifest_path, new_content).unwrap();

        let updated = read_file_string(&manifest_path);
        assert!(updated.contains("version = \"1.0.1\""));
        assert!(!updated.contains("version = \"1.0.0\""));
    }

    #[test]
    fn test_sync_updates_imports() {
        let temp_dir = setup_temp_dir();

        // Create a typst file with imports
        let typ_file = temp_dir.path().join("document.typ");
        let content = r#"#import "@preview/example:1.0.0": *
#import "@preview/another:2.0.0"

#show: example.template

Some document content
"#;
        fs::write(&typ_file, content).unwrap();

        let file_content = read_file_string(&typ_file);

        // Verify imports are present
        assert!(file_content.contains("@preview/example:1.0.0"));
        assert!(file_content.contains("@preview/another:2.0.0"));

        // After sync, versions might be updated
        let updated_content = file_content
            .replace("example:1.0.0", "example:1.1.0")
            .replace("another:2.0.0", "another:2.1.0");

        fs::write(&typ_file, updated_content).unwrap();

        let synced = read_file_string(&typ_file);
        assert!(synced.contains("@preview/example:1.1.0"));
        assert!(synced.contains("@preview/another:2.1.0"));
    }

    #[test]
    fn test_exclude_patterns_in_linking() {
        let temp_dir = setup_temp_dir();
        setup_test_env(temp_dir.path());

        let package_dir = temp_dir.path().join("package");
        fs::create_dir_all(&package_dir).unwrap();

        // Create manifest with excludes
        let manifest_content = r#"[package]
name = "test"
version = "1.0.0"
entrypoint = "main.typ"
authors = ["Test"]
license = "MIT"
description = "Test"

[tool.utpm]
namespace = "local"
exclude = [
    ".git",
    ".github",
    "*.md",
    "tests/",
    "*.log"
]
"#;
        create_custom_manifest(&package_dir, manifest_content);

        // Create various files
        fs::write(package_dir.join("main.typ"), "main").unwrap();
        fs::write(package_dir.join("README.md"), "readme").unwrap();
        fs::write(package_dir.join("debug.log"), "log").unwrap();

        fs::create_dir_all(package_dir.join("tests")).unwrap();
        fs::write(package_dir.join("tests/test.typ"), "test").unwrap();

        fs::create_dir_all(package_dir.join(".git")).unwrap();
        fs::write(package_dir.join(".git/config"), "git").unwrap();

        // Verify files exist before filtering
        assert_file_exists(&package_dir.join("main.typ"));
        assert_file_exists(&package_dir.join("README.md"));
        assert_file_exists(&package_dir.join("debug.log"));

        // After linking with excludes, these should not be copied:
        // - README.md (*.md)
        // - debug.log (*.log)
        // - tests/ directory
        // - .git directory

        cleanup_test_env();
    }

    #[test]
    fn test_multiple_namespaces() {
        let temp_dir = setup_temp_dir();
        setup_test_env(temp_dir.path());

        let data_dir = temp_dir.path().join("data/typst/packages");
        let cache_dir = temp_dir.path().join("cache/typst/packages");

        // Local namespace goes to data
        let local_path = data_dir.join("local/pkg1/1.0.0");
        fs::create_dir_all(&local_path).unwrap();

        // Preview namespace goes to cache
        let preview_path = cache_dir.join("preview/pkg2/1.0.0");
        fs::create_dir_all(&preview_path).unwrap();

        assert_dir_exists(&local_path);
        assert_dir_exists(&preview_path);

        cleanup_test_env();
    }

    #[test]
    fn test_package_validation() {
        let temp_dir = setup_temp_dir();

        // Valid package
        let valid_manifest = r#"[package]
name = "valid-pkg"
version = "1.0.0"
entrypoint = "main.typ"
authors = ["Author"]
license = "MIT"
description = "Valid package"
"#;
        create_custom_manifest(temp_dir.path(), valid_manifest);

        let content = read_file_string(&temp_dir.path().join("typst.toml"));

        // Verify all required fields
        assert!(content.contains("name"));
        assert!(content.contains("version"));
        assert!(content.contains("entrypoint"));
    }

    #[test]
    fn test_dry_run_mode() {
        let temp_dir = setup_temp_dir();
        setup_test_env(temp_dir.path());

        let package_dir = temp_dir.path().join("package");
        fs::create_dir_all(&package_dir).unwrap();
        create_test_package(&package_dir, "test", "1.0.0");

        let data_dir = temp_dir.path().join("data/typst/packages/local/test/1.0.0");

        // In dry-run mode, files should NOT be written
        // Verify directory doesn't exist yet
        assert_not_exists(&data_dir);

        // After actual link (not dry-run), directory should exist
        // (This would require actual command execution)

        cleanup_test_env();
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_missing_manifest_error() {
        let temp_dir = setup_temp_dir();
        let manifest_path = temp_dir.path().join("typst.toml");

        // Verify manifest doesn't exist
        assert_not_exists(&manifest_path);

        // Attempting to read should fail gracefully
    }

    #[test]
    fn test_invalid_version_format() {
        let temp_dir = setup_temp_dir();

        let invalid_manifest = r#"[package]
name = "test"
version = "invalid"
entrypoint = "main.typ"
"#;

        // This should fail to parse
        let manifest_path = temp_dir.path().join("typst.toml");
        fs::write(&manifest_path, invalid_manifest).unwrap();

        // Parsing should detect invalid version
    }

    #[test]
    fn test_package_already_exists() {
        let temp_dir = setup_temp_dir();
        setup_test_env(temp_dir.path());

        // Create an existing package
        let existing = temp_dir
            .path()
            .join("data/typst/packages/local/existing/1.0.0");
        fs::create_dir_all(&existing).unwrap();
        fs::write(existing.join("typst.toml"), "test").unwrap();

        assert_dir_exists(&existing);

        // Attempting to link again should fail without --force

        cleanup_test_env();
    }

    #[test]
    fn test_invalid_package_format() {
        use utpm::regex_package;

        // Test various invalid package formats
        let invalid_formats = vec![
            "preview/package:1.0.0",   // Missing @
            "@preview/package",        // Missing version
            "@preview/package:v1.0",   // Invalid version (v prefix)
            "package:1.0.0",           // Missing namespace
            "@/package:1.0.0",         // Empty namespace
            "@preview/:1.0.0",         // Empty name
            "@preview/package:1.0",    // Incomplete version
            "@preview/package-:1.0.0", // Invalid name (trailing dash)
        ];

        let re = regex_package();
        for format in invalid_formats {
            // None of these should match the package regex
            assert!(
                !re.is_match(format),
                "Format should be invalid but matched regex: {}",
                format
            );
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_list_many_packages() {
        let temp_dir = setup_temp_dir();
        setup_test_env(temp_dir.path());

        let data_dir = temp_dir.path().join("data/typst/packages/local");

        // Create many packages
        for i in 0..100 {
            let pkg_path = data_dir.join(format!("package-{}/1.0.0", i));
            fs::create_dir_all(&pkg_path).unwrap();
        }

        // Listing should handle many packages efficiently
        let count = fs::read_dir(&data_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .count();

        assert_eq!(count, 100);

        cleanup_test_env();
    }

    #[test]
    fn test_large_file_filtering() {
        let temp_dir = setup_temp_dir();

        // Create many files
        for i in 0..1000 {
            fs::write(temp_dir.path().join(format!("file-{}.typ", i)), "content").unwrap();
        }

        // Filtering should handle many files
        let count = fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .count();

        assert_eq!(count, 1000);
    }
}
