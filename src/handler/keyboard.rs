//! Keyboard event handler

use crate::app::model::App;
use crate::app::msg::AppMsg;
use crate::app::state::{AppState, ViewMode};
use crate::runtime::executor::Runtime;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle keyboard event
pub fn handle_key_event(key: KeyEvent, app: &mut App, runtime: &Runtime) {
    // Handle Ctrl+C globally
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        let _ = app.msg_tx.try_send(AppMsg::Quit);
        return;
    }

    // State-based handling (priority order)
    match &app.state {
        AppState::Cloning { .. } => {
            handle_cloning_keys(key, app);
        }
        AppState::ShowingActions { .. } => {
            handle_action_menu_keys(key, app, runtime);
        }
        AppState::ShowingHelp { .. } => {
            handle_help_keys(key, app);
        }
        AppState::ChoosingDir { mode, .. } => {
            let mode = mode.clone();
            handle_chooser_keys(key, app, runtime, mode);
        }
        AppState::ManagingDirs { editing, .. } => {
            if editing.is_some() {
                handle_main_dir_edit_keys(key, app);
            } else {
                handle_main_dir_manager_keys(key, app, runtime);
            }
        }
        AppState::SelectingTheme { .. } => {
            handle_theme_selector_keys(key, app);
        }
        AppState::Running | AppState::Loading { .. } | AppState::Error { .. } => {
            if app.search_active {
                handle_search_input(key, app, runtime);
            } else {
                handle_running_keys(key, app, runtime);
            }
        }
        AppState::Quit => {
            // Already quitting
        }
    }
}

/// Handle keys in action menu
fn handle_action_menu_keys(key: KeyEvent, app: &mut App, _runtime: &Runtime) {
    match key.code {
        KeyCode::Esc => {
            let _ = app.msg_tx.try_send(AppMsg::CloseActions);
        }
        KeyCode::Char('1') => {
            let action = crate::action::Action::CdAndCloud;
            if app.selection_mode && app.selected_count() > 0 {
                let _ = app.msg_tx.try_send(AppMsg::ExecuteBatchAction(action));
            } else {
                let _ = app.msg_tx.try_send(AppMsg::ExecuteAction(action));
            }
        }
        KeyCode::Char('2') => {
            let action = crate::action::Action::OpenWebStorm;
            if app.selection_mode && app.selected_count() > 0 {
                let _ = app.msg_tx.try_send(AppMsg::ExecuteBatchAction(action));
            } else {
                let _ = app.msg_tx.try_send(AppMsg::ExecuteAction(action));
            }
        }
        KeyCode::Char('3') => {
            let action = crate::action::Action::OpenVsCode;
            if app.selection_mode && app.selected_count() > 0 {
                let _ = app.msg_tx.try_send(AppMsg::ExecuteBatchAction(action));
            } else {
                let _ = app.msg_tx.try_send(AppMsg::ExecuteAction(action));
            }
        }
        KeyCode::Char('4') => {
            let action = crate::action::Action::OpenFileManager;
            if app.selection_mode && app.selected_count() > 0 {
                let _ = app.msg_tx.try_send(AppMsg::ExecuteBatchAction(action));
            } else {
                let _ = app.msg_tx.try_send(AppMsg::ExecuteAction(action));
            }
        }
        KeyCode::Char('5') => {
            let action = crate::action::Action::OpenIntelliJ;
            if app.selection_mode && app.selected_count() > 0 {
                let _ = app.msg_tx.try_send(AppMsg::ExecuteBatchAction(action));
            } else {
                let _ = app.msg_tx.try_send(AppMsg::ExecuteAction(action));
            }
        }
        KeyCode::Char('6') => {
            let action = crate::action::Action::OpenOpenCode;
            if app.selection_mode && app.selected_count() > 0 {
                let _ = app.msg_tx.try_send(AppMsg::ExecuteBatchAction(action));
            } else {
                let _ = app.msg_tx.try_send(AppMsg::ExecuteAction(action));
            }
        }
        KeyCode::Down => {
            // Navigate down in menu
            let _ = app.msg_tx.try_send(AppMsg::ActionMenuNavDown);
        }
        KeyCode::Up => {
            // Navigate up in menu
            let _ = app.msg_tx.try_send(AppMsg::ActionMenuNavUp);
        }
        KeyCode::Enter => {
            // Execute selected action
            if let Some(action) = get_selected_action(app) {
                if app.selection_mode && app.selected_count() > 0 {
                    let _ = app.msg_tx.try_send(AppMsg::ExecuteBatchAction(action));
                } else {
                    let _ = app.msg_tx.try_send(AppMsg::ExecuteAction(action));
                }
            }
        }
        _ => {}
    }
}

