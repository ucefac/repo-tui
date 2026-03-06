//! Mock filesystem for testing
//!
//! Provides a temporary directory with helper methods to create
//! git repositories and other test structures.

use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Mock filesystem for testing
pub struct MockFs {
    temp_dir: TempDir,
}

impl MockFs {
    /// Create a new mock filesystem
    pub fn new() -> Self {
        Self {
            temp_dir: TempDir::new().unwrap(),
        }
    }

    /// Get the root path of the mock filesystem
    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Create a git repository
    pub fn create_repo(&self, name: &str) -> PathBuf {
        let repo_path = self.temp_dir.path().join(name);
        fs::create_dir(&repo_path).unwrap();

        let git_path = repo_path.join(".git");
        fs::create_dir(&git_path).unwrap();

        repo_path
    }

    /// Create a git repository with a submodule-style .git file
    pub fn create_repo_with_gitfile(&self, name: &str) -> PathBuf {
        let repo_path = self.temp_dir.path().join(name);
        fs::create_dir(&repo_path).unwrap();

        let gitfile_path = repo_path.join(".git");
        fs::write(&gitfile_path, "gitdir: /path/to/actual/git").unwrap();

        repo_path
    }

    /// Create multiple repositories
    pub fn create_repos(&self, names: &[&str]) -> Vec<PathBuf> {
        names.iter().map(|name| self.create_repo(name)).collect()
    }

    /// Create a nested directory structure with repos
    pub fn create_nested_repos(&self, parent: &str, repo_names: &[&str]) -> PathBuf {
        let parent_path = self.temp_dir.path().join(parent);
        fs::create_dir(&parent_path).unwrap();

        for name in repo_names {
            let repo_path = parent_path.join(name);
            fs::create_dir(&repo_path).unwrap();
            fs::create_dir(repo_path.join(".git")).unwrap();
        }

        parent_path
    }

    /// Create a directory that is NOT a git repo
    pub fn create_non_repo_dir(&self, name: &str) -> PathBuf {
        let dir_path = self.temp_dir.path().join(name);
        fs::create_dir(&dir_path).unwrap();
        dir_path
    }

    /// Create a regular file (should be ignored by repo discovery)
    pub fn create_file(&self, name: &str, content: &str) -> PathBuf {
        let file_path = self.temp_dir.path().join(name);
        fs::write(&file_path, content).unwrap();
        file_path
    }

    /// Create a symlink to a directory
    #[cfg(unix)]
    pub fn create_symlink(&self, name: &str, target: &Path) -> PathBuf {
        let link_path = self.temp_dir.path().join(name);
        std::os::unix::fs::symlink(target, &link_path).unwrap();
        link_path
    }

    /// Get the number of repositories found
    pub fn repo_count(&self) -> usize {
        fs::read_dir(self.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().join(".git").exists())
            .count()
    }
}

impl Default for MockFs {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_fs_new() {
        let mock = MockFs::new();
        assert!(mock.path().exists());
    }

    #[test]
    fn test_create_repo() {
        let mock = MockFs::new();
        let repo = mock.create_repo("test-repo");

        assert!(repo.exists());
        assert!(repo.join(".git").exists());
    }

    #[test]
    fn test_create_repos() {
        let mock = MockFs::new();
        let repos = mock.create_repos(&["repo1", "repo2", "repo3"]);

        assert_eq!(repos.len(), 3);
        for repo in &repos {
            assert!(repo.join(".git").exists());
        }
    }

    #[test]
    fn test_create_non_repo_dir() {
        let mock = MockFs::new();
        let dir = mock.create_non_repo_dir("not-a-repo");

        assert!(dir.exists());
        assert!(!dir.join(".git").exists());
    }

    #[test]
    fn test_create_file() {
        let mock = MockFs::new();
        let file = mock.create_file("test.txt", "hello");

        assert!(file.exists());
        assert_eq!(fs::read_to_string(file).unwrap(), "hello");
    }

    #[test]
    fn test_create_repo_with_gitfile() {
        let mock = MockFs::new();
        let repo = mock.create_repo_with_gitfile("submodule");

        assert!(repo.join(".git").exists());
        assert!(repo.join(".git").is_file());
    }

    #[test]
    fn test_repo_count() {
        let mock = MockFs::new();
        mock.create_repo("repo1");
        mock.create_repo("repo2");
        mock.create_non_repo_dir("not-repo");

        assert_eq!(mock.repo_count(), 2);
    }
}
