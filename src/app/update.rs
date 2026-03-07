//! Application update logic

use crate::app::model::App;
use crate::app::msg::AppMsg;
use crate::app::state::AppState;
use crate::config;
use crate::constants;
use crate::runtime::executor::Runtime;
use std::path::PathBuf;
use std::sync::Arc;

/// Update application state based on message
pub fn update(msg: AppMsg, app: &mut App, runtime: &Runtime) {
    match msg {
        // === Search Input ===
        AppMsg::SearchInput(c) => {
            // Always accept search input
            app.search_query.push(c);
            app.search_active = true; // Ensure focus
            app.pending_search = Some(app.search_query.clone());

            // Schedule debounce
            runtime.dispatch_after(
                crate::app::msg::AppMsg::Tick,
                std::time::Duration::from_millis(constants::SEARCH_DEBOUNCE_MS),
            );
        }

        AppMsg::SearchBackspace => {
            app.search_query.pop();

            // Exit search focus if query is empty
            if app.search_query.is_empty() {
                app.search_active = false;
                app.pending_search = None;
                app.apply_filter();
            } else {
                app.pending_search = Some(app.search_query.clone());

                runtime.dispatch_after(
                    crate::app::msg::AppMsg::Tick,
                    std::time::Duration::from_millis(constants::SEARCH_DEBOUNCE_MS),
                );
            }
        }

        AppMsg::SearchClear => {
            app.search_query.clear();
            app.pending_search = None;
            app.apply_filter();
        }

        // === Navigation ===
        AppMsg::NextRepo => {
            if app.filtered_indices.is_empty() {
                return;
            }
            let current = app.selected_index().unwrap_or(0);
            let next = (current + 1).min(app.filtered_indices.len() - 1);
            app.set_selected_index(Some(next));
        }

        AppMsg::PreviousRepo => {
            if app.filtered_indices.is_empty() {
                return;
            }
            let current = app.selected_index().unwrap_or(0);
            let prev = current.saturating_sub(1);
            app.set_selected_index(Some(prev));
        }

        AppMsg::JumpToTop => {
            if !app.filtered_indices.is_empty() {
                app.set_selected_index(Some(0));
            }
        }

        AppMsg::JumpToBottom => {
            if !app.filtered_indices.is_empty() {
                app.set_selected_index(Some(app.filtered_indices.len() - 1));
            }
        }

        AppMsg::ScrollDown | AppMsg::ScrollUp => {
            // Handled in view with terminal height
        }

        // === Async Events ===
        AppMsg::ConfigLoaded(result) => {
            match result {
                Ok(config) => {
                    app.main_dir = Some(config.main_directory.clone());
                    app.config = Some(config.clone());

                    // Start loading repositories
                    runtime.dispatch(crate::app::msg::Cmd::LoadRepositories(
                        config.main_directory,
                    ));
                }
                Err(e) => {
                    app.error_message = Some(e.user_message());

                    // All configuration errors trigger directory chooser
                    // This handles: NotFound, empty path, invalid path, etc.
                    app.state = AppState::ChoosingDir {
                        path: dirs::home_dir().unwrap_or_default(),
                        entries: Vec::new(),
                        selected_index: 0,
                        scroll_offset: 0,
                    };
                    runtime.dispatch(crate::app::msg::Cmd::ScanDirectory(
                        dirs::home_dir().unwrap_or_default(),
                    ));
                }
            }
            app.loading = false;
            app.loading_message = None;
        }

        AppMsg::RepositoriesLoaded(result) => match result {
            Ok(repos) => {
                app.repositories = repos;
                app.apply_filter();
                app.state = AppState::Running;

                // Schedule Git status checking in background
                if let Some(ref scheduler) = app.git_scheduler {
                    let repos_with_idx: Vec<_> = app
                        .repositories
                        .iter()
                        .enumerate()
                        .map(|(i, r)| (i, r.clone()))
                        .collect();

                    // Clone scheduler for async task
                    let scheduler_clone = Arc::clone(scheduler);
                    tokio::spawn(async move {
                        scheduler_clone.schedule_batch(repos_with_idx).await;
                    });
                }
            }
            Err(e) => {
                app.error_message = Some(e.to_string());
                app.state = AppState::Error {
                    message: e.to_string(),
                };
            }
        },

        AppMsg::GitStatusChecked(idx, result) => {
            if let Ok(status) = result {
                if let Some(repo) = app.repositories.get_mut(idx) {
                    repo.is_dirty = status.is_dirty;
                    repo.branch = status.branch;
                }
            }
        }

        AppMsg::ActionExecuted(result) => {
            if let Err(e) = result {
                app.error_message = Some(e.user_message());
            }
        }

        // === State Transitions ===
        AppMsg::OpenActions => {
            if let Some(repo) = app.selected_repository().cloned() {
                app.selected_repo = Some(repo.clone());
                app.state = AppState::ShowingActions { repo };
            }
        }

        AppMsg::CloseActions => {
            app.selected_repo = None;
            app.state = AppState::Running;
        }

        AppMsg::ExecuteAction(action) => {
            if let Some(repo) = app.selected_repo.clone() {
                runtime.dispatch(crate::app::msg::Cmd::ExecuteAction(action, repo));
                app.state = AppState::Running;
                app.selected_repo = None;
            }
        }

        AppMsg::ShowHelp => {
            app.state = AppState::ShowingHelp;
        }

        AppMsg::CloseHelp => {
            app.state = AppState::Running;
        }

        AppMsg::ShowDirectoryChooser => {
            // Ignore if search is active (prevent stale messages from opening chooser)
            if app.search_active {
                return;
            }

            app.state = AppState::ChoosingDir {
                path: dirs::home_dir().unwrap_or_default(),
                entries: Vec::new(),
                selected_index: 0,
                scroll_offset: 0,
            };
            runtime.dispatch(crate::app::msg::Cmd::ScanDirectory(
                dirs::home_dir().unwrap_or_default(),
            ));
        }

        AppMsg::DirectorySelected(path) => {
            // Save configuration
            let mut config = app.config.clone().unwrap_or_default();
            config.main_directory = PathBuf::from(path);

            match config::save_config(&config) {
                Ok(()) => {
                    // Update state
                    app.config = Some(config.clone());
                    app.main_dir = Some(config.main_directory.clone());

                    // Clear any previous error messages
                    app.error_message = None;

                    // Set state to Loading while repositories are being discovered
                    app.state = AppState::Loading {
                        message: "Discovering repositories...".to_string(),
                    };

                    // Load repositories
                    runtime.dispatch(crate::app::msg::Cmd::LoadRepositories(
                        config.main_directory,
                    ));
                }
                Err(e) => {
                    app.error_message = Some(format!("Failed to save config: {}", e));
                    app.state = AppState::Error {
                        message: format!("Failed to save configuration: {}", e),
                    };
                }
            }
        }

        AppMsg::DirectoryEntriesScanned(entries) => {
            if let AppState::ChoosingDir {
                entries: ref mut e,
                selected_index: ref mut idx,
                ..
            } = app.state
            {
                *e = entries;
                *idx = 0;
            }
        }

        AppMsg::ScanError(error) => {
            if let AppState::ChoosingDir {
                entries: ref mut e, ..
            } = app.state
            {
                *e = vec![format!("Error: {}", error)];
            }
        }

        AppMsg::DirectoryNavDown => {
            if let AppState::ChoosingDir {
                entries,
                selected_index,
                scroll_offset,
                ..
            } = &mut app.state
            {
                if !entries.is_empty() {
                    *selected_index = (*selected_index + 1).min(entries.len() - 1);
                    // Auto-scroll: ensure selected item is visible
                    // Assuming visible height of ~15 items (adjust as needed)
                    let visible_count = 15usize;
                    if *selected_index >= *scroll_offset + visible_count {
                        *scroll_offset = selected_index.saturating_sub(visible_count - 1);
                    }
                }
            }
        }

        AppMsg::DirectoryNavUp => {
            if let AppState::ChoosingDir {
                entries: _,
                selected_index,
                scroll_offset,
                ..
            } = &mut app.state
            {
                *selected_index = selected_index.saturating_sub(1);
                // Auto-scroll: ensure selected item is visible
                if *selected_index < *scroll_offset {
                    *scroll_offset = *selected_index;
                }
            }
        }

        AppMsg::Refresh => {
            if let Some(path) = app.main_dir.clone() {
                app.state = AppState::Loading {
                    message: "Refreshing repositories...".to_string(),
                };
                runtime.dispatch(crate::app::msg::Cmd::LoadRepositories(path));
            }
        }

        // === Global ===
        AppMsg::Tick => {
            // Handle search debounce
            if let Some(query) = app.pending_search.take() {
                app.search_query = query;
                app.apply_filter();
            }
        }

        AppMsg::Quit => {
            app.state = AppState::Quit;
        }

        AppMsg::Cancel => match &app.state {
            AppState::ShowingActions { .. } => {
                app.state = AppState::Running;
                app.selected_repo = None;
            }
            AppState::ShowingHelp => {
                app.state = AppState::Running;
            }
            AppState::Running => {
                // Cancel from search focus
                if app.search_active {
                    app.search_active = false;
                    app.search_query.clear();
                    app.apply_filter();
                }
            }
            _ => {}
        },

        AppMsg::ActionMenuNavDown => {
            // Navigate down in action menu
            // This would be handled by the ActionMenu widget state
            // For now, just log
        }

        AppMsg::ActionMenuNavUp => {
            // Navigate up in action menu
            // This would be handled by the ActionMenu widget state
            // For now, just log
        }

        AppMsg::ShowError(message) => {
            app.state = AppState::Error { message };
        }

        AppMsg::CloseError => {
            if matches!(app.state, AppState::Error { .. }) {
                app.state = AppState::Running;
            }
        }

        AppMsg::CopyPathToClipboard(path) => {
            use arboard::Clipboard;

            match Clipboard::new().and_then(|mut c| c.set_text(path.to_string_lossy().to_string()))
            {
                Ok(()) => {
                    app.loading_message = Some("✅ Path copied to clipboard".to_string());
                }
                Err(e) => {
                    app.error_message = Some(format!("Failed to copy path: {}", e));
                }
            }
        }

        AppMsg::ThemeChanged => {
            // Toggle theme
            app.theme = app.theme.toggle();

            // Update theme in config
            if let Some(ref mut config) = app.config {
                config.ui.theme = app.theme.name.clone();

                // Save configuration with better error handling
                match config::save_config(config) {
                    Ok(()) => {
                        // Show success message briefly
                        app.loading_message = Some("Theme saved".to_string());
                    }
                    Err(e) => {
                        app.error_message = Some(format!(
                            "Failed to save theme: {}. Theme will reset on restart.",
                            e
                        ));
                        // Rollback theme in app state
                        app.theme = app.theme.toggle();
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::msg::AppMsg;
    use crate::app::state::AppState;
    use crate::repo::Repository;

    #[test]
    fn test_directory_nav_down() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        app.state = AppState::ChoosingDir {
            path: std::path::PathBuf::from("/tmp"),
            entries: vec!["dir1".to_string(), "dir2".to_string(), "dir3".to_string()],
            selected_index: 0,
            scroll_offset: 0,
        };

        update(AppMsg::DirectoryNavDown, &mut app, &runtime);

        if let AppState::ChoosingDir { selected_index, .. } = app.state {
            assert_eq!(selected_index, 1);
        } else {
            panic!("State should be ChoosingDir");
        }
    }

    #[test]
    fn test_directory_nav_up() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        app.state = AppState::ChoosingDir {
            path: std::path::PathBuf::from("/tmp"),
            entries: vec!["dir1".to_string(), "dir2".to_string(), "dir3".to_string()],
            selected_index: 2,
            scroll_offset: 0,
        };

        update(AppMsg::DirectoryNavUp, &mut app, &runtime);

        if let AppState::ChoosingDir { selected_index, .. } = app.state {
            assert_eq!(selected_index, 1);
        } else {
            panic!("State should be ChoosingDir");
        }
    }

    #[test]
    fn test_update_next_repo() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        app.filtered_indices = vec![0, 1, 2];
        app.set_selected_index(Some(0));

        update(AppMsg::NextRepo, &mut app, &runtime);
        assert_eq!(app.selected_index(), Some(1));

        update(AppMsg::NextRepo, &mut app, &runtime);
        assert_eq!(app.selected_index(), Some(2));
    }

    #[test]
    fn test_update_previous_repo() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        app.filtered_indices = vec![0, 1, 2];
        app.set_selected_index(Some(2));

        update(AppMsg::PreviousRepo, &mut app, &runtime);
        assert_eq!(app.selected_index(), Some(1));

        update(AppMsg::PreviousRepo, &mut app, &runtime);
        assert_eq!(app.selected_index(), Some(0));
    }

    #[test]
    fn test_update_jump_to_top() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        app.filtered_indices = vec![0, 1, 2, 3, 4];
        app.set_selected_index(Some(3));

        update(AppMsg::JumpToTop, &mut app, &runtime);
        assert_eq!(app.selected_index(), Some(0));
    }

    #[test]
    fn test_update_jump_to_bottom() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        app.filtered_indices = vec![0, 1, 2, 3, 4];
        app.set_selected_index(Some(0));

        update(AppMsg::JumpToBottom, &mut app, &runtime);
        assert_eq!(app.selected_index(), Some(4));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_update_search_input() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        app.state = AppState::Running;

        update(AppMsg::SearchInput('t'), &mut app, &runtime);
        assert_eq!(app.search_query, "t");
        assert!(app.search_active);

        update(AppMsg::SearchInput('e'), &mut app, &runtime);
        assert_eq!(app.search_query, "te");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_update_search_backspace() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        app.search_active = true;
        app.search_query = "test".to_string();

        update(AppMsg::SearchBackspace, &mut app, &runtime);
        assert_eq!(app.search_query, "tes");
    }

    #[test]
    fn test_update_search_clear() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        app.repositories = vec![Repository {
            name: "repo1".to_string(),
            path: std::path::PathBuf::from("/tmp/repo1"),
            last_modified: None,
            is_dirty: false,
            branch: Some("main".to_string()),
        }];
        app.apply_filter();

        app.search_query = "xyz".to_string();
        update(AppMsg::SearchClear, &mut app, &runtime);

        assert!(app.search_query.is_empty());
        assert_eq!(app.filtered_count(), 1);
    }

    #[test]
    fn test_update_open_close_actions() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        app.repositories = vec![Repository {
            name: "repo1".to_string(),
            path: std::path::PathBuf::from("/tmp/repo1"),
            last_modified: None,
            is_dirty: false,
            branch: Some("main".to_string()),
        }];
        app.apply_filter();
        app.set_selected_index(Some(0));
        app.state = AppState::Running;

        // Open actions
        update(AppMsg::OpenActions, &mut app, &runtime);
        assert!(matches!(app.state, AppState::ShowingActions { .. }));
        assert!(app.selected_repo.is_some());

        // Close actions
        update(AppMsg::CloseActions, &mut app, &runtime);
        assert!(matches!(app.state, AppState::Running));
        assert!(app.selected_repo.is_none());
    }

    #[test]
    fn test_update_show_close_help() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        app.state = AppState::Running;

        update(AppMsg::ShowHelp, &mut app, &runtime);
        assert!(matches!(app.state, AppState::ShowingHelp));

        update(AppMsg::CloseHelp, &mut app, &runtime);
        assert!(matches!(app.state, AppState::Running));
    }

    #[test]
    fn test_update_cancel() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        // Cancel from ShowingActions
        app.state = AppState::ShowingActions {
            repo: Repository {
                name: "test".to_string(),
                path: std::path::PathBuf::from("/tmp/test"),
                last_modified: None,
                is_dirty: false,
                branch: None,
            },
        };
        app.selected_repo = Some(Repository {
            name: "test".to_string(),
            path: std::path::PathBuf::from("/tmp/test"),
            last_modified: None,
            is_dirty: false,
            branch: None,
        });

        update(AppMsg::Cancel, &mut app, &runtime);
        assert!(matches!(app.state, AppState::Running));
        assert!(app.selected_repo.is_none());

        // Cancel from search focus
        app.search_active = true;
        app.search_query = "test".to_string();

        update(AppMsg::Cancel, &mut app, &runtime);
        assert!(!app.search_active);
        assert!(app.search_query.is_empty());
    }

    #[test]
    fn test_update_quit() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        app.state = AppState::Running;

        update(AppMsg::Quit, &mut app, &runtime);
        assert!(matches!(app.state, AppState::Quit));
    }
}
