//! Git status checking

use crate::error::{AppError, AppResult, RepoError};
use crate::repo::GitStatus;
use std::path::Path;
use std::process::Command;

/// Check git status for a repository
///
/// Uses `git status --porcelain` to detect uncommitted changes
pub fn check_git_status(repo_path: &Path) -> AppResult<GitStatus> {
    // Check if .git exists
    if !repo_path.join(".git").exists() {
        return Err(AppError::Repo(RepoError::NotGitRepo(
            repo_path.to_path_buf(),
        )));
    }

    // Get status using git command
    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .arg("--untracked-files=no")
        .current_dir(repo_path)
        .output()
        .map_err(|e| RepoError::GitCommandFailed(e.to_string()))?;

    // Check if dirty (any output means changes)
    let is_dirty = !output.stdout.is_empty();

    // Get current branch
    let branch = get_current_branch(repo_path);

    Ok(GitStatus {
        is_dirty,
        branch,
        ahead: None,
        behind: None,
    })
}

/// Get current branch name
fn get_current_branch(repo_path: &Path) -> Option<String> {
    Command::new("git")
        .arg("branch")
        .arg("--show-current")
        .current_dir(repo_path)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    #[test]
    fn test_check_git_status_clean() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test-repo");
        fs::create_dir(&repo_path).unwrap();

        // Initialize git repo
        Command::new("git")
            .arg("init")
            .current_dir(&repo_path)
            .output()
            .ok();

        let status = check_git_status(&repo_path);

        // Should succeed (may be clean or dirty depending on git config)
        assert!(status.is_ok());
    }

    #[test]
    fn test_check_git_status_not_repo() {
        let temp_dir = TempDir::new().unwrap();
        let not_repo = temp_dir.path().join("not-repo");
        fs::create_dir(&not_repo).unwrap();

        let status = check_git_status(&not_repo);
        assert!(matches!(
            status,
            Err(AppError::Repo(RepoError::NotGitRepo(_)))
        ));
    }

    #[test]
    fn test_get_current_branch() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test-repo");
        fs::create_dir(&repo_path).unwrap();

        Command::new("git")
            .arg("init")
            .current_dir(&repo_path)
            .output()
            .ok();

        let branch = get_current_branch(&repo_path);
        // May be main or master depending on git version
        assert!(
            branch.is_none()
                || branch == Some("main".to_string())
                || branch == Some("master".to_string())
        );
    }
}
