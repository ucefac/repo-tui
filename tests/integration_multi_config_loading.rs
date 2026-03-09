//! Multi-directory configuration loading integration tests
//!
//! Tests for loading and managing multi-directory configurations

use std::path::{Path, PathBuf};
use tempfile::TempDir;

// === Helper functions (inlined from config_helper.rs) ===

/// Create a temporary directory with a multi-directory config
fn create_temp_config_with_dirs(dirs: &[&str]) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let dirs_toml = dirs
        .iter()
        .map(|d| format!("    \"{}\"", d))
        .collect::<Vec<_>>()
        .join(",\n");

    let config_content = format!(
        r#"version = "2.0"
main_directories = [
{}
]
single_repos = []

[editors]
vscode = "code"
webstorm = "webstorm"

[ui]
theme = "dark"
show_git_status = true
show_branch = true

[security]
allow_symlinks = false
max_search_depth = 2
"#,
        dirs_toml
    );

    std::fs::write(&config_path, config_content).unwrap();
    (temp_dir, config_path)
}

/// Create a temporary directory with an old format (v1) config
fn create_old_format_config() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let config_content = r#"version = "1.0"
main_directory = "/home/user/projects"

[editors]
vscode = "code"
webstorm = "webstorm"

[ui]
theme = "dark"
show_git_status = true
show_branch = true

[security]
allow_symlinks = false
max_search_depth = 2
"#;

    std::fs::write(&config_path, config_content).unwrap();
    (temp_dir, config_path)
}

/// Create a temporary directory with an empty main_directories config
fn create_empty_main_dirs_config() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let config_content = r#"version = "2.0"
main_directories = []
single_repos = []

[editors]
vscode = "code"
webstorm = "webstorm"

[ui]
theme = "dark"
show_git_status = true
show_branch = true

[security]
allow_symlinks = false
max_search_depth = 2
"#;

    std::fs::write(&config_path, config_content).unwrap();
    (temp_dir, config_path)
}

/// Create a config with duplicate paths
fn create_config_with_duplicates() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let config_content = r#"version = "2.0"
main_directories = [
    "/home/user/projects",
    "/home/user/projects",
    "/home/user/work"
]
single_repos = []

[editors]
vscode = "code"
webstorm = "webstorm"

[ui]
theme = "dark"
show_git_status = true
show_branch = true

[security]
allow_symlinks = false
max_search_depth = 2
"#;

    std::fs::write(&config_path, config_content).unwrap();
    (temp_dir, config_path)
}

/// Create a config with both old and new fields
fn create_config_with_both_fields() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let config_content = r#"version = "1.5"
main_directory = "/home/user/legacy"
main_directories = [
    "/home/user/projects",
    "/home/user/work"
]
single_repos = []

[editors]
vscode = "code"
webstorm = "webstorm"

[ui]
theme = "dark"
show_git_status = true
show_branch = true

[security]
allow_symlinks = false
max_search_depth = 2
"#;

    std::fs::write(&config_path, config_content).unwrap();
    (temp_dir, config_path)
}

/// Read the version from a config file
fn read_config_version(path: &Path) -> String {
    let content = std::fs::read_to_string(path).unwrap();
    for line in content.lines() {
        if line.starts_with("version") {
            return line
                .split('=')
                .nth(1)
                .unwrap()
                .trim()
                .trim_matches('"')
                .to_string();
        }
    }
    panic!("Version not found in config")
}

/// Count the number of main directories in a config file
fn count_main_directories(path: &Path) -> usize {
    let content = std::fs::read_to_string(path).unwrap();
    let value: toml::Value = content.parse().unwrap();

    if let Some(arr) = value.get("main_directories").and_then(|v| v.as_array()) {
        arr.len()
    } else {
        0
    }
}

// === Test code ===

/// Simulate loading a multi-directory configuration
fn load_multi_dir_config(config_path: &PathBuf) -> Result<MockConfig, ConfigError> {
    let content = std::fs::read_to_string(config_path).map_err(|_| ConfigError::FileNotFound)?;

    let config: MockConfig = toml::from_str(&content).map_err(|_| ConfigError::InvalidFormat)?;

    // Validate: Check for empty main_directories
    if config.main_directories.is_empty() && config.single_repos.is_empty() {
        return Err(ConfigError::EmptyDirectories);
    }

    // Validate: Check for duplicates
    let unique_count: std::collections::HashSet<_> = config.main_directories.iter().collect();
    if unique_count.len() != config.main_directories.len() {
        return Err(ConfigError::DuplicatePaths);
    }

    Ok(config)
}

