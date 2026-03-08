//! Repository discovery

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{RepoError, Result};
use crate::repo::source::RepoSource;
use crate::repo::Repository;

/// Configuration for repository scanning
#[derive(Debug, Clone)]
pub struct ScanConfig {
    /// Maximum search depth
    pub max_depth: usize,
    /// Whether to follow symlinks
    pub follow_symlinks: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            max_depth: crate::constants::security::DEFAULT_MAX_SEARCH_DEPTH,
            follow_symlinks: crate::constants::security::DEFAULT_ALLOW_SYMLINKS,
        }
    }
}

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

        // Add all directories, mark git status
        let is_git = is_git_repository(&path);
        let mut repo = Repository::from_path(path);
        repo.is_git_repo = is_git;
        repos.push(repo);
    }

    // Sort by name
    repos.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(repos)
}

/// Discover repositories from multiple main directories
///
/// Scans all enabled main directories and adds standalone repositories.
/// Removes duplicates based on path.
pub fn discover_repositories_multi(
    main_dirs: &[(usize, &Path, Option<usize>)],
    single_repos: &[PathBuf],
    config: &ScanConfig,
) -> Result<Vec<Repository>> {
    let mut all_repos = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();

    // Scan main directories
    for (dir_index, dir_path, max_depth) in main_dirs {
        let depth = max_depth.unwrap_or(config.max_depth);
        let repos = discover_in_directory(dir_path, depth, config)?;

        for repo_path in repos {
            if seen_paths.insert(repo_path.clone()) {
                let source = RepoSource::MainDirectory {
                    dir_index: *dir_index,
                    dir_path: (*dir_path).to_path_buf(),
                };
                all_repos.push(Repository::from_path_with_source(repo_path, source));
            }
        }
    }

    // Add standalone repositories
    for repo_path in single_repos {
        if seen_paths.insert(repo_path.clone()) && repo_path.exists() {
            let source = RepoSource::Standalone;
            all_repos.push(Repository::from_path_with_source(repo_path.clone(), source));
        }
    }

    // Sort by last modified time (newest first)
    all_repos.sort_by(|a, b| {
        b.last_modified
            .cmp(&a.last_modified)
            .then_with(|| a.name.cmp(&b.name))
    });

    Ok(all_repos)
}

/// Discover repositories in a directory with depth limit
fn discover_in_directory(
    dir: &Path,
    max_depth: usize,
    config: &ScanConfig,
) -> Result<Vec<PathBuf>> {
    let mut repos = Vec::new();
    let mut to_visit = vec![(dir.to_path_buf(), 0usize)];

    while let Some((current_dir, depth)) = to_visit.pop() {
        if depth > max_depth {
            continue;
        }

        let entries = match fs::read_dir(&current_dir) {
            Ok(entries) => entries,
            Err(_) => continue, // Skip directories we can't read
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();

            // Handle symlinks
            if path.is_symlink() && !config.follow_symlinks {
                continue;
            }

            // Skip if not a directory
            if !path.is_dir() {
                continue;
            }

            // Check if it's a git repository
            if is_git_repository(&path) {
                repos.push(path);
            } else if depth < max_depth {
                // Add to visit list for deeper scan
                to_visit.push((path, depth + 1));
            }
        }
    }

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

        // Should discover all directories (including non-git)
        assert_eq!(repos.len(), 3);
        assert!(repos.iter().any(|r| r.name == "repo1" && r.is_git_repo));
        assert!(repos.iter().any(|r| r.name == "repo2" && r.is_git_repo));
        assert!(repos.iter().any(|r| r.name == "not-repo" && !r.is_git_repo));
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
        // All should be git repos
        assert!(repos.iter().all(|r| r.is_git_repo));
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
