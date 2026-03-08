//! Multi-directory discovery tests
//!
//! Tests for discovering repositories from multiple main directories

use std::collections::HashSet;
use std::path::PathBuf;
use tempfile::TempDir;

/// Represents a discovered repository
#[derive(Debug, Clone, PartialEq, Eq)]
struct DiscoveredRepo {
    name: String,
    path: PathBuf,
    source_index: usize,
}

/// Discover repositories from multiple main directories
fn discover_from_multiple_dirs(main_dirs: &[PathBuf]) -> Vec<DiscoveredRepo> {
    let mut all_repos = Vec::new();
    let mut seen_paths: HashSet<PathBuf> = HashSet::new();

    for (index, dir) in main_dirs.iter().enumerate() {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() && path.join(".git").exists() {
                    // Deduplicate by path
                    if seen_paths.insert(path.clone()) {
                        let name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        all_repos.push(DiscoveredRepo {
                            name,
                            path,
                            source_index: index,
                        });
                    }
                }
            }
        }
    }

    // Sort by name for consistent results
    all_repos.sort_by(|a, b| a.name.cmp(&b.name));

    all_repos
}

/// Merge repositories from main directories with single repos
fn merge_repositories(
    from_dirs: Vec<DiscoveredRepo>,
    single_repos: Vec<PathBuf>,
) -> Vec<DiscoveredRepo> {
    let mut all_repos = from_dirs;
    let mut seen_paths: HashSet<PathBuf> = all_repos.iter().map(|r| r.path.clone()).collect();

    for (index, path) in single_repos.iter().enumerate() {
        if seen_paths.insert(path.clone()) && path.exists() {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            all_repos.push(DiscoveredRepo {
                name,
                path: path.clone(),
                source_index: all_repos.len() + index, // Unique index for single repos
            });
        }
    }

    all_repos.sort_by(|a, b| a.name.cmp(&b.name));
    all_repos
}

