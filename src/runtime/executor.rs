//! Runtime executor for async tasks

use crate::action;
use crate::app::msg::{AppMsg, Cmd};
use crate::config;
use crate::error::{ActionError, AppError, ConfigError, RepoError};
use crate::repo;
use tokio::sync::mpsc;

/// Runtime for executing async commands
pub struct Runtime {
    msg_tx: mpsc::Sender<AppMsg>,
}

impl Runtime {
    /// Create a new runtime
    pub fn new(msg_tx: mpsc::Sender<AppMsg>) -> Self {
        Self { msg_tx }
    }

    /// Dispatch a command for execution
    pub fn dispatch(&self, cmd: Cmd) {
        let msg_tx = self.msg_tx.clone();

        match cmd {
            Cmd::LoadConfig => {
                tokio::spawn(async move {
                    let result = config::load_or_create_config().map_err(|e| {
                        if let AppError::Config(ce) = e {
                            ce
                        } else {
                            ConfigError::PathError(e.to_string())
                        }
                    });
                    let _ = msg_tx.send(AppMsg::ConfigLoaded(Box::new(result))).await;
                });
            }

            Cmd::LoadRepositories(path) => {
                tokio::spawn(async move {
                    let result: Result<Vec<_>, RepoError> =
                        tokio::task::spawn_blocking(move || {
                            repo::discover_repositories(&path).map_err(|e| {
                                if let AppError::Repo(re) = e {
                                    re
                                } else {
                                    RepoError::ScanFailed(e.to_string())
                                }
                            })
                        })
                        .await
                        .map_err(|_| RepoError::ScanFailed("Task join failed".to_string()))
                        .and_then(|r| r);

                    let _ = msg_tx.send(AppMsg::RepositoriesLoaded(result)).await;
                });
            }

            Cmd::CheckGitStatus(idx, path) => {
                tokio::spawn(async move {
                    let result: Result<_, RepoError> = tokio::task::spawn_blocking(move || {
                        repo::check_git_status(&path).map_err(|e| {
                            if let AppError::Repo(re) = e {
                                re
                            } else {
                                RepoError::GitCommandFailed(e.to_string())
                            }
                        })
                    })
                    .await
                    .map_err(|_| RepoError::GitCommandFailed("Task join failed".to_string()))
                    .and_then(|r| r);

                    let _ = msg_tx.send(AppMsg::GitStatusChecked(idx, result)).await;
                });
            }

            Cmd::ExecuteAction(action, repo) => {
                tokio::spawn(async move {
                    let inner_result = tokio::task::spawn_blocking(move || {
                        action::execute_action(&action, &repo).map_err(|e| {
                            if let AppError::Action(ae) = e {
                                ae
                            } else {
                                ActionError::ExecutionFailed(e.to_string())
                            }
                        })
                    })
                    .await;

                    let result: Result<(), ActionError> = match inner_result {
                        Ok(Ok(())) => Ok(()),
                        Ok(Err(e)) => Err(e),
                        Err(_) => Err(ActionError::ExecutionFailed("Task join failed".to_string())),
                    };

                    let _ = msg_tx.send(AppMsg::ActionExecuted(result)).await;
                });
            }

            Cmd::ExecuteBatchAction(action, repos) => {
                tokio::spawn(async move {
                    // Execute batch action with concurrency limit of 5
                    let result = action::execute_batch(&action, repos, 5).await;
                    let _ = msg_tx.send(AppMsg::BatchActionExecuted(result)).await;
                });
            }

            Cmd::ScanDirectory(path) => {
                tokio::spawn(async move {
                    let result = tokio::task::spawn_blocking(move || {
                        std::fs::read_dir(&path)
                            .map(|entries| {
                                entries
                                    .filter_map(|e| {
                                        e.ok().and_then(|entry| {
                                            entry.path().is_dir().then(|| {
                                                entry.file_name().to_string_lossy().to_string()
                                            })
                                        })
                                    })
                                    .collect::<Vec<_>>()
                            })
                            .map_err(|e| RepoError::ScanFailed(e.to_string()))
                    })
                    .await
                    .map_err(|_| RepoError::ScanFailed("Task join failed".to_string()))
                    .and_then(|r| r);

                    let msg = match result {
                        Ok(entries) => AppMsg::DirectoryEntriesScanned(entries),
                        Err(e) => AppMsg::ScanError(e.to_string()),
                    };

                    let _ = msg_tx.send(msg).await;
                });
            }
        }
    }

    /// Dispatch a message after a delay
    pub fn dispatch_after(&self, msg: AppMsg, delay: std::time::Duration) {
        let msg_tx = self.msg_tx.clone();

        tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            let _ = msg_tx.send(msg).await;
        });
    }
}

impl Clone for Runtime {
    fn clone(&self) -> Self {
        Self {
            msg_tx: self.msg_tx.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runtime_dispatch() {
        let (tx, mut rx) = mpsc::channel(100);
        let runtime = Runtime::new(tx);

        runtime.dispatch(Cmd::LoadConfig);

        let msg = rx.recv().await;
        assert!(matches!(msg, Some(AppMsg::ConfigLoaded(_))));
    }
}
