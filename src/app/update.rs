//! Application update logic

use crate::app::model::App;
use crate::app::msg::AppMsg;
use crate::app::state::{AppState, ViewMode};
use crate::config;
use crate::constants;
use crate::error::{ActionError, ConfigError};
use crate::repo::Repository;
use crate::runtime::executor::Runtime;
use crate::ui::Theme;
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
            let len = app.filtered_indices.len();
            let next = (current + 1) % len;
            app.set_selected_index(Some(next));
        }

        AppMsg::PreviousRepo => {
            if app.filtered_indices.is_empty() {
                return;
            }
            let current = app.selected_index().unwrap_or(0);
            let len = app.filtered_indices.len();
            let prev = if current == 0 { len - 1 } else { current - 1 };
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
            match *result {
                Ok(config) => {
                    // Load main directories info from config
                    app.main_directories = config
                        .main_directories
                        .iter()
                        .map(|d| {
                            crate::app::model::MainDirectoryInfo {
                                path: d.path.clone(),
                                display_name: d.display_name.clone().unwrap_or_else(|| {
                                    d.path
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("unknown")
                                        .to_string()
                                }),
                                enabled: d.enabled,
                                repo_count: 0, // Will be updated later
                            }
                        })
                        .collect();

                    // Load standalone repositories
                    app.single_repositories = config
                        .single_repositories
                        .iter()
                        .map(|r| r.path.clone())
                        .collect();

                    // Keep backward compatibility with old main_directory field
                    if let Some(ref old_dir) = config.main_directory {
                        if !old_dir.as_os_str().is_empty() {
                            app.main_dir = Some(old_dir.clone());
                        }
                    }

                    app.config = Some(config.clone());

                    // Load favorites from config
                    app.favorites = config.favorites.to_store();

                    // Load recent repositories from config
                    app.recent = config.recent.to_store();

                    // Handle random theme configuration
                    app.theme = if config.ui.theme == "random" {
                        crate::ui::themes::get_random_theme()
                    } else {
                        Theme::new(&config.ui.theme)
                    };

                    // Load repositories from multiple main directories
                    let main_dirs: Vec<_> = config
                        .enabled_main_dirs_with_meta()
                        .into_iter()
                        .map(|(_, path, max_depth)| (path.clone(), max_depth))
                        .collect();

                    if !main_dirs.is_empty() || !app.single_repositories.is_empty() {
                        runtime.dispatch(crate::app::msg::Cmd::LoadRepositoriesMulti {
                            main_dirs,
                            single_repos: app.single_repositories.clone(),
                        });
                    } else if let Some(ref main_dir) = app.main_dir {
                        // Fallback to legacy single directory loading
                        runtime.dispatch(crate::app::msg::Cmd::LoadRepositories(main_dir.clone()));
                    } else {
                        // No directories configured, show directory chooser
                        app.state = AppState::ChoosingDir {
                            path: dirs::home_dir().unwrap_or_default(),
                            entries: Vec::new(),
                            selected_index: 0,
                            scroll_offset: 0,
                            mode: crate::app::state::DirectoryChooserMode::default(),
                            return_to: crate::app::state::ReturnTarget::Running,
                        };
                        runtime.dispatch(crate::app::msg::Cmd::ScanDirectory(
                            dirs::home_dir().unwrap_or_default(),
                        ));
                    }
                }
                Err(e) => {
                    // 首次运行时配置文件不存在是正常流程，不显示错误消息
                    if !matches!(e, ConfigError::NotFound(_)) {
                        app.error_message = Some(e.user_message());
                    }

                    // All configuration errors trigger directory chooser
                    app.state = AppState::ChoosingDir {
                        path: dirs::home_dir().unwrap_or_default(),
                        entries: Vec::new(),
                        selected_index: 0,
                        scroll_offset: 0,
                        mode: crate::app::state::DirectoryChooserMode::default(),
                        return_to: crate::app::state::ReturnTarget::Running,
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
                    // Only update branch if this is a git repo
                    if repo.is_git_repo {
                        repo.branch = status.branch;
                    }
                }
            }
        }

        AppMsg::ActionExecuted(result) => {
            match result {
                Ok(()) => {}
                Err(ActionError::TerminalNeedsReinit) => {
                    // Signal that terminal needs reinitialization
                    let _ = app.msg_tx.try_send(AppMsg::TerminalNeedsReinit);
                }
                Err(e) => {
                    app.error_message = Some(e.user_message());
                }
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
                // Record as recently opened
                app.recent.add(&repo.path);

                // Save recent to config
                if let Some(ref mut config) = app.config {
                    config.recent.repositories = app.recent.get_all().to_vec();
                    let _ = config::save_config(config);
                }

                runtime.dispatch(crate::app::msg::Cmd::ExecuteAction(action, repo));
                app.state = AppState::Running;
                app.selected_repo = None;
            }
        }

        AppMsg::ShowHelp => {
            app.state = AppState::ShowingHelp { scroll_offset: 0 };
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
                mode: crate::app::state::DirectoryChooserMode::SelectMainDirectory {
                    allow_multiple: false,
                    edit_mode: false,
                },
                return_to: crate::app::state::ReturnTarget::Running,
            };
            runtime.dispatch(crate::app::msg::Cmd::ScanDirectory(
                dirs::home_dir().unwrap_or_default(),
            ));
        }

        AppMsg::DirectorySelected(path) => {
            // Create default config if not exists (first launch scenario)
            if app.config.is_none() {
                app.config = Some(config::Config::default());
            }

            let path_buf = PathBuf::from(&path);

            // Get current chooser mode and return target
            let (mode, return_to) = if let AppState::ChoosingDir {
                mode, return_to, ..
            } = &app.state
            {
                (mode.clone(), return_to.clone())
            } else {
                (
                    crate::app::state::DirectoryChooserMode::SelectMainDirectory {
                        allow_multiple: false,
                        edit_mode: false,
                    },
                    crate::app::state::ReturnTarget::Running,
                )
            };

            match mode {
                crate::app::state::DirectoryChooserMode::AddSingleRepository => {
                    // Validate it's a git repository
                    if !path_buf.join(".git").exists() {
                        app.error_message =
                            Some("Selected directory is not a Git repository".to_string());
                        return;
                    }

                    // Add as standalone repository
                    if let Some(ref mut config) = app.config {
                        if let Err(e) = config.add_single_repository(path_buf.clone()) {
                            app.error_message = Some(format!("Failed to add repository: {}", e));
                        } else {
                            app.single_repositories.push(path_buf);
                            let _ = config::save_config(config);
                            runtime.dispatch(crate::app::msg::Cmd::LoadConfig);
                            app.state = AppState::Running;
                        }
                    }
                }
                crate::app::state::DirectoryChooserMode::SelectMainDirectory {
                    edit_mode, ..
                } => {
                    if edit_mode {
                        // Update existing main directory
                        if let Some(ref mut config) = app.config {
                            config.main_directory = Some(path_buf.clone());
                            let _ = config::save_config(config);
                            app.main_dir = Some(path_buf);
                            runtime.dispatch(crate::app::msg::Cmd::LoadConfig);
                        }
                    } else {
                        // Add as new main directory
                        if let Some(ref mut config) = app.config {
                            if let Err(e) = config.add_main_directory(path_buf.clone()) {
                                app.error_message =
                                    Some(format!("Failed to add main directory: {}", e));
                            } else {
                                let display_name = path_buf
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .map(|s| s.to_string());
                                app.main_directories
                                    .push(crate::app::model::MainDirectoryInfo {
                                        path: path_buf.clone(),
                                        display_name: display_name
                                            .unwrap_or_else(|| "unknown".to_string()),
                                        enabled: true,
                                        repo_count: 0,
                                    });
                                let _ = config::save_config(config);
                                runtime.dispatch(crate::app::msg::Cmd::LoadConfig);
                            }
                        }
                    }
                    // Return to appropriate state based on return_to
                    match return_to {
                        crate::app::state::ReturnTarget::ManagingDirs => {
                            // Re-initialize ManagingDirs state
                            let mut list_state = ratatui::widgets::ListState::default();
                            list_state.select(Some(0));
                            app.state = AppState::ManagingDirs {
                                list_state,
                                selected_dir_index: 0,
                                editing: None,
                                confirming_delete: false,
                            };
                        }
                        crate::app::state::ReturnTarget::Running => {
                            app.state = AppState::Running;
                        }
                    }
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
                path: _,
                entries,
                selected_index,
                scroll_offset,
                mode: _,
                return_to: _,
            } = &mut app.state
            {
                if !entries.is_empty() {
                    let len = entries.len();
                    *selected_index = (*selected_index + 1) % len;
                    let visible_count = 15usize;
                    if *selected_index >= *scroll_offset + visible_count {
                        *scroll_offset = selected_index.saturating_sub(visible_count - 1);
                    }
                }
            }
        }

        AppMsg::DirectoryNavUp => {
            if let AppState::ChoosingDir {
                path: _,
                entries,
                selected_index,
                scroll_offset,
                mode: _,
                return_to: _,
            } = &mut app.state
            {
                if !entries.is_empty() {
                    let len = entries.len();
                    *selected_index = if *selected_index == 0 {
                        len - 1
                    } else {
                        *selected_index - 1
                    };
                    if *selected_index < *scroll_offset {
                        *scroll_offset = *selected_index;
                    }
                }
            }
        }

        AppMsg::Refresh => {
            app.state = AppState::Loading {
                message: "Refreshing repositories...".to_string(),
            };
            // Load from all configured sources
            let main_dirs: Vec<_> = app
                .config
                .as_ref()
                .map(|c| {
                    c.enabled_main_dirs_with_meta()
                        .into_iter()
                        .map(|(_, path, max_depth)| (path.clone(), max_depth))
                        .collect()
                })
                .unwrap_or_default();

            runtime.dispatch(crate::app::msg::Cmd::LoadRepositoriesMulti {
                main_dirs,
                single_repos: app.single_repositories.clone(),
            });
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
            AppState::ShowingHelp { .. } => {
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

        AppMsg::OpenThemeSelector => {
            let mut theme_list_state = ratatui::widgets::ListState::default();

            // 根据当前配置决定选中项
            let default_index = if app.config.as_ref().is_some_and(|c| c.ui.theme == "random") {
                0 // "🎲 Random (随机)"
            } else {
                // 查找当前主题在列表中的索引
                app.config
                    .as_ref()
                    .and_then(|c| {
                        crate::ui::themes::THEME_NAMES
                            .iter()
                            .position(|&t| t == c.ui.theme.as_str())
                    })
                    .unwrap_or(1) // 默认选中 dark
            };

            theme_list_state.select(Some(default_index));

            // 固定预览主题（不再随机生成）
            let preview_theme = Theme::dark();

            app.state = AppState::SelectingTheme {
                theme_list_state,
                preview_theme,
            };
        }

        AppMsg::CloseThemeSelector => {
            app.state = AppState::Running;
        }

        AppMsg::SelectTheme(theme_name) => {
            // 确定要应用的主题
            let theme_to_apply = if theme_name.contains("Random") {
                // 选择 random 时，立即随机选择一个具体主题
                crate::ui::themes::get_random_theme()
            } else {
                Theme::new(&theme_name)
            };

            app.theme = theme_to_apply;

            if let Some(ref mut config) = app.config {
                // Save "random" in config if random was selected
                config.ui.theme = if theme_name.contains("Random") {
                    "random".to_string()
                } else {
                    theme_name
                };

                // 保存配置
                match config::save_config(config) {
                    Ok(()) => {
                        app.loading_message = Some(format!("Theme '{}' saved", config.ui.theme));
                    }
                    Err(e) => {
                        app.error_message = Some(format!("Failed to save theme: {}", e));
                    }
                }
            }

            app.state = AppState::Running;
        }

        AppMsg::ThemeNavUp => {
            if let AppState::SelectingTheme {
                theme_list_state, ..
            } = &mut app.state
            {
                let themes = crate::ui::themes::THEME_NAMES;
                if themes.is_empty() {
                    return;
                }
                let current = theme_list_state.selected().unwrap_or(0);
                let len = themes.len();
                let prev = if current == 0 { len - 1 } else { current - 1 };
                theme_list_state.select(Some(prev));
            }
        }

        AppMsg::ThemeNavDown => {
            if let AppState::SelectingTheme {
                theme_list_state, ..
            } = &mut app.state
            {
                let themes = crate::ui::themes::THEME_NAMES;
                if themes.is_empty() {
                    return;
                }
                let current = theme_list_state.selected().unwrap_or(0);
                let len = themes.len();
                let next = (current + 1) % len;
                theme_list_state.select(Some(next));
            }
        }

        // === Favorites ===
        AppMsg::ToggleFavorite => {
            app.toggle_favorite();
            // Save favorites to config
            if let Some(ref mut config) = app.config {
                config.favorites.repositories = app.favorites.get_all().to_vec();
                let _ = config::save_config(config);
            }
        }

        AppMsg::ShowFavorites => {
            app.toggle_view_mode();
        }

        AppMsg::ShowAllRepos => {
            if app.view_mode != ViewMode::All {
                app.view_mode = ViewMode::All;
                app.filter_by_view_mode();
            }
        }

        AppMsg::ShowRecent => {
            if app.view_mode != ViewMode::Recent {
                app.view_mode = ViewMode::Recent;
                app.filter_by_view_mode();
            }
        }

        // === Batch Operations ===
        AppMsg::ToggleSelectionMode => {
            app.toggle_selection_mode();
        }

        AppMsg::ToggleSelection => {
            if app.selection_mode {
                app.toggle_selection();
            }
        }

        AppMsg::SelectAll => {
            if app.selection_mode {
                app.select_all();
            }
        }

        AppMsg::ClearSelection => {
            app.clear_selection();
        }

        AppMsg::ExecuteBatchAction(action) => {
            // Exit selection mode
            app.selection_mode = false;

            // Get selected repositories
            let selected_repos: Vec<Repository> = app
                .get_selected_repos()
                .into_iter()
                .cloned()
                .collect::<Vec<_>>();

            if selected_repos.is_empty() {
                app.error_message = Some("No repositories selected".to_string());
                return;
            }

            // Execute batch action with concurrency limit of 5
            runtime.dispatch(crate::app::msg::Cmd::ExecuteBatchAction(
                action,
                selected_repos,
            ));

            // Clear selection after execution
            app.clear_selection();
        }

        AppMsg::BatchActionExecuted(result) => {
            if result.all_succeeded() {
                app.loading_message = Some(format!(
                    "✓ Batch completed: {}/{} succeeded",
                    result.success, result.total
                ));
            } else {
                app.error_message = Some(format!(
                    "Batch completed with errors: {}/{} succeeded, {} failed",
                    result.success, result.total, result.failed
                ));
            }
        }

        // === Main Directory Management ===
        AppMsg::ShowMainDirectoryManager => {
            let mut list_state = ratatui::widgets::ListState::default();
            list_state.select(Some(0));
            app.state = AppState::ManagingDirs {
                list_state,
                selected_dir_index: 0,
                editing: None,
                confirming_delete: false,
            };
        }

        AppMsg::CloseMainDirectoryManager => {
            app.state = AppState::Running;
        }

        AppMsg::AddMainDirectory(path) => {
            if let Some(ref mut config) = app.config {
                if let Err(e) = config.add_main_directory(path.clone()) {
                    app.error_message = Some(format!("Failed to add main directory: {}", e));
                } else {
                    let display_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.to_string());
                    app.main_directories
                        .push(crate::app::model::MainDirectoryInfo {
                            path: path.clone(),
                            display_name: display_name.unwrap_or_else(|| "unknown".to_string()),
                            enabled: true,
                            repo_count: 0,
                        });
                    let _ = config::save_config(config);
                    runtime.dispatch(crate::app::msg::Cmd::LoadConfig);
                }
            }
        }

        AppMsg::RemoveMainDirectory(index) => {
            if let Some(ref mut config) = app.config {
                if let Err(e) = config.remove_main_directory(index) {
                    app.error_message = Some(format!("Failed to remove main directory: {}", e));
                } else {
                    if index < app.main_directories.len() {
                        app.main_directories.remove(index);
                    }
                    let _ = config::save_config(config);
                    runtime.dispatch(crate::app::msg::Cmd::LoadConfig);
                    // Reset confirmation state after successful deletion
                    if let AppState::ManagingDirs { confirming_delete, .. } = &mut app.state {
                        *confirming_delete = false;
                    }
                }
            }
        }

        AppMsg::ToggleMainDirectoryEnabled(index) => {
            if let Some(ref mut config) = app.config {
                match config.toggle_main_directory(index) {
                    Ok(enabled) => {
                        if let Some(dir) = app.main_directories.get_mut(index) {
                            dir.enabled = enabled;
                        }
                        let _ = config::save_config(config);
                        runtime.dispatch(crate::app::msg::Cmd::LoadConfig);
                    }
                    Err(e) => {
                        app.error_message = Some(format!("Failed to toggle main directory: {}", e));
                    }
                }
            }
        }

        AppMsg::UpdateMainDirectoryName(index, name) => {
            if let Some(ref mut config) = app.config {
                if let Some(dir_config) = config.main_directories.get_mut(index) {
                    dir_config.display_name = Some(name.clone());
                    if let Some(dir) = app.main_directories.get_mut(index) {
                        dir.display_name = name;
                    }
                    let _ = config::save_config(config);
                }
            }
        }

        AppMsg::MainDirNavUp => {
            if let AppState::ManagingDirs {
                selected_dir_index,
                list_state,
                ..
            } = &mut app.state
            {
                if !app.main_directories.is_empty() {
                    *selected_dir_index = if *selected_dir_index == 0 {
                        app.main_directories.len() - 1
                    } else {
                        *selected_dir_index - 1
                    };
                    list_state.select(Some(*selected_dir_index));
                }
            }
        }

        AppMsg::MainDirNavDown => {
            if let AppState::ManagingDirs {
                selected_dir_index,
                list_state,
                ..
            } = &mut app.state
            {
                if !app.main_directories.is_empty() {
                    *selected_dir_index = (*selected_dir_index + 1) % app.main_directories.len();
                    list_state.select(Some(*selected_dir_index));
                }
            }
        }

        AppMsg::EditMainDirectory(index) => {
            if let AppState::ManagingDirs { editing, .. } = &mut app.state {
                if let Some(dir) = app.main_directories.get(index) {
                    *editing = Some(crate::app::state::MainDirEdit {
                        index: Some(index),
                        path: dir.path.clone(),
                        display_name: dir.display_name.clone(),
                        enabled: dir.enabled,
                    });
                }
            }
        }

        AppMsg::ConfirmEditMainDirectory => {
            if let AppState::ManagingDirs { editing, .. } = &mut app.state {
                if let Some(ref edit) = editing {
                    if let Some(ref mut config) = app.config {
                        if let Some(dir_config) =
                            config.main_directories.get_mut(edit.index.unwrap_or(0))
                        {
                            dir_config.display_name = Some(edit.display_name.clone());
                            if let Some(dir) = app.main_directories.get_mut(edit.index.unwrap_or(0))
                            {
                                dir.display_name = edit.display_name.clone();
                            }
                            let _ = config::save_config(config);
                        }
                    }
                }
                *editing = None;
            }
        }

        AppMsg::CancelEditMainDirectory => {
            if let AppState::ManagingDirs { editing, .. } = &mut app.state {
                *editing = None;
            }
        }

        AppMsg::ShowDeleteMainDirConfirmation => {
            if let AppState::ManagingDirs { confirming_delete, .. } = &mut app.state {
                *confirming_delete = true;
            }
        }

        AppMsg::CancelDeleteMainDirConfirmation => {
            if let AppState::ManagingDirs { confirming_delete, .. } = &mut app.state {
                *confirming_delete = false;
            }
        }

        // === Single Repository Management ===
        AppMsg::ShowAddSingleRepoChooser => {
            app.state = AppState::ChoosingDir {
                path: dirs::home_dir().unwrap_or_default(),
                entries: Vec::new(),
                selected_index: 0,
                scroll_offset: 0,
                mode: crate::app::state::DirectoryChooserMode::AddSingleRepository,
                return_to: crate::app::state::ReturnTarget::Running,
            };
            runtime.dispatch(crate::app::msg::Cmd::ScanDirectory(
                dirs::home_dir().unwrap_or_default(),
            ));
        }

        AppMsg::AddSingleRepository(path) => {
            if let Some(ref mut config) = app.config {
                if let Err(e) = config.add_single_repository(path.clone()) {
                    app.error_message = Some(format!("Failed to add repository: {}", e));
                } else {
                    app.single_repositories.push(path.clone());
                    let _ = config::save_config(config);
                    runtime.dispatch(crate::app::msg::Cmd::LoadConfig);
                }
            }
        }

        AppMsg::RemoveSingleRepository(path) => {
            if let Some(ref mut config) = app.config {
                if let Err(e) = config.remove_single_repository(&path) {
                    app.error_message = Some(format!("Failed to remove repository: {}", e));
                } else {
                    app.single_repositories.retain(|p| p != &path);
                    let _ = config::save_config(config);
                    runtime.dispatch(crate::app::msg::Cmd::LoadConfig);
                }
            }
        }

        // === Directory Chooser Enhanced ===
        AppMsg::ShowDirectoryChooserWithMode(mode) => {
            // Determine return target based on mode
            let return_to = match &mode {
                crate::app::state::DirectoryChooserMode::SelectMainDirectory { .. } => {
                    crate::app::state::ReturnTarget::ManagingDirs
                }
                _ => crate::app::state::ReturnTarget::Running,
            };

            app.state = AppState::ChoosingDir {
                path: dirs::home_dir().unwrap_or_default(),
                entries: Vec::new(),
                selected_index: 0,
                scroll_offset: 0,
                mode,
                return_to,
            };
            runtime.dispatch(crate::app::msg::Cmd::ScanDirectory(
                dirs::home_dir().unwrap_or_default(),
            ));
        }

        AppMsg::DirectoriesSelected(paths) => {
            for path in paths {
                if let Some(ref mut config) = app.config {
                    let path_buf = PathBuf::from(&path);
                    let _ = config.add_main_directory(path_buf);
                }
            }
            if let Some(ref config) = app.config {
                let _ = config::save_config(config);
            }
            runtime.dispatch(crate::app::msg::Cmd::LoadConfig);
            app.state = AppState::Running;
        }

        AppMsg::CancelDirectoryChooser => {
            // Get current return target
            let return_to = if let AppState::ChoosingDir { return_to, .. } = &app.state {
                return_to.clone()
            } else {
                crate::app::state::ReturnTarget::Running
            };

            // Return to appropriate state based on return_to
            match return_to {
                crate::app::state::ReturnTarget::ManagingDirs => {
                    // Re-initialize ManagingDirs state
                    let mut list_state = ratatui::widgets::ListState::default();
                    list_state.select(Some(0));
                    app.state = AppState::ManagingDirs {
                        list_state,
                        selected_dir_index: 0,
                        editing: None,
                        confirming_delete: false,
                    };
                }
                crate::app::state::ReturnTarget::Running => {
                    app.state = AppState::Running;
                }
            }
        }

        // === Clone Operations (Phase 1: basic structure) ===
        AppMsg::StartClone => {
            // Initialize clone state and transition to Cloning state
            let clone_state = crate::app::state::CloneState::new();
            app.state = AppState::Cloning { clone_state };
        }

        AppMsg::CloneUrlInput(c) => {
            if let Some(clone_state) = app.state.clone_state_mut() {
                clone_state.insert_char(c);
            }
        }

        AppMsg::CloneUrlPaste(text) => {
            if let Some(clone_state) = app.state.clone_state_mut() {
                clone_state.paste(&text);
            }
        }

        AppMsg::CloneUrlBackspace => {
            if let Some(clone_state) = app.state.clone_state_mut() {
                clone_state.backspace();
            }
        }

        AppMsg::CloneUrlClear => {
            if let Some(clone_state) = app.state.clone_state_mut() {
                clone_state.clear_from_cursor();
            }
        }

        AppMsg::CloneUrlConfirm => {
            // Validate URL and check target folder
            if let Some(clone_state) = app.state.clone_state_mut() {
                let url = clone_state.url_input.trim().to_string();

                // Validate URL
                if let Err(e) = crate::repo::clone::validate_git_url(&url, crate::constants::MAX_URL_LENGTH) {
                    clone_state.stage = crate::app::state::CloneStage::Error(e);
                    return;
                }

                // Parse URL
                let parsed = match crate::repo::clone::parse_git_url(&url) {
                    Ok(p) => p,
                    Err(e) => {
                        clone_state.stage = crate::app::state::CloneStage::Error(e);
                        return;
                    }
                };

                // Store parsed URL
                clone_state.parsed_url = Some(parsed.clone());

                // Get target main directory
                let target_idx = clone_state.selected_main_dir();
                let target_dir = app
                    .main_directories
                    .get(target_idx)
                    .map(|d| d.path.clone())
                    .or_else(|| app.main_dir.clone())
                    .unwrap_or_else(|| {
                        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
                    });

                // Generate folder name
                let folder_name = crate::repo::clone::generate_folder_name(&parsed);
                let target_path = target_dir.join(&folder_name);

                // Check if folder already exists
                if target_path.exists() {
                    // Check if it's a valid git repository that can be replaced
                    match crate::repo::clone::validate_folder_replace(&target_path, &[target_dir]) {
                        Ok(()) => {
                            // Can replace - show confirmation
                            clone_state.stage = crate::app::state::CloneStage::ConfirmReplace {
                                existing_path: target_path,
                            };
                        }
                        Err(e) => {
                            // Cannot replace (not a git repo or other issue)
                            clone_state.stage = crate::app::state::CloneStage::Error(e);
                        }
                    }
                } else {
                    // Folder doesn't exist - start clone
                    clone_state.stage = crate::app::state::CloneStage::Executing;
                    clone_state.target_main_dir = Some(target_idx);

                    // Dispatch clone command
                    runtime.dispatch(crate::app::msg::Cmd::CloneRepository {
                        url: url.clone(),
                        target_path: target_path.clone(),
                    });
                }
            }
        }

        AppMsg::CloneNextMainDir => {
            if let Some(clone_state) = app.state.clone_state_mut() {
                let max_dirs = app.main_directories.len();
                clone_state.next_main_dir(max_dirs);
            }
        }

        AppMsg::ClonePreviousMainDir => {
            if let Some(clone_state) = app.state.clone_state_mut() {
                clone_state.previous_main_dir();
            }
        }

        AppMsg::CloneSelectMainDir(index) => {
            if let Some(clone_state) = app.state.clone_state_mut() {
                clone_state.set_selected_main_dir(index);
            }
        }

        AppMsg::CloneConfirmReplace(confirmed) => {
            if confirmed {
                // User confirmed replacement - delete old folder and clone
                if let Some(clone_state) = app.state.clone_state_mut() {
                    if let crate::app::state::CloneStage::ConfirmReplace { existing_path } = clone_state.stage.clone() {
                        // Delete the existing folder
                        if let Err(e) = std::fs::remove_dir_all(&existing_path) {
                            clone_state.stage = crate::app::state::CloneStage::Error(
                                crate::error::CloneError::Io(e.to_string()),
                            );
                            return;
                        }

                        // Get URL and start clone
                        let url = clone_state.url_input.clone();
                        clone_state.stage = crate::app::state::CloneStage::Executing;

                        // Dispatch clone command
                        runtime.dispatch(crate::app::msg::Cmd::CloneRepository {
                            url,
                            target_path: existing_path,
                        });
                    }
                }
            } else {
                // User cancelled - go back to input stage
                if let Some(clone_state) = app.state.clone_state_mut() {
                    clone_state.stage = crate::app::state::CloneStage::InputUrl;
                }
            }
        }

        AppMsg::CloneProgress(line) => {
            if let Some(clone_state) = app.state.clone_state_mut() {
                clone_state.add_progress(line);
            }
        }

        AppMsg::CloneCompleted(result) => {
            match result {
                Ok(_repo) => {
                    // Clone successful - refresh repositories
                    if let Some(config) = &app.config {
                        let main_dirs: Vec<(PathBuf, Option<usize>)> = config
                            .main_directories
                            .iter()
                            .enumerate()
                            .filter(|(_, d)| d.enabled)
                            .map(|(idx, d)| (d.path.clone(), Some(idx)))
                            .collect();

                        let single_repos: Vec<PathBuf> = config
                            .single_repositories
                            .iter()
                            .map(|r| r.path.clone())
                            .collect();

                        runtime.dispatch(crate::app::msg::Cmd::LoadRepositoriesMulti {
                            main_dirs,
                            single_repos,
                        });
                    }

                    // Return to Running state
                    app.state = AppState::Running;
                }
                Err(e) => {
                    // Clone failed - show error
                    if let Some(clone_state) = app.state.clone_state_mut() {
                        clone_state.stage = crate::app::state::CloneStage::Error(e);
                    }
                }
            }
        }

        AppMsg::CloneRetry => {
            // Retry clone - go back to input stage
            if let Some(clone_state) = app.state.clone_state_mut() {
                clone_state.stage = crate::app::state::CloneStage::InputUrl;
                clone_state.progress_lines.clear();
            }
        }

        AppMsg::CancelClone => {
            // Cancel clone operation and return to Running state
            if let Some(clone_state) = app.state.clone_state() {
                clone_state.cancel();
            }
            app.state = AppState::Running;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::msg::AppMsg;
    use crate::app::state::AppState;
    use crate::repo::{RepoSource, Repository};

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
            mode: crate::app::state::DirectoryChooserMode::default(),
            return_to: crate::app::state::ReturnTarget::Running,
        };

        update(AppMsg::DirectoryNavDown, &mut app, &runtime);

        if let AppState::ChoosingDir { selected_index, .. } = app.state {
            assert_eq!(selected_index, 1);
        } else {
            panic!("State should be ChoosingDir");
        }
    }

    #[test]
    fn test_directory_nav_down_cyclic() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        app.state = AppState::ChoosingDir {
            path: std::path::PathBuf::from("/tmp"),
            entries: vec!["dir1".to_string(), "dir2".to_string(), "dir3".to_string()],
            selected_index: 2,
            scroll_offset: 0,
            mode: crate::app::state::DirectoryChooserMode::default(),
            return_to: crate::app::state::ReturnTarget::Running,
        };

        update(AppMsg::DirectoryNavDown, &mut app, &runtime);

        if let AppState::ChoosingDir { selected_index, .. } = app.state {
            assert_eq!(selected_index, 0);
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
            mode: crate::app::state::DirectoryChooserMode::default(),
            return_to: crate::app::state::ReturnTarget::Running,
        };

        update(AppMsg::DirectoryNavUp, &mut app, &runtime);

        if let AppState::ChoosingDir { selected_index, .. } = app.state {
            assert_eq!(selected_index, 1);
        } else {
            panic!("State should be ChoosingDir");
        }
    }

    #[test]
    fn test_directory_nav_up_cyclic() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        app.state = AppState::ChoosingDir {
            path: std::path::PathBuf::from("/tmp"),
            entries: vec!["dir1".to_string(), "dir2".to_string(), "dir3".to_string()],
            selected_index: 0,
            scroll_offset: 0,
            mode: crate::app::state::DirectoryChooserMode::default(),
            return_to: crate::app::state::ReturnTarget::Running,
        };

        update(AppMsg::DirectoryNavUp, &mut app, &runtime);

        if let AppState::ChoosingDir { selected_index, .. } = app.state {
            assert_eq!(selected_index, 2);
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
    fn test_update_next_repo_cyclic() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        app.filtered_indices = vec![0, 1, 2];
        app.set_selected_index(Some(2));

        update(AppMsg::NextRepo, &mut app, &runtime);
        assert_eq!(app.selected_index(), Some(0));
    }

    #[test]
    fn test_update_previous_repo_cyclic() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        app.filtered_indices = vec![0, 1, 2];
        app.set_selected_index(Some(0));

        update(AppMsg::PreviousRepo, &mut app, &runtime);
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
    fn test_update_previous_repo_from_middle() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        app.filtered_indices = vec![0, 1, 2, 3, 4];
        app.set_selected_index(Some(2));

        update(AppMsg::PreviousRepo, &mut app, &runtime);
        assert_eq!(app.selected_index(), Some(1));
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
            is_git_repo: true,
            source: RepoSource::Standalone,
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
            is_git_repo: true,
            source: RepoSource::Standalone,
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
        assert!(matches!(app.state, AppState::ShowingHelp { .. }));

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
                is_git_repo: true,
                source: RepoSource::Standalone,
            },
        };
        app.selected_repo = Some(Repository {
            name: "test".to_string(),
            path: std::path::PathBuf::from("/tmp/test"),
            last_modified: None,
            is_dirty: false,
            branch: None,
            is_git_repo: true,
            source: RepoSource::Standalone,
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

    #[test]
    fn test_update_open_theme_selector() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        app.state = AppState::Running;

        update(AppMsg::OpenThemeSelector, &mut app, &runtime);

        assert!(matches!(app.state, AppState::SelectingTheme { .. }));
        assert!(app.state.is_modal());
    }

    #[test]
    fn test_update_theme_nav() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        app.state = AppState::SelectingTheme {
            theme_list_state: ratatui::widgets::ListState::default(),
            preview_theme: Theme::dark(),
        };

        update(AppMsg::ThemeNavDown, &mut app, &runtime);
        if let AppState::SelectingTheme {
            theme_list_state, ..
        } = &app.state
        {
            assert_eq!(theme_list_state.selected(), Some(1));
        }

        update(AppMsg::ThemeNavUp, &mut app, &runtime);
        if let AppState::SelectingTheme {
            theme_list_state, ..
        } = &app.state
        {
            assert_eq!(theme_list_state.selected(), Some(0));
        }
    }

    #[test]
    fn test_update_theme_nav_cyclic() {
        use crate::ui::themes::THEME_NAMES;

        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        let theme_count = THEME_NAMES.len();
        app.state = AppState::SelectingTheme {
            theme_list_state: ratatui::widgets::ListState::default(),
            preview_theme: Theme::dark(),
        };

        if let AppState::SelectingTheme {
            theme_list_state, ..
        } = &mut app.state
        {
            theme_list_state.select(Some(theme_count - 1));
        }

        update(AppMsg::ThemeNavDown, &mut app, &runtime);

        if let AppState::SelectingTheme {
            theme_list_state, ..
        } = &app.state
        {
            assert_eq!(theme_list_state.selected(), Some(0));
        }

        if let AppState::SelectingTheme {
            theme_list_state, ..
        } = &mut app.state
        {
            theme_list_state.select(Some(0));
        }

        update(AppMsg::ThemeNavUp, &mut app, &runtime);

        if let AppState::SelectingTheme {
            theme_list_state, ..
        } = &app.state
        {
            assert_eq!(theme_list_state.selected(), Some(theme_count - 1));
        }
    }

    #[test]
    fn test_update_select_theme() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        app.state = AppState::SelectingTheme {
            theme_list_state: ratatui::widgets::ListState::default(),
            preview_theme: Theme::dark(),
        };
        app.config = Some(crate::config::Config::default());

        update(AppMsg::SelectTheme("nord".to_string()), &mut app, &runtime);

        assert!(matches!(app.state, AppState::Running));
        assert_eq!(app.theme.name, "nord");
    }

    #[test]
    fn test_update_toggle_favorite() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        app.repositories = vec![Repository {
            name: "repo1".to_string(),
            path: std::path::PathBuf::from("/tmp/repo1"),
            last_modified: None,
            is_dirty: false,
            branch: Some("main".to_string()),
            is_git_repo: true,
            source: RepoSource::Standalone,
        }];
        app.filtered_indices = vec![0];
        app.set_selected_index(Some(0));
        app.config = Some(crate::config::Config::default());

        // Initially not favorited
        assert!(!app.favorites.contains(&app.repositories[0].path));

        // Toggle favorite
        update(AppMsg::ToggleFavorite, &mut app, &runtime);
        assert!(app.favorites.contains(&app.repositories[0].path));

        // Toggle again to remove
        update(AppMsg::ToggleFavorite, &mut app, &runtime);
        assert!(!app.favorites.contains(&app.repositories[0].path));
    }

    #[test]
    fn test_update_show_favorites() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        app.repositories = vec![
            Repository {
                name: "repo1".to_string(),
                path: std::path::PathBuf::from("/tmp/repo1"),
                last_modified: None,
                is_dirty: false,
                branch: Some("main".to_string()),
                is_git_repo: true,
                source: RepoSource::Standalone,
            },
            Repository {
                name: "repo2".to_string(),
                path: std::path::PathBuf::from("/tmp/repo2"),
                last_modified: None,
                is_dirty: false,
                branch: Some("main".to_string()),
                is_git_repo: true,
                source: RepoSource::Standalone,
            },
        ];
        app.filtered_indices = vec![0, 1];
        app.set_selected_index(Some(0));

        // Add one to favorites
        app.favorites.add(&app.repositories[0].path);

        // Initially All mode
        assert_eq!(app.view_mode, ViewMode::All);

        // Switch to Favorites
        update(AppMsg::ShowFavorites, &mut app, &runtime);
        assert_eq!(app.view_mode, ViewMode::Favorites);
        assert_eq!(app.filtered_indices.len(), 1);

        // Switch back to All
        update(AppMsg::ShowAllRepos, &mut app, &runtime);
        assert_eq!(app.view_mode, ViewMode::All);
        assert_eq!(app.filtered_indices.len(), 2);
    }

    #[test]
    fn test_update_show_favorites_no_favorites() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        app.repositories = vec![Repository {
            name: "repo1".to_string(),
            path: std::path::PathBuf::from("/tmp/repo1"),
            last_modified: None,
            is_dirty: false,
            branch: Some("main".to_string()),
            is_git_repo: true,
            source: RepoSource::Standalone,
        }];
        app.filtered_indices = vec![0];
        app.set_selected_index(Some(0));

        // No favorites
        update(AppMsg::ShowFavorites, &mut app, &runtime);
        assert_eq!(app.view_mode, ViewMode::Favorites);
        assert_eq!(app.filtered_indices.len(), 0);
    }

    #[tokio::test]
    async fn test_directory_selected_creates_config_when_none() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        // Ensure initial config is None
        assert!(app.config.is_none(), "Initial config should be None");

        // Setup ChoosingDir state for first launch
        app.state = AppState::ChoosingDir {
            path: std::path::PathBuf::from("/tmp"),
            entries: vec![],
            selected_index: 0,
            scroll_offset: 0,
            mode: crate::app::state::DirectoryChooserMode::SelectMainDirectory {
                allow_multiple: false,
                edit_mode: false,
            },
            return_to: crate::app::state::ReturnTarget::Running,
        };

        // Send DirectorySelected message
        let test_path = "/tmp/test_repo";
        update(
            AppMsg::DirectorySelected(test_path.to_string()),
            &mut app,
            &runtime,
        );

        // Verify config was created
        assert!(app.config.is_some(), "Config should be created when None");
        assert_eq!(
            app.config.as_ref().unwrap().version,
            crate::constants::CONFIG_VERSION
        );
    }

    #[tokio::test]
    async fn test_directory_selected_preserves_existing_config() {
        use crate::config::Config;

        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        // Set up existing config with custom values
        let existing_config = Config {
            version: "2.0".to_string(),
            ui: crate::config::UiConfig {
                theme: "nord".to_string(),
                show_git_status: false,
                show_branch: false,
            },
            ..Config::default()
        };
        app.config = Some(existing_config.clone());

        // Setup ChoosingDir state
        app.state = AppState::ChoosingDir {
            path: std::path::PathBuf::from("/tmp"),
            entries: vec![],
            selected_index: 0,
            scroll_offset: 0,
            mode: crate::app::state::DirectoryChooserMode::SelectMainDirectory {
                allow_multiple: false,
                edit_mode: false,
            },
            return_to: crate::app::state::ReturnTarget::Running,
        };

        // Send DirectorySelected message
        let test_path = "/tmp/test_repo";
        update(
            AppMsg::DirectorySelected(test_path.to_string()),
            &mut app,
            &runtime,
        );

        // Verify existing config was preserved
        assert!(app.config.is_some());
        assert_eq!(app.config.as_ref().unwrap().version, "2.0");
        assert_eq!(app.config.as_ref().unwrap().ui.theme, "nord");
        assert!(!app.config.as_ref().unwrap().ui.show_git_status);
    }
}