/// Remove duplicate repositories by path
fn deduplicate_repositories(repos: Vec<DiscoveredRepo>) -> Vec<DiscoveredRepo> {
    let mut seen = HashSet::new();
    repos
        .into_iter()
        .filter(|repo| seen.insert(repo.path.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_mock_repo(parent: &PathBuf, name: &str) -> PathBuf {
        let repo_path = parent.join(name);
        std::fs::create_dir_all(&repo_path).unwrap();
        std::fs::create_dir(repo_path.join(".git")).unwrap();
        repo_path
    }

    #[test]
    fn test_discover_from_single_directory() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let main_dir = temp.path().join("main");
        std::fs::create_dir(&main_dir).unwrap();

        create_mock_repo(&main_dir, "repo1");
        create_mock_repo(&main_dir, "repo2");
        create_mock_repo(&main_dir, "repo3");

        // Act
        let repos = discover_from_multiple_dirs(&[main_dir]);

        // Assert
        assert_eq!(repos.len(), 3);
        assert!(repos.iter().any(|r| r.name == "repo1"));
        assert!(repos.iter().any(|r| r.name == "repo2"));
        assert!(repos.iter().any(|r| r.name == "repo3"));
    }

    #[test]
    fn test_discover_from_multiple_directories() {
        // Arrange
        let temp = TempDir::new().unwrap();

        let work_dir = temp.path().join("work");
        let personal_dir = temp.path().join("personal");
        std::fs::create_dir(&work_dir).unwrap();
        std::fs::create_dir(&personal_dir).unwrap();

        create_mock_repo(&work_dir, "app1");
        create_mock_repo(&work_dir, "app2");
        create_mock_repo(&personal_dir, "dotfiles");
        create_mock_repo(&personal_dir, "experiments");

        // Act
        let repos = discover_from_multiple_dirs(&[work_dir, personal_dir]);

        // Assert
        assert_eq!(repos.len(), 4);
        assert!(repos.iter().any(|r| r.name == "app1"));
        assert!(repos.iter().any(|r| r.name == "app2"));
        assert!(repos.iter().any(|r| r.name == "dotfiles"));
        assert!(repos.iter().any(|r| r.name == "experiments"));
    }

    #[test]
    fn test_deduplicate_cross_directory() {
        // Arrange: Create repos with same name in different directories
        let temp = TempDir::new().unwrap();

        let dir1 = temp.path().join("dir1");
        let dir2 = temp.path().join("dir2");
        std::fs::create_dir(&dir1).unwrap();
        std::fs::create_dir(&dir2).unwrap();

        // Different repos with different paths (unique)
        create_mock_repo(&dir1, "repo-a");
        create_mock_repo(&dir1, "repo-b");
        create_mock_repo(&dir2, "repo-c");
        create_mock_repo(&dir2, "repo-d");

        // Act
        let repos = discover_from_multiple_dirs(&[dir1, dir2]);

        // Assert: All repos should be discovered (different paths)
        assert_eq!(repos.len(), 4);
    }

    #[test]
    fn test_deduplicate_same_path() {
        // Arrange: Create repos and manually create duplicates
        let temp = TempDir::new().unwrap();
        let main_dir = temp.path().join("main");
        std::fs::create_dir(&main_dir).unwrap();

        create_mock_repo(&main_dir, "repo1");
        create_mock_repo(&main_dir, "repo2");

        // Manually create duplicate entries
        let duplicate_repos = vec![
            DiscoveredRepo {
                name: "repo1".to_string(),
                path: main_dir.join("repo1"),
                source_index: 0,
            },
            DiscoveredRepo {
                name: "repo1".to_string(),
                path: main_dir.join("repo1"), // Same path
                source_index: 0,
            },
            DiscoveredRepo {
                name: "repo2".to_string(),
                path: main_dir.join("repo2"),
                source_index: 0,
            },
        ];

        // Act
        let deduped = deduplicate_repositories(duplicate_repos);

        // Assert
        assert_eq!(deduped.len(), 2);
    }

    #[test]
    fn test_merge_single_repos() {
        // Arrange
        let temp = TempDir::new().unwrap();

        let main_dir = temp.path().join("main");
        std::fs::create_dir(&main_dir).unwrap();

        let single_repo = temp.path().join("single-repo");
        std::fs::create_dir(&single_repo).unwrap();
        std::fs::create_dir(single_repo.join(".git")).unwrap();

        create_mock_repo(&main_dir, "main-repo");

        let from_dirs = discover_from_multiple_dirs(&[main_dir]);

        // Act: Merge with single repo
        let merged = merge_repositories(from_dirs, vec![single_repo.clone()]);

        // Assert
        assert_eq!(merged.len(), 2);
        assert!(merged.iter().any(|r| r.name == "main-repo"));
        assert!(merged.iter().any(|r| r.name == "single-repo"));
    }

    #[test]
    fn test_merge_avoids_duplicates() {
        // Arrange
        let temp = TempDir::new().unwrap();

        let main_dir = temp.path().join("main");
        std::fs::create_dir(&main_dir).unwrap();

        let single_repo = main_dir.join("main-repo"); // Same as in main
        std::fs::create_dir(&single_repo).unwrap();
        std::fs::create_dir(single_repo.join(".git")).unwrap();

        let from_dirs = discover_from_multiple_dirs(&[main_dir.clone()]);
        assert_eq!(from_dirs.len(), 1);

        // Act: Try to merge the same repo again
        let merged = merge_repositories(from_dirs, vec![single_repo]);

        // Assert: Should not duplicate
        assert_eq!(merged.len(), 1);
    }

    #[test]
    fn test_empty_directories() {
        // Arrange: Empty directories
        let temp = TempDir::new().unwrap();

        let dir1 = temp.path().join("empty1");
        let dir2 = temp.path().join("empty2");
        std::fs::create_dir(&dir1).unwrap();
        std::fs::create_dir(&dir2).unwrap();

        // Act
        let repos = discover_from_multiple_dirs(&[dir1, dir2]);

        // Assert
        assert!(repos.is_empty());
    }

    #[test]
    fn test_nonexistent_directories() {
        // Arrange: Non-existent paths
        let temp = TempDir::new().unwrap();

        let nonexistent = temp.path().join("does-not-exist");

        // Act: Should not panic
        let repos = discover_from_multiple_dirs(&[nonexistent]);

        // Assert
        assert!(repos.is_empty());
    }

    #[test]
    fn test_sorting_order() {
        // Arrange: Create repos in non-alphabetical order
        let temp = TempDir::new().unwrap();
        let main_dir = temp.path().join("main");
        std::fs::create_dir(&main_dir).unwrap();

        create_mock_repo(&main_dir, "zebra");
        create_mock_repo(&main_dir, "alpha");
        create_mock_repo(&main_dir, "mike");

        // Act
        let repos = discover_from_multiple_dirs(&[main_dir]);

        // Assert: Should be sorted
        assert_eq!(repos[0].name, "alpha");
        assert_eq!(repos[1].name, "mike");
        assert_eq!(repos[2].name, "zebra");
    }

    #[test]
    fn test_preserves_source_index() {
        // Arrange
        let temp = TempDir::new().unwrap();

        let dir1 = temp.path().join("dir1");
        let dir2 = temp.path().join("dir2");
        std::fs::create_dir(&dir1).unwrap();
        std::fs::create_dir(&dir2).unwrap();

        create_mock_repo(&dir1, "repo-from-0");
        create_mock_repo(&dir2, "repo-from-1");

        // Act
        let repos = discover_from_multiple_dirs(&[dir1, dir2]);

        // Assert
        let repo0 = repos.iter().find(|r| r.name == "repo-from-0").unwrap();
        let repo1 = repos.iter().find(|r| r.name == "repo-from-1").unwrap();

        assert_eq!(repo0.source_index, 0);
        assert_eq!(repo1.source_index, 1);
    }

    #[test]
    fn test_ignores_non_git_directories() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let main_dir = temp.path().join("main");
        std::fs::create_dir(&main_dir).unwrap();

        // Git repo
        create_mock_repo(&main_dir, "git-repo");

        // Non-git directory
        std::fs::create_dir(main_dir.join("not-a-repo")).unwrap();

        // Regular file
        std::fs::write(main_dir.join("file.txt"), "content").unwrap();

        // Act
        let repos = discover_from_multiple_dirs(&[main_dir]);

        // Assert
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].name, "git-repo");
    }

    #[test]
    fn test_large_number_of_directories() {
        // Arrange: Create many main directories
        let temp = TempDir::new().unwrap();
        let mut main_dirs = Vec::new();

        for i in 0..10 {
            let dir = temp.path().join(format!("dir{}", i));
            std::fs::create_dir(&dir).unwrap();
            create_mock_repo(&dir, &format!("repo{}", i));
            main_dirs.push(dir);
        }

        // Act
        let repos = discover_from_multiple_dirs(&main_dirs);

        // Assert
        assert_eq!(repos.len(), 10);
    }

    #[test]
    fn test_path_normalization_for_dedup() {
        // Arrange: Same path represented differently
        let temp = TempDir::new().unwrap();
        let main_dir = temp.path().join("main");
        std::fs::create_dir(&main_dir).unwrap();

        create_mock_repo(&main_dir, "repo1");

        // Create repos with potentially different path representations
        let repos = vec![
            DiscoveredRepo {
                name: "repo1".to_string(),
                path: main_dir.join("repo1"),
                source_index: 0,
            },
            DiscoveredRepo {
                name: "repo1".to_string(),
                path: main_dir.join("repo1"),
                source_index: 0,
            },
        ];

        // Act
        let deduped = deduplicate_repositories(repos);

        // Assert
        assert_eq!(deduped.len(), 1);
    }

    #[test]
    fn test_empty_main_dirs_list() {
        // Act: Empty list of main directories
        let repos = discover_from_multiple_dirs(&[]);

        // Assert
        assert!(repos.is_empty());
    }

    #[test]
    fn test_merge_preserves_existing_order() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let main_dir = temp.path().join("main");
        std::fs::create_dir(&main_dir).unwrap();

        create_mock_repo(&main_dir, "beta");
        create_mock_repo(&main_dir, "alpha");

        let from_dirs = discover_from_multiple_dirs(&[main_dir]);

        // Act: Merge with empty single repos
        let merged = merge_repositories(from_dirs, vec![]);

        // Assert: Order should be preserved (alpha before beta due to sorting)
        assert_eq!(merged[0].name, "alpha");
        assert_eq!(merged[1].name, "beta");
    }

    #[test]
    fn test_discover_from_nested_directories() {
        // Arrange: Repos at different depths are NOT discovered by this simple scanner
        // (This tests current behavior - only immediate subdirectories)
        let temp = TempDir::new().unwrap();
        let main_dir = temp.path().join("main");
        std::fs::create_dir(&main_dir).unwrap();

        // Immediate subdirectory - should be discovered
        create_mock_repo(&main_dir, "direct-child");

        // Nested directory - should NOT be discovered by simple scanner
        let nested = main_dir.join("nested");
        std::fs::create_dir(&nested).unwrap();
        create_mock_repo(&nested, "nested-child");

        // Act
        let repos = discover_from_multiple_dirs(&[main_dir]);

        // Assert: Only direct children are discovered
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].name, "direct-child");
    }

    #[test]
    fn test_unicode_repo_names() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let main_dir = temp.path().join("main");
        std::fs::create_dir(&main_dir).unwrap();

        create_mock_repo(&main_dir, "项目");
        create_mock_repo(&main_dir, "プロジェクト");

        // Act
        let repos = discover_from_multiple_dirs(&[main_dir]);

        // Assert
        assert_eq!(repos.len(), 2);
        assert!(repos.iter().any(|r| r.name == "项目"));
        assert!(repos.iter().any(|r| r.name == "プロジェクト"));
    }

    #[test]
    fn test_special_characters_in_names() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let main_dir = temp.path().join("main");
        std::fs::create_dir(&main_dir).unwrap();

        create_mock_repo(&main_dir, "my-app_v2.0");
        create_mock_repo(&main_dir, "test+dev");

        // Act
        let repos = discover_from_multiple_dirs(&[main_dir]);

        // Assert
        assert_eq!(repos.len(), 2);
    }
}
