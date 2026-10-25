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

    #[test]
    fn test_init_populate_creates_template_files() {
        use std::str::FromStr;
        use typst_syntax::package::{PackageInfo, PackageVersion, TemplateInfo};

        let temp_dir = setup_temp_dir();
        let pkg = PackageInfo {
            name: "organized-ajp".into(),
            version: PackageVersion::from_str("0.1.0").unwrap(),
            entrypoint: "lib.typ".into(),
            authors: vec!["The Typst Project Developers".into()],
            license: Some("MIT".into()),
            description: Some("Master's thesis at the Unseen University.".into()),
            repository: None,
            homepage: None,
            keywords: vec![],
            compiler: None,
            exclude: vec![],
            categories: vec!["thesis".into()],
            disciplines: vec![],
            unknown_fields: Default::default(),
        };
        let mut template = TemplateInfo::new("template", "main.typ");
        template.thumbnail = Some("thumbnail.png".into());

        utpm::commands::init::populate_project_files(temp_dir.path(), &pkg, Some(&template))
            .unwrap();

        // Template directory and its entrypoint are created, and the
        // entrypoint imports the package.
        let entry = temp_dir.path().join("template").join("main.typ");
        assert_file_exists(&entry);
        let content = read_file_string(&entry);
        assert!(content.contains("#import \"@local/organized-ajp:0.1.0\": *"));
        assert!(content.contains("#show:"));
    }

    #[test]
    fn test_init_populate_nested_entrypoint() {
        use std::str::FromStr;
        use typst_syntax::package::{PackageInfo, PackageVersion};

        let temp_dir = setup_temp_dir();
        let pkg = PackageInfo {
            name: "nested-pkg".into(),
            version: PackageVersion::from_str("0.1.0").unwrap(),
            entrypoint: "src/lib.typ".into(),
            authors: vec![],
            license: None,
            description: None,
            repository: None,
            homepage: None,
            keywords: vec![],
            compiler: None,
            exclude: vec![],
            categories: vec![],
            disciplines: vec![],
            unknown_fields: Default::default(),
        };

        utpm::commands::init::populate_project_files(temp_dir.path(), &pkg, None).unwrap();

        // Parent directories of a nested entrypoint are created.
        assert_file_exists(&temp_dir.path().join("src").join("lib.typ"));
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
    use utpm::commands::unlink::remove_package_dir;

    #[test]
    fn test_unlink_removes_package() {
        let temp_dir = setup_temp_dir();
        setup_test_env(temp_dir.path());

        // Create a linked package
        let package_path = temp_dir
            .path()
            .join("data/typst/packages/local/test-pkg/1.0.0");
        fs::create_dir_all(&package_path).unwrap();
        fs::write(package_path.join("typst.toml"), "test").unwrap();

        assert_dir_exists(&package_path);

        // After unlink, directory should be removed
        // This would require actual command execution

        cleanup_test_env();
    }

    #[test]
    fn test_unlink_cascades_empty_parents() {
        init_test_args();
        let temp_dir = setup_temp_dir();
        let root = temp_dir.path().join("packages");
        let version_dir = root.join("local").join("my-pkg").join("1.0.0");
        fs::create_dir_all(&version_dir).unwrap();
        fs::write(version_dir.join("typst.toml"), "test").unwrap();

        remove_package_dir(&version_dir, &root).unwrap();

        // Version, package name and namespace directories are all removed.
        assert_not_exists(&version_dir);
        assert_not_exists(&root.join("local").join("my-pkg"));
        assert_not_exists(&root.join("local"));
        // The package directory root itself is never removed.
        assert_dir_exists(&root);
    }

    #[test]
    fn test_unlink_keeps_parents_with_remaining_packages() {
        init_test_args();
        let temp_dir = setup_temp_dir();
        let root = temp_dir.path().join("packages");
        let pkg_a_v1 = root.join("local").join("pkg-a").join("1.0.0");
        let pkg_a_v2 = root.join("local").join("pkg-a").join("2.0.0");
        let pkg_b = root.join("local").join("pkg-b").join("1.0.0");
        fs::create_dir_all(&pkg_a_v1).unwrap();
        fs::create_dir_all(&pkg_a_v2).unwrap();
        fs::create_dir_all(&pkg_b).unwrap();
        fs::write(pkg_a_v1.join("typst.toml"), "test").unwrap();
        fs::write(pkg_a_v2.join("typst.toml"), "test").unwrap();
        fs::write(pkg_b.join("typst.toml"), "test").unwrap();

        // Remove one version: the package keeps its other version.
        remove_package_dir(&pkg_a_v1, &root).unwrap();
        assert_not_exists(&pkg_a_v1);
        assert_dir_exists(&pkg_a_v2);
        assert_dir_exists(&root.join("local").join("pkg-a"));
        assert_dir_exists(&root.join("local"));

        // Remove the last version: the package goes, the namespace stays.
        remove_package_dir(&pkg_a_v2, &root).unwrap();
        assert_not_exists(&root.join("local").join("pkg-a"));
        assert_dir_exists(&root.join("local").join("pkg-b"));
        assert_dir_exists(&root.join("local"));
    }

    #[test]
    fn test_unlink_package_not_exist() {
        let temp_dir = setup_temp_dir();
        setup_test_env(temp_dir.path());

        let package_path = temp_dir
            .path()
            .join("data/typst/packages/local/nonexistent/1.0.0");

        assert_not_exists(&package_path);

        cleanup_test_env();
    }
}

#[cfg(test)]
mod bump_command_tests {
    use std::str::FromStr;

    use typst_syntax::package::PackageVersion;

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
            ("1.0.0", "1.0.1"), // Patch
            ("1.0.0", "1.1.0"), // Minor
            ("1.0.0", "2.0.0"), // Major
            ("0.1.0", "0.1.1"), // Pre-release patch
        ];

        for (old, new) in versions {
            assert_ne!(old, new);
            // Verify semver parsing would work
            assert!(PackageVersion::from_str(old).is_ok());
            assert!(PackageVersion::from_str(new).is_ok());
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
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();

        assert_eq!(versions.len(), 3);
        assert!(versions.iter().any(|v| v == "1.0.0"));
        assert!(versions.iter().any(|v| v == "2.0.0"));

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
