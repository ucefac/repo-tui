//! Application state enum

use std::path::PathBuf;

use crate::repo::Repository;

/// Application state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppState {
    /// Normal running state - main UI
    Running,

    /// Choosing main directory (first launch)
    ChoosingDir {
        /// Current directory path
        path: PathBuf,

        /// Directory entries (store names only for Clone compatibility)
        entries: Vec<String>,

        /// Selected index in the directory list
        selected_index: usize,

        /// Scroll offset for viewport tracking
        scroll_offset: usize,
    },

    /// Showing action menu
    ShowingActions {
        /// Selected repository
        repo: Repository,
    },

    /// Showing help panel
    ShowingHelp,

    /// Loading state
    Loading {
        /// Loading message
        message: String,
    },

    /// Error state
    Error {
        /// Error message
        message: String,
    },

    /// Quitting
    Quit,

    /// Selecting theme
    SelectingTheme {
        /// Theme list state
        theme_list_state: ratatui::widgets::ListState,
        /// Preview theme (stored to ensure "what you see is what you get")
        preview_theme: crate::ui::theme::Theme,
    },
}

/// View mode for repository list
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewMode {
    /// Show all repositories
    All,
    /// Show only favorited repositories
    Favorites,
    /// Show recently opened repositories
    Recent,
}

impl AppState {
    /// Get state priority for key handling
    ///
    /// Higher priority states intercept all key events
    pub fn priority(&self) -> u8 {
        match self {
            AppState::ShowingActions { .. } => 5,
            AppState::ShowingHelp => 4,
            AppState::ChoosingDir { .. } => 3,
            AppState::SelectingTheme { .. } => 3,
            AppState::Running => 1,
            AppState::Loading { .. } | AppState::Error { .. } => 0,
            AppState::Quit => 0,
        }
    }

    /// Check if current state is modal (shows overlay)
    pub fn is_modal(&self) -> bool {
        matches!(
            self,
            AppState::ShowingActions { .. }
                | AppState::ShowingHelp
                | AppState::ChoosingDir { .. }
                | AppState::SelectingTheme { .. }
        )
    }

    /// Check if application is running normally
    pub fn is_running(&self) -> bool {
        matches!(self, AppState::Running)
    }

    /// Check if application is in a loading/error state
    pub fn is_blocking(&self) -> bool {
        matches!(self, AppState::Loading { .. } | AppState::Error { .. })
    }

    /// Get mutable reference to theme list state if in SelectingTheme state
    pub fn theme_list_state_mut(&mut self) -> Option<&mut ratatui::widgets::ListState> {
        if let AppState::SelectingTheme {
            theme_list_state, ..
        } = self
        {
            Some(theme_list_state)
        } else {
            None
        }
    }

    /// Get reference to preview theme if in SelectingTheme state
    pub fn preview_theme(&self) -> Option<&crate::ui::theme::Theme> {
        if let AppState::SelectingTheme { preview_theme, .. } = self {
            Some(preview_theme)
        } else {
            None
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        AppState::Loading {
            message: "Initializing...".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_priority() {
        assert_eq!(
            AppState::ShowingActions {
                repo: Repository::test_repo()
            }
            .priority(),
            5
        );
        assert_eq!(AppState::ShowingHelp.priority(), 4);
        assert_eq!(AppState::Running.priority(), 1);
    }

    #[test]
    fn test_is_modal() {
        assert!(AppState::ShowingHelp.is_modal());
        assert!(!AppState::Running.is_modal());
    }

    #[test]
    fn test_is_running() {
        assert!(AppState::Running.is_running());
        assert!(!AppState::ShowingHelp.is_running());
    }

    #[test]
    fn test_selecting_theme_state() {
        let mut state = AppState::SelectingTheme {
            theme_list_state: ratatui::widgets::ListState::default(),
            preview_theme: crate::ui::theme::Theme::dark(),
        };

        assert!(state.is_modal());
        assert_eq!(state.priority(), 3);
        assert!(state.theme_list_state_mut().is_some());
    }

    #[test]
    fn test_theme_list_state_mut_wrong_state() {
        let mut state = AppState::Running;
        assert!(state.theme_list_state_mut().is_none());
    }
}