/// Get currently selected action from app state
fn get_selected_action(_app: &App) -> Option<crate::action::Action> {
    // This would need to be implemented in app/model.rs
    // For now, default to CdAndCloud
    Some(crate::action::Action::CdAndCloud)
}

/// Maximum scroll offset for help panel (total lines - visible lines)
const HELP_MAX_SCROLL: usize = 32;

/// Handle keys in help panel
fn handle_help_keys(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Up => {
            if let AppState::ShowingHelp { scroll_offset, .. } = &mut app.state {
                if *scroll_offset > 0 {
                    *scroll_offset -= 1;
                } else {
                    *scroll_offset = HELP_MAX_SCROLL;
                }
            }
        }
        KeyCode::Down => {
            if let AppState::ShowingHelp { scroll_offset, .. } = &mut app.state {
                if *scroll_offset < HELP_MAX_SCROLL {
                    *scroll_offset += 1;
                } else {
                    *scroll_offset = 0;
                }
            }
        }
        KeyCode::Esc => {
            let _ = app.msg_tx.try_send(AppMsg::CloseHelp);
        }
        _ => {}
    }
}

/// Handle keys in theme selector
fn handle_theme_selector_keys(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Esc => {
            let _ = app.msg_tx.try_send(AppMsg::CloseThemeSelector);
        }
        KeyCode::Char('j') | KeyCode::Down => {
            let _ = app.msg_tx.try_send(AppMsg::ThemeNavDown);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let _ = app.msg_tx.try_send(AppMsg::ThemeNavUp);
        }
        KeyCode::Enter => {
            // Get selected theme name and send it
            let theme_name = get_selected_theme_name(app);
            if let Some(name) = theme_name {
                let _ = app.msg_tx.try_send(AppMsg::SelectTheme(name));
            }
        }
        _ => {}
    }
}

/// Get currently selected theme name from app state
fn get_selected_theme_name(app: &App) -> Option<String> {
    use crate::ui::themes::THEME_NAMES;

    if let AppState::SelectingTheme {
        theme_list_state, ..
    } = &app.state
    {
        if let Some(selected) = theme_list_state.selected() {
            if selected < THEME_NAMES.len() {
                let name = THEME_NAMES[selected];
                // Return "random" for the random option
                if name.contains("Random") {
                    return Some("random".to_string());
                }
                return Some(name.to_string());
            }
        }
    }
    None
}

/// Handle keys in directory chooser
fn handle_chooser_keys(
    key: KeyEvent,
    app: &mut App,
    runtime: &Runtime,
    mode: crate::app::state::DirectoryChooserMode,
) {
    match key.code {
        KeyCode::Esc => {
            // Cancel directory chooser and return to previous state
            let _ = app.msg_tx.try_send(AppMsg::CancelDirectoryChooser);
        }
        KeyCode::Left => {
            // Go to parent directory
            handle_directory_back(app, runtime, mode);
        }
        KeyCode::Right => {
            // Enter selected directory
            handle_directory_enter(app, runtime, mode);
        }
        KeyCode::Char(' ') => {
            // Space to select current directory and confirm
            if let AppState::ChoosingDir {
                path,
                entries,
                selected_index,
                ..
            } = &app.state
            {
                // If there are entries and a valid selection, use the selected subdirectory
                if !entries.is_empty() && *selected_index < entries.len() {
                    let selected_name = &entries[*selected_index];
                    let selected_path = path.join(selected_name);
                    let path_str = selected_path.to_string_lossy().to_string();
                    let _ = app.msg_tx.try_send(AppMsg::DirectorySelected(path_str));
                } else {
                    // No entries or invalid selection, use current path
                    let path_str = path.to_string_lossy().to_string();
                    let _ = app.msg_tx.try_send(AppMsg::DirectorySelected(path_str));
                }
            }
        }
        KeyCode::Down => {
            let _ = app.msg_tx.try_send(AppMsg::DirectoryNavDown);
        }
        KeyCode::Up => {
            let _ = app.msg_tx.try_send(AppMsg::DirectoryNavUp);
        }
        KeyCode::Home => {
            // Jump to first entry
            if let AppState::ChoosingDir { selected_index, .. } = &mut app.state {
                *selected_index = 0;
            }
        }
        KeyCode::End => {
            // Jump to last entry
            if let AppState::ChoosingDir {
                entries,
                selected_index,
                ..
            } = &mut app.state
            {
                if !entries.is_empty() {
                    *selected_index = entries.len() - 1;
                }
            }
        }
        _ => {}
    }
}

