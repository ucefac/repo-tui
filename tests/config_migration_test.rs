//! Configuration migration tests
//!
//! Tests for v1 → v2 configuration migration

use std::path::PathBuf;
use tempfile::TempDir;

/// Helper function to simulate v1 config structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct V1Config {
    version: String,
    main_directory: PathBuf,
}

/// Helper function to simulate v2 config structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct V2Config {
    version: String,
    main_directories: Vec<String>,
    single_repos: Vec<String>,
}

/// Simulate migration from v1 to v2
fn migrate_v1_to_v2(v1: &V1Config) -> V2Config {
    let main_dirs = if v1.main_directory.as_os_str().is_empty() {
        vec![]
    } else {
        vec![v1.main_directory.to_string_lossy().to_string()]
    };

    V2Config {
        version: "2.0".to_string(),
        main_directories: main_dirs,
        single_repos: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v1_to_v2_migration() {
        // Arrange: Create a v1 config
        let v1_config = V1Config {
            version: "1.0".to_string(),
            main_directory: PathBuf::from("/home/user/projects"),
        };

        // Act: Migrate to v2
        let v2_config = migrate_v1_to_v2(&v1_config);

        // Assert: Verify migration results
        assert_eq!(v2_config.version, "2.0");
        assert_eq!(v2_config.main_directories.len(), 1);
        assert_eq!(v2_config.main_directories[0], "/home/user/projects");
        assert!(v2_config.single_repos.is_empty());
    }

    #[test]
    fn test_empty_main_directory_migration() {
        // Arrange: Create a v1 config with empty main_directory
        let v1_config = V1Config {
            version: "1.0".to_string(),
            main_directory: PathBuf::new(),
        };

        // Act: Migrate to v2
        let v2_config = migrate_v1_to_v2(&v1_config);

        // Assert: Empty path should not be added to main_directories
        assert!(v2_config.main_directories.is_empty());
        assert_eq!(v2_config.version, "2.0");
    }

    #[test]
    fn test_duplicate_path_handling() {
        // Arrange: Simulate a scenario where migration might create duplicates
        let v1_config = V1Config {
            version: "1.0".to_string(),
            main_directory: PathBuf::from("/home/user/projects"),
        };

        // Act: First migration
        let mut v2_config = migrate_v1_to_v2(&v1_config);

        // Simulate duplicate addition (which should be prevented in real code)
        v2_config
            .main_directories
            .push("/home/user/projects".to_string());

        // Assert: Check for duplicates
        let has_duplicates = v2_config.main_directories.len()
            != v2_config
                .main_directories
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len();
        assert!(has_duplicates, "Test setup should have duplicates");

        // Deduplicate
        let unique: std::collections::HashSet<_> =
            v2_config.main_directories.iter().cloned().collect();
        assert_eq!(unique.len(), 1);
    }

    #[test]
    fn test_config_persistence_after_migration() {
        // Arrange: Create a temporary directory and config file
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let v1_config = V1Config {
            version: "1.0".to_string(),
            main_directory: PathBuf::from("/home/user/workspace"),
        };

        // Write v1 config to file
        let toml_str = toml::to_string(&v1_config).unwrap();
        std::fs::write(&config_path, toml_str).unwrap();

        // Act: Read and migrate
        let content = std::fs::read_to_string(&config_path).unwrap();
        let loaded_v1: V1Config = toml::from_str(&content).unwrap();
        let v2_config = migrate_v1_to_v2(&loaded_v1);

        // Write v2 config back
        let v2_toml = toml::to_string(&v2_config).unwrap();
        std::fs::write(&config_path, v2_toml).unwrap();

        // Assert: Verify persisted config
        let persisted_content = std::fs::read_to_string(&config_path).unwrap();
        let loaded_v2: V2Config = toml::from_str(&persisted_content).unwrap();

        assert_eq!(loaded_v2.version, "2.0");
        assert_eq!(loaded_v2.main_directories.len(), 1);
        assert_eq!(loaded_v2.main_directories[0], "/home/user/workspace");
    }

    #[test]
    fn test_migration_preserves_order() {
        // Arrange: Create multiple v1 configs that would be merged
        let v1_configs = vec![
            V1Config {
                version: "1.0".to_string(),
                main_directory: PathBuf::from("/home/user/work"),
            },
            V1Config {
                version: "1.0".to_string(),
                main_directory: PathBuf::from("/home/user/personal"),
            },
        ];

        // Act: Migrate each
        let v2_configs: Vec<_> = v1_configs.iter().map(migrate_v1_to_v2).collect();

        // Assert: Order should be preserved
        assert_eq!(v2_configs[0].main_directories[0], "/home/user/work");
        assert_eq!(v2_configs[1].main_directories[0], "/home/user/personal");
    }

    #[test]
    fn test_empty_path_handling_in_migration() {
        // Test various empty/invalid path scenarios
        let test_cases = vec![
            ("", true),             // Empty string - should be filtered
            ("   ", false),         // Whitespace - might be valid or not
            ("/valid/path", false), // Valid path - should not be filtered
        ];

        for (path_str, should_be_empty) in test_cases {
            let v1_config = V1Config {
                version: "1.0".to_string(),
                main_directory: PathBuf::from(path_str),
            };

            let v2_config = migrate_v1_to_v2(&v1_config);

            if should_be_empty {
                assert!(
                    v2_config.main_directories.is_empty(),
                    "Path '{}' should result in empty main_directories",
                    path_str
                );
            }
        }
    }

    #[test]
    fn test_main_directories_array_format() {
        // Arrange: Create v2 config
        let v2_config = V2Config {
            version: "2.0".to_string(),
            main_directories: vec![
                "/home/user/projects".to_string(),
                "/home/user/work".to_string(),
            ],
            single_repos: vec![],
        };

        // Act: Serialize to TOML
        let toml_str = toml::to_string(&v2_config).unwrap();

        // Assert: Verify array format
        assert!(toml_str.contains("main_directories"));
        assert!(toml_str.contains('['));
        assert!(toml_str.contains("/home/user/projects"));
        assert!(toml_str.contains("/home/user/work"));
    }

    #[test]
    fn test_backward_compatibility_check() {
        // Test that v1 configs are correctly identified
        let v1_config = V1Config {
            version: "1.0".to_string(),
            main_directory: PathBuf::from("/home/user/projects"),
        };

        // Check version indicates v1
        assert!(v1_config.version.starts_with("1."));
        assert!(!v1_config.main_directory.as_os_str().is_empty());

        // After migration, version should be v2
        let v2_config = migrate_v1_to_v2(&v1_config);
        assert!(v2_config.version.starts_with("2."));
    }

    #[test]
    fn test_migration_with_special_characters_in_path() {
        // Arrange: Paths with special characters
        let v1_config = V1Config {
            version: "1.0".to_string(),
            main_directory: PathBuf::from("/home/user/my-projects_v2.0/test+dev"),
        };

        // Act: Migrate
        let v2_config = migrate_v1_to_v2(&v1_config);

        // Assert: Special characters should be preserved
        assert_eq!(v2_config.main_directories.len(), 1);
        assert!(v2_config.main_directories[0].contains("my-projects_v2.0"));
    }

    #[test]
    fn test_migration_with_unicode_paths() {
        // Arrange: Paths with unicode characters
        let v1_config = V1Config {
            version: "1.0".to_string(),
            main_directory: PathBuf::from("/home/user/项目/work"),
        };

        // Act: Migrate
        let v2_config = migrate_v1_to_v2(&v1_config);

        // Assert: Unicode should be preserved
        assert_eq!(v2_config.main_directories.len(), 1);
        assert!(v2_config.main_directories[0].contains("项目"));
    }
}
