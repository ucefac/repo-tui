//! Repository types

use crate::repo::source::RepoSource;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Git repository information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repository {
    /// Repository name
    pub name: String,

    /// Repository path
    pub path: PathBuf,

    /// Last modified time
    pub last_modified: Option<SystemTime>,

    /// Has uncommitted changes
    pub is_dirty: bool,

    /// Current branch name
    pub branch: Option<String>,

    /// Is a git repository
    pub is_git_repo: bool,

    /// Repository source (main directory or standalone)
    pub source: RepoSource,
}

impl Repository {
    /// Create a new repository from path
    pub fn from_path(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let last_modified = path.metadata().ok().and_then(|m| m.modified().ok());
        let is_git_repo = path.join(".git").exists();

        Self {
            name,
            path,
            last_modified,
            is_dirty: false,
            branch: None,
            is_git_repo,
            source: RepoSource::Standalone,
        }
    }

    /// Create a new repository from path with source
    pub fn from_path_with_source(path: PathBuf, source: RepoSource) -> Self {
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let last_modified = path.metadata().ok().and_then(|m| m.modified().ok());
        let is_git_repo = path.join(".git").exists();

        Self {
            name,
            path,
            last_modified,
            is_dirty: false,
            branch: None,
            is_git_repo,
            source,
        }
    }

    /// Create a new repository from path with git status
    pub fn from_path_with_git_status(path: &Path) -> Self {
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let last_modified = path.metadata().ok().and_then(|m| m.modified().ok());
        let is_git_repo = path.join(".git").exists();

        Self {
            name,
            path: path.to_path_buf(),
            last_modified,
            is_dirty: false,
            branch: None,
            is_git_repo,
            source: RepoSource::Standalone,
        }
    }

    /// Generate display name: @scope/repo-name
    pub fn display_name(&self) -> String {
        format!("@{}/{}", self.source.scope(), self.name)
    }

    /// Get full display info
    pub fn display_info(&self) -> String {
        if self.is_git_repo {
            match &self.branch {
                Some(branch) => format!(
                    "{} [{}]{}",
                    self.display_name(),
                    branch,
                    if self.is_dirty { " *" } else { "" }
                ),
                None => self.display_name(),
            }
        } else {
            format!("{} (not git)", self.display_name())
        }
    }

    /// Test repository for testing
    #[cfg(test)]
    pub fn test_repo() -> Self {
        Self {
            name: "test-repo".to_string(),
            path: PathBuf::from("/tmp/test-repo"),
            last_modified: None,
            is_dirty: false,
            branch: Some("main".to_string()),
            is_git_repo: true,
            source: RepoSource::Standalone,
        }
    }

    /// Test non-git repository for testing
    #[cfg(test)]
    pub fn test_non_git_repo() -> Self {
        Self {
            name: "test-non-git-repo".to_string(),
            path: PathBuf::from("/tmp/test-non-git-repo"),
            last_modified: None,
            is_dirty: false,
            branch: None,
            is_git_repo: false,
            source: RepoSource::Standalone,
        }
    }
}

/// Git status information
#[derive(Debug, Clone, Default)]
pub struct GitStatus {
    /// Has uncommitted changes
    pub is_dirty: bool,

    /// Current branch
    pub branch: Option<String>,

    /// Ahead count
    pub ahead: Option<usize>,

    /// Behind count
    pub behind: Option<usize>,
}

impl GitStatus {
    /// Create a clean status
    pub fn clean() -> Self {
        Self {
            is_dirty: false,
            branch: None,
            ahead: None,
            behind: None,
        }
    }

    /// Create a dirty status
    pub fn dirty() -> Self {
        Self {
            is_dirty: true,
            branch: None,
            ahead: None,
            behind: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_repository_from_path() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test-repo");
        fs::create_dir(&repo_path).unwrap();

        let repo = Repository::from_path(repo_path.clone());
        assert_eq!(repo.name, "test-repo");
        assert_eq!(repo.path, repo_path);
    }

    #[test]
    fn test_git_status() {
        let clean = GitStatus::clean();
        assert!(!clean.is_dirty);

        let dirty = GitStatus::dirty();
        assert!(dirty.is_dirty);
    }
}