/// Handle going back to parent directory
fn handle_directory_back(
    app: &mut App,
    runtime: &Runtime,
    mode: crate::app::state::DirectoryChooserMode,
) {
    if let AppState::ChoosingDir {
        path, return_to, ..
    } = &app.state
    {
        if let Some(parent) = path.parent() {
            let parent_path = parent.to_path_buf();

            // Scan parent directory
            runtime.dispatch(crate::app::msg::Cmd::ScanDirectory(parent_path.clone()));

            // Update state
            app.state = AppState::ChoosingDir {
                path: parent_path,
                entries: Vec::new(),
                selected_index: 0,
                scroll_offset: 0,
                mode,
                return_to: return_to.clone(),
            };
        }
    }
}

/// Handle entering a directory
fn handle_directory_enter(
    app: &mut App,
    runtime: &Runtime,
    mode: crate::app::state::DirectoryChooserMode,
) {
    if let AppState::ChoosingDir {
        path,
        entries,
        selected_index,
        return_to,
        ..
    } = &app.state
    {
        if !entries.is_empty() && *selected_index < entries.len() {
            let selected_name = &entries[*selected_index];
            let selected_path = path.join(selected_name);

            // Navigate into the selected directory
            runtime.dispatch(crate::app::msg::Cmd::ScanDirectory(selected_path));

            // Update state to new path
            app.state = AppState::ChoosingDir {
                path: path.join(selected_name),
                entries: Vec::new(), // Will be populated by async scan
                selected_index: 0,
                scroll_offset: 0,
                mode,
                return_to: return_to.clone(),
            };
        }
    }
}

/// Handle search input (when search box is focused)
///
/// Core principle: When search is focused, only arrow keys navigate,
/// all letter keys are for input only (no shortcut functions).
fn handle_search_input(key: KeyEvent, app: &mut App, _runtime: &Runtime) {
    match key.code {
        // === Exit/Confirm ===
        KeyCode::Esc => {
            // Only exit focus, keep query
            app.search_active = false;
        }
        KeyCode::Enter => {
            // Confirm search, exit focus
            app.search_active = false;
        }

        // === Edit keys ===
        KeyCode::Backspace => {
            let _ = app.msg_tx.try_send(AppMsg::SearchBackspace);
        }

        // === Navigation keys (arrow keys, don't affect search box content) ===
        KeyCode::Up => {
            let _ = app.msg_tx.try_send(AppMsg::PreviousRepo);
        }
        KeyCode::Down => {
            let _ = app.msg_tx.try_send(AppMsg::NextRepo);
        }
        KeyCode::Home => {
            let _ = app.msg_tx.try_send(AppMsg::JumpToTop);
        }
        KeyCode::End => {
            let _ = app.msg_tx.try_send(AppMsg::JumpToBottom);
        }

        // === All other characters: input only ===
        // Including g/G/m/r/? etc. - all function keys are blocked
        KeyCode::Char(c) => {
            let _ = app.msg_tx.try_send(AppMsg::SearchInput(c));
        }

        _ => {}
    }
}