/// Simulate saving a configuration
fn save_multi_dir_config(config_path: &PathBuf, config: &MockConfig) -> Result<(), ConfigError> {
    let toml = toml::to_string(config).map_err(|_| ConfigError::SerializationError)?;

    std::fs::write(config_path, toml).map_err(|_| ConfigError::WriteError)?;

    Ok(())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct MockConfig {
    version: String,
    main_directories: Vec<String>,
    single_repos: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
enum ConfigError {
    FileNotFound,
    InvalidFormat,
    EmptyDirectories,
    DuplicatePaths,
    SerializationError,
    WriteError,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_multi_directory_config() {
        // Arrange
        let (_temp, config_path) = create_temp_config_with_dirs(&[
            "/home/user/projects",
            "/home/user/work",
            "/home/user/personal",
        ]);

        // Act
        let config = load_multi_dir_config(&config_path).unwrap();

        // Assert
        assert_eq!(config.version, "2.0");
        assert_eq!(config.main_directories.len(), 3);
        assert!(config
            .main_directories
            .contains(&"/home/user/projects".to_string()));
        assert!(config
            .main_directories
            .contains(&"/home/user/work".to_string()));
        assert!(config
            .main_directories
            .contains(&"/home/user/personal".to_string()));
    }

    #[test]
    fn test_load_empty_main_directories_fails() {
        // Arrange
        let (_temp, config_path) = create_empty_main_dirs_config();

        // Act
        let result = load_multi_dir_config(&config_path);

        // Assert
        assert!(matches!(result, Err(ConfigError::EmptyDirectories)));
    }

    #[test]
    fn test_load_config_with_duplicates_fails() {
        // Arrange
        let (_temp, config_path) = create_config_with_duplicates();

        // Act
        let result = load_multi_dir_config(&config_path);

        // Assert
        assert!(matches!(result, Err(ConfigError::DuplicatePaths)));
    }

    #[test]
    fn test_save_and_reload_config() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("config.toml");

        let config = MockConfig {
            version: "2.0".to_string(),
            main_directories: vec!["/home/user/dir1".to_string(), "/home/user/dir2".to_string()],
            single_repos: vec!["/opt/single".to_string()],
        };

        // Act: Save
        save_multi_dir_config(&config_path, &config).unwrap();

        // Act: Reload
        let loaded = load_multi_dir_config(&config_path).unwrap();

        // Assert
        assert_eq!(loaded.version, config.version);
        assert_eq!(loaded.main_directories, config.main_directories);
        assert_eq!(loaded.single_repos, config.single_repos);
    }

    #[test]
    fn test_config_persistence_across_sessions() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("config.toml");

        // Session 1: Create config
        let config1 = MockConfig {
            version: "2.0".to_string(),
            main_directories: vec!["/home/user/projects".to_string()],
            single_repos: vec![],
        };
        save_multi_dir_config(&config_path, &config1).unwrap();

        // Session 2: Load and modify
        let mut config2 = load_multi_dir_config(&config_path).unwrap();
        config2.main_directories.push("/home/user/work".to_string());
        save_multi_dir_config(&config_path, &config2).unwrap();

        // Session 3: Verify persistence
        let config3 = load_multi_dir_config(&config_path).unwrap();

        assert_eq!(config3.main_directories.len(), 2);
        assert!(config3
            .main_directories
            .contains(&"/home/user/projects".to_string()));
        assert!(config3
            .main_directories
            .contains(&"/home/user/work".to_string()));
    }

    #[test]
    fn test_backward_compatibility_migration() {
        // Arrange: Create old format config
        let (_temp, config_path) = create_old_format_config();

        // Read the old config content
        let content = std::fs::read_to_string(&config_path).unwrap();

        // Simulate migration: Parse old format and convert
        let old_config: toml::Value = content.parse().unwrap();
        let old_path = old_config
            .get("main_directory")
            .and_then(|v| v.as_str())
            .unwrap();

        // Create new format config
        let new_config = MockConfig {
            version: "2.0".to_string(),
            main_directories: vec![old_path.to_string()],
            single_repos: vec![],
        };

        // Save as new format
        save_multi_dir_config(&config_path, &new_config).unwrap();

        // Act: Load migrated config
        let loaded = load_multi_dir_config(&config_path).unwrap();

        // Assert
        assert_eq!(loaded.version, "2.0");
        assert_eq!(loaded.main_directories.len(), 1);
        assert_eq!(loaded.main_directories[0], "/home/user/projects");
    }

    #[test]
    fn test_config_with_single_repos() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("config.toml");

        let config = MockConfig {
            version: "2.0".to_string(),
            main_directories: vec!["/home/user/projects".to_string()],
            single_repos: vec![
                "/opt/external/repo1".to_string(),
                "/opt/external/repo2".to_string(),
            ],
        };

        // Act
        save_multi_dir_config(&config_path, &config).unwrap();
        let loaded = load_multi_dir_config(&config_path).unwrap();

        // Assert
        assert_eq!(loaded.single_repos.len(), 2);
        assert!(loaded
            .single_repos
            .contains(&"/opt/external/repo1".to_string()));
        assert!(loaded
            .single_repos
            .contains(&"/opt/external/repo2".to_string()));
    }

    #[test]
    fn test_add_main_directory_to_existing_config() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("config.toml");

        let mut config = MockConfig {
            version: "2.0".to_string(),
            main_directories: vec!["/home/user/dir1".to_string()],
            single_repos: vec![],
        };
        save_multi_dir_config(&config_path, &config).unwrap();

        // Act: Add new directory
        config.main_directories.push("/home/user/dir2".to_string());
        save_multi_dir_config(&config_path, &config).unwrap();

        // Assert
        let loaded = load_multi_dir_config(&config_path).unwrap();
        assert_eq!(loaded.main_directories.len(), 2);
        assert!(loaded
            .main_directories
            .contains(&"/home/user/dir2".to_string()));
    }

    #[test]
    fn test_remove_main_directory_from_config() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("config.toml");

        let mut config = MockConfig {
            version: "2.0".to_string(),
            main_directories: vec!["/home/user/dir1".to_string(), "/home/user/dir2".to_string()],
            single_repos: vec![],
        };
        save_multi_dir_config(&config_path, &config).unwrap();

        // Act: Remove directory
        config.main_directories.retain(|d| d != "/home/user/dir1");
        save_multi_dir_config(&config_path, &config).unwrap();

        // Assert
        let loaded = load_multi_dir_config(&config_path).unwrap();
        assert_eq!(loaded.main_directories.len(), 1);
        assert!(!loaded
            .main_directories
            .contains(&"/home/user/dir1".to_string()));
        assert!(loaded
            .main_directories
            .contains(&"/home/user/dir2".to_string()));
    }

    #[test]
    fn test_config_validation_preserves_order() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("config.toml");

        let config = MockConfig {
            version: "2.0".to_string(),
            main_directories: vec![
                "/home/user/first".to_string(),
                "/home/user/second".to_string(),
                "/home/user/third".to_string(),
            ],
            single_repos: vec![],
        };
        save_multi_dir_config(&config_path, &config).unwrap();

        // Act
        let loaded = load_multi_dir_config(&config_path).unwrap();

        // Assert: Order should be preserved
        assert_eq!(loaded.main_directories[0], "/home/user/first");
        assert_eq!(loaded.main_directories[1], "/home/user/second");
        assert_eq!(loaded.main_directories[2], "/home/user/third");
    }

    #[test]
    fn test_config_with_both_old_and_new_fields() {
        // Arrange: Config that has both main_directory and main_directories
        let (_temp, config_path) = create_config_with_both_fields();

        // Act: Should load successfully (new format takes precedence)
        let content = std::fs::read_to_string(&config_path).unwrap();
        let config: MockConfig = toml::from_str(&content).unwrap();

        // Assert: main_directories should be used
        assert_eq!(config.main_directories.len(), 2);
        assert!(config
            .main_directories
            .contains(&"/home/user/projects".to_string()));
        assert!(config
            .main_directories
            .contains(&"/home/user/work".to_string()));
    }
}
