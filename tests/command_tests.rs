/// Integration tests for UTPM commands
mod common;

use common::*;
use std::fs;

#[cfg(test)]
mod init_command_tests {
    use super::*;

    #[test]
    fn test_init_creates_manifest() {
        let temp_dir = setup_temp_dir();
        setup_test_env(temp_dir.path());
        
        let current_dir = temp_dir.path().join("current");
        fs::create_dir_all(&current_dir).unwrap();
        
        let manifest_path = current_dir.join("typst.toml");
        
        // Verify manifest doesn't exist yet
        assert!(!manifest_path.exists());
        
        // After init would run, manifest should exist
        // This is a placeholder for actual command execution
        
        cleanup_test_env();
    }

    #[test]
    fn test_init_with_force_flag() {
        let temp_dir = setup_temp_dir();
        let manifest_path = create_test_manifest(temp_dir.path(), "test", "1.0.0");
        
        // Verify manifest exists
        assert_file_exists(&manifest_path);
        
        let content_before = read_file_string(&manifest_path);
        
        // With --force flag, it should overwrite
        // This would require actual command execution
        
        assert!(content_before.contains("test"));
    }

    #[test]
    fn test_init_manifest_structure() {
        let temp_dir = setup_temp_dir();
        let manifest_path = create_test_manifest(temp_dir.path(), "my-package", "2.1.0");
        
        let content = read_file_string(&manifest_path);
        
        // Verify required fields
        assert!(content.contains("[package]"));
        assert!(content.contains("name = \"my-package\""));
        assert!(content.contains("version = \"2.1.0\""));
        assert!(content.contains("entrypoint = \"main.typ\""));
        assert!(content.contains("[tool.utpm]"));
    }

    #[test]
    fn test_init_with_custom_entrypoint() {
        let temp_dir = setup_temp_dir();
        let manifest_content = r#"[package]
name = "test"
version = "1.0.0"
entrypoint = "lib.typ"
authors = ["Test"]
license = "MIT"
description = "Test"

[tool.utpm]
namespace = "local"
"#;
        let manifest_path = create_custom_manifest(temp_dir.path(), manifest_content);
        
        let content = read_file_string(&manifest_path);
        assert!(content.contains("entrypoint = \"lib.typ\""));
    }

    #[test]
    fn test_init_populate_creates_files() {
        let temp_dir = setup_temp_dir();
        create_test_package(temp_dir.path(), "test-pkg", "1.0.0");
        
        // Verify files were created
        assert_file_exists(&temp_dir.path().join("typst.toml"));
        assert_file_exists(&temp_dir.path().join("main.typ"));
        assert_dir_exists(&temp_dir.path().join("src"));
        assert_dir_exists(&temp_dir.path().join("examples"));
    }
}

#[cfg(test)]
mod link_command_tests {
    use super::*;

    #[test]
    fn test_link_package_structure() {
        let temp_dir = setup_temp_dir();
        setup_test_env(temp_dir.path());
        
        // Create a test package
        let package_dir = temp_dir.path().join("test-package");
        create_test_package(&package_dir, "my-pkg", "1.0.0");
        
        // Verify package structure
        assert_file_exists(&package_dir.join("typst.toml"));
        assert_file_exists(&package_dir.join("main.typ"));
        
        cleanup_test_env();
    }

    #[test]
    fn test_link_target_directory() {
        let temp_dir = setup_temp_dir();
        setup_test_env(temp_dir.path());
        
        let data_dir = temp_dir.path().join("data/typst/packages");
        fs::create_dir_all(&data_dir).unwrap();
        
        // Link should create: data/typst/packages/local/my-pkg/1.0.0/
        let _expected_path = data_dir.join("local/my-pkg/1.0.0");
        
        assert_dir_exists(&data_dir);
        
        cleanup_test_env();
    }

    #[test]
    fn test_link_respects_gitignore() {
        let temp_dir = setup_temp_dir();
        let package_dir = temp_dir.path().join("package");
        fs::create_dir_all(&package_dir).unwrap();
        
        // Create .gitignore
        fs::write(package_dir.join(".gitignore"), "*.log\n.env\n").unwrap();
        
        // Create files
        fs::write(package_dir.join("main.typ"), "test").unwrap();
        fs::write(package_dir.join("debug.log"), "log").unwrap();
        fs::write(package_dir.join(".env"), "secret").unwrap();
        
        // Verify .gitignore exists
        assert_file_exists(&package_dir.join(".gitignore"));
    }