/// Handle keys in running state
fn handle_running_keys(key: KeyEvent, app: &mut App, _runtime: &Runtime) {
    match key.code {
        // Navigation
        KeyCode::Down => {
            let _ = app.msg_tx.try_send(AppMsg::NextRepo);
        }
        KeyCode::Up => {
            let _ = app.msg_tx.try_send(AppMsg::PreviousRepo);
        }
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            let _ = app.msg_tx.try_send(AppMsg::ScrollDown);
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            let _ = app.msg_tx.try_send(AppMsg::ScrollUp);
        }
        KeyCode::Home => {
            let _ = app.msg_tx.try_send(AppMsg::JumpToTop);
        }
        KeyCode::End => {
            let _ = app.msg_tx.try_send(AppMsg::JumpToBottom);
        }

        // Search - focus search box (only via '/' key)
        KeyCode::Char('/') => {
            app.search_active = true;
        }

        // Toggle theme selector (t key)
        KeyCode::Char('t') => {
            let _ = app.msg_tx.try_send(AppMsg::OpenThemeSelector);
        }

        // Change main directory - open main directory manager
        KeyCode::Char('m') => {
            let _ = app.msg_tx.try_send(AppMsg::ShowMainDirectoryManager);
        }

        // Actions
        KeyCode::Enter => {
            let _ = app.msg_tx.try_send(AppMsg::OpenActions);
        }
        KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Ctrl+f: Toggle favorites view
            if app.view_mode == ViewMode::Favorites {
                app.view_mode = ViewMode::All;
                app.filter_by_view_mode();
            } else {
                app.view_mode = ViewMode::Favorites;
                app.filter_by_view_mode();
            }
        }

        KeyCode::Char('f') => {
            // f: Toggle favorite for current repo
            let _ = app.msg_tx.try_send(AppMsg::ToggleFavorite);
        }

        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Ctrl+A: Select all in selection mode
            if app.selection_mode {
                let _ = app.msg_tx.try_send(AppMsg::SelectAll);
            }
        }

        KeyCode::Char('a') => {
            // Add single repository
            let _ = app.msg_tx.try_send(AppMsg::ShowAddSingleRepoChooser);
        }

        KeyCode::Char(' ') => {
            // Space: Toggle selection for current repo (in selection mode)
            if app.selection_mode {
                let _ = app.msg_tx.try_send(AppMsg::ToggleSelection);
            }
        }

        // Refresh repository list (r key)
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Ctrl+r: Toggle recent view
            if app.view_mode == ViewMode::Recent {
                app.view_mode = ViewMode::All;
                app.filter_by_view_mode();
            } else {
                app.view_mode = ViewMode::Recent;
                app.filter_by_view_mode();
            }
        }
        KeyCode::Char('r') => {
            let _ = app.msg_tx.try_send(AppMsg::Refresh);
        }
        KeyCode::Char('?') => {
            let _ = app.msg_tx.try_send(AppMsg::ShowHelp);
        }

        KeyCode::Char('v') => {
            // v: Toggle selection mode
            let _ = app.msg_tx.try_send(AppMsg::ToggleSelectionMode);
        }

        KeyCode::Char('c') => {
            // c: Start clone operation
            let _ = app.msg_tx.try_send(AppMsg::StartClone);
        }

        KeyCode::Char('U') => {
            // U: Trigger manual update check
            let _ = app.msg_tx.try_send(AppMsg::TriggerUpdateCheck);
        }

        _ => {}
    }
}

// Main directory management functions
fn handle_main_dir_manager_keys(key: KeyEvent, app: &mut App, _runtime: &Runtime) {
    // Check if we're in delete confirmation mode
    let is_confirming = if let AppState::ManagingDirs { confirming_delete, .. } = &app.state {
        *confirming_delete
    } else {
        false
    };

    // If confirming delete, handle confirmation keys
    if is_confirming {
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter => {
                // Confirm deletion
                if let AppState::ManagingDirs { selected_dir_index, .. } = &app.state {
                    let _ = app
                        .msg_tx
                        .try_send(AppMsg::RemoveMainDirectory(*selected_dir_index));
                }
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                // Cancel deletion
                let _ = app.msg_tx.try_send(AppMsg::CancelDeleteMainDirConfirmation);
            }
            _ => {}
        }
        return;
    }

    // Normal mode - handle regular keys
    match key.code {
        KeyCode::Esc => {
            let _ = app.msg_tx.try_send(AppMsg::CloseMainDirectoryManager);
        }
        KeyCode::Char('a') => {
            // Add new main directory - open chooser via message system
            let _ = app.msg_tx.try_send(AppMsg::ShowDirectoryChooserWithMode(
                crate::app::state::DirectoryChooserMode::SelectMainDirectory {
                    allow_multiple: false,
                    edit_mode: false,
                },
            ));
        }
        KeyCode::Char('d') => {
            // Show delete confirmation
            let _ = app.msg_tx.try_send(AppMsg::ShowDeleteMainDirConfirmation);
        }
        KeyCode::Char('e') => {
            // Edit selected main directory
            if let AppState::ManagingDirs {
                selected_dir_index, ..
            } = &app.state
            {
                let _ = app
                    .msg_tx
                    .try_send(AppMsg::EditMainDirectory(*selected_dir_index));
            }
        }
        KeyCode::Char(' ') => {
            // Toggle enabled state
            if let AppState::ManagingDirs {
                selected_dir_index, ..
            } = &app.state
            {
                let _ = app
                    .msg_tx
                    .try_send(AppMsg::ToggleMainDirectoryEnabled(*selected_dir_index));
            }
        }
        KeyCode::Up => {
            let _ = app.msg_tx.try_send(AppMsg::MainDirNavUp);
        }
        KeyCode::Down => {
            let _ = app.msg_tx.try_send(AppMsg::MainDirNavDown);
        }
        _ => {}
    }
}

