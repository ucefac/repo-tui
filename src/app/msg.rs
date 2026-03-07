//! Application messages

use crate::action::Action;
use crate::config::Config;
use crate::error::{ActionError, ConfigError, RepoError};
use crate::repo::{GitStatus, Repository};
use std::path::PathBuf;

/// Commands for async execution
#[derive(Debug, Clone)]
pub enum Cmd {
    /// Load configuration
    LoadConfig,

    /// Load repositories from directory
    LoadRepositories(PathBuf),

    /// Check git status for a repository
    CheckGitStatus(usize, PathBuf),

    /// Execute an action
    ExecuteAction(Action, Repository),

    /// Execute batch actions
    ExecuteBatchAction(Action, Vec<Repository>),

    /// Scan a directory (for directory chooser)
    ScanDirectory(PathBuf),
}

/// Application messages
#[derive(Debug, Clone)]
pub enum AppMsg {
    // === User Input ===
    /// Search input character
    SearchInput(char),

    /// Search backspace
    SearchBackspace,

    /// Clear search
    SearchClear,

    /// Navigate to next repository
    NextRepo,

    /// Navigate to previous repository
    PreviousRepo,

    /// Jump to first repository
    JumpToTop,

    /// Jump to last repository
    JumpToBottom,

    /// Scroll down half page
    ScrollDown,

    /// Scroll up half page
    ScrollUp,

    /// Navigate down in directory chooser
    DirectoryNavDown,

    /// Navigate up in directory chooser
    DirectoryNavUp,

    /// Navigate down in action menu
    ActionMenuNavDown,

    /// Navigate up in action menu
    ActionMenuNavUp,

    // === Async Events ===
    /// Configuration loaded
    ConfigLoaded(Box<Result<Config, ConfigError>>),

    /// Repositories loaded
    RepositoriesLoaded(Result<Vec<Repository>, RepoError>),

    /// Git status checked for a repository
    GitStatusChecked(usize, Result<GitStatus, RepoError>),

    /// Action executed
    ActionExecuted(Result<(), ActionError>),

    // === State Transitions ===
    /// Open action menu
    OpenActions,

    /// Close action menu
    CloseActions,

    /// Execute action
    ExecuteAction(Action),

    /// Show help panel
    ShowHelp,

    /// Close help panel
    CloseHelp,

    /// Show directory chooser
    ShowDirectoryChooser,

    /// Directory selected
    DirectorySelected(String),

    /// Directory entries scanned
    DirectoryEntriesScanned(Vec<String>),

    /// Scan error
    ScanError(String),

    /// Refresh repository list
    Refresh,

    // === Global ===
    /// Tick (for debounce/timer)
    Tick,

    /// Quit application
    Quit,

    /// Cancel current operation
    Cancel,

    /// Show error dialog
    ShowError(String),

    /// Close error dialog
    CloseError,

    /// Copy path to clipboard
    CopyPathToClipboard(PathBuf),

    /// Theme changed (dark/light toggle)
    ThemeChanged,

    /// Open theme selector
    OpenThemeSelector,

    /// Close theme selector
    CloseThemeSelector,

    /// Select specified theme
    SelectTheme(String),

    /// Navigate up in theme selector
    ThemeNavUp,

    /// Navigate down in theme selector
    ThemeNavDown,

    /// Toggle favorite for current repository
    ToggleFavorite,

    /// Switch to favorites view
    ShowFavorites,

    /// Switch to all repositories view
    ShowAllRepos,

    /// Switch to recent repositories view
    ShowRecent,

    /// Toggle selection mode
    ToggleSelectionMode,

    /// Toggle selection for current repository
    ToggleSelection,

    /// Select all filtered repositories
    SelectAll,

    /// Clear all selections
    ClearSelection,

    /// Execute batch action
    ExecuteBatchAction(Action),

    /// Batch action completed
    BatchActionExecuted(crate::action::batch::BatchResult),
}

impl AppMsg {
    /// Check if message is a search input
    pub fn is_search_input(&self) -> bool {
        matches!(
            self,
            AppMsg::SearchInput(_) | AppMsg::SearchBackspace | AppMsg::SearchClear
        )
    }

    /// Check if message is a navigation input
    pub fn is_navigation(&self) -> bool {
        matches!(
            self,
            AppMsg::NextRepo
                | AppMsg::PreviousRepo
                | AppMsg::JumpToTop
                | AppMsg::JumpToBottom
                | AppMsg::ScrollDown
                | AppMsg::ScrollUp
        )
    }

    /// Check if message is a view mode switch
    pub fn is_view_switch(&self) -> bool {
        matches!(
            self,
            AppMsg::ShowFavorites | AppMsg::ShowAllRepos | AppMsg::ShowRecent
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_search_input() {
        assert!(AppMsg::SearchInput('a').is_search_input());
        assert!(AppMsg::SearchBackspace.is_search_input());
        assert!(AppMsg::SearchClear.is_search_input());
        assert!(!AppMsg::NextRepo.is_search_input());
    }

    #[test]
    fn test_is_navigation() {
        assert!(AppMsg::NextRepo.is_navigation());
        assert!(AppMsg::PreviousRepo.is_navigation());
        assert!(!AppMsg::SearchInput('a').is_navigation());
    }

    #[test]
    fn test_is_view_switch() {
        assert!(AppMsg::ShowFavorites.is_view_switch());
        assert!(AppMsg::ShowAllRepos.is_view_switch());
        assert!(!AppMsg::NextRepo.is_view_switch());
        assert!(!AppMsg::ToggleFavorite.is_view_switch());
    }
}