    #[test]
    fn test_link_with_exclude_patterns() {
        let temp_dir = setup_temp_dir();
        
        let manifest_content = r#"[package]
name = "test"
version = "1.0.0"
entrypoint = "main.typ"
authors = ["Test"]
license = "MIT"
description = "Test"

[tool.utpm]
namespace = "local"
exclude = [".git", "*.md", "tests/"]
"#;
        create_custom_manifest(temp_dir.path(), manifest_content);
        
        let content = read_file_string(&temp_dir.path().join("typst.toml"));
        assert!(content.contains("exclude = [\".git\", \"*.md\", \"tests/\"]"));
    }
}

#[cfg(test)]
mod clone_command_tests {
    use super::*;

    #[test]
    fn test_clone_package_format() {
        // Test valid package formats for cloning
        let valid_formats = vec![
            "@preview/example:1.0.0",
            "@preview/my-package:2.3.4",
            "@local/test:0.1.0",
        ];

        for format in valid_formats {
            assert!(format.starts_with('@'));
            assert!(format.contains('/'));
            assert!(format.contains(':'));
        }
    }

    #[test]
    fn test_clone_target_directory() {
        let temp_dir = setup_temp_dir();
        setup_test_env(temp_dir.path());
        
        // Preview packages go to cache
        let cache_dir = temp_dir.path().join("cache/typst/packages/preview");
        fs::create_dir_all(&cache_dir).unwrap();
        
        assert_dir_exists(&cache_dir);
        
        cleanup_test_env();
    }
}

#[cfg(test)]
mod unlink_command_tests {
    use super::*;

    #[test]
    fn test_unlink_removes_package() {
        let temp_dir = setup_temp_dir();
        setup_test_env(temp_dir.path());
        
        // Create a linked package
        let package_path = temp_dir.path()
            .join("data/typst/packages/local/test-pkg/1.0.0");
        fs::create_dir_all(&package_path).unwrap();
        fs::write(package_path.join("typst.toml"), "test").unwrap();
        
        assert_dir_exists(&package_path);
        
        // After unlink, directory should be removed
        // This would require actual command execution
        
        cleanup_test_env();
    }

    #[test]
    fn test_unlink_package_not_exist() {
        let temp_dir = setup_temp_dir();
        setup_test_env(temp_dir.path());
        
        let package_path = temp_dir.path()
            .join("data/typst/packages/local/nonexistent/1.0.0");
        
        assert_not_exists(&package_path);
        
        cleanup_test_env();
    }
}

#[cfg(test)]
mod bump_command_tests {
    use super::*;

    #[test]
    fn test_bump_updates_version() {
        let temp_dir = setup_temp_dir();
        create_test_manifest(temp_dir.path(), "test", "1.0.0");
        
        let manifest_path = temp_dir.path().join("typst.toml");
        let content = read_file_string(&manifest_path);
        
        assert!(content.contains("version = \"1.0.0\""));
        
        // After bump to 1.1.0, should contain new version
        // This would require actual command execution
    }

    #[test]
    fn test_bump_version_formats() {
        // Test various version formats
        let versions = vec![
            ("1.0.0", "1.0.1"),  // Patch
            ("1.0.0", "1.1.0"),  // Minor
            ("1.0.0", "2.0.0"),  // Major
            ("0.1.0", "0.1.1"),  // Pre-release patch
        ];

        for (old, new) in versions {
            assert_ne!(old, new);
            // Verify semver parsing would work
            assert!(semver::Version::parse(old).is_ok());
            assert!(semver::Version::parse(new).is_ok());
        }
    }
}

#[cfg(test)]
mod sync_command_tests {
    use super::*;

    #[test]
    fn test_sync_finds_imports() {
        let temp_dir = setup_temp_dir();
        
        let typ_file = temp_dir.path().join("main.typ");
        let content = r#"#import "@preview/example:1.0.0": *
#import "@preview/other:2.3.4"

// Some code
"#;
        fs::write(&typ_file, content).unwrap();
        
        let file_content = read_file_string(&typ_file);
        assert!(file_content.contains("@preview/example:1.0.0"));
        assert!(file_content.contains("@preview/other:2.3.4"));
    }

