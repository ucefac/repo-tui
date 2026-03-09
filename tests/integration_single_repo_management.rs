//! Single repository management integration tests
//!
//! Tests for adding and managing individual repositories

use std::collections::HashSet;
use std::path::PathBuf;
use tempfile::TempDir;

/// Represents a single repository entry
#[derive(Debug, Clone, PartialEq, Eq)]
struct SingleRepo {
    path: PathBuf,
    display_name: Option<String>,
    added_at: Option<String>, // Using string for simplicity
}

/// Manages single repositories
#[derive(Debug, Clone)]
struct SingleRepoManager {
    repos: Vec<SingleRepo>,
}

impl SingleRepoManager {
    fn new() -> Self {
        Self { repos: Vec::new() }
    }

    /// Add a single repository (must be a valid git repo)
    fn add_repo(&mut self, path: PathBuf) -> Result<(), SingleRepoError> {
        // Validate: Check if path exists
        if !path.exists() {
            return Err(SingleRepoError::PathNotFound);
        }

        // Validate: Check if it's a git repository
        if !path.join(".git").exists() {
            return Err(SingleRepoError::NotGitRepository);
        }

        // Validate: Check for duplicates
        if self.repos.iter().any(|r| r.path == path) {
            return Err(SingleRepoError::DuplicatePath);
        }

        let repo = SingleRepo {
            path,
            display_name: None,
            added_at: Some("2024-01-01T00:00:00".to_string()), // Mock timestamp
        };

        self.repos.push(repo);
        Ok(())
    }

    /// Remove a single repository by path
    fn remove_repo(&mut self, path: &PathBuf) -> Result<(), SingleRepoError> {
        let index = self
            .repos
            .iter()
            .position(|r| &r.path == path)
            .ok_or(SingleRepoError::NotFound)?;

        self.repos.remove(index);
        Ok(())
    }

    /// Get display name for a repo (@scope/name format)
    fn get_display_name(&self, repo: &SingleRepo) -> String {
        let scope = "single";
        let name = repo
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        if let Some(ref custom_name) = repo.display_name {
            format!("@{}/{}", scope, custom_name)
        } else {
            format!("@{}/{}", scope, name)
        }
    }

    /// Get all repo paths
    fn get_paths(&self) -> Vec<&PathBuf> {
        self.repos.iter().map(|r| &r.path).collect()
    }

    /// Check if a path is already added
    fn contains(&self, path: &PathBuf) -> bool {
        self.repos.iter().any(|r| r.path == *path)
    }

    fn len(&self) -> usize {
        self.repos.len()
    }

    fn is_empty(&self) -> bool {
        self.repos.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
enum SingleRepoError {
    PathNotFound,
    NotGitRepository,
    DuplicatePath,
    NotFound,
}

/// Check if a path is a valid git repository
fn is_valid_git_repo(path: &PathBuf) -> bool {
    path.exists() && path.join(".git").exists()
}

/// Extract scope from path (parent directory name)
fn extract_scope(path: &PathBuf) -> String {
    path.parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_mock_repo(parent: &std::path::Path, name: &str) -> PathBuf {
        let repo_path = parent.join(name);
        std::fs::create_dir_all(&repo_path).unwrap();
        std::fs::create_dir(repo_path.join(".git")).unwrap();
        repo_path
    }

    #[test]
    fn test_add_valid_git_repository() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let repo_path = create_mock_repo(temp.path(), "my-repo");
        let mut manager = SingleRepoManager::new();

        // Act
        let result = manager.add_repo(repo_path.clone());

        // Assert
        assert!(result.is_ok());
        assert_eq!(manager.len(), 1);
        assert!(manager.contains(&repo_path));
    }

    #[test]
    fn test_add_nonexistent_path_fails() {
        // Arrange
        let mut manager = SingleRepoManager::new();
        let nonexistent = PathBuf::from("/does/not/exist");

        // Act
        let result = manager.add_repo(nonexistent);

        // Assert
        assert!(matches!(result, Err(SingleRepoError::PathNotFound)));
        assert!(manager.is_empty());
    }

    #[test]
    fn test_add_non_git_directory_fails() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let not_a_repo = temp.path().join("not-a-repo");
        std::fs::create_dir(&not_a_repo).unwrap();

        let mut manager = SingleRepoManager::new();

        // Act
        let result = manager.add_repo(not_a_repo);

        // Assert
        assert!(matches!(result, Err(SingleRepoError::NotGitRepository)));
        assert!(manager.is_empty());
    }

    #[test]
    fn test_add_duplicate_repository_fails() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let repo_path = create_mock_repo(temp.path(), "my-repo");
        let mut manager = SingleRepoManager::new();

