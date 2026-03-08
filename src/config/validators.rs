//! Configuration validators

use crate::config::types::{Config, MainDirectoryConfig, SingleRepoConfig};
use crate::error::{AppError, AppResult, ConfigError};
use path_absolutize::Absolutize;
use std::path::{Path, PathBuf};

/// Validate configuration
pub fn validate_config(config: &Config) -> AppResult<()> {
    // Validate main directories (new v2.0 format)
    for (index, dir_config) in config.main_directories.iter().enumerate() {
        validate_main_directory(dir_config, index)?;
    }

    // Validate standalone repositories
    for (index, repo_config) in config.single_repositories.iter().enumerate() {
        validate_single_repository(repo_config, index)?;
    }

    // Validate old main_directory if present (backward compatibility)
    if let Some(ref main_dir) = config.main_directory {
        if !main_dir.as_os_str().is_empty() {
            validate_directory(main_dir)?;
        }
    }

    // Validate editor commands
    if let Some(ref webstorm) = config.editors.webstorm {
        validate_editor_command(webstorm)?;
    }
    if let Some(ref vscode) = config.editors.vscode {
        validate_editor_command(vscode)?;
    }

    // Validate default command
    if let Some(ref cmd) = config.default_command {
        validate_command_name(cmd)?;
    }

    Ok(())
}

/// Validate main directory configuration
pub fn validate_main_directory(
    dir_config: &MainDirectoryConfig,
    _index: usize,
) -> AppResult<PathBuf> {
    validate_directory(&dir_config.path)
}

/// Validate single repository configuration
pub fn validate_single_repository(
    repo_config: &SingleRepoConfig,
    _index: usize,
) -> AppResult<PathBuf> {
    let path = &repo_config.path;

    // 0. Check for empty path
    if path.as_os_str().is_empty() {
        return Err(AppError::Config(ConfigError::PathError(
            "Repository path cannot be empty".to_string(),
        )));
    }

    // 1. Normalize to absolute path
    let abs_path = path
        .absolutize()
        .map_err(|e| ConfigError::PathError(e.to_string()))?
        .to_path_buf();

    // 2. Check existence
    if !abs_path.exists() {
        return Err(AppError::Config(ConfigError::DirectoryNotFound(abs_path)));
    }

    // 3. Check is directory
    if !abs_path.is_dir() {
        return Err(AppError::Config(ConfigError::NotADirectory(abs_path)));
    }

    // 4. Check within home directory (security constraint)
    let home = dirs::home_dir().ok_or(ConfigError::HomeNotFound)?;
    let home = home
        .absolutize()
        .map_err(|e| ConfigError::PathError(format!("Failed to resolve home directory: {}", e)))?;

    if !abs_path.starts_with(home.as_ref()) {
        return Err(AppError::Config(ConfigError::DirectoryOutsideHome(
            abs_path,
        )));
    }

    // 5. Check if it's a git repository (optional validation)
    // Note: We don't require standalone repos to be git repos at config level
    // They will be validated when added through the UI

    Ok(abs_path)
}

/// Validate directory path
pub fn validate_directory(path: &Path) -> AppResult<PathBuf> {
    // 0. Check for empty path before absolutize()
    if path.as_os_str().is_empty() {
        return Err(AppError::Config(ConfigError::PathError(
            "Main directory path cannot be empty".to_string(),
        )));
    }

    // 1. Normalize to absolute path
    let abs_path = path
        .absolutize()
        .map_err(|e| ConfigError::PathError(e.to_string()))?
        .to_path_buf();

    // 2. Check existence
    if !abs_path.exists() {
        return Err(AppError::Config(ConfigError::DirectoryNotFound(abs_path)));
    }

    // 3. Check is directory
    if !abs_path.is_dir() {
        return Err(AppError::Config(ConfigError::NotADirectory(abs_path)));
    }

    // 4. Check within home directory (security constraint)
    let home = dirs::home_dir().ok_or(ConfigError::HomeNotFound)?;
    let home = home
        .absolutize()
        .map_err(|e| ConfigError::PathError(format!("Failed to resolve home directory: {}", e)))?;

    if !abs_path.starts_with(home.as_ref()) {
        return Err(AppError::Config(ConfigError::DirectoryOutsideHome(
            abs_path,
        )));
    }

    // 5. Check read permission
    if abs_path.read_dir().is_err() {
        return Err(AppError::Config(ConfigError::NoReadPermission(
            abs_path.clone(),
        )));
    }

    // Check symlinks if not allowed
    if path.is_symlink() {
        let config = crate::config::load_config().ok();
        if !config.is_some_and(|c| c.security.allow_symlinks) {
            return Err(AppError::Config(ConfigError::SymlinkNotAllowed(abs_path)));
        }
    }

    Ok(abs_path)
}

