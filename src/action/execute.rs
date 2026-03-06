//! Action execution

use crate::action::{validators, Action};
use crate::error::{ActionError, AppError, AppResult};
use crate::repo::Repository;
use std::path::Path;
use std::process::Command;

/// Execute an action on a repository
pub fn execute_action(action: &Action, repo: &Repository) -> AppResult<()> {
    // Validate action first
    validators::validate_action(action)?;

    // Validate repository path
    validate_repo_path(&repo.path)?;

    match action {
        Action::CdAndCloud => {
            execute_cd_and_cloud(&repo.path)?;
        }

        Action::OpenWebStorm => {
            execute_editor("webstorm", &repo.path)?;
        }

        Action::OpenVsCode => {
            execute_editor("code", &repo.path)?;
        }

        Action::OpenFileManager => {
            open_file_manager(&repo.path)?;
        }
    }

    Ok(())
}

/// Validate repository path (security check)
fn validate_repo_path(path: &Path) -> AppResult<()> {
    // Check path exists
    if !path.exists() {
        return Err(AppError::Action(ActionError::PathValidationFailed(
            "Repository path does not exist".to_string(),
        )));
    }

    // Check path is absolute or within allowed directory
    if !path.is_absolute() {
        return Err(AppError::Action(ActionError::PathValidationFailed(
            "Repository path must be absolute".to_string(),
        )));
    }

    // Check for unsafe characters (basic check)
    if let Some(path_str) = path.to_str() {
        // Check for shell injection attempts
        if path_str.contains(';')
            || path_str.contains('|')
            || path_str.contains('&')
            || path_str.contains('$')
            || path_str.contains('`')
        {
            return Err(AppError::Action(ActionError::UnsafePath(
                "Path contains unsafe characters".to_string(),
            )));
        }
    }

    Ok(())
}

/// Execute cd + cloud (claude)
///
/// Security: Uses current_dir() instead of shell cd command
fn execute_cd_and_cloud(repo_path: &Path) -> AppResult<()> {
    // Use which to get full path
    let claude_path =
        which::which("claude").map_err(|_| ActionError::CommandNotFound("claude".to_string()))?;

    // Execute with current_dir (safer than shell cd)
    let status = Command::new(claude_path).current_dir(repo_path).status()?;

    if !status.success() {
        return Err(AppError::Action(ActionError::ExecutionFailed(format!(
            "claude exited with code: {:?}",
            status.code()
        ))));
    }

    Ok(())
}

/// Execute editor with repository path
fn execute_editor(editor: &str, repo_path: &Path) -> AppResult<()> {
    // Try to find full path
    let editor_path = which::which(editor).unwrap_or_else(|_| std::path::PathBuf::from(editor));

    // Execute with arg (automatically escapes special characters)
    let status = Command::new(editor_path).arg(repo_path).status()?;

    if !status.success() {
        return Err(AppError::Action(ActionError::ExecutionFailed(format!(
            "{} exited with code: {:?}",
            editor,
            status.code()
        ))));
    }

    Ok(())
}

/// Open file manager
fn open_file_manager(repo_path: &Path) -> AppResult<()> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(repo_path).status()?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(repo_path).status()?;
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer").arg(repo_path).status()?;
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        return Err(AppError::Action(ActionError::ExecutionFailed(format!(
            "Unsupported platform for file manager"
        ))));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_repo_path() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test-repo");
        fs::create_dir(&repo_path).unwrap();

        assert!(validate_repo_path(&repo_path).is_ok());
    }

    #[test]
    fn test_validate_repo_path_not_exists() {
        let fake_path = Path::new("/nonexistent/path");
        assert!(validate_repo_path(fake_path).is_err());
    }

    #[test]
    fn test_validate_repo_path_unsafe() {
        let unsafe_path = Path::new("/tmp/test;rm -rf /");
        assert!(validate_repo_path(unsafe_path).is_err());
    }
}
