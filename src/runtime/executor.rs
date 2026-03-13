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
                                let mut entries: Vec<_> = entries
                                    .filter_map(|e| {
                                        e.ok().and_then(|entry| {
                                            let file_name = entry.file_name();
                                            let name = file_name.to_string_lossy();
                                            // Filter hidden folders (starting with ".")
                                            (entry.path().is_dir() && !name.starts_with('.'))
                                                .then(|| name.to_string())
                                        })
                                    })
                                    .collect();
                                // Sort directory entries alphabetically
                                entries.sort();
                                entries
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

            Cmd::LoadRepositoriesMulti {
                main_dirs,
                single_repos,
            } => {
                tokio::spawn(async move {
                    let result: Result<Vec<_>, RepoError> =
                        tokio::task::spawn_blocking(move || {
                            let mut all_repos = Vec::new();
                            let mut seen_paths = std::collections::HashSet::new();

                            // Load from main directories
                            for (dir_index, (path, _max_depth)) in main_dirs.iter().enumerate() {
                                match repo::discover_repositories(path) {
                                    Ok(repos) => {
                                        for mut repo in repos {
                                            if seen_paths.insert(repo.path.clone()) {
                                                // Set correct source for main directory repos
                                                repo.source =
                                                    crate::repo::RepoSource::MainDirectory {
                                                        dir_index,
                                                        dir_path: path.clone(),
                                                    };
                                                all_repos.push(repo);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        return Err(RepoError::ScanFailed(format!(
                                            "Failed to scan {}: {}",
                                            path.display(),
                                            e
                                        )));
                                    }
                                }
                            }

                            // Load standalone repositories
                            for path in single_repos {
                                if seen_paths.insert(path.clone()) {
                                    let repo = crate::repo::Repository::from_path_with_source(
                                        path,
                                        crate::repo::RepoSource::Standalone,
                                    );
                                    all_repos.push(repo);
                                }
                            }

                            Ok(all_repos)
                        })
                        .await
                        .map_err(|_| RepoError::ScanFailed("Task join failed".to_string()))
                        .and_then(|r| r);

                    let _ = msg_tx.send(AppMsg::RepositoriesLoaded(result)).await;
                });
            }

            Cmd::SaveConfig(_config) => {
                // TODO: Implement config saving
                tokio::spawn(async move {
                    let _ = msg_tx
                        .send(AppMsg::ShowError(
                            "Config saving not yet implemented".to_string(),
                        ))
                        .await;
                });
            }

            Cmd::ValidateDirectory(_path) => {
                // TODO: Implement directory validation
                tokio::spawn(async move {
                    let _ = msg_tx
                        .send(AppMsg::ShowError(
                            "Directory validation not yet implemented".to_string(),
                        ))
                        .await;
                });
            }

            Cmd::CloneRepository { url, target_path } => {
                tokio::spawn(async move {
                    use tokio::io::AsyncBufReadExt;
                    use tokio::process::Command;

                    tracing::info!("Starting git clone: {} -> {:?}", url, target_path);

                    // Create parent directory if it doesn't exist
                    if let Some(parent) = target_path.parent() {
                        if let Err(e) = tokio::fs::create_dir_all(parent).await {
                            let _ = msg_tx
                                .send(AppMsg::CloneCompleted(Err(crate::error::CloneError::Io(
                                    e.to_string(),
                                ))))
                                .await;
                            return;
                        }
                    }

                    // Spawn git clone process
                    let mut child = match Command::new("git")
                        .args([
                            "clone",
                            "--progress",
                            &url,
                            target_path.to_string_lossy().as_ref(),
                        ])
                        .stderr(std::process::Stdio::piped())
                        .stdout(std::process::Stdio::null())
                        .spawn()
                    {
                        Ok(c) => c,
                        Err(e) => {
                            tracing::error!("Failed to spawn git clone: {}", e);
                            let error = if e.kind() == std::io::ErrorKind::NotFound {
                                crate::error::CloneError::GitNotFound
                            } else {
                                crate::error::CloneError::Io(e.to_string())
                            };
                            let _ = msg_tx.send(AppMsg::CloneCompleted(Err(error))).await;
                            return;
                        }
                    };

                    // Read progress from stderr using a simple line reader
                    if let Some(stderr) = child.stderr.take() {
                        let mut buf_reader = tokio::io::BufReader::new(stderr);
                        let mut line = String::new();

                        loop {
                            match buf_reader.read_line(&mut line).await {
                                Ok(0) => break, // EOF
                                Ok(_) => {
                                    let trimmed = line.trim_end().to_string();
                                    if !trimmed.is_empty() {
                                        tracing::debug!("git clone progress: {}", trimmed);
                                        let _ = msg_tx.send(AppMsg::CloneProgress(trimmed)).await;
                                    }
                                    line.clear();
                                }
                                Err(e) => {
                                    tracing::error!("Error reading git clone output: {}", e);
                                    break;
                                }
                            }
                        }
                    }

                    // Wait for process to complete
                    match child.wait().await {
                        Ok(status) => {
                            if status.success() {
                                tracing::info!("git clone completed successfully");
                                // Create a minimal repository info
                                let repo = crate::repo::Repository::from_path(target_path);
                                let _ = msg_tx.send(AppMsg::CloneCompleted(Ok(repo))).await;
                            } else {
                                tracing::error!(
                                    "git clone failed with status: {:?}",
                                    status.code()
                                );
                                let _ = msg_tx
                                    .send(AppMsg::CloneCompleted(Err(
                                        crate::error::CloneError::GitFailed(status.code()),
                                    )))
                                    .await;
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to wait for git clone: {}", e);
                            let _ = msg_tx
                                .send(AppMsg::CloneCompleted(Err(crate::error::CloneError::Io(
                                    e.to_string(),
                                ))))
                                .await;
                        }
                    }
                });
            }

            Cmd::CheckForUpdate => {
                tokio::spawn(async move {
                    // Get config for GitHub repo info
                    let config = config::load_or_create_config();
                    let (owner, repo) = match config {
                        Ok(ref cfg) => (
                            cfg.update.github_owner.clone(),
                            cfg.update.github_repo.clone(),
                        ),
                        Err(_) => ("yyyyyyh".to_string(), "ghclone".to_string()),
                    };

                    let result = crate::update::check_for_update(&owner, &repo).await;
                    let _ = msg_tx
                        .send(AppMsg::UpdateCheckCompleted(Box::new(result)))
                        .await;
                });
            }

            Cmd::MoveRepository(repo, target_main_dir) => {
                tokio::spawn(async move {
                    tracing::info!(
                        "Moving repository {} to {:?}",
                        repo.name,
                        target_main_dir
                    );

                    // Perform the move operation
                    let result = tokio::task::spawn_blocking(move || {
                        repo::move_module::move_repository(&repo, &target_main_dir)
                    })
                    .await;

                    // Handle the result
                    let result = match result {
                        Ok(Ok(new_path)) => Ok(new_path),
                        Ok(Err(e)) => Err(e),
                        Err(e) => Err(crate::error::MoveError::Io(e.to_string())),
                    };

                    let _ = msg_tx.send(AppMsg::MoveCompleted(result)).await;
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
