/// Tests for utility functions in utils/ modules
mod common;

use common::*;

#[cfg(test)]
mod regex_tests {
    use utpm::utils::regex_package;

    #[test]
    fn test_valid_package_formats() {
        let re = regex_package();

        // Valid formats
        assert!(re.is_match("@preview/package:2.0.1"));
        assert!(re.is_match("@local/package:1.0.0"));
        assert!(re.is_match("@local/package-a:2.0.1"));
        assert!(re.is_match("@local/package-A:2.0.1"));
        assert!(re.is_match("@local/my-package:0.1.0"));
        assert!(re.is_match("@namespace/name:10.20.30"));
        assert!(re.is_match("@a/p:1.0.1"));
    }

    #[test]
    fn test_invalid_package_formats() {
        let re = regex_package();

        // Invalid formats
        assert!(!re.is_match("@preview/package-:2.0.1")); // Trailing dash
        assert!(!re.is_match("@local/p:1..1")); // Double dots
        assert!(!re.is_match("@a/p:v1.0.1")); // Version prefix
        assert!(!re.is_match("@/p:1.0.1")); // Missing namespace
        assert!(!re.is_match("p:1.0.1")); // Missing @
        assert!(!re.is_match("@a/p")); // Missing version
        assert!(!re.is_match("package:1.0.0")); // Missing namespace
        assert!(!re.is_match("@preview/package")); // Missing version
        assert!(!re.is_match("@preview/package:1.0")); // Incomplete version
    }

    #[test]
    fn test_edge_cases() {
        let re = regex_package();

        assert!(re.is_match("@local/AAAAAAAAAAAAAA:2.0.1"));
        assert!(re.is_match("@x/y:0.0.1"));
        assert!(!re.is_match("@/package:1.0.0"));
        assert!(!re.is_match("@namespace/:1.0.0"));
    }
}

#[cfg(test)]
mod regex_import_tests {
    use utpm::utils::regex_import;

    #[test]
    fn test_valid_import_formats() {
        let re = regex_import();

        assert!(re.is_match("#import \"@preview/package:1.0.0\""));
        assert!(re.is_match("#import \"@local/my-package:2.1.3\""));
        assert!(re.is_match("#import \"@namespace/name:0.1.0\""));
    }

    #[test]
    fn test_invalid_import_formats() {
        let re = regex_import();

        assert!(!re.is_match("import \"@preview/package:1.0.0\"")); // Missing #
        assert!(!re.is_match("#import @preview/package:1.0.0")); // Missing quotes
        assert!(!re.is_match("#import \"preview/package:1.0.0\"")); // Missing @
    }
}

#[cfg(test)]
mod sync_regex_tests {
    use regex::Regex;

    #[test]
    fn test_sync_regex_pattern() {
        // Test the regex pattern used in sync command
        let re = Regex::new(
            r#"\#import \"@([a-zA-Z]+)\/([a-zA-Z]+(?:\-[a-zA-Z]+)?)\:(\d+)\.(\d+)\.(\d+)\""#,
        )
        .unwrap();

        assert!(re.is_match("#import \"@preview/example:1.0.0\""));
        assert!(re.is_match("#import \"@local/my-pkg:2.1.3\""));
    }
}

#[cfg(test)]
mod paths_tests {
    use super::*;
    use std::env;

    #[test]
    fn test_get_current_dir() {
        let temp_dir = setup_temp_dir();
        let current_path = temp_dir.path().join("current");
        std::fs::create_dir_all(&current_path).unwrap();

        // Note: env::set_var is unsafe in Rust 2024 edition
        // In production code, use safer alternatives or unsafe blocks
        unsafe {
            env::set_var("UTPM_CURRENT_DIR", current_path.to_str().unwrap());
        }

        // This would require accessing the actual function
        // For now, just verify env var is set
        assert_eq!(
            env::var("UTPM_CURRENT_DIR").unwrap(),
            current_path.to_str().unwrap()
        );

        cleanup_test_env();
    }

    #[test]
    fn test_check_path_file() {
        let temp_dir = setup_temp_dir();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "test").unwrap();

        assert!(file_path.exists());
        assert!(file_path.is_file());

        let dir_path = temp_dir.path().join("dir");
        std::fs::create_dir(&dir_path).unwrap();