        manager.add_repo(repo_path.clone()).unwrap();

        // Act: Try to add again
        let result = manager.add_repo(repo_path.clone());

        // Assert
        assert!(matches!(result, Err(SingleRepoError::DuplicatePath)));
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_remove_repository() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let repo1 = create_mock_repo(temp.path(), "repo1");
        let repo2 = create_mock_repo(temp.path(), "repo2");

        let mut manager = SingleRepoManager::new();
        manager.add_repo(repo1.clone()).unwrap();
        manager.add_repo(repo2.clone()).unwrap();

        // Act
        let result = manager.remove_repo(&repo1);

        // Assert
        assert!(result.is_ok());
        assert_eq!(manager.len(), 1);
        assert!(!manager.contains(&repo1));
        assert!(manager.contains(&repo2));
    }

    #[test]
    fn test_remove_nonexistent_repository() {
        // Arrange
        let mut manager = SingleRepoManager::new();
        let nonexistent = PathBuf::from("/does/not/exist");

        // Act
        let result = manager.remove_repo(&nonexistent);

        // Assert
        assert!(matches!(result, Err(SingleRepoError::NotFound)));
    }

    #[test]
    fn test_display_name_format() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let repo_path = create_mock_repo(temp.path(), "my-awesome-app");

        let manager = SingleRepoManager::new();
        let repo = SingleRepo {
            path: repo_path,
            display_name: None,
            added_at: None,
        };

        // Act
        let display_name = manager.get_display_name(&repo);

        // Assert
        assert_eq!(display_name, "@single/my-awesome-app");
    }

    #[test]
    fn test_display_name_with_custom_name() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let repo_path = create_mock_repo(temp.path(), "repo123");

        let manager = SingleRepoManager::new();
        let repo = SingleRepo {
            path: repo_path,
            display_name: Some("My Custom Name".to_string()),
            added_at: None,
        };

        // Act
        let display_name = manager.get_display_name(&repo);

        // Assert
        assert_eq!(display_name, "@single/My Custom Name");
    }

    #[test]
    fn test_scope_extraction() {
        // Arrange
        let test_cases = vec![
            ("/home/user/projects/app1", "projects"),
            ("/work/repos/service", "repos"),
            ("/opt/external/lib", "external"),
        ];

        for (path_str, expected_scope) in test_cases {
            let path = PathBuf::from(path_str);
            let scope = extract_scope(&path);
            assert_eq!(scope, expected_scope, "Path: {}", path_str);
        }
    }

    #[test]
    fn test_is_valid_git_repo() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let valid_repo = create_mock_repo(temp.path(), "valid");
        let not_a_repo = temp.path().join("not-a-repo");
        std::fs::create_dir(&not_a_repo).unwrap();

        // Act & Assert
        assert!(is_valid_git_repo(&valid_repo));
        assert!(!is_valid_git_repo(&not_a_repo));
        assert!(!is_valid_git_repo(&PathBuf::from("/nonexistent")));
    }

    #[test]
    fn test_add_multiple_repositories() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let mut manager = SingleRepoManager::new();

        // Act
        for i in 1..=5 {
            let repo_path = create_mock_repo(temp.path(), &format!("repo{}", i));
            manager.add_repo(repo_path).unwrap();
        }

        // Assert
        assert_eq!(manager.len(), 5);
    }

    #[test]
    fn test_get_paths_returns_all() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let mut manager = SingleRepoManager::new();

        let paths: Vec<_> = (1..=3)
            .map(|i| create_mock_repo(temp.path(), &format!("repo{}", i)))
            .collect();

        for path in &paths {
            manager.add_repo(path.clone()).unwrap();
        }

        // Act
        let retrieved_paths = manager.get_paths();

        // Assert
        assert_eq!(retrieved_paths.len(), 3);
        for path in &paths {
            assert!(retrieved_paths.contains(&path));
        }
    }

    #[test]
    fn test_scope_extraction_nested() {
        // Arrange
        let test_cases = vec![
            ("/home/user/company/dept/team/projects/app", "projects"),
            ("/a/b/c/d/e/f/repo", "f"),
        ];

        for (path_str, expected_scope) in test_cases {
            let path = PathBuf::from(path_str);
            let scope = extract_scope(&path);
            assert_eq!(
                scope, expected_scope,
                "Path {} should have scope {}",
                path_str, expected_scope
            );
        }
    }

    #[test]
    fn test_add_similar_paths() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let mut manager = SingleRepoManager::new();

        // These should all be considered different
        let repo1 = create_mock_repo(temp.path(), "repo");
        let repo2 = create_mock_repo(temp.path().join("subdir").as_path(), "repo");

        // Act
        manager.add_repo(repo1.clone()).unwrap();
        let result = manager.add_repo(repo2.clone());

        // Assert
        assert!(result.is_ok());
        assert_eq!(manager.len(), 2);
    }

    #[test]
    fn test_empty_manager() {
        // Arrange
        let manager = SingleRepoManager::new();

        // Assert
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
        assert!(manager.get_paths().is_empty());
    }

    #[test]
    fn test_repo_with_gitfile() {
        // Arrange: Submodule-style .git file
        let temp = TempDir::new().unwrap();
        let repo_path = temp.path().join("submodule-repo");
        std::fs::create_dir(&repo_path).unwrap();
        std::fs::write(repo_path.join(".git"), "gitdir: /path/to/git").unwrap();

        let mut manager = SingleRepoManager::new();

        // Act
        let result = manager.add_repo(repo_path.clone());

        // Assert: Should be recognized as git repo
        assert!(result.is_ok());
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_unicode_repo_names() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let repo1 = create_mock_repo(temp.path(), "项目");
        let repo2 = create_mock_repo(temp.path(), "プロジェクト");

        let mut manager = SingleRepoManager::new();

        // Act
        manager.add_repo(repo1).unwrap();
        manager.add_repo(repo2).unwrap();

        // Assert
        assert_eq!(manager.len(), 2);
    }

    #[test]
    fn test_special_characters_in_repo_names() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let repo = create_mock_repo(temp.path(), "my-app_v2.0+test");

        let manager = SingleRepoManager::new();
        let single_repo = SingleRepo {
            path: repo,
            display_name: None,
            added_at: None,
        };

        // Act
        let display_name = manager.get_display_name(&single_repo);

        // Assert
        assert!(display_name.contains("my-app_v2.0+test"));
    }

    #[test]
    fn test_add_and_remove_sequence() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let mut manager = SingleRepoManager::new();

        let repo1 = create_mock_repo(temp.path(), "repo1");
        let repo2 = create_mock_repo(temp.path(), "repo2");
        let repo3 = create_mock_repo(temp.path(), "repo3");

        // Act: Add all
        manager.add_repo(repo1.clone()).unwrap();
        manager.add_repo(repo2.clone()).unwrap();
        manager.add_repo(repo3.clone()).unwrap();
        assert_eq!(manager.len(), 3);

        // Act: Remove middle
        manager.remove_repo(&repo2).unwrap();
        assert_eq!(manager.len(), 2);
        assert!(!manager.contains(&repo2));

        // Act: Remove first
        manager.remove_repo(&repo1).unwrap();
        assert_eq!(manager.len(), 1);

        // Act: Remove last
        manager.remove_repo(&repo3).unwrap();
        assert!(manager.is_empty());
    }

    #[test]
    fn test_repo_path_uniqueness() {
        // Arrange: Two different paths that might look similar
        let temp = TempDir::new().unwrap();
        let mut manager = SingleRepoManager::new();

        #[cfg(unix)]
        {
            let repo1 = create_mock_repo(temp.path(), "repo");
            let repo2 = temp.path().join("repo");

            // Add first
            manager.add_repo(repo1.clone()).unwrap();

            // Try to add the same path again
            let result = manager.add_repo(repo1.clone());
            assert!(matches!(result, Err(SingleRepoError::DuplicatePath)));
        }

        #[cfg(windows)]
        {
            // On Windows, path comparison is case-insensitive
            // This test would need platform-specific handling
        }
    }

    #[test]
    fn test_manager_contains_check() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let repo = create_mock_repo(temp.path(), "my-repo");
        let not_added = create_mock_repo(temp.path(), "not-added");

        let mut manager = SingleRepoManager::new();
        manager.add_repo(repo.clone()).unwrap();

        // Act & Assert
        assert!(manager.contains(&repo));
        assert!(!manager.contains(&not_added));
    }

    #[test]
    fn test_display_name_edge_cases() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let manager = SingleRepoManager::new();

        // Empty name fallback
        let repo_with_empty_name = SingleRepo {
            path: PathBuf::from("/"), // Root has no file_name
            display_name: None,
            added_at: None,
        };
        assert_eq!(
            manager.get_display_name(&repo_with_empty_name),
            "@single/unknown"
        );

        // Custom empty name
        let repo_with_custom_empty = SingleRepo {
            path: temp.path().join("test"),
            display_name: Some("".to_string()),
            added_at: None,
        };
        assert_eq!(
            manager.get_display_name(&repo_with_custom_empty),
            "@single/"
        );
    }
}
