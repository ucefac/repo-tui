//! Recent repositories store

use crate::recent::RecentEntry;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Maximum number of recent repositories to keep
const MAX_RECENT_ENTRIES: usize = 20;

/// Recent repositories store with LRU eviction
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecentStore {
    /// List of recent repositories (newest first)
    #[serde(default)]
    pub repositories: Vec<RecentEntry>,
}

impl RecentStore {
    /// Create a new empty recent store
    pub fn new() -> Self {
        Self {
            repositories: Vec::new(),
        }
    }

    /// Create from a list of recent entries
    pub fn from_entries(entries: Vec<RecentEntry>) -> Self {
        let mut store = Self {
            repositories: entries,
        };
        store.sort_by_time();
        store.truncate_to_max();
        store
    }

    /// Add a repository to recent list
    /// If already exists, moves it to the top
    pub fn add(&mut self, path: &Path) {
        let path_str = path.to_string_lossy().to_string();

        // Remove existing entry if present
        self.repositories.retain(|e| e.path != path_str);

        // Add new entry at the top
        let entry = RecentEntry::new(path);
        self.repositories.insert(0, entry);

        // Enforce maximum limit
        self.truncate_to_max();
    }

    /// Get all recent repositories (sorted by time, newest first)
    pub fn get_all(&self) -> &[RecentEntry] {
        &self.repositories
    }

    /// Get the number of recent repositories
    pub fn len(&self) -> usize {
        self.repositories.len()
    }

    /// Check if recent store is empty
    pub fn is_empty(&self) -> bool {
        self.repositories.is_empty()
    }

    /// Clear all recent repositories
    pub fn clear(&mut self) {
        self.repositories.clear();
    }

    /// Remove a repository from recent list
    pub fn remove(&mut self, path: &Path) {
        let path_str = path.to_string_lossy().to_string();
        self.repositories.retain(|e| e.path != path_str);
    }

    /// Check if a repository is in recent list
    pub fn contains(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_string();
        self.repositories.iter().any(|e| e.path == path_str)
    }

    /// Get recent repositories up to a limit
    pub fn get_recent(&self, limit: usize) -> &[RecentEntry] {
        if limit >= self.repositories.len() {
            &self.repositories
        } else {
            &self.repositories[..limit]
        }
    }