fn handle_main_dir_edit_keys(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Esc => {
            let _ = app.msg_tx.try_send(AppMsg::CancelEditMainDirectory);
        }
        KeyCode::Enter => {
            let _ = app.msg_tx.try_send(AppMsg::ConfirmEditMainDirectory);
        }
        KeyCode::Char(c) => {
            // Add character to editing name
            if let AppState::ManagingDirs {
                editing: Some(ref mut edit),
                ..
            } = &mut app.state
            {
                edit.display_name.push(c);
            }
        }
        KeyCode::Backspace => {
            // Remove last character from editing name
            if let AppState::ManagingDirs {
                editing: Some(ref mut edit),
                ..
            } = &mut app.state
            {
                edit.display_name.pop();
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::DirectoryChooserMode;
    use crate::ui::Theme;
    use crossterm::event::KeyEventKind;
    use std::path::PathBuf;

    fn create_test_key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }
    }

    #[test]
    fn test_handle_running_keys_navigation() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx);
        let runtime = Runtime::new(app.msg_tx.clone());

        handle_running_keys(create_test_key(KeyCode::Char('j')), &mut app, &runtime);
        // Should dispatch NextRepo
    }

    #[test]
    fn test_handle_running_keys_search() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        // Test '/' key enters search mode
        assert!(!app.search_active);
        handle_running_keys(create_test_key(KeyCode::Char('/')), &mut app, &runtime);
        assert!(app.search_active);
    }

    #[test]
    fn test_handle_running_keys_direct_action() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        // Add a mock repository
        use crate::repo::{RepoSource, Repository};
        use std::path::PathBuf;
        app.repositories = vec![Repository {
            name: "test-repo".to_string(),
            path: PathBuf::from("/tmp/test-repo"),
            last_modified: None,
            is_dirty: false,
            branch: Some("main".to_string()),
            is_git_repo: true,
            source: RepoSource::Standalone,
        }];
        app.filtered_indices = vec![0];
        app.set_selected_index(Some(0));

        // Test direct action shortcuts
        handle_running_keys(create_test_key(KeyCode::Char('c')), &mut app, &runtime);
        handle_running_keys(create_test_key(KeyCode::Char('w')), &mut app, &runtime);
        handle_running_keys(create_test_key(KeyCode::Char('v')), &mut app, &runtime);
        handle_running_keys(create_test_key(KeyCode::Char('f')), &mut app, &runtime);
    }

    #[test]
    fn test_handle_action_menu_keys() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());

        // Test close action menu
        handle_action_menu_keys(
            create_test_key(KeyCode::Esc),
            &mut app,
            &Runtime::new(tx.clone()),
        );

        // Test execute actions
        handle_action_menu_keys(
            create_test_key(KeyCode::Char('c')),
            &mut app,
            &Runtime::new(tx.clone()),
        );
        handle_action_menu_keys(
            create_test_key(KeyCode::Char('w')),
            &mut app,
            &Runtime::new(tx.clone()),
        );
        handle_action_menu_keys(
            create_test_key(KeyCode::Char('1')),
            &mut app,
            &Runtime::new(tx.clone()),
        );
    }

    #[test]
    fn test_handle_search_input() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        app.search_active = true;
        app.search_query = "test".to_string();

        // Test esc - exits search focus but keeps query
        handle_search_input(
            create_test_key(KeyCode::Esc),
            &mut app,
            &Runtime::new(tx.clone()),
        );
        assert_eq!(app.search_query, "test"); // Query is preserved
        assert!(!app.search_active);

        // Test backspace - sends message (doesn't directly modify)
        app.search_active = true;
        app.search_query = "hello".to_string();
        handle_search_input(
            create_test_key(KeyCode::Backspace),
            &mut app,
            &Runtime::new(tx.clone()),
        );
        // Backspace sends a message, doesn't directly modify
    }

    #[test]
    fn test_handle_chooser_keys() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        // Setup directory chooser state
        app.state = AppState::ChoosingDir {
            path: PathBuf::from("/home/user"),
            entries: vec!["Documents".to_string(), "Projects".to_string()],
            selected_index: 0,
            scroll_offset: 0,
            mode: DirectoryChooserMode::default(),
            return_to: crate::app::state::ReturnTarget::Running,
        };

        // Test navigation
        handle_chooser_keys(
            create_test_key(KeyCode::Down),
            &mut app,
            &runtime,
            DirectoryChooserMode::default(),
        );
        handle_chooser_keys(
            create_test_key(KeyCode::Up),
            &mut app,
            &runtime,
            DirectoryChooserMode::default(),
        );

        // Test home/end
        handle_chooser_keys(
            create_test_key(KeyCode::Home),
            &mut app,
            &runtime,
            DirectoryChooserMode::default(),
        );
        handle_chooser_keys(
            create_test_key(KeyCode::End),
            &mut app,
            &runtime,
            DirectoryChooserMode::default(),
        );

        // Test quit
        handle_chooser_keys(
            create_test_key(KeyCode::Esc),
            &mut app,
            &runtime,
            DirectoryChooserMode::default(),
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_handle_chooser_navigation_keys() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        app.state = AppState::ChoosingDir {
            path: PathBuf::from("/home/user"),
            entries: vec!["dir1".to_string(), "dir2".to_string(), "dir3".to_string()],
            selected_index: 0,
            scroll_offset: 0,
            mode: DirectoryChooserMode::default(),
            return_to: crate::app::state::ReturnTarget::Running,
        };

        // Test ←/→ navigation
        let mode = crate::app::state::DirectoryChooserMode::SelectMainDirectory {
            allow_multiple: false,
            edit_mode: false,
        };
        handle_chooser_keys(
            create_test_key(KeyCode::Left),
            &mut app,
            &runtime,
            mode.clone(),
        );
        handle_chooser_keys(
            create_test_key(KeyCode::Right),
            &mut app,
            &runtime,
            mode.clone(),
        );
        handle_chooser_keys(
            create_test_key(KeyCode::Right),
            &mut app,
            &runtime,
            mode.clone(),
        );
    }

    #[test]
    fn test_handle_theme_selector_keys() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());

        // Setup theme selector state
        app.state = AppState::SelectingTheme {
            theme_list_state: ratatui::widgets::ListState::default(),
            preview_theme: Theme::dark(),
        };

        // Test close
        handle_theme_selector_keys(create_test_key(KeyCode::Esc), &mut app);

        // Test navigation
        handle_theme_selector_keys(create_test_key(KeyCode::Char('j')), &mut app);
        handle_theme_selector_keys(create_test_key(KeyCode::Char('k')), &mut app);
        handle_theme_selector_keys(create_test_key(KeyCode::Down), &mut app);
        handle_theme_selector_keys(create_test_key(KeyCode::Up), &mut app);

        // Test enter (select)
        handle_theme_selector_keys(create_test_key(KeyCode::Enter), &mut app);
    }

    #[test]
    fn test_handle_running_keys_opens_theme_selector() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        app.state = AppState::Running;

        // Test 't' key opens theme selector
        handle_running_keys(create_test_key(KeyCode::Char('t')), &mut app, &runtime);
    }

    #[test]
    fn test_f_key_toggles_favorite() {
        use crate::repo::Repository;
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        app.repositories = vec![Repository::test_repo()];
        app.filtered_indices = vec![0];
        app.set_selected_index(Some(0));

        // Test 'f' key sends ToggleFavorite message
        handle_running_keys(create_test_key(KeyCode::Char('f')), &mut app, &runtime);

        // Check that a message was sent (we can't test the actual toggle here as it's async)
        // The actual toggle happens in update.rs when processing ToggleFavorite
        assert!(rx.try_recv().is_ok()); // Should have received a message
    }

    #[test]
    fn test_v_key_toggles_selection_mode() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        assert!(!app.selection_mode);

        // Test 'v' key toggles selection mode
        handle_running_keys(create_test_key(KeyCode::Char('v')), &mut app, &runtime);

        // Check that ToggleSelectionMode message was sent
        assert!(rx.try_recv().is_ok());
    }

    #[test]
    fn test_f_key_sends_toggle_favorite() {
        use crate::repo::Repository;
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        app.repositories = vec![Repository::test_repo()];
        app.filtered_indices = vec![0];
        app.set_selected_index(Some(0));

        // Test 'f' key sends ToggleFavorite
        handle_running_keys(create_test_key(KeyCode::Char('f')), &mut app, &runtime);

        // Verify message was sent (actual toggle is async)
        assert!(rx.try_recv().is_ok());
    }

    #[test]
    fn test_ctrl_f_toggles_favorites_view() {
        use crate::app::state::ViewMode;
        use crate::repo::Repository;
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        app.repositories = vec![Repository::test_repo()];
        app.filtered_indices = vec![0];
        app.set_selected_index(Some(0));
        app.view_mode = ViewMode::All;

        // Test Ctrl+f switches to Favorites view
        let ctrl_f = KeyEvent {
            code: KeyCode::Char('f'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };
        handle_running_keys(ctrl_f, &mut app, &runtime);
        assert_eq!(app.view_mode, ViewMode::Favorites);

        // Test Ctrl+f again switches back to All
        handle_running_keys(ctrl_f, &mut app, &runtime);
        assert_eq!(app.view_mode, ViewMode::All);
    }

    #[test]
    fn test_ctrl_r_toggles_recent_view() {
        use crate::app::state::ViewMode;
        use crate::repo::Repository;
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        app.repositories = vec![Repository::test_repo()];
        app.filtered_indices = vec![0];
        app.set_selected_index(Some(0));
        app.view_mode = ViewMode::All;

        // Test Ctrl+r switches to Recent view
        let ctrl_r = KeyEvent {
            code: KeyCode::Char('r'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };
        handle_running_keys(ctrl_r, &mut app, &runtime);
        assert_eq!(app.view_mode, ViewMode::Recent);

        // Test Ctrl+r again switches back to All
        handle_running_keys(ctrl_r, &mut app, &runtime);
        assert_eq!(app.view_mode, ViewMode::All);
    }

    #[test]
    fn test_c_key_starts_clone() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx.clone());
        let runtime = Runtime::new(tx);

        // Test 'c' key sends StartClone message
        handle_running_keys(
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: crossterm::event::KeyEventState::NONE,
            },
            &mut app,
            &runtime,
        );
    }
}

