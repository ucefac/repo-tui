//! Multi-directory keyboard operation integration tests
//!
//! Tests for keyboard shortcuts related to multi-directory management

/// Represents a key event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyCode {
    Char(char),
    Enter,
    Esc,
    Up,
    Down,
    Left,
    Right,
    Space,
    Backspace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct KeyEvent {
    code: KeyCode,
    ctrl: bool,
    shift: bool,
}

impl KeyEvent {
    fn new(code: KeyCode) -> Self {
        Self {
            code,
            ctrl: false,
            shift: false,
        }
    }

    fn ctrl(code: KeyCode) -> Self {
        Self {
            code,
            ctrl: true,
            shift: false,
        }
    }

    fn shift(code: KeyCode) -> Self {
        Self {
            code,
            ctrl: false,
            shift: true,
        }
    }
}

/// Application states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppState {
    Running,
    ManagingDirs,
    ChoosingDir {
        mode: ChooserMode,
        return_to: ReturnTarget,
    },
    ShowingHelp,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChooserMode {
    AddMainDirectory,
    AddSingleRepo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReturnTarget {
    Running,
    ManagingDirs,
}

/// Keyboard handler for multi-directory operations
struct MultiDirKeyboardHandler {
    state: AppState,
    previous_state: Option<AppState>,
    messages: Vec<AppMessage>,
}

#[derive(Debug, Clone, PartialEq)]
enum AppMessage {
    ShowMainDirectoryManager,
    CloseMainDirectoryManager,
    AddMainDirectory,
    AddSingleRepo,
    RemoveMainDirectory,
    ToggleDirectoryEnabled,
    NavigateUp,
    NavigateDown,
    Confirm,
    Cancel,
    ShowHelp,
    Quit,
}

impl MultiDirKeyboardHandler {
    fn new() -> Self {
        Self {
            state: AppState::Running,
            previous_state: None,
            messages: Vec::new(),
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Vec<AppMessage> {
        self.messages.clear();

        match self.state {
            AppState::Running => self.handle_running_keys(key),
            AppState::ManagingDirs => self.handle_managing_dirs_keys(key),
            AppState::ChoosingDir { mode, .. } => self.handle_chooser_keys(key, mode),
            AppState::ShowingHelp => self.handle_help_keys(key),
            AppState::Quit => vec![],
        }
    }

    fn handle_running_keys(&mut self, key: KeyEvent) -> Vec<AppMessage> {
        match key.code {
            KeyCode::Char('m') if !key.ctrl && !key.shift => {
                self.state = AppState::ManagingDirs;
                self.messages.push(AppMessage::ShowMainDirectoryManager);
            }
            KeyCode::Char('a') if !key.ctrl && !key.shift => {
                self.state = AppState::ChoosingDir {
                    mode: ChooserMode::AddMainDirectory,
                    return_to: ReturnTarget::ManagingDirs,
                };
                self.messages.push(AppMessage::AddMainDirectory);
            }
            KeyCode::Char('A') | KeyCode::Char('a') if key.shift => {
                self.state = AppState::ChoosingDir {
                    mode: ChooserMode::AddSingleRepo,
                    return_to: ReturnTarget::Running,
                };
                self.messages.push(AppMessage::AddSingleRepo);
            }
            KeyCode::Char('?') | KeyCode::Char('h') => {
                self.previous_state = Some(self.state);
                self.state = AppState::ShowingHelp;
                self.messages.push(AppMessage::ShowHelp);
            }
            KeyCode::Char('q') if !key.ctrl => {
                self.state = AppState::Quit;
                self.messages.push(AppMessage::Quit);
            }
            _ => {}
        }

        self.messages.clone()
    }

    fn handle_managing_dirs_keys(&mut self, key: KeyEvent) -> Vec<AppMessage> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.state = AppState::Running;
                self.messages.push(AppMessage::CloseMainDirectoryManager);
            }
            KeyCode::Char('a') => {
                self.previous_state = Some(self.state);
                self.state = AppState::ChoosingDir {
                    mode: ChooserMode::AddMainDirectory,
                    return_to: ReturnTarget::ManagingDirs,
                };
                self.messages.push(AppMessage::AddMainDirectory);
            }
            KeyCode::Char('d') => {
                self.messages.push(AppMessage::RemoveMainDirectory);
            }
            KeyCode::Char('e') => {
                self.messages.push(AppMessage::ToggleDirectoryEnabled);
            }
            KeyCode::Space => {
                self.messages.push(AppMessage::ToggleDirectoryEnabled);
            }
            KeyCode::Up => {
                self.messages.push(AppMessage::NavigateUp);
            }
            KeyCode::Down => {
                self.messages.push(AppMessage::NavigateDown);
            }
            KeyCode::Char('?') => {
                self.previous_state = Some(self.state);
                self.state = AppState::ShowingHelp;
                self.messages.push(AppMessage::ShowHelp);
            }
            _ => {}
        }

        self.messages.clone()
    }

    fn handle_chooser_keys(&mut self, key: KeyEvent, _mode: ChooserMode) -> Vec<AppMessage> {
        match key.code {
            KeyCode::Esc => {
                // Return to previous state (managing dirs or running)
                if let Some(prev) = self.previous_state {
                    self.state = prev;
                } else {
                    self.state = AppState::Running;
                }
                self.messages.push(AppMessage::Cancel);
            }
            KeyCode::Enter => {
                self.state = AppState::ManagingDirs;
                self.messages.push(AppMessage::Confirm);
            }
            KeyCode::Up => {
                self.messages.push(AppMessage::NavigateUp);
            }
            KeyCode::Down => {
                self.messages.push(AppMessage::NavigateDown);
            }
            _ => {}
        }

        self.messages.clone()
    }

    fn handle_help_keys(&mut self, key: KeyEvent) -> Vec<AppMessage> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                // Return to previous state
                if let Some(prev) = self.previous_state {
                    self.state = prev;
                } else {
                    self.state = AppState::Running;
                }
                self.previous_state = None;
            }
            _ => {}
        }

        self.messages.clone()
    }

    fn current_state(&self) -> AppState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_m_key_enters_main_directory_manager() {
        // Arrange
        let mut handler = MultiDirKeyboardHandler::new();
        assert_eq!(handler.current_state(), AppState::Running);

        // Act
        let messages = handler.handle_key(KeyEvent::new(KeyCode::Char('m')));

        // Assert
        assert_eq!(handler.current_state(), AppState::ManagingDirs);
        assert!(messages.contains(&AppMessage::ShowMainDirectoryManager));
    }

    #[test]
    fn test_esc_exits_main_directory_manager() {
        // Arrange
        let mut handler = MultiDirKeyboardHandler::new();
        handler.handle_key(KeyEvent::new(KeyCode::Char('m'))); // Enter manager
        assert_eq!(handler.current_state(), AppState::ManagingDirs);

        // Act
        let messages = handler.handle_key(KeyEvent::new(KeyCode::Esc));

        // Assert
        assert_eq!(handler.current_state(), AppState::Running);
        assert!(messages.contains(&AppMessage::CloseMainDirectoryManager));
    }

    #[test]
    fn test_a_key_adds_main_directory_from_manager() {
        // Arrange
        let mut handler = MultiDirKeyboardHandler::new();
        handler.handle_key(KeyEvent::new(KeyCode::Char('m'))); // Enter manager

        // Act
        let messages = handler.handle_key(KeyEvent::new(KeyCode::Char('a')));

        // Assert
        assert_eq!(
            handler.current_state(),
            AppState::ChoosingDir {
                mode: ChooserMode::AddMainDirectory,
                return_to: ReturnTarget::ManagingDirs,
            }
        );

        // Act
        let messages = handler.handle_key(KeyEvent::new(KeyCode::Esc));

        // Assert: Should return to manager
        assert_eq!(handler.current_state(), AppState::ManagingDirs);
        assert!(messages.contains(&AppMessage::Cancel));
    }

    #[test]
    fn test_enter_confirms_chooser_selection() {
        // Arrange
        let mut handler = MultiDirKeyboardHandler::new();
        handler.handle_key(KeyEvent::new(KeyCode::Char('m')));
        handler.handle_key(KeyEvent::new(KeyCode::Char('a'))); // Open chooser

        // Act
        let messages = handler.handle_key(KeyEvent::new(KeyCode::Enter));

        // Assert
        assert_eq!(handler.current_state(), AppState::ManagingDirs);
        assert!(messages.contains(&AppMessage::Confirm));
    }

    #[test]
    fn test_help_key_shows_help() {
        // Arrange
        let mut handler = MultiDirKeyboardHandler::new();

        // Act
        let messages = handler.handle_key(KeyEvent::new(KeyCode::Char('?')));

        // Assert
        assert_eq!(handler.current_state(), AppState::ShowingHelp);
        assert!(messages.contains(&AppMessage::ShowHelp));
    }

    #[test]
    fn test_esc_closes_help() {
        // Arrange
        let mut handler = MultiDirKeyboardHandler::new();
        handler.handle_key(KeyEvent::new(KeyCode::Char('?'))); // Open help
        assert_eq!(handler.current_state(), AppState::ShowingHelp);

        // Act
        handler.handle_key(KeyEvent::new(KeyCode::Esc));

        // Assert: Should return to Running (previous state)
        assert_eq!(handler.current_state(), AppState::Running);
    }

    #[test]
    fn test_help_from_manager_returns_to_manager() {
        // Arrange
        let mut handler = MultiDirKeyboardHandler::new();
        handler.handle_key(KeyEvent::new(KeyCode::Char('m'))); // Enter manager
        assert_eq!(handler.current_state(), AppState::ManagingDirs);

        handler.handle_key(KeyEvent::new(KeyCode::Char('?'))); // Open help
        assert_eq!(handler.current_state(), AppState::ShowingHelp);

        // Act
        handler.handle_key(KeyEvent::new(KeyCode::Esc)); // Close help

        // Assert: Should return to ManagingDirs
        assert_eq!(handler.current_state(), AppState::ManagingDirs);
    }

    #[test]
    fn test_q_quits_from_running() {
        // Arrange
        let mut handler = MultiDirKeyboardHandler::new();

        // Act
        let messages = handler.handle_key(KeyEvent::new(KeyCode::Char('q')));

        // Assert
        assert_eq!(handler.current_state(), AppState::Quit);
        assert!(messages.contains(&AppMessage::Quit));
    }

    #[test]
    fn test_no_conflict_between_a_and_shift_a() {
        // Arrange
        let mut handler1 = MultiDirKeyboardHandler::new();
        let mut handler2 = MultiDirKeyboardHandler::new();

        // Act
        let messages1 = handler1.handle_key(KeyEvent::new(KeyCode::Char('a')));
        let messages2 = handler2.handle_key(KeyEvent::shift(KeyCode::Char('a')));

        // Assert: Different messages
        assert!(messages1.contains(&AppMessage::AddMainDirectory));
        assert!(messages2.contains(&AppMessage::AddSingleRepo));
    }

    #[test]
    fn test_navigation_in_chooser() {
        // Arrange
        let mut handler = MultiDirKeyboardHandler::new();
        handler.handle_key(KeyEvent::new(KeyCode::Char('m')));
        handler.handle_key(KeyEvent::new(KeyCode::Char('a'))); // Open chooser

        // Act: Navigate in chooser
        let messages_up = handler.handle_key(KeyEvent::new(KeyCode::Up));
        let messages_down = handler.handle_key(KeyEvent::new(KeyCode::Down));

        // Assert
        assert!(messages_up.contains(&AppMessage::NavigateUp));
        assert!(messages_down.contains(&AppMessage::NavigateDown));
    }

    #[test]
    fn test_key_handling_in_quit_state() {
        // Arrange
        let mut handler = MultiDirKeyboardHandler::new();
        handler.handle_key(KeyEvent::new(KeyCode::Char('q'))); // Quit
        assert_eq!(handler.current_state(), AppState::Quit);

        // Act: Keys should be ignored in Quit state
        let messages = handler.handle_key(KeyEvent::new(KeyCode::Char('m')));

        // Assert
        assert!(messages.is_empty());
        assert_eq!(handler.current_state(), AppState::Quit);
    }

    #[test]
    fn test_ignored_keys_in_manager() {
        // Arrange
        let mut handler = MultiDirKeyboardHandler::new();
        handler.handle_key(KeyEvent::new(KeyCode::Char('m'))); // Enter manager

        // Act: Keys that should be ignored
        let messages = handler.handle_key(KeyEvent::new(KeyCode::Char('z')));

        // Assert: Should be in manager with no messages
        assert_eq!(handler.current_state(), AppState::ManagingDirs);
        assert!(messages.is_empty());
    }

    #[test]
    fn test_m_key_in_manager_does_nothing() {
        // Arrange
        let mut handler = MultiDirKeyboardHandler::new();
        handler.handle_key(KeyEvent::new(KeyCode::Char('m'))); // Enter manager

        // Act: Press 'm' again
        let messages = handler.handle_key(KeyEvent::new(KeyCode::Char('m')));

        // Assert: Should stay in manager
        assert_eq!(handler.current_state(), AppState::ManagingDirs);
        assert!(messages.is_empty());
    }

    #[test]
    fn test_chooser_navigation_preserves_mode() {
        // Arrange
        let mut handler = MultiDirKeyboardHandler::new();
        handler.handle_key(KeyEvent::shift(KeyCode::Char('a'))); // Open single repo chooser

        // Act: Navigate
        handler.handle_key(KeyEvent::new(KeyCode::Down));
        handler.handle_key(KeyEvent::new(KeyCode::Up));

        // Assert: Should still be in single repo chooser mode
        assert_eq!(
            handler.current_state(),
            AppState::ChoosingDir {
                mode: ChooserMode::AddSingleRepo,
                return_to: ReturnTarget::Running,
            }
        );
    }

    #[test]
    fn test_h_key_also_shows_help() {
        // Arrange
        let mut handler = MultiDirKeyboardHandler::new();

        // Act
        let messages = handler.handle_key(KeyEvent::new(KeyCode::Char('h')));

        // Assert
        assert_eq!(handler.current_state(), AppState::ShowingHelp);
        assert!(messages.contains(&AppMessage::ShowHelp));
    }

    #[test]
    fn test_complete_workflow_add_main_directory() {
        // Arrange
        let mut handler = MultiDirKeyboardHandler::new();

        // Step 1: Enter manager
        handler.handle_key(KeyEvent::new(KeyCode::Char('m')));
        assert_eq!(handler.current_state(), AppState::ManagingDirs);

        // Step 2: Add directory
        let messages = handler.handle_key(KeyEvent::new(KeyCode::Char('a')));
        assert!(messages.contains(&AppMessage::AddMainDirectory));
        assert!(matches!(
            handler.current_state(),
            AppState::ChoosingDir {
                mode: ChooserMode::AddMainDirectory,
                return_to: ReturnTarget::ManagingDirs,
            }
        ));

        // Step 3: Confirm
        handler.handle_key(KeyEvent::new(KeyCode::Enter));
        assert_eq!(handler.current_state(), AppState::ManagingDirs);

        // Step 4: Exit manager
        handler.handle_key(KeyEvent::new(KeyCode::Esc));
        assert_eq!(handler.current_state(), AppState::Running);
    }

    #[test]
    fn test_complete_workflow_add_single_repo() {
        // Arrange
        let mut handler = MultiDirKeyboardHandler::new();

        // Step 1: Press Shift+A
        let messages = handler.handle_key(KeyEvent::shift(KeyCode::Char('a')));
        assert!(messages.contains(&AppMessage::AddSingleRepo));
        assert!(matches!(
            handler.current_state(),
            AppState::ChoosingDir {
                mode: ChooserMode::AddSingleRepo,
                return_to: ReturnTarget::Running,
            }
        ));

        // Step 2: Cancel
        handler.handle_key(KeyEvent::new(KeyCode::Esc));
        assert_eq!(handler.current_state(), AppState::Running);
    }

    #[test]
    fn test_manager_navigation_wraps_around() {
        // Arrange: This test verifies navigation behavior
        let mut handler = MultiDirKeyboardHandler::new();
        handler.handle_key(KeyEvent::new(KeyCode::Char('m'))); // Enter manager

        // Act & Assert: Up from beginning should wrap (implementation dependent)
        // The actual wrapping is handled by the App model, not the keyboard handler
        // Here we just verify navigation messages are sent
        let messages = handler.handle_key(KeyEvent::new(KeyCode::Up));
        assert!(messages.contains(&AppMessage::NavigateUp));
    }
}
