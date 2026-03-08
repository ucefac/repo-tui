//! Configuration test helpers
//!
//! Provides utilities for creating and managing test configurations

use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Create a temporary directory with a multi-directory config
pub fn create_temp_config_with_dirs(dirs: &[&str]) -> (TempDir, PathBuf) {
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
pub fn create_old_format_config() -> (TempDir, PathBuf) {
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
pub fn create_empty_main_dirs_config() -> (TempDir, PathBuf) {
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
pub fn create_config_with_duplicates() -> (TempDir, PathBuf) {
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

/// Create a config with empty path strings
pub fn create_config_with_empty_paths() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let config_content = r#"version = "2.0"
main_directories = [
    "",
    "/home/user/projects"
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
pub fn create_config_with_both_fields() -> (TempDir, PathBuf) {
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
pub fn read_config_version(path: &Path) -> String {
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

/// Assert that a config file contains main_directories array
pub fn assert_config_has_main_directories(path: &Path) {
    let content = std::fs::read_to_string(path).unwrap();
    assert!(
        content.contains("main_directories"),
        "Config should contain main_directories field"
    );
}

/// Assert that a config file is valid TOML
pub fn assert_config_valid(path: &Path) {
    let content = std::fs::read_to_string(path).unwrap();
    let result: Result<toml::Value, _> = content.parse();
    assert!(result.is_ok(), "Config should be valid TOML: {:?}", result);
}

/// Count the number of main directories in a config file
pub fn count_main_directories(path: &Path) -> usize {
    let content = std::fs::read_to_string(path).unwrap();
    let value: toml::Value = content.parse().unwrap();

    if let Some(arr) = value.get("main_directories").and_then(|v| v.as_array()) {
        arr.len()
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_temp_config_with_dirs() {
        let (_temp, path) =
            create_temp_config_with_dirs(&["/home/user/projects", "/home/user/work"]);

        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("/home/user/projects"));
        assert!(content.contains("/home/user/work"));
    }

    #[test]
    fn test_create_old_format_config() {
        let (_temp, path) = create_old_format_config();

        assert!(path.exists());
        let version = read_config_version(&path);
        assert_eq!(version, "1.0");
    }

    #[test]
    fn test_read_config_version() {
        let (_temp, path) = create_temp_config_with_dirs(&["/test"]);
        let version = read_config_version(&path);
        assert_eq!(version, "2.0");
    }

    #[test]
    fn test_count_main_directories() {
        let (_temp, path) = create_temp_config_with_dirs(&[
            "/home/user/projects",
            "/home/user/work",
            "/home/user/personal",
        ]);

        let count = count_main_directories(&path);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_create_empty_main_dirs_config() {
        let (_temp, path) = create_empty_main_dirs_config();
        let count = count_main_directories(&path);
        assert_eq!(count, 0);
    }
}
