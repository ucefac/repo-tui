//! Keyboard event handler

use crate::app::model::App;
use crate::app::msg::AppMsg;
use crate::app::state::AppState;
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
        AppState::ShowingActions { .. } => {
            handle_action_menu_keys(key, app, runtime);
        }
        AppState::ShowingHelp => {
            handle_help_keys(key, app);
        }
        AppState::ChoosingDir { .. } => {
            handle_chooser_keys(key, app, runtime);
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
        KeyCode::Char('q') | KeyCode::Esc => {
            let _ = app.msg_tx.try_send(AppMsg::CloseActions);
        }
        KeyCode::Char('c') | KeyCode::Char('1') => {
            let _ = app
                .msg_tx
                .try_send(AppMsg::ExecuteAction(crate::action::Action::CdAndCloud));
        }
        KeyCode::Char('w') | KeyCode::Char('2') => {
            let _ = app
                .msg_tx
                .try_send(AppMsg::ExecuteAction(crate::action::Action::OpenWebStorm));
        }
        KeyCode::Char('v') | KeyCode::Char('3') => {
            let _ = app
                .msg_tx
                .try_send(AppMsg::ExecuteAction(crate::action::Action::OpenVsCode));
        }
        KeyCode::Char('f') | KeyCode::Char('4') => {
            let _ = app.msg_tx.try_send(AppMsg::ExecuteAction(
                crate::action::Action::OpenFileManager,
            ));
        }
        KeyCode::Char('j') | KeyCode::Down => {
            // Navigate down in menu
            let _ = app.msg_tx.try_send(AppMsg::ActionMenuNavDown);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            // Navigate up in menu
            let _ = app.msg_tx.try_send(AppMsg::ActionMenuNavUp);
        }
        KeyCode::Enter => {
            // Execute selected action
            if let Some(action) = get_selected_action(app) {
                let _ = app.msg_tx.try_send(AppMsg::ExecuteAction(action));
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

/// Handle keys in help panel
fn handle_help_keys(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
            let _ = app.msg_tx.try_send(AppMsg::CloseHelp);
        }
        _ => {}
    }
}

/// Handle keys in directory chooser
fn handle_chooser_keys(key: KeyEvent, app: &mut App, runtime: &Runtime) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            let _ = app.msg_tx.try_send(AppMsg::Quit);
        }
        KeyCode::Enter => {
            // Enter selected directory or confirm selection
            handle_directory_enter(app, runtime);
        }
        KeyCode::Left | KeyCode::Char('h') => {
            // Go to parent directory
            handle_directory_back(app, runtime);
        }
        KeyCode::Right | KeyCode::Char('l') => {
            // Enter selected directory
            handle_directory_enter(app, runtime);
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
        KeyCode::Char('j') | KeyCode::Down => {
            let _ = app.msg_tx.try_send(AppMsg::DirectoryNavDown);
        }
        KeyCode::Char('k') | KeyCode::Up => {
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
fn handle_directory_back(app: &mut App, runtime: &Runtime) {
    if let AppState::ChoosingDir { path, .. } = &app.state {
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
            };
        }
    }
}

/// Handle entering a directory
fn handle_directory_enter(app: &mut App, runtime: &Runtime) {
    if let AppState::ChoosingDir {
        path,
        entries,
        selected_index,
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
        // Including j/k/g/G/m/r/q/? etc. - all function keys are blocked
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
        KeyCode::Char('j') | KeyCode::Down => {
            let _ = app.msg_tx.try_send(AppMsg::NextRepo);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let _ = app.msg_tx.try_send(AppMsg::PreviousRepo);
        }
        KeyCode::Char('g') => {
            if key.modifiers.contains(KeyModifiers::NONE) {
                let _ = app.msg_tx.try_send(AppMsg::JumpToTop);
            }
        }
        KeyCode::Char('G') => {
            let _ = app.msg_tx.try_send(AppMsg::JumpToBottom);
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

        // Change main directory
        KeyCode::Char('m') => {
            let _ = app.msg_tx.try_send(AppMsg::ShowDirectoryChooser);
        }

        // Actions
        KeyCode::Enter | KeyCode::Char('o') => {
            let _ = app.msg_tx.try_send(AppMsg::OpenActions);
        }

        // Direct action shortcuts (without opening menu)
        KeyCode::Char('c') => {
            if let Some(repo) = app.selected_repository().cloned() {
                app.selected_repo = Some(repo);
                let _ = app
                    .msg_tx
                    .try_send(AppMsg::ExecuteAction(crate::action::Action::CdAndCloud));
            }
        }
        KeyCode::Char('w') => {
            if let Some(repo) = app.selected_repository().cloned() {
                app.selected_repo = Some(repo);
                let _ = app
                    .msg_tx
                    .try_send(AppMsg::ExecuteAction(crate::action::Action::OpenWebStorm));
            }
        }
        KeyCode::Char('v') => {
            if let Some(repo) = app.selected_repository().cloned() {
                app.selected_repo = Some(repo);
                let _ = app
                    .msg_tx
                    .try_send(AppMsg::ExecuteAction(crate::action::Action::OpenVsCode));
            }
        }
        KeyCode::Char('f') => {
            if let Some(repo) = app.selected_repository().cloned() {
                app.selected_repo = Some(repo);
                let _ = app.msg_tx.try_send(AppMsg::ExecuteAction(
                    crate::action::Action::OpenFileManager,
                ));
            }
        }

        // Global
        KeyCode::Char('r') => {
            let _ = app.msg_tx.try_send(AppMsg::Refresh);
        }
        KeyCode::Char('?') => {
            let _ = app.msg_tx.try_send(AppMsg::ShowHelp);
        }
        KeyCode::Char('q') => {
            let _ = app.msg_tx.try_send(AppMsg::Quit);
        }

        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        use crate::repo::Repository;
        use std::path::PathBuf;
        app.repositories = vec![Repository {
            name: "test-repo".to_string(),
            path: PathBuf::from("/tmp/test-repo"),
            last_modified: None,
            is_dirty: false,
            branch: Some("main".to_string()),
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
            create_test_key(KeyCode::Char('q')),
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
        };

        // Test navigation
        handle_chooser_keys(create_test_key(KeyCode::Char('j')), &mut app, &runtime);
        handle_chooser_keys(create_test_key(KeyCode::Char('k')), &mut app, &runtime);
        handle_chooser_keys(create_test_key(KeyCode::Down), &mut app, &runtime);
        handle_chooser_keys(create_test_key(KeyCode::Up), &mut app, &runtime);

        // Test home/end
        handle_chooser_keys(create_test_key(KeyCode::Home), &mut app, &runtime);
        handle_chooser_keys(create_test_key(KeyCode::End), &mut app, &runtime);

        // Test quit
        handle_chooser_keys(create_test_key(KeyCode::Char('q')), &mut app, &runtime);
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
        };

        // Test h/l navigation
        handle_chooser_keys(create_test_key(KeyCode::Char('h')), &mut app, &runtime);
        handle_chooser_keys(create_test_key(KeyCode::Char('l')), &mut app, &runtime);
        handle_chooser_keys(create_test_key(KeyCode::Left), &mut app, &runtime);
        handle_chooser_keys(create_test_key(KeyCode::Right), &mut app, &runtime);
    }
}