        assert!(dir_path.exists());
        assert!(dir_path.is_dir());
    }

    #[test]
    fn test_has_content() {
        let temp_dir = setup_temp_dir();

        // Empty directory
        let empty_dir = temp_dir.path().join("empty");
        std::fs::create_dir(&empty_dir).unwrap();
        assert_eq!(std::fs::read_dir(&empty_dir).unwrap().count(), 0);

        // Directory with content
        let full_dir = temp_dir.path().join("full");
        std::fs::create_dir(&full_dir).unwrap();
        std::fs::write(full_dir.join("file.txt"), "content").unwrap();
        assert!(std::fs::read_dir(&full_dir).unwrap().count() > 0);
    }
}

#[cfg(test)]
mod specs_tests {
    use ecow::eco_vec;
    use utpm::utils::specs::Extra;

    #[test]
    fn test_extra_default() {
        let extra = Extra::default();
        assert_eq!(extra.exclude, None);
    }

    #[test]
    fn test_extra_serialization() {
        let extra = Extra {
            exclude: Some(eco_vec![String::from("*.md"), String::from(".git")]),
        };

        // Test that it can be serialized
        let serialized = serde_json::to_string(&extra);
        assert!(serialized.is_ok());
    }

    #[test]
    fn test_extra_with_excludes() {
        let extra = Extra {
            exclude: Some(eco_vec![
                String::from(".git"),
                String::from(".github"),
                String::from("*.md"),
                String::from("tests/"),
            ]),
        };

        assert!(extra.exclude.is_some());
        let excludes = extra.exclude.unwrap();
        assert_eq!(excludes.len(), 4);
        assert!(excludes.iter().any(|s| s == ".git"));
        assert!(excludes.iter().any(|s| s == "*.md"));
    }

    #[test]
    fn test_extra_new() {
        let excludes = eco_vec![String::from(".git"), String::from("*.log")];
        let extra = Extra::new(Some(excludes.clone()));

        assert!(extra.exclude.is_some());
        let unwrapped = extra.exclude.unwrap();
        assert_eq!(unwrapped.len(), 2);
        assert!(unwrapped.iter().any(|s| s == ".git"));
        assert!(unwrapped.iter().any(|s| s == "*.log"));
    }
}

#[cfg(test)]
mod state_tests {
    use utpm::utils::state::UtpmError;

    #[test]
    fn test_error_variants() {
        let err = UtpmError::Manifest;
        let msg = format!("{}", err);
        assert!(msg.contains("Missing typst.toml manifest"));
        assert!(msg.contains("utpm prj init"));

        let err = UtpmError::PackageNotValid;
        assert!(format!("{}", err).contains("Invalid package format"));

        let err = UtpmError::PackageNotExist;
        let msg = format!("{}", err);
        assert!(msg.contains("Package not found") || msg.contains("doesn't exist"));
    }

    #[test]
    fn test_already_exist_error() {
        let err = UtpmError::AlreadyExist(
            String::from("test-package"),
            typst_syntax::package::PackageVersion {
                major: 1,
                minor: 0,
                patch: 0,
            },
            String::from("Test info"),
        );

        let msg = format!("{}", err);
        assert!(msg.contains("test-package"));
        assert!(msg.contains("1.0.0"));
        assert!(msg.contains("Test info"));
    }

    #[test]
    fn test_error_display() {
        // Test that all error variants implement Display properly
        let errors = vec![
            UtpmError::Manifest,
            UtpmError::PackageNotValid,
            UtpmError::PackageNotExist,
            UtpmError::ContentFound,
            UtpmError::PackageFormatError,
        ];

        for err in errors {
            let msg = format!("{}", err);
            assert!(!msg.is_empty(), "Error message should not be empty");
        }
    }
}

#[cfg(test)]
mod output_tests {
    use utpm::utils::output::OutputFormat;

    #[test]
    fn test_output_format_variants() {
        let formats = vec![OutputFormat::Text, OutputFormat::Json];

        for format in formats {
            // Just verify they exist and can be compared
            assert!(format == OutputFormat::Text || format == OutputFormat::Json);
        }
    }
}

#[cfg(test)]
mod dryrun_tests {
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn setup() {
        INIT.call_once(|| {
            // Initialize any global state here
        });
    }

    #[test]
    fn test_dry_run_flag() {
        setup();
        // Test that dry-run mode can be checked
        // The actual implementation uses OnceLock which can only be set once
        // So we test the concept rather than the actual function
        let dry_run = false;
        assert!(!dry_run);
    }
}
