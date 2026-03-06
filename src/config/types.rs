//! Configuration types

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Configuration version for backward compatibility
    pub version: String,

    /// Main directory to scan for repositories
    pub main_directory: PathBuf,

    /// Editor configurations
    #[serde(default)]
    pub editors: EditorConfig,

    /// Default command (optional)
    #[serde(default)]
    pub default_command: Option<String>,

    /// UI configuration
    #[serde(default)]
    pub ui: UiConfig,

    /// Security configuration
    #[serde(default)]
    pub security: SecurityConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: crate::constants::CONFIG_VERSION.to_string(),
            main_directory: PathBuf::new(),
            editors: EditorConfig::default(),
            default_command: None,
            ui: UiConfig::default(),
            security: SecurityConfig::default(),
        }
    }
}

/// Editor path configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EditorConfig {
    /// WebStorm path
    #[serde(default)]
    pub webstorm: Option<String>,

    /// VS Code path
    #[serde(default)]
    pub vscode: Option<String>,

    /// Other editors
    #[serde(flatten)]
    pub others: std::collections::HashMap<String, String>,
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Theme: "dark" or "light"
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Show git status
    #[serde(default = "default_true")]
    pub show_git_status: bool,

    /// Show branch name
    #[serde(default = "default_true")]
    pub show_branch: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            show_git_status: default_true(),
            show_branch: default_true(),
        }
    }
}

fn default_theme() -> String {
    crate::constants::ui::DEFAULT_THEME.to_string()
}

fn default_true() -> bool {
    true
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Allow symlinks
    #[serde(default = "default_false")]
    pub allow_symlinks: bool,

    /// Maximum search depth
    #[serde(default = "default_max_depth")]
    pub max_search_depth: usize,

    /// Additional allowed commands
    #[serde(default)]
    pub allowed_commands: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            allow_symlinks: crate::constants::security::DEFAULT_ALLOW_SYMLINKS,
            max_search_depth: crate::constants::security::DEFAULT_MAX_SEARCH_DEPTH,
            allowed_commands: Vec::new(),
        }
    }
}

fn default_false() -> bool {
    false
}

fn default_max_depth() -> usize {
    crate::constants::security::DEFAULT_MAX_SEARCH_DEPTH
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.version, crate::constants::CONFIG_VERSION);
        assert!(!config.security.allow_symlinks);
        assert_eq!(config.security.max_search_depth, 2);
    }

    #[test]
    fn test_config_serialize() {
        let config = Config::default();
        let serialized = toml::to_string(&config).unwrap();
        assert!(serialized.contains("version"));
        assert!(serialized.contains("main_directory"));
    }

    #[test]
    fn test_config_deserialize() {
        let toml_str = r#"
            version = "1.0"
            main_directory = "/test"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.version, "1.0");
        assert_eq!(config.main_directory, PathBuf::from("/test"));
    }
}
