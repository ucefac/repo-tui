//! Configuration loading

use crate::config::types::Config;
use crate::config::validators::validate_config;
use crate::constants::{CONFIG_DIR_NAME, CONFIG_FILE_NAME, MAX_SUPPORTED_VERSION};
use crate::error::{AppError, AppResult, ConfigError};
use std::fs;
use std::path::{Path, PathBuf};

/// Get configuration directory path
pub fn get_config_dir() -> AppResult<PathBuf> {
    // Use ~/.config/repotui on all platforms for consistency
    let home_dir = dirs::home_dir().ok_or(ConfigError::HomeNotFound)?;
    let config_dir = home_dir.join(".config").join(CONFIG_DIR_NAME);

    // Create directory if not exists
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).map_err(|e| {
            ConfigError::PathError(format!("Failed to create config directory: {}", e))
        })?;
    }

    Ok(config_dir)
}

/// Get configuration file path
pub fn get_config_path() -> AppResult<PathBuf> {
    Ok(get_config_dir()?.join(CONFIG_FILE_NAME))
}

/// Load configuration from file
pub fn load_config() -> AppResult<Config> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        return Err(AppError::Config(ConfigError::NotFound(config_path)));
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| ConfigError::PathError(format!("Failed to read config file: {}", e)))?;

    let config: Config = toml::from_str(&content).map_err(ConfigError::from)?;

    // Validate configuration
    validate_config(&config).map_err(|e| {
        // If validation fails, backup and return error
        let _ = backup_corrupted_config(&config_path);
        e
    })?;

    Ok(config)
}

/// Check if version is supported
fn is_version_supported(version: &str) -> bool {
    // Parse version numbers
    let parts: Vec<&str> = version.split('.').collect();
    if parts.is_empty() {
        return false;
    }

    let major: u32 = parts[0].parse().unwrap_or(0);
    let max_parts: Vec<&str> = MAX_SUPPORTED_VERSION.split('.').collect();
    let max_major: u32 = max_parts[0].parse().unwrap_or(0);

    // Support same major version
    major == max_major
}

/// Check if version is newer than supported (downgrade scenario)
fn is_version_newer_than_supported(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    let max_parts: Vec<&str> = MAX_SUPPORTED_VERSION.split('.').collect();

    if parts.is_empty() || max_parts.is_empty() {
        return false;
    }

    let major: u32 = parts[0].parse().unwrap_or(0);
    let max_major: u32 = max_parts[0].parse().unwrap_or(0);

    major > max_major
}

/// Load configuration with version check and auto-migration
pub fn load_config_with_version_check() -> AppResult<Config> {
    let mut config = load_config()?;

    // Check version compatibility
    if !is_version_supported(&config.version) && is_version_newer_than_supported(&config.version) {
        return Err(ConfigError::VersionTooNew {
            current: config.version,
            max_supported: MAX_SUPPORTED_VERSION.to_string(),
            message: "Please upgrade repotui or delete the configuration file".to_string(),
        }
        .into());
    }

    // Auto-migrate if needed
    if config.needs_migration() {
        tracing::info!("Auto-migrating config from version {}", config.version);
        config.migrate();
        save_config(&config)?;
    }

    Ok(config)
}

/// Load configuration or return error for directory chooser
pub fn load_or_create_config() -> AppResult<Config> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        // Return error to trigger directory chooser UI
        return Err(AppError::Config(ConfigError::NotFound(config_path)));
    }

    match load_config_with_version_check() {
        Ok(config) => {
            // Check if we have at least one main directory or standalone repo
            if config.main_directories.is_empty() && config.single_repositories.is_empty() {
                // Also check old main_directory field for backward compatibility
                if config.main_directory.is_none()
                    || config
                        .main_directory
                        .as_ref()
                        .unwrap()
                        .as_os_str()
                        .is_empty()
                {
                    return Err(AppError::Config(ConfigError::NotFound(config_path)));
                }
            }
            Ok(config)
        }
        Err(AppError::Config(ConfigError::NotFound(_))) => {
            // Return error to trigger directory chooser UI
            Err(AppError::Config(ConfigError::NotFound(config_path)))
        }
        Err(e) => Err(e),
    }
}

/// Save configuration to file
pub fn save_config(config: &Config) -> AppResult<()> {
    let config_path = get_config_path()?;
    let config_dir = config_path
        .parent()
        .ok_or_else(|| ConfigError::PathError("Config path has no parent".to_string()))?;

    // Ensure directory exists
    if !config_dir.exists() {
        fs::create_dir_all(config_dir).map_err(|e| {
            ConfigError::PathError(format!("Failed to create config directory: {}", e))
        })?;
    }

    // Serialize to TOML
    let content = toml::to_string_pretty(config).map_err(ConfigError::from)?;

    // Write to file
    fs::write(&config_path, &content)
        .map_err(|e| ConfigError::PathError(format!("Failed to write config file: {}", e)))?;

    // Set file permissions (Unix: chmod 600)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&config_path)?.permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(&config_path, permissions)?;
    }

    Ok(())
}

/// Backup corrupted configuration file
pub fn backup_corrupted_config(config_path: &Path) -> AppResult<PathBuf> {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_path = config_path.with_extension(format!("toml.backup.{}", timestamp));

    fs::copy(config_path, &backup_path)
        .map_err(|e| ConfigError::PathError(format!("Failed to backup config: {}", e)))?;

    Ok(backup_path)
}

/// Check if configuration exists
pub fn config_exists() -> bool {
    get_config_path().map(|path| path.exists()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_config_dir() {
        let dir = get_config_dir().unwrap();
        assert!(dir.ends_with(CONFIG_DIR_NAME));
    }

    #[test]
    fn test_config_exists() {
        // Just check the function works
        let _ = config_exists();
    }
}

#[cfg(test)]
mod debug_tests {
    use super::*;

    #[test]
    fn test_debug_home_dir() {
        println!("HOME env: {:?}", std::env::var("HOME").ok());
        println!("dirs::home_dir(): {:?}", dirs::home_dir());
        println!("dirs::config_dir(): {:?}", dirs::config_dir());

        let config_dir = get_config_dir().unwrap();
        println!("Config dir: {:?}", config_dir);

        let config_path = get_config_path().unwrap();
        println!("Config path: {:?}", config_path);
        println!("Config exists: {}", config_path.exists());
    }
}