/// Validate editor command
pub fn validate_editor_command(cmd: &str) -> AppResult<()> {
    // Check whitelist
    if is_in_editor_whitelist(cmd) {
        return Ok(());
    }

    // Check if absolute path
    if cmd.starts_with('/') {
        let path = Path::new(cmd);
        return validate_absolute_command(path);
    }

    // Check in PATH
    if which::which(cmd).is_ok() {
        return Ok(());
    }

    Err(AppError::Config(ConfigError::EditorNotFound(
        cmd.to_string(),
    )))
}

fn validate_command_name(cmd: &str) -> AppResult<()> {
    use crate::constants::{ALLOWED_COMMANDS, ALLOWED_EDITORS};

    if ALLOWED_COMMANDS.contains(&cmd) || ALLOWED_EDITORS.contains(&cmd) {
        return Ok(());
    }

    if which::which(cmd).is_ok() {
        return Ok(());
    }

    Err(AppError::Config(ConfigError::InvalidEditorCommand(
        cmd.to_string(),
    )))
}

fn validate_absolute_command(path: &Path) -> AppResult<()> {
    if !path.exists() {
        return Err(AppError::Config(ConfigError::EditorNotFound(
            path.display().to_string(),
        )));
    }

    if !path.is_file() {
        return Err(AppError::Config(ConfigError::NotADirectory(
            path.to_path_buf(),
        )));
    }

    // Check executable permission (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(path)
            .map_err(|_| ConfigError::EditorNotFound(path.display().to_string()))?;

        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(AppError::Config(ConfigError::EditorNotExecutable(
                path.display().to_string(),
            )));
        }
    }

    Ok(())
}

/// Check if command is in editor whitelist
fn is_in_editor_whitelist(cmd: &str) -> bool {
    use crate::constants::ALLOWED_EDITORS;

    // Check exact match
    if ALLOWED_EDITORS.contains(&cmd) {
        return true;
    }

    // Check base name (for paths like /usr/bin/code)
    if let Some(name) = Path::new(cmd).file_name().and_then(|s| s.to_str()) {
        if ALLOWED_EDITORS.contains(&name) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_directory_exists() {
        // Skip this test if not running in home directory
        // because of security constraint
        if let Some(home) = dirs::home_dir() {
            let result = validate_directory(&home);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_validate_directory_empty_path() {
        let result = validate_directory(Path::new(""));
        assert!(matches!(
            result,
            Err(AppError::Config(ConfigError::PathError(_)))
        ));
    }

    #[test]
    fn test_validate_directory_not_exists() {
        let result = validate_directory(Path::new("/nonexistent/path"));
        assert!(matches!(
            result,
            Err(AppError::Config(ConfigError::DirectoryNotFound(_)))
        ));
    }

    #[test]
    fn test_validate_directory_not_a_dir() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("file.txt");
        fs::write(&file_path, "test").unwrap();

        let result = validate_directory(&file_path);
        assert!(matches!(
            result,
            Err(AppError::Config(ConfigError::NotADirectory(_)))
        ));
    }

    #[test]
    fn test_validate_editor_whitelist() {
        assert!(validate_editor_command("code").is_ok());
        assert!(validate_editor_command("webstorm").is_ok());
        assert!(validate_editor_command("vim").is_ok());
    }

    #[test]
    fn test_validate_command_name() {
        assert!(validate_command_name("claude").is_ok());
        assert!(validate_command_name("cursor").is_ok());
    }
}
