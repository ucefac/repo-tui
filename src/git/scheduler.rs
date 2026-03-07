//! Background Git status scheduler

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::app::msg::AppMsg;
use crate::git::cache::StatusCache;
use crate::git::status::check_git_status;
use crate::repo::Repository;

/// Background scheduler for Git status checks
pub struct GitStatusScheduler {
    /// Shared cache
    cache: Arc<StatusCache>,
    /// Message sender for UI updates
    msg_tx: mpsc::Sender<AppMsg>,
}

impl GitStatusScheduler {
    /// Create a new scheduler
    pub fn new(cache: Arc<StatusCache>, msg_tx: mpsc::Sender<AppMsg>) -> Self {
        Self { cache, msg_tx }
    }

    /// Schedule a Git status check for a single repository
    pub async fn schedule_check(&self, index: usize, path: PathBuf) {
        // Check cache first
        if let Some(cached) = self.cache.get(&path) {
            // Cache hit - send immediately
            let _ = self
                .msg_tx
                .send(AppMsg::GitStatusChecked(index, Ok(cached.status)))
                .await;
            return;
        }

        // Cache miss - spawn async task
        let cache = Arc::clone(&self.cache);
        let msg_tx = self.msg_tx.clone();
        let path_clone = path.clone();

        tokio::spawn(async move {
            match check_git_status(&path).await {
                Ok(status) => {
                    // Cache the result
                    cache.insert(path_clone.clone(), status.clone());

                    // Send to UI
                    let _ = msg_tx
                        .send(AppMsg::GitStatusChecked(index, Ok(status)))
                        .await;
                }
                Err(e) => {
                    // Send error to UI (non-fatal)
                    let repo_error = crate::error::RepoError::GitError(e.to_string());
                    let _ = msg_tx
                        .send(AppMsg::GitStatusChecked(index, Err(repo_error)))
                        .await;
                }
            }
        });
    }

    /// Schedule batch Git status checks with concurrency limit
    pub async fn schedule_batch(&self, repos: Vec<(usize, Repository)>) {
        if repos.is_empty() {
            return;
        }

        // Process in batches to limit concurrency
        const BATCH_SIZE: usize = 10;

        for chunk in repos.chunks(BATCH_SIZE) {
            let futures: Vec<_> = chunk
                .iter()
                .map(|(idx, repo)| self.schedule_check(*idx, repo.path.clone()))
                .collect();

            // Wait for this batch to complete
            futures::future::join_all(futures).await;
        }
    }

    /// Schedule status checks for all repositories
    pub async fn schedule_all(&self, repositories: &mut [Repository]) {
        let repos: Vec<(usize, Repository)> = repositories
            .iter()
            .enumerate()
            .map(|(idx, repo)| (idx, repo.clone()))
            .collect();

        self.schedule_batch(repos).await;
    }

    /// Refresh status for a specific repository (bypass cache)
    pub async fn refresh_status(&self, index: usize, path: PathBuf) {
        // Remove from cache first
        self.cache.remove(&path);

        // Then check fresh status
        self.schedule_check(index, path).await;
    }

    /// Get cache statistics
    pub fn cache_len(&self) -> usize {
        self.cache.len()
    }

    /// Cleanup expired cache entries
    pub fn cleanup_cache(&self) {
        self.cache.cleanup();
    }
}

/// Helper function to check if a path needs git status check
pub fn needs_status_check(path: &Path) -> bool {
    path.join(".git").exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::GitStatus;
    use std::time::Duration;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_schedule_check_cache_miss() {
        let cache = Arc::new(StatusCache::new(60));
        let (tx, mut rx) = mpsc::channel::<AppMsg>(100);
        let scheduler = GitStatusScheduler::new(cache, tx);

        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        // Non-git directory
        scheduler.schedule_check(0, path.clone()).await;

        // Give async task time to complete
        tokio::time::sleep(Duration::from_millis(100)).await;

        if let Ok(msg) = rx.try_recv() {
            if let AppMsg::GitStatusChecked(idx, result) = msg {
                assert_eq!(idx, 0);
                assert!(result.is_ok());
            }
        }
    }

    #[tokio::test]
    async fn test_schedule_check_cache_hit() {
        let cache = Arc::new(StatusCache::new(60));
        let (tx, mut rx) = mpsc::channel::<AppMsg>(100);
        let scheduler = GitStatusScheduler::new(cache.clone(), tx);

        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        // Pre-populate cache
        let status = GitStatus {
            is_dirty: true,
            branch: Some("main".to_string()),
            ahead: None,
            behind: None,
        };
        cache.insert(path.clone(), status.clone());

        // This should use cache
        scheduler.schedule_check(0, path.clone()).await;

        // Should receive immediately
        if let Ok(msg) = rx.try_recv() {
            if let AppMsg::GitStatusChecked(idx, result) = msg {
                assert_eq!(idx, 0);
                let returned_status = result.unwrap();
                assert!(returned_status.is_dirty);
                assert_eq!(returned_status.branch, Some("main".to_string()));
            }
        }
    }

    #[tokio::test]
    async fn test_schedule_batch() {
        let cache = Arc::new(StatusCache::new(60));
        let (tx, mut rx) = mpsc::channel::<AppMsg>(100);
        let scheduler = GitStatusScheduler::new(cache, tx);

        let temp_dir = TempDir::new().unwrap();
        let repos: Vec<(usize, Repository)> = (0..5)
            .map(|i| {
                (
                    i,
                    Repository {
                        name: format!("repo{}", i),
                        path: temp_dir.path().join(format!("repo{}", i)),
                        last_modified: None,
                        is_dirty: false,
                        branch: None,
                    },
                )
            })
            .collect();

        scheduler.schedule_batch(repos).await;

        // Give async tasks time to complete
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Should have received 5 messages
        let mut count = 0;
        while let Ok(msg) = rx.try_recv() {
            if let AppMsg::GitStatusChecked(_, _) = msg {
                count += 1;
            }
        }
        assert_eq!(count, 5);
    }

    #[tokio::test]
    async fn test_refresh_status() {
        let cache = Arc::new(StatusCache::new(60));
        let (tx, _rx) = mpsc::channel::<AppMsg>(100);
        let scheduler = GitStatusScheduler::new(cache.clone(), tx);

        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        // Pre-populate cache
        cache.insert(path.clone(), GitStatus::clean());
        assert!(cache.get(&path).is_some());

        // Refresh should remove from cache
        scheduler.refresh_status(0, path.clone()).await;

        // Cache should be cleared for this path
        assert!(cache.get(&path).is_none());
    }

    #[test]
    fn test_cache_len() {
        let cache = Arc::new(StatusCache::new(60));
        let (tx, _rx) = mpsc::channel::<AppMsg>(100);
        let scheduler = GitStatusScheduler::new(cache.clone(), tx);

        assert_eq!(scheduler.cache_len(), 0);

        cache.insert(PathBuf::from("/tmp/repo1"), GitStatus::clean());
        assert_eq!(scheduler.cache_len(), 1);
    }

    #[test]
    fn test_cleanup_cache() {
        let cache = Arc::new(StatusCache::new(1)); // 1 second TTL
        let (tx, _rx) = mpsc::channel::<AppMsg>(100);
        let scheduler = GitStatusScheduler::new(cache.clone(), tx);

        cache.insert(PathBuf::from("/tmp/repo1"), GitStatus::clean());
        std::thread::sleep(Duration::from_secs(2));

        scheduler.cleanup_cache();
        assert_eq!(scheduler.cache_len(), 0);
    }
}