    /// Sort entries by time (newest first)
    fn sort_by_time(&mut self) {
        self.repositories.sort_by(|a, b| {
            let time_a = a.parsed_timestamp();
            let time_b = b.parsed_timestamp();

            match (time_a, time_b) {
                (Some(ta), Some(tb)) => tb.cmp(&ta), // Descending order
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });
    }

    /// Truncate to maximum allowed entries
    fn truncate_to_max(&mut self) {
        if self.repositories.len() > MAX_RECENT_ENTRIES {
            self.repositories.truncate(MAX_RECENT_ENTRIES);
        }
    }

    /// Load from config format
    pub fn from_config_entries(entries: Vec<RecentEntry>) -> Self {
        let mut store = Self::from_entries(entries);
        store.sort_by_time();
        store
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_recent_store_new() {
        let store = RecentStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_recent_store_add() {
        let mut store = RecentStore::new();
        let path = PathBuf::from("/home/user/repo1");

        store.add(&path);
        assert_eq!(store.len(), 1);
        assert!(store.contains(&path));
        assert_eq!(store.get_all()[0].path, "/home/user/repo1");
    }

    #[test]
    fn test_recent_store_add_multiple() {
        let mut store = RecentStore::new();
        let path1 = PathBuf::from("/home/user/repo1");
        let path2 = PathBuf::from("/home/user/repo2");

        store.add(&path1);
        store.add(&path2);

        assert_eq!(store.len(), 2);
        // Most recent should be first
        assert_eq!(store.get_all()[0].path, "/home/user/repo2");
        assert_eq!(store.get_all()[1].path, "/home/user/repo1");
    }

    #[test]
    fn test_recent_store_add_existing() {
        let mut store = RecentStore::new();
        let path1 = PathBuf::from("/home/user/repo1");
        let path2 = PathBuf::from("/home/user/repo2");

        store.add(&path1);
        store.add(&path2);
        // Add path1 again - should move to top
        store.add(&path1);

        assert_eq!(store.len(), 2);
        assert_eq!(store.get_all()[0].path, "/home/user/repo1");
        assert_eq!(store.get_all()[1].path, "/home/user/repo2");
    }

    #[test]
    fn test_recent_store_remove() {
        let mut store = RecentStore::new();
        let path1 = PathBuf::from("/home/user/repo1");
        let path2 = PathBuf::from("/home/user/repo2");

        store.add(&path1);
        store.add(&path2);

        store.remove(&path1);

        assert_eq!(store.len(), 1);
        assert!(!store.contains(&path1));
        assert!(store.contains(&path2));
    }

    #[test]
    fn test_recent_store_clear() {
        let mut store = RecentStore::new();
        let path1 = PathBuf::from("/home/user/repo1");
        let path2 = PathBuf::from("/home/user/repo2");

        store.add(&path1);
        store.add(&path2);

        store.clear();

        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_recent_store_lru_eviction() {
        let mut store = RecentStore::new();

        // Add 25 entries (more than MAX_RECENT_ENTRIES = 20)
        for i in 0..25 {
            let path = PathBuf::from(format!("/home/user/repo{}", i));
            store.add(&path);
        }

        // Should only keep 20 most recent
        assert_eq!(store.len(), MAX_RECENT_ENTRIES);
        // Most recent should be repo24
        assert_eq!(store.get_all()[0].path, "/home/user/repo24");
        // Oldest kept should be repo5 (entries 0-4 were evicted)
        assert_eq!(store.get_all()[19].path, "/home/user/repo5");
    }

    #[test]
    fn test_recent_store_get_recent() {
        let mut store = RecentStore::new();

        for i in 0..10 {
            let path = PathBuf::from(format!("/home/user/repo{}", i));
            store.add(&path);
        }

        // Get only 5 most recent
        let recent = store.get_recent(5);
        assert_eq!(recent.len(), 5);
        assert_eq!(recent[0].path, "/home/user/repo9");
        assert_eq!(recent[4].path, "/home/user/repo5");
    }

    #[test]
    fn test_recent_store_from_entries() {
        let entries = vec![
            RecentEntry::from_parts(
                "/home/user/repo1".to_string(),
                "2026-03-07T10:00:00Z".to_string(),
            ),
            RecentEntry::from_parts(
                "/home/user/repo2".to_string(),
                "2026-03-07T09:00:00Z".to_string(),
            ),
        ];

        let store = RecentStore::from_entries(entries.clone());
        assert_eq!(store.len(), 2);
        // Should be sorted by time (newest first)
        assert_eq!(store.get_all()[0].path, "/home/user/repo1");
    }

    #[test]
    fn test_recent_store_serialize() {
        let mut store = RecentStore::new();
        store.add(&PathBuf::from("/home/user/repo1"));
        store.add(&PathBuf::from("/home/user/repo2"));

        let serialized = toml::to_string(&store).unwrap();
        assert!(serialized.contains("repositories"));
        assert!(serialized.contains("repo1"));
    }

    #[test]
    fn test_recent_store_deserialize() {
        let toml_str = r#"
            [[repositories]]
            path = "/home/user/repo1"
            opened_at = "2026-03-07T10:00:00Z"
            
            [[repositories]]
            path = "/home/user/repo2"
            opened_at = "2026-03-07T09:00:00Z"
        "#;

        let store: RecentStore = toml::from_str(toml_str).unwrap();
        assert_eq!(store.len(), 2);
        assert!(store.contains(&PathBuf::from("/home/user/repo1")));
    }
}
