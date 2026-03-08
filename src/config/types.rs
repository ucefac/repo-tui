//! Configuration types

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Configuration version for backward compatibility
    pub version: String,

    /// Main directory list (v2.0+)
    #[serde(default)]
    pub main_directories: Vec<MainDirectoryConfig>,

    /// Standalone repository list (v2.0+)
    #[serde(default)]
    pub single_repositories: Vec<SingleRepoConfig>,

    /// Backward compatibility: old single main directory (v1.0)
    #[serde(default)]
    pub main_directory: Option<PathBuf>,

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

    /// Favorites configuration
    #[serde(default)]
    pub favorites: FavoritesConfig,

    /// Recent repositories configuration
    #[serde(default)]
    pub recent: RecentConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: crate::constants::CONFIG_VERSION.to_string(),
            main_directories: Vec::new(),
            single_repositories: Vec::new(),
            main_directory: None,
            editors: EditorConfig::default(),
            default_command: None,
            ui: UiConfig::default(),
            security: SecurityConfig::default(),
            favorites: FavoritesConfig::default(),
            recent: RecentConfig::default(),
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
    crate::ui::themes::default_theme_name().to_string()
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

/// Main directory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MainDirectoryConfig {
    /// Directory path
    pub path: PathBuf,

    /// Display name (optional, defaults to directory name)
    #[serde(default)]
    pub display_name: Option<String>,

    /// Scan depth (overrides global setting)
    #[serde(default)]
    pub max_depth: Option<usize>,

    /// Whether this directory is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Single repository configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleRepoConfig {
    /// Repository path
    pub path: PathBuf,

    /// Display name (optional, defaults to directory name)
    #[serde(default)]
    pub display_name: Option<String>,

    /// When the repository was added
    #[serde(default)]
    pub added_at: Option<chrono::DateTime<chrono::Local>>,
}

/// Favorites configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FavoritesConfig {
    /// List of favorite repository paths
    #[serde(default)]
    pub repositories: Vec<String>,
}

impl FavoritesConfig {
    /// Convert to FavoritesStore
    pub fn to_store(&self) -> crate::favorites::FavoritesStore {
        crate::favorites::FavoritesStore::from_paths(self.repositories.clone())
    }

    /// Create from FavoritesStore
    pub fn from_store(store: &crate::favorites::FavoritesStore) -> Self {
        Self {
            repositories: store.get_all().to_vec(),
        }
    }
}

/// Recent repositories configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecentConfig {
    /// List of recent repositories with timestamps
    #[serde(default)]
    pub repositories: Vec<crate::recent::RecentEntry>,
}

impl RecentConfig {
    /// Convert to RecentStore
    pub fn to_store(&self) -> crate::recent::RecentStore {
        crate::recent::RecentStore::from_config_entries(self.repositories.clone())
    }

    /// Create from RecentStore
    pub fn from_store(store: &crate::recent::RecentStore) -> Self {
        Self {
            repositories: store.get_all().to_vec(),
        }
    }
}

impl Config {
    /// Get all enabled main directory paths
    pub fn enabled_main_dirs(&self) -> Vec<&PathBuf> {
        self.main_directories
            .iter()
            .filter(|d| d.enabled)
            .map(|d| &d.path)
            .collect()
    }

    /// Get all enabled main directories with their indices and depths
    pub fn enabled_main_dirs_with_meta(&self) -> Vec<(usize, &PathBuf, Option<usize>)> {
        self.main_directories
            .iter()
            .enumerate()
            .filter(|(_, d)| d.enabled)
            .map(|(i, d)| (i, &d.path, d.max_depth))
            .collect()
    }

    /// Check if migration is needed (old version with single main_directory)
    pub fn needs_migration(&self) -> bool {
        // If main_directories is empty but main_directory has a value, migration is needed
        self.main_directories.is_empty()
            && self.main_directory.is_some()
            && !self.main_directory.as_ref().unwrap().as_os_str().is_empty()
    }

    /// Migrate from v1 to v2 format
    pub fn migrate(&mut self) {
        // Migrate old main_directory to main_directories
        if let Some(old_dir) = self.main_directory.take() {
            if !old_dir.as_os_str().is_empty() {
                // Check if already exists in main_directories (avoid duplicates)
                let exists = self.main_directories.iter().any(|d| d.path == old_dir);

                if !exists {
                    // Add to main_directories as first entry
                    self.main_directories.insert(
                        0,
                        MainDirectoryConfig {
                            path: old_dir,
                            display_name: None,
                            max_depth: None,
                            enabled: true,
                        },
                    );
                }
            }
        }

        // Update version
        self.version = crate::constants::CONFIG_VERSION.to_string();
    }

    /// Get display name for a main directory
    pub fn get_main_dir_display_name(&self, index: usize) -> Option<String> {
        self.main_directories.get(index).map(|d| {
            d.display_name.clone().unwrap_or_else(|| {
                d.path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            })
        })
    }

    /// Add a new main directory
    pub fn add_main_directory(&mut self, path: PathBuf) -> Result<(), crate::error::ConfigError> {
        // Check for duplicates
        if self.main_directories.iter().any(|d| d.path == path) {
            return Err(crate::error::ConfigError::PathError(
                "Directory already exists in main directories".to_string(),
            ));
        }

        let display_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string());

        self.main_directories.push(MainDirectoryConfig {
            path,
            display_name,
            max_depth: None,
            enabled: true,
        });

        Ok(())
    }

    /// Remove a main directory by index
    pub fn remove_main_directory(&mut self, index: usize) -> Result<(), crate::error::ConfigError> {
        if index >= self.main_directories.len() {
            return Err(crate::error::ConfigError::PathError(
                "Invalid directory index".to_string(),
            ));
        }

        self.main_directories.remove(index);
        Ok(())
    }

    /// Toggle main directory enabled state
    pub fn toggle_main_directory(
        &mut self,
        index: usize,
    ) -> Result<bool, crate::error::ConfigError> {
        if let Some(dir) = self.main_directories.get_mut(index) {
            dir.enabled = !dir.enabled;
            Ok(dir.enabled)
        } else {
            Err(crate::error::ConfigError::PathError(
                "Invalid directory index".to_string(),
            ))
        }
    }

    /// Add a standalone repository
    pub fn add_single_repository(
        &mut self,
        path: PathBuf,
    ) -> Result<(), crate::error::ConfigError> {
        // Check for duplicates
        if self.single_repositories.iter().any(|r| r.path == path) {
            return Err(crate::error::ConfigError::PathError(
                "Repository already exists".to_string(),
            ));
        }

        self.single_repositories.push(SingleRepoConfig {
            path,
            display_name: None,
            added_at: Some(chrono::Local::now()),
        });

        Ok(())
    }

    /// Remove a standalone repository by path
    pub fn remove_single_repository(
        &mut self,
        path: &PathBuf,
    ) -> Result<(), crate::error::ConfigError> {
        let initial_len = self.single_repositories.len();
        self.single_repositories.retain(|r| &r.path != path);

        if self.single_repositories.len() == initial_len {
            return Err(crate::error::ConfigError::PathError(
                "Repository not found".to_string(),
            ));
        }

        Ok(())
    }
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
