//! Repository discovery

use crate::error::{RepoError, Result};
use crate::repo::Repository;
use std::fs;
use std::path::Path;

/// Discover git repositories in a directory
///
/// Scans immediate subdirectories for git repositories
pub fn discover_repositories(main_dir: &Path) -> Result<Vec<Repository>> {
    let mut repos = Vec::new();

    let entries = fs::read_dir(main_dir).map_err(|e| {
        RepoError::ScanFailed(format!(
            "Failed to read directory {}: {}",
            main_dir.display(),
            e
        ))
    })?;

    for entry in entries {
        let entry =
            entry.map_err(|e| RepoError::ScanFailed(format!("Failed to read entry: {}", e)))?;

        let path = entry.path();

        // Skip if not a directory
        if !path.is_dir() {
            continue;
        }

        // Check if it's a git repository
        if is_git_repository(&path) {
            let repo = Repository::from_path(path);
            repos.push(repo);
        }
    }

    // Sort by name
    repos.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(repos)
}

/// Check if a path is a git repository
fn is_git_repository(path: &Path) -> bool {
    path.join(".git").exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_discover_repositories() {
        let temp_dir = TempDir::new().unwrap();

        // Create mock git repos
        let repo1 = temp_dir.path().join("repo1");
        let repo2 = temp_dir.path().join("repo2");
        let not_repo = temp_dir.path().join("not-repo");

        fs::create_dir(&repo1).unwrap();
        fs::create_dir(&repo2).unwrap();
        fs::create_dir(&not_repo).unwrap();

        // Add .git directories
        fs::create_dir(repo1.join(".git")).unwrap();
        fs::create_dir(repo2.join(".git")).unwrap();

        let repos = discover_repositories(temp_dir.path()).unwrap();

        assert_eq!(repos.len(), 2);
        assert!(repos.iter().any(|r| r.name == "repo1"));
        assert!(repos.iter().any(|r| r.name == "repo2"));
    }

    #[test]
    fn test_is_git_repository() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test-repo");
        fs::create_dir(&repo_path).unwrap();
        fs::create_dir(repo_path.join(".git")).unwrap();

        assert!(is_git_repository(&repo_path));

        let not_repo = temp_dir.path().join("not-repo");
        fs::create_dir(&not_repo).unwrap();
        assert!(!is_git_repository(&not_repo));
    }

    #[test]
    fn test_discover_empty_directory() {
        let temp_dir = TempDir::new().unwrap();

        let repos = discover_repositories(temp_dir.path()).unwrap();
        assert!(repos.is_empty());
    }

    #[test]
    fn test_discover_sorting() {
        let temp_dir = TempDir::new().unwrap();

        // Create repos in non-alphabetical order
        let repo_c = temp_dir.path().join("charlie");
        let repo_a = temp_dir.path().join("alpha");
        let repo_b = temp_dir.path().join("bravo");

        fs::create_dir(&repo_c).unwrap();
        fs::create_dir(&repo_a).unwrap();
        fs::create_dir(&repo_b).unwrap();

        fs::create_dir(repo_c.join(".git")).unwrap();
        fs::create_dir(repo_a.join(".git")).unwrap();
        fs::create_dir(repo_b.join(".git")).unwrap();

        let repos = discover_repositories(temp_dir.path()).unwrap();

        assert_eq!(repos.len(), 3);
        assert_eq!(repos[0].name, "alpha");
        assert_eq!(repos[1].name, "bravo");
        assert_eq!(repos[2].name, "charlie");
    }

    #[test]
    fn test_discover_with_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create a repo
        let repo = temp_dir.path().join("repo1");
        fs::create_dir(&repo).unwrap();
        fs::create_dir(repo.join(".git")).unwrap();

        // Create a regular file (should be ignored)
        fs::write(temp_dir.path().join("file.txt"), "content").unwrap();

        let repos = discover_repositories(temp_dir.path()).unwrap();
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].name, "repo1");
    }

    #[test]
    fn test_discover_gitfile() {
        let temp_dir = TempDir::new().unwrap();

        // Create repo with .git file (submodule style)
        let repo = temp_dir.path().join("submodule");
        fs::create_dir(&repo).unwrap();
        fs::write(repo.join(".git"), "gitdir: /path/to/git").unwrap();

        let repos = discover_repositories(temp_dir.path()).unwrap();
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].name, "submodule");
    }

    #[test]
    fn test_discover_skips_nested_repos() {
        let temp_dir = TempDir::new().unwrap();

        // Create parent repo
        let parent = temp_dir.path().join("parent");
        fs::create_dir(&parent).unwrap();
        fs::create_dir(parent.join(".git")).unwrap();

        // Create nested repo (should not be discovered at this level)
        let nested = parent.join("child");
        fs::create_dir(&nested).unwrap();
        fs::create_dir(nested.join(".git")).unwrap();

        let repos = discover_repositories(temp_dir.path()).unwrap();
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].name, "parent");
    }
}