/// Handle keys in cloning state
fn handle_cloning_keys(key: KeyEvent, app: &mut App) {
    use crate::app::state::CloneStage;

    // Get clone state info
    let stage = app.state.clone_state().map(|s| s.stage.clone());

    match stage {
        Some(CloneStage::InputUrl) => {
            match key.code {
                KeyCode::Esc => {
                    let _ = app.msg_tx.try_send(AppMsg::CancelClone);
                }
                KeyCode::Enter => {
                    let _ = app.msg_tx.try_send(AppMsg::CloneUrlConfirm);
                }
                KeyCode::Backspace => {
                    let _ = app.msg_tx.try_send(AppMsg::CloneUrlBackspace);
                }
                KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    let _ = app.msg_tx.try_send(AppMsg::CloneUrlClear);
                }
                KeyCode::Up => {
                    let _ = app.msg_tx.try_send(AppMsg::ClonePreviousMainDir);
                }
                KeyCode::Down => {
                    let _ = app.msg_tx.try_send(AppMsg::CloneNextMainDir);
                }
                KeyCode::Char(c) => {
                    let _ = app.msg_tx.try_send(AppMsg::CloneUrlInput(c));
                }
                _ => {}
            }
        }
        Some(CloneStage::ConfirmReplace { .. }) => {
            match key.code {
                KeyCode::Esc | KeyCode::Char('2') => {
                    // Cancel - go back to input
                    let _ = app.msg_tx.try_send(AppMsg::CloneConfirmReplace(false));
                }
                KeyCode::Char('1') | KeyCode::Enter => {
                    // Confirm replace
                    let _ = app.msg_tx.try_send(AppMsg::CloneConfirmReplace(true));
                }
                _ => {}
            }
        }
        Some(CloneStage::Executing) => {
            match key.code {
                KeyCode::Esc => {
                    // Cancel clone operation
                    let _ = app.msg_tx.try_send(AppMsg::CancelClone);
                }
                _ => {}
            }
        }
        Some(CloneStage::Error(_)) => {
            match key.code {
                KeyCode::Esc => {
                    let _ = app.msg_tx.try_send(AppMsg::CancelClone);
                }
                _ => {}
            }
        }
        None => {}
    }
}
