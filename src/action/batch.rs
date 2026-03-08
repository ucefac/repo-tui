//! Batch action execution

use crate::action::{execute_action, Action};
use crate::error::{ActionError, AppError};
use crate::repo::Repository;
use std::sync::Arc;
use tokio::sync::Semaphore;

/// Batch execution result
#[derive(Debug, Clone)]
pub struct BatchResult {
    /// Total number of operations
    pub total: usize,
    /// Number of successful operations
    pub success: usize,
    /// Number of failed operations
    pub failed: usize,
    /// Errors from failed operations
    pub errors: Vec<(String, ActionError)>,
}

impl BatchResult {
    /// Create a new batch result
    pub fn new(total: usize) -> Self {
        Self {
            total,
            success: 0,
            failed: 0,
            errors: Vec::new(),
        }
    }

    /// Check if all operations succeeded
    pub fn all_succeeded(&self) -> bool {
        self.failed == 0 && self.success == self.total
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.success as f64 / self.total as f64
        }
    }
}

/// Execute batch actions with concurrency control
pub async fn execute_batch(
    action: &Action,
    repos: Vec<Repository>,
    concurrency_limit: usize,
) -> BatchResult {
    let mut result = BatchResult::new(repos.len());

    if repos.is_empty() {
        return result;
    }

    // Create semaphore for concurrency control
    let semaphore = Arc::new(Semaphore::new(concurrency_limit));
    let mut handles = Vec::new();

    // Spawn tasks for each repository
    for repo in repos {
        let sem = Arc::clone(&semaphore);
        let action_clone = action.clone();
        let repo_name = repo.name.clone();

        let handle = tokio::spawn(async move {
            // Acquire permit
            let _permit = sem.acquire().await.unwrap();

            // Execute action
            let result = execute_action(&action_clone, &repo);

            (repo_name, result)
        });

        handles.push(handle);
    }

    // Collect results
    for handle in handles {
        match handle.await {
            Ok((repo_name, exec_result)) => match exec_result {
                Ok(()) => {
                    result.success += 1;
                }
                Err(AppError::Action(action_err)) => {
                    result.failed += 1;
                    result.errors.push((repo_name, action_err));
                }
                Err(other_err) => {
                    result.failed += 1;
                    result.errors.push((
                        repo_name,
                        ActionError::ExecutionFailed(other_err.to_string()),
                    ));
                }
            },
            Err(join_err) => {
                result.failed += 1;
                result.errors.push((
                    "unknown".to_string(),
                    ActionError::ExecutionFailed(format!("Task join error: {}", join_err)),
                ));
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_repo(name: &str) -> Repository {
        Repository {
            name: name.to_string(),
            path: PathBuf::from(format!("/tmp/{}", name)),
            last_modified: None,
            is_dirty: false,
            branch: Some("main".to_string()),
            is_git_repo: true,
            source: crate::repo::source::RepoSource::Standalone,
        }
    }

    #[tokio::test]
    async fn test_batch_result_new() {
        let result = BatchResult::new(5);
        assert_eq!(result.total, 5);
        assert_eq!(result.success, 0);
        assert_eq!(result.failed, 0);
        assert!(result.errors.is_empty());
    }

    #[tokio::test]
    async fn test_batch_result_all_succeeded() {
        let mut result = BatchResult::new(3);
        result.success = 3;
        assert!(result.all_succeeded());

        result.failed = 1;
        assert!(!result.all_succeeded());
    }

    #[tokio::test]
    async fn test_batch_result_success_rate() {
        let mut result = BatchResult::new(10);
        result.success = 7;
        result.failed = 3;

        assert!((result.success_rate() - 0.7).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_batch_result_empty_repos() {
        let repos: Vec<Repository> = vec![];
        let result = execute_batch(&Action::OpenVsCode, repos, 5).await;

        assert_eq!(result.total, 0);
        assert_eq!(result.success, 0);
        assert_eq!(result.failed, 0);
    }

    #[tokio::test]
    async fn test_batch_concurrency_limit() {
        let repos: Vec<Repository> = (0..10)
            .map(|i| create_test_repo(&format!("repo{}", i)))
            .collect();

        // This test just verifies the function doesn't panic with concurrency limit
        let result = execute_batch(&Action::OpenFileManager, repos, 3).await;

        // All should fail since paths don't exist, but function should complete
        assert_eq!(result.total, 10);
    }
}
