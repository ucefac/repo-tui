//! Git status TTL cache implementation

use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::repo::GitStatus;

/// Git status with timestamp
#[derive(Debug, Clone)]
pub struct CachedGitStatus {
    /// Git status information
    pub status: GitStatus,
    /// When this status was cached
    pub cached_at: Instant,
}

/// Thread-safe TTL cache for Git status
pub struct StatusCache {
    /// Internal cache storage
    cache: DashMap<PathBuf, CachedGitStatus>,
    /// Time-to-live duration
    ttl: Duration,
}

impl StatusCache {
    /// Create a new cache with specified TTL
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            cache: DashMap::new(),
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    /// Create a new cache with default 5-minute TTL
    pub fn default_cache() -> Self {
        Self::new(300) // 5 minutes
    }

    /// Get cached status if it exists and is not expired
    pub fn get(&self, path: &Path) -> Option<CachedGitStatus> {
        self.cache.get(path).and_then(|entry| {
            if entry.cached_at.elapsed() < self.ttl {
                Some(entry.value().clone())
            } else {
                None // Expired, needs refresh
            }
        })
    }

    /// Insert a new status into the cache
    pub fn insert(&self, path: PathBuf, status: GitStatus) {
        let cached = CachedGitStatus {
            status,
            cached_at: Instant::now(),
        };
        self.cache.insert(path, cached);
    }

    /// Remove a specific path from cache
    pub fn remove(&self, path: &Path) {
        self.cache.remove(path);
    }

    /// Clear all expired entries
    pub fn cleanup(&self) {
        self.cache.retain(|_, v| v.cached_at.elapsed() < self.ttl);
    }

    /// Get the number of cached entries
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Clear all entries
    pub fn clear(&self) {
        self.cache.clear();
    }
}

impl Default for StatusCache {
    fn default() -> Self {
        Self::default_cache()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_cache_insert_and_get() {
        let cache = StatusCache::new(60);
        let path = PathBuf::from("/tmp/test-repo");
        let status = GitStatus {
            is_dirty: true,
            branch: Some("main".to_string()),
            ahead: None,
            behind: None,
        };

        cache.insert(path.clone(), status.clone());
        let cached = cache.get(&path);

        assert!(cached.is_some());
        let cached = cached.unwrap();
        assert!(cached.status.is_dirty);
        assert_eq!(cached.status.branch, Some("main".to_string()));
    }

    #[test]
    fn test_cache_ttl_expiry() {
        let cache = StatusCache::new(1); // 1 second TTL
        let path = PathBuf::from("/tmp/test-repo");
        let status = GitStatus::clean();

        cache.insert(path.clone(), status);
        assert!(cache.get(&path).is_some());

        // Wait for TTL to expire
        thread::sleep(Duration::from_secs(2));

        assert!(cache.get(&path).is_none());
    }

    #[test]
    fn test_cache_cleanup() {
        let cache = StatusCache::new(1); // 1 second TTL
        let path1 = PathBuf::from("/tmp/repo1");
        let path2 = PathBuf::from("/tmp/repo2");

        cache.insert(path1.clone(), GitStatus::clean());
        thread::sleep(Duration::from_secs(2));
        cache.insert(path2.clone(), GitStatus::dirty());

        // path1 is expired, path2 is fresh
        cache.cleanup();

        assert!(cache.get(&path1).is_none());
        assert!(cache.get(&path2).is_some());
    }

    #[test]
    fn test_cache_remove() {
        let cache = StatusCache::new(60);
        let path = PathBuf::from("/tmp/test-repo");

        cache.insert(path.clone(), GitStatus::clean());
        assert!(cache.get(&path).is_some());

        cache.remove(&path);
        assert!(cache.get(&path).is_none());
    }

    #[test]
    fn test_cache_len_and_clear() {
        let cache = StatusCache::new(60);
        let path1 = PathBuf::from("/tmp/repo1");
        let path2 = PathBuf::from("/tmp/repo2");

        assert!(cache.is_empty());

        cache.insert(path1, GitStatus::clean());
        cache.insert(path2, GitStatus::dirty());
        assert_eq!(cache.len(), 2);

        cache.clear();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_default() {
        let cache = StatusCache::default();
        assert_eq!(cache.ttl, Duration::from_secs(300));
    }

    #[test]
    fn test_cache_concurrent_access() {
        let cache = Arc::new(StatusCache::new(60));
        let mut handles = vec![];

        for i in 0..10 {
            let cache = Arc::clone(&cache);
            handles.push(thread::spawn(move || {
                let path = PathBuf::from(format!("/tmp/repo{}", i));
                cache.insert(path.clone(), GitStatus::clean());
                assert!(cache.get(&path).is_some());
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(cache.len(), 10);
    }
}
