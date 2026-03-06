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

    /// Search box focused
    Searching,

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
            AppState::Searching => 2,
            AppState::Running => 1,
            AppState::Loading { .. } | AppState::Error { .. } => 0,
            AppState::Quit => 0,
        }
    }

    /// Check if current state is modal (shows overlay)
    pub fn is_modal(&self) -> bool {
        matches!(
            self,
            AppState::ShowingActions { .. } | AppState::ShowingHelp | AppState::ChoosingDir { .. }
        )
    }

    /// Check if application is running normally
    pub fn is_running(&self) -> bool {
        matches!(self, AppState::Running | AppState::Searching)
    }

    /// Check if application is in a loading/error state
    pub fn is_blocking(&self) -> bool {
        matches!(self, AppState::Loading { .. } | AppState::Error { .. })
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
        assert!(AppState::Searching.is_running());
        assert!(!AppState::ShowingHelp.is_running());
    }
}
