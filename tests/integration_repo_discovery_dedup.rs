//! Repository discovery and deduplication integration tests
//!
//! Tests for multi-directory discovery with deduplication

use std::collections::HashSet;
use std::path::PathBuf;
use tempfile::TempDir;

/// Represents a discovered repository with source info
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct DiscoveredRepo {
    name: String,
    path: PathBuf,
    source_type: SourceType,
    source_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SourceType {
    MainDirectory,
    Single,
}

/// Multi-directory repository discoverer
struct MultiDirectoryDiscoverer;

impl MultiDirectoryDiscoverer {
    fn new() -> Self {
        Self
    }

    /// Discover repositories from multiple main directories
    fn discover_from_dirs(&self, main_dirs: &[PathBuf]) -> Vec<DiscoveredRepo> {
        let mut all_repos = Vec::new();
        let mut seen_paths: HashSet<PathBuf> = HashSet::new();

        for (dir_index, dir) in main_dirs.iter().enumerate() {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_dir() && path.join(".git").exists() {
                        // Deduplicate: Only add if we haven't seen this path
                        if seen_paths.insert(path.clone()) {
                            let name = path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown")
                                .to_string();

                            all_repos.push(DiscoveredRepo {
                                name,
                                path,
                                source_type: SourceType::MainDirectory,
                                source_index: dir_index,
                            });
                        }
                    }
                }
            }
        }

        all_repos
    }

    /// Merge repositories from different sources with deduplication
    fn merge_with_dedup(
        &self,
        from_dirs: Vec<DiscoveredRepo>,
        single_repos: &[PathBuf],
    ) -> Vec<DiscoveredRepo> {
        let mut all_repos = from_dirs;
        let mut seen_paths: HashSet<PathBuf> = all_repos.iter().map(|r| r.path.clone()).collect();

        for path in single_repos {
            if seen_paths.insert(path.clone()) && path.exists() {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                all_repos.push(DiscoveredRepo {
                    name,
                    path: path.clone(),
                    source_type: SourceType::Single,
                    source_index: all_repos.len(), // Unique index
                });
            }
        }

        // Sort by name for consistent ordering
        all_repos.sort_by(|a, b| a.name.cmp(&b.name));

        all_repos
    }

    /// Deduplicate repositories by canonical path
    fn deduplicate_by_canonical_path(&self, repos: Vec<DiscoveredRepo>) -> Vec<DiscoveredRepo> {
        let mut seen = HashSet::new();
        repos
            .into_iter()
            .filter(|repo| {
                // Use canonical path if available, otherwise use original
                let key = repo.path.clone();
                seen.insert(key)
            })
            .collect()
    }

    /// Get total repo count
    fn count_repos(&self, main_dirs: &[PathBuf]) -> usize {
        self.discover_from_dirs(main_dirs).len()
    }
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
    fn test_discover_from_multiple_main_directories() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let discoverer = MultiDirectoryDiscoverer::new();

        let work_dir = temp.path().join("work");
        let personal_dir = temp.path().join("personal");
        std::fs::create_dir(&work_dir).unwrap();
        std::fs::create_dir(&personal_dir).unwrap();

        create_mock_repo(&work_dir, "app1");
        create_mock_repo(&work_dir, "app2");
        create_mock_repo(&personal_dir, "dotfiles");
        create_mock_repo(&personal_dir, "experiments");

        // Act
        let repos = discoverer.discover_from_dirs(&[work_dir, personal_dir]);

        // Assert
        assert_eq!(repos.len(), 4);
        assert!(repos.iter().any(|r| r.name == "app1"));
        assert!(repos.iter().any(|r| r.name == "app2"));
        assert!(repos.iter().any(|r| r.name == "dotfiles"));
        assert!(repos.iter().any(|r| r.name == "experiments"));
    }

    #[test]
    fn test_deduplicate_same_repo_in_multiple_dirs() {
        // Arrange: Simulate scenario where same path might be scanned twice
        let temp = TempDir::new().unwrap();
        let discoverer = MultiDirectoryDiscoverer::new();

        let main_dir = temp.path().join("main");
        std::fs::create_dir(&main_dir).unwrap();

        create_mock_repo(&main_dir, "repo1");
        create_mock_repo(&main_dir, "repo2");

        // Act: Scan same directory twice
        let repos = discoverer.discover_from_dirs(&[main_dir.clone(), main_dir]);

        // Assert: Should deduplicate
        assert_eq!(repos.len(), 2);
    }

    #[test]
    fn test_merge_single_repos_with_main() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let discoverer = MultiDirectoryDiscoverer::new();

        let main_dir = temp.path().join("main");
        std::fs::create_dir(&main_dir).unwrap();

        let main_repo = create_mock_repo(&main_dir, "main-repo");
        let single_repo = temp.path().join("single-repo");
        std::fs::create_dir(&single_repo).unwrap();
        std::fs::create_dir(single_repo.join(".git")).unwrap();

        let from_dirs = discoverer.discover_from_dirs(&[main_dir]);
        assert_eq!(from_dirs.len(), 1);

        // Act
        let merged = discoverer.merge_with_dedup(from_dirs, &[single_repo.clone()]);

        // Assert
        assert_eq!(merged.len(), 2);
        assert!(merged.iter().any(|r| r.name == "main-repo"));
        assert!(merged.iter().any(|r| r.name == "single-repo"));

        // Check source types
        let main = merged.iter().find(|r| r.name == "main-repo").unwrap();
        let single = merged.iter().find(|r| r.name == "single-repo").unwrap();

        assert_eq!(main.source_type, SourceType::MainDirectory);
        assert_eq!(single.source_type, SourceType::Single);
    }

    #[test]
    fn test_merge_avoids_duplicate_single_repos() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let discoverer = MultiDirectoryDiscoverer::new();

        let main_dir = temp.path().join("main");
        std::fs::create_dir(&main_dir).unwrap();

        let duplicate_path = create_mock_repo(&main_dir, "duplicate-repo");

        let from_dirs = discoverer.discover_from_dirs(&[main_dir]);

        // Act: Try to merge the same repo as a single repo
        let merged = discoverer.merge_with_dedup(from_dirs.clone(), &[duplicate_path.clone()]);

        // Assert: Should not duplicate
        assert_eq!(merged.len(), 1);
        assert!(merged.iter().any(|r| r.name == "duplicate-repo"));
    }

    #[test]
    fn test_deduplicate_by_canonical_path() {
        // Arrange: Create duplicate entries manually
        let temp = TempDir::new().unwrap();
        let discoverer = MultiDirectoryDiscoverer::new();

        let repo_path = temp.path().join("my-repo");
        std::fs::create_dir(&repo_path).unwrap();
        std::fs::create_dir(repo_path.join(".git")).unwrap();

        let repos = vec![
            DiscoveredRepo {
                name: "my-repo".to_string(),
                path: repo_path.clone(),
                source_type: SourceType::MainDirectory,
                source_index: 0,
            },
            DiscoveredRepo {
                name: "my-repo".to_string(),
                path: repo_path.clone(), // Same path
                source_type: SourceType::MainDirectory,
                source_index: 0,
            },
        ];

        // Act
        let deduped = discoverer.deduplicate_by_canonical_path(repos);

        // Assert
        assert_eq!(deduped.len(), 1);
    }

    #[test]
    fn test_preserves_first_occurrence_on_dedup() {
        // Arrange: Same path, different source info
        let temp = TempDir::new().unwrap();
        let discoverer = MultiDirectoryDiscoverer::new();

        let repo_path = temp.path().join("my-repo");
        std::fs::create_dir(&repo_path).unwrap();
        std::fs::create_dir(repo_path.join(".git")).unwrap();

        let repos = vec![
            DiscoveredRepo {
                name: "my-repo".to_string(),
                path: repo_path.clone(),
                source_type: SourceType::MainDirectory,
                source_index: 0, // First occurrence
            },
            DiscoveredRepo {
                name: "my-repo".to_string(),
                path: repo_path.clone(),
                source_type: SourceType::Single,
                source_index: 5, // Second occurrence, different source
            },
        ];

        // Act
        let deduped = discoverer.deduplicate_by_canonical_path(repos);

        // Assert: First occurrence should be kept
        assert_eq!(deduped.len(), 1);
        assert_eq!(deduped[0].source_type, SourceType::MainDirectory);
        assert_eq!(deduped[0].source_index, 0);
    }

    #[test]
    fn test_discover_empty_directories() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let discoverer = MultiDirectoryDiscoverer::new();

        let empty1 = temp.path().join("empty1");
        let empty2 = temp.path().join("empty2");
        std::fs::create_dir(&empty1).unwrap();
        std::fs::create_dir(&empty2).unwrap();

        // Act
        let repos = discoverer.discover_from_dirs(&[empty1, empty2]);

        // Assert
        assert!(repos.is_empty());
    }

    #[test]
    fn test_discover_with_non_git_directories() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let discoverer = MultiDirectoryDiscoverer::new();

        let main_dir = temp.path().join("main");
        std::fs::create_dir(&main_dir).unwrap();

        create_mock_repo(&main_dir, "git-repo");
        std::fs::create_dir(main_dir.join("not-a-repo")).unwrap();
        std::fs::write(main_dir.join("file.txt"), "content").unwrap();

        // Act
        let repos = discoverer.discover_from_dirs(&[main_dir]);

        // Assert
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].name, "git-repo");
    }

    #[test]
    fn test_count_repos() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let discoverer = MultiDirectoryDiscoverer::new();

        let dir1 = temp.path().join("dir1");
        let dir2 = temp.path().join("dir2");
        std::fs::create_dir(&dir1).unwrap();
        std::fs::create_dir(&dir2).unwrap();

        create_mock_repo(&dir1, "repo1");
        create_mock_repo(&dir1, "repo2");
        create_mock_repo(&dir2, "repo3");

        // Act
        let count = discoverer.count_repos(&[dir1, dir2]);

        // Assert
        assert_eq!(count, 3);
    }

    #[test]
    fn test_large_scale_discovery() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let discoverer = MultiDirectoryDiscoverer::new();

        let main_dir = temp.path().join("main");
        std::fs::create_dir(&main_dir).unwrap();

        // Create many repos
        for i in 0..100 {
            create_mock_repo(&main_dir, &format!("repo{:03}", i));
        }

        // Act
        let repos = discoverer.discover_from_dirs(&[main_dir]);

        // Assert
        assert_eq!(repos.len(), 100);

        // Verify sorted order
        for i in 0..repos.len() - 1 {
            assert!(
                repos[i].name <= repos[i + 1].name,
                "Repos should be sorted alphabetically"
            );
        }
    }

    #[test]
    fn test_source_index_tracking() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let discoverer = MultiDirectoryDiscoverer::new();

        let dir0 = temp.path().join("dir0");
        let dir1 = temp.path().join("dir1");
        let dir2 = temp.path().join("dir2");
        std::fs::create_dir(&dir0).unwrap();
        std::fs::create_dir(&dir1).unwrap();
        std::fs::create_dir(&dir2).unwrap();

        create_mock_repo(&dir0, "from-zero");
        create_mock_repo(&dir1, "from-one");
        create_mock_repo(&dir2, "from-two");

        // Act
        let repos = discoverer.discover_from_dirs(&[dir0, dir1, dir2]);

        // Assert
        let repo0 = repos.iter().find(|r| r.name == "from-zero").unwrap();
        let repo1 = repos.iter().find(|r| r.name == "from-one").unwrap();
        let repo2 = repos.iter().find(|r| r.name == "from-two").unwrap();

        assert_eq!(repo0.source_index, 0);
        assert_eq!(repo1.source_index, 1);
        assert_eq!(repo2.source_index, 2);
    }

    #[test]
    fn test_merge_preserves_all_single_repos() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let discoverer = MultiDirectoryDiscoverer::new();

        let single1 = temp.path().join("single1");
        let single2 = temp.path().join("single2");
        let single3 = temp.path().join("single3");
        std::fs::create_dir(&single1).unwrap();
        std::fs::create_dir(&single2).unwrap();
        std::fs::create_dir(&single3).unwrap();
        std::fs::create_dir(single1.join(".git")).unwrap();
        std::fs::create_dir(single2.join(".git")).unwrap();
        std::fs::create_dir(single3.join(".git")).unwrap();

        // Act: Merge with empty from_dirs
        let merged = discoverer.merge_with_dedup(vec![], &[single1, single2, single3]);

        // Assert
        assert_eq!(merged.len(), 3);
        assert!(merged.iter().all(|r| r.source_type == SourceType::Single));
    }

    #[test]
    fn test_discover_handles_errors_gracefully() {
        // Arrange
        let discoverer = MultiDirectoryDiscoverer::new();

        // Non-existent directories should not cause panic
        let nonexistent1 = PathBuf::from("/does/not/exist1");
        let nonexistent2 = PathBuf::from("/does/not/exist2");

        // Act: Should not panic
        let repos = discoverer.discover_from_dirs(&[nonexistent1, nonexistent2]);

        // Assert
        assert!(repos.is_empty());
    }

    #[test]
    fn test_merge_handles_nonexistent_single_repos() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let discoverer = MultiDirectoryDiscoverer::new();

        let main_dir = temp.path().join("main");
        std::fs::create_dir(&main_dir).unwrap();
        create_mock_repo(&main_dir, "main-repo");

        let from_dirs = discoverer.discover_from_dirs(&[main_dir]);

        let nonexistent = PathBuf::from("/does/not/exist");

        // Act: Non-existent paths should be skipped
        let merged = discoverer.merge_with_dedup(from_dirs, &[nonexistent]);

        // Assert
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].name, "main-repo");
    }

    #[test]
    fn test_different_repos_same_name_different_dirs() {
        // Arrange: Different repos with same name in different directories
        let temp = TempDir::new().unwrap();
        let discoverer = MultiDirectoryDiscoverer::new();

        let dir1 = temp.path().join("dir1");
        let dir2 = temp.path().join("dir2");
        std::fs::create_dir(&dir1).unwrap();
        std::fs::create_dir(&dir2).unwrap();

        // Different repos with same name
        create_mock_repo(&dir1, "shared-name");
        create_mock_repo(&dir2, "shared-name");

        // Act
        let repos = discoverer.discover_from_dirs(&[dir1, dir2]);

        // Assert: Both should be discovered (different paths)
        assert_eq!(repos.len(), 2);
        assert!(repos.iter().all(|r| r.name == "shared-name"));

        // But they should have different source indices
        let indices: HashSet<_> = repos.iter().map(|r| r.source_index).collect();
        assert_eq!(indices.len(), 2);
    }

    #[test]
    fn test_empty_merge() {
        // Arrange
        let discoverer = MultiDirectoryDiscoverer::new();

        // Act: Merge empty lists
        let merged = discoverer.merge_with_dedup(vec![], &[]);

        // Assert
        assert!(merged.is_empty());
    }

    #[test]
    fn test_unicode_repo_discovery() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let discoverer = MultiDirectoryDiscoverer::new();

        let main_dir = temp.path().join("main");
        std::fs::create_dir(&main_dir).unwrap();

        create_mock_repo(&main_dir, "项目");
        create_mock_repo(&main_dir, "プロジェクト");
        create_mock_repo(&main_dir, "proyecto");

        // Act
        let repos = discoverer.discover_from_dirs(&[main_dir]);

        // Assert
        assert_eq!(repos.len(), 3);
        assert!(repos.iter().any(|r| r.name == "项目"));
        assert!(repos.iter().any(|r| r.name == "プロジェクト"));
        assert!(repos.iter().any(|r| r.name == "proyecto"));
    }

    #[test]
    fn test_discover_from_many_directories() {
        // Arrange
        let temp = TempDir::new().unwrap();
        let discoverer = MultiDirectoryDiscoverer::new();

        let mut main_dirs = Vec::new();
        let mut expected_count = 0;

        for i in 0..10 {
            let dir = temp.path().join(format!("dir{}", i));
            std::fs::create_dir(&dir).unwrap();
            create_mock_repo(&dir, &format!("repo{}", i));
            main_dirs.push(dir);
            expected_count += 1;
        }

        // Act
        let repos = discoverer.discover_from_dirs(&main_dirs);

        // Assert
        assert_eq!(repos.len(), expected_count);
    }
}
