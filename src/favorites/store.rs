//! Favorites storage and management

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

/// Favorites store for managing bookmarked repositories
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FavoritesStore {
    /// List of favorite repository paths
    #[serde(default)]
    pub repositories: Vec<String>,
}

impl FavoritesStore {
    /// Create a new empty favorites store
    pub fn new() -> Self {
        Self {
            repositories: Vec::new(),
        }
    }

    /// Create from a list of repository paths
    pub fn from_paths(paths: Vec<String>) -> Self {
        Self {
            repositories: paths,
        }
    }

    /// Add a repository to favorites
    pub fn add(&mut self, path: &Path) {
        let path_str = path.to_string_lossy().to_string();
        if !self.repositories.contains(&path_str) {
            self.repositories.push(path_str);
        }
    }

    /// Remove a repository from favorites
    pub fn remove(&mut self, path: &Path) {
        let path_str = path.to_string_lossy().to_string();
        self.repositories.retain(|p| p != &path_str);
    }

    /// Check if a repository is favorited
    pub fn contains(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_string();
        self.repositories.contains(&path_str)
    }

    /// Get all favorite repository paths
    pub fn get_all(&self) -> &[String] {
        &self.repositories
    }

    /// Get the number of favorites
    pub fn len(&self) -> usize {
        self.repositories.len()
    }

    /// Check if favorites is empty
    pub fn is_empty(&self) -> bool {
        self.repositories.is_empty()
    }

    /// Clear all favorites
    pub fn clear(&mut self) {
        self.repositories.clear();
    }

    /// Toggle favorite status for a repository
    /// Returns true if added, false if removed
    pub fn toggle(&mut self, path: &Path) -> bool {
        if self.contains(path) {
            self.remove(path);
            false
        } else {
            self.add(path);
            true
        }
    }

    /// Load favorites from a HashSet for validation
    pub fn from_set(set: HashSet<String>) -> Self {
        let mut repos: Vec<String> = set.into_iter().collect();
        repos.sort();
        Self {
            repositories: repos,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_favorites_new() {
        let store = FavoritesStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_favorites_add() {
        let mut store = FavoritesStore::new();
        let path = PathBuf::from("/home/user/repo1");

        store.add(&path);
        assert_eq!(store.len(), 1);
        assert!(store.contains(&path));
    }

    #[test]
    fn test_favorites_add_duplicate() {
        let mut store = FavoritesStore::new();
        let path = PathBuf::from("/home/user/repo1");

        store.add(&path);
        store.add(&path);
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_favorites_remove() {
        let mut store = FavoritesStore::new();
        let path = PathBuf::from("/home/user/repo1");

        store.add(&path);
        store.remove(&path);
        assert_eq!(store.len(), 0);
        assert!(!store.contains(&path));
    }

    #[test]
    fn test_favorites_remove_nonexistent() {
        let mut store = FavoritesStore::new();
        let path1 = PathBuf::from("/home/user/repo1");
        let path2 = PathBuf::from("/home/user/repo2");

        store.add(&path1);
        store.remove(&path2);
        assert_eq!(store.len(), 1);
        assert!(store.contains(&path1));
    }

    #[test]
    fn test_favorites_contains() {
        let mut store = FavoritesStore::new();
        let path = PathBuf::from("/home/user/repo1");

        store.add(&path);
        assert!(store.contains(&path));

        let other_path = PathBuf::from("/home/user/repo2");
        assert!(!store.contains(&other_path));
    }

    #[test]
    fn test_favorites_toggle() {
        let mut store = FavoritesStore::new();
        let path = PathBuf::from("/home/user/repo1");

        // Toggle from not favorited to favorited
        let added = store.toggle(&path);
        assert!(added);
        assert!(store.contains(&path));

        // Toggle from favorited to not favorited
        let added = store.toggle(&path);
        assert!(!added);
        assert!(!store.contains(&path));
    }

    #[test]
    fn test_favorites_clear() {
        let mut store = FavoritesStore::new();
        let path1 = PathBuf::from("/home/user/repo1");
        let path2 = PathBuf::from("/home/user/repo2");

        store.add(&path1);
        store.add(&path2);
        assert_eq!(store.len(), 2);

        store.clear();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_favorites_get_all() {
        let mut store = FavoritesStore::new();
        let path1 = PathBuf::from("/home/user/repo1");
        let path2 = PathBuf::from("/home/user/repo2");

        store.add(&path1);
        store.add(&path2);

        let all = store.get_all();
        assert_eq!(all.len(), 2);
        assert!(all.contains(&path1.to_string_lossy().to_string()));
        assert!(all.contains(&path2.to_string_lossy().to_string()));
    }

    #[test]
    fn test_favorites_from_paths() {
        let paths = vec![
            "/home/user/repo1".to_string(),
            "/home/user/repo2".to_string(),
        ];

        let store = FavoritesStore::from_paths(paths.clone());
        assert_eq!(store.len(), 2);
        assert_eq!(store.get_all(), &paths);
    }

    #[test]
    fn test_favorites_serialize() {
        let mut store = FavoritesStore::new();
        store.add(&PathBuf::from("/home/user/repo1"));
        store.add(&PathBuf::from("/home/user/repo2"));

        let serialized = toml::to_string(&store).unwrap();
        assert!(serialized.contains("repositories"));
        assert!(serialized.contains("repo1"));
        assert!(serialized.contains("repo2"));
    }

    #[test]
    fn test_favorites_deserialize() {
        let toml_str = r#"
            repositories = [
                "/home/user/repo1",
                "/home/user/repo2",
            ]
        "#;

        let store: FavoritesStore = toml::from_str(toml_str).unwrap();
        assert_eq!(store.len(), 2);
        assert!(store.contains(&PathBuf::from("/home/user/repo1")));
        assert!(store.contains(&PathBuf::from("/home/user/repo2")));
    }

    #[test]
    fn test_favorites_from_set() {
        let mut set = HashSet::new();
        set.insert("/home/user/repo1".to_string());
        set.insert("/home/user/repo2".to_string());

        let store = FavoritesStore::from_set(set);
        assert_eq!(store.len(), 2);
    }
}
