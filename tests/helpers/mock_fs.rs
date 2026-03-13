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
    created_repos: Vec<PathBuf>,
}

impl MockFs {
    /// Create a new mock filesystem
    pub fn new() -> Self {
        Self {
            temp_dir: TempDir::new().unwrap(),
            created_repos: Vec::new(),
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
        fs::create_dir_all(&parent_path).unwrap();

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

    // === Multi-directory support ===

    /// Create a multi-directory structure for testing
    ///
    /// Creates multiple main directories with repositories in each
    pub fn create_multi_directory_structure(&mut self) -> Vec<PathBuf> {
        let mut main_dirs = Vec::new();

        // Main directory 1: work/projects
        let work_dir = self.create_nested_repos("work/projects", &["app1", "app2"]);
        self.create_non_repo_dir("work/projects/docs");
        main_dirs.push(work_dir);

        // Main directory 2: personal
        let personal_dir = self.create_nested_repos("personal", &["dotfiles", "experiments"]);
        main_dirs.push(personal_dir);

        // Main directory 3: company
        let company_dir = self.create_nested_repos("company", &["service1", "service2"]);
        main_dirs.push(company_dir);

        main_dirs
    }

    /// Create a repository with a specific scope (parent directory name)
    pub fn create_repo_with_scope(&mut self, scope: &str, name: &str) -> PathBuf {
        let scope_path = self.temp_dir.path().join(scope);
        if !scope_path.exists() {
            fs::create_dir(&scope_path).unwrap();
        }

        let repo_path = scope_path.join(name);
        fs::create_dir(&repo_path).unwrap();
        fs::create_dir(repo_path.join(".git")).unwrap();

        self.created_repos.push(repo_path.clone());
        repo_path
    }

    /// Get all created repository paths
    pub fn get_all_repo_paths(&self) -> Vec<PathBuf> {
        let mut repos = self.created_repos.clone();

        // Also collect repos from temp_dir
        fn collect_repos(dir: &Path, repos: &mut Vec<PathBuf>) {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_dir() {
                        if path.join(".git").exists() {
                            repos.push(path);
                        } else {
                            collect_repos(&path, repos);
                        }
                    }
                }
            }
        }

        collect_repos(self.temp_dir.path(), &mut repos);

        // Remove duplicates while preserving order
        let mut seen = std::collections::HashSet::new();
        repos
            .into_iter()
            .filter(|p| seen.insert(p.clone()))
            .collect()
    }

    /// Create duplicate repositories with the same name in different locations
    /// Returns the paths to the duplicate repos
    pub fn create_duplicate_repos(&self, name: &str, parent_dirs: &[&str]) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        for parent in parent_dirs {
            let parent_path = self.temp_dir.path().join(parent);
            // Use create_dir_all to create parent directories recursively
            fs::create_dir_all(&parent_path).unwrap();

            let repo_path = parent_path.join(name);
            fs::create_dir(&repo_path).unwrap();
            fs::create_dir(repo_path.join(".git")).unwrap();
            paths.push(repo_path);
        }

        paths
    }

    /// Create a main directory with nested repos at various depths
    pub fn create_main_dir_with_depth(
        &self,
        main_dir: &str,
        repos_at_depth: &[(usize, &[&str])],
    ) -> PathBuf {
        let main_path = self.temp_dir.path().join(main_dir);
        fs::create_dir(&main_path).unwrap();

        for (depth, repo_names) in repos_at_depth {
            let mut current_path = main_path.clone();
            for _ in 0..*depth {
                current_path = current_path.join(format!("level{}", depth));
                fs::create_dir(&current_path).unwrap();
            }

            for name in *repo_names {
                let repo_path = current_path.join(name);
                fs::create_dir(&repo_path).unwrap();
                fs::create_dir(repo_path.join(".git")).unwrap();
            }
        }

        main_path
    }

    /// Count repositories in a specific directory
    pub fn count_repos_in(&self, dir: &Path) -> usize {
        fn count_recursive(dir: &Path) -> usize {
            let mut count = 0;
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_dir() {
                        if path.join(".git").exists() {
                            count += 1;
                        } else {
                            count += count_recursive(&path);
                        }
                    }
                }
            }
            count
        }

        count_recursive(dir)
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

    #[test]
    fn test_create_multi_directory_structure() {
        let mut mock = MockFs::new();
        let main_dirs = mock.create_multi_directory_structure();

        assert_eq!(main_dirs.len(), 3);

        // Check work/projects
        assert!(main_dirs[0].join("app1/.git").exists());
        assert!(main_dirs[0].join("app2/.git").exists());
        assert!(!main_dirs[0].join("docs/.git").exists()); // Non-repo

        // Check personal
        assert!(main_dirs[1].join("dotfiles/.git").exists());

        // Check company
        assert!(main_dirs[2].join("service1/.git").exists());
    }

    #[test]
    fn test_create_repo_with_scope() {
        let mut mock = MockFs::new();
        let repo = mock.create_repo_with_scope("mycompany", "myapp");

        assert!(repo.exists());
        assert!(repo.join(".git").exists());
        assert!(repo.to_string_lossy().contains("mycompany"));
    }

    #[test]
    fn test_get_all_repo_paths() {
        let mut mock = MockFs::new();
        mock.create_repo_with_scope("work", "app1");
        mock.create_repo_with_scope("work", "app2");
        mock.create_repo_with_scope("personal", "blog");

        let paths = mock.get_all_repo_paths();
        assert_eq!(paths.len(), 3);
    }

    #[test]
    fn test_create_duplicate_repos() {
        let mock = MockFs::new();
        let paths = mock.create_duplicate_repos("shared-lib", &["work/repos", "personal/repos"]);

        assert_eq!(paths.len(), 2);
        assert!(paths[0].ends_with("shared-lib"));
        assert!(paths[1].ends_with("shared-lib"));
        assert_ne!(paths[0], paths[1]); // Different paths
    }

    #[test]
    fn test_count_repos_in() {
        let mock = MockFs::new();
        let main_dir = mock.create_nested_repos("main", &["repo1", "repo2", "repo3"]);
        mock.create_nested_repos("main/subdir", &["nested1"]);

        let count = mock.count_repos_in(&main_dir);
        assert!(count >= 3);
    }
}