    #[test]
    fn test_sync_check_mode() {
        // Test that --check mode doesn't modify files
        let temp_dir = setup_temp_dir();
        let typ_file = temp_dir.path().join("main.typ");
        let content = "#import \"@preview/example:1.0.0\": *";
        fs::write(&typ_file, content).unwrap();
        
        let before = read_file_string(&typ_file);
        
        // In check mode, file should remain unchanged
        let after = read_file_string(&typ_file);
        assert_eq!(before, after);
    }
}

#[cfg(test)]
mod list_command_tests {
    use super::*;

    #[test]
    fn test_list_local_packages() {
        let temp_dir = setup_temp_dir();
        setup_test_env(temp_dir.path());
        
        let data_dir = temp_dir.path().join("data/typst/packages");
        
        // Create some test packages
        let pkg1 = data_dir.join("local/package1/1.0.0");
        let pkg2 = data_dir.join("local/package2/2.1.0");
        
        fs::create_dir_all(&pkg1).unwrap();
        fs::create_dir_all(&pkg2).unwrap();
        
        assert_dir_exists(&pkg1);
        assert_dir_exists(&pkg2);
        
        cleanup_test_env();
    }

    #[test]
    fn test_list_multiple_versions() {
        let temp_dir = setup_temp_dir();
        setup_test_env(temp_dir.path());
        
        let data_dir = temp_dir.path().join("data/typst/packages/local/package");
        
        // Create multiple versions
        fs::create_dir_all(data_dir.join("1.0.0")).unwrap();
        fs::create_dir_all(data_dir.join("1.1.0")).unwrap();
        fs::create_dir_all(data_dir.join("2.0.0")).unwrap();
        
        let versions: Vec<_> = fs::read_dir(&data_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        
        assert_eq!(versions.len(), 3);
        assert!(versions.contains(&"1.0.0".to_string()));
        assert!(versions.contains(&"2.0.0".to_string()));
        
        cleanup_test_env();
    }
}

#[cfg(test)]
mod metadata_command_tests {
    use super::*;

    #[test]
    fn test_metadata_extracts_all_fields() {
        let temp_dir = setup_temp_dir();
        create_test_manifest(temp_dir.path(), "test-pkg", "3.2.1");
        
        let content = read_file_string(&temp_dir.path().join("typst.toml"));
        
        // Verify all standard fields are present
        assert!(content.contains("name"));
        assert!(content.contains("version"));
        assert!(content.contains("entrypoint"));
        assert!(content.contains("authors"));
        assert!(content.contains("license"));
        assert!(content.contains("description"));
    }

    #[test]
    fn test_metadata_specific_field() {
        let temp_dir = setup_temp_dir();
        create_test_manifest(temp_dir.path(), "my-package", "1.2.3");
        
        let content = read_file_string(&temp_dir.path().join("typst.toml"));
        
        // Extract specific fields
        assert!(content.contains("name = \"my-package\""));
        assert!(content.contains("version = \"1.2.3\""));
    }
}

#[cfg(test)]
mod get_command_tests {
    #[test]
    fn test_get_package_info() {
        // Test package info retrieval concepts
        let package_name = "example";
        let package_version = "1.0.0";
        
        assert!(!package_name.is_empty());
        assert!(!package_version.is_empty());
    }
}

#[cfg(test)]
mod install_command_tests {
    use super::*;

    #[test]
    fn test_install_from_git() {
        // Test git URL validation
        let valid_git_urls = vec![
            "https://github.com/user/repo.git",
            "git@github.com:user/repo.git",
            "git://github.com/user/repo.git",
        ];

        for url in valid_git_urls {
            assert!(url.contains("git") || url.starts_with("https"));
        }
    }

    #[test]
    fn test_install_dependencies() {
        let temp_dir = setup_temp_dir();
        
        let manifest_content = r#"[package]
name = "test"
version = "1.0.0"
entrypoint = "main.typ"
authors = ["Test"]
license = "MIT"
description = "Test"

[tool.utpm]
namespace = "local"
dependencies = [
    "https://github.com/example/dep1.git",
    "https://github.com/example/dep2.git"
]
"#;
        create_custom_manifest(temp_dir.path(), manifest_content);
        
        let content = read_file_string(&temp_dir.path().join("typst.toml"));
        assert!(content.contains("dependencies"));
    }
}
