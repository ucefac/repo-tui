//! Application messages

use crate::action::Action;
use crate::config::Config;
use crate::error::{ActionError, ConfigError, RepoError};
use crate::repo::{GitStatus, Repository};
use std::path::PathBuf;

/// Commands for async execution
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Cmd {
    /// Load configuration
    LoadConfig,

    /// Load repositories from directory (legacy)
    LoadRepositories(PathBuf),

    /// Load repositories from multiple main directories and standalone repos
    LoadRepositoriesMulti {
        /// Main directories with optional max depth
        main_dirs: Vec<(PathBuf, Option<usize>)>,
        /// Standalone repository paths
        single_repos: Vec<PathBuf>,
    },

    /// Check git status for a repository
    CheckGitStatus(usize, PathBuf),

    /// Execute an action
    ExecuteAction(Action, Repository),

    /// Execute batch actions
    ExecuteBatchAction(Action, Vec<Repository>),

    /// Scan a directory (for directory chooser)
    ScanDirectory(PathBuf),

    /// Save configuration
    SaveConfig(Config),

    /// Validate directory path
    ValidateDirectory(PathBuf),

    /// Clone a repository
    CloneRepository {
        /// Repository URL
        url: String,
        /// Target path
        target_path: PathBuf,
    },

    /// Check for updates
    CheckForUpdate,

    /// Delete a repository from filesystem
    DeleteRepository(usize, PathBuf, String),
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

    // === Clone Operations ===
    /// Start clone operation
    StartClone,

    /// Clone URL input character
    CloneUrlInput(char),

    /// Clone URL paste text
    CloneUrlPaste(String),

    /// Clone URL backspace
    CloneUrlBackspace,

    /// Clone URL clear from cursor
    CloneUrlClear,

    /// Clone URL confirm
    CloneUrlConfirm,

    /// Navigate to next main directory for clone
    CloneNextMainDir,

    /// Navigate to previous main directory for clone
    ClonePreviousMainDir,

    /// Select main directory for clone
    CloneSelectMainDir(usize),

    /// Confirm replace existing folder
    CloneConfirmReplace(bool),

    /// Clone progress update
    CloneProgress(String),

    /// Clone completed
    CloneCompleted(Result<crate::repo::Repository, crate::error::CloneError>),

    /// Clone retry after error
    CloneRetry,

    /// Cancel clone operation
    CancelClone,

    /// Terminal needs reinitialization (after interactive CLI command)
    TerminalNeedsReinit,

    // === Main Directory Management ===
    /// Show main directory manager
    ShowMainDirectoryManager,

    /// Close main directory manager
    CloseMainDirectoryManager,

    /// Add main directory
    AddMainDirectory(PathBuf),

    /// Remove main directory
    RemoveMainDirectory(usize),

    /// Toggle main directory enabled
    ToggleMainDirectoryEnabled(usize),

    /// Update main directory display name
    UpdateMainDirectoryName(usize, String),

    /// Navigate up in main directory manager
    MainDirNavUp,

    /// Navigate down in main directory manager
    MainDirNavDown,

    /// Edit main directory
    EditMainDirectory(usize),

    /// Confirm main directory edit
    ConfirmEditMainDirectory,

    /// Cancel main directory edit
    CancelEditMainDirectory,

    /// Show delete confirmation for main directory
    ShowDeleteMainDirConfirmation,

    /// Cancel delete confirmation for main directory
    CancelDeleteMainDirConfirmation,

    /// Show delete confirmation for repository
    ShowDeleteRepoConfirmation,

    /// Cancel delete confirmation for repository
    CancelDeleteRepoConfirmation,

    /// Delete repository from filesystem
    DeleteRepository(usize),

    // === Single Repository Management ===
    /// Show add single repo chooser
    ShowAddSingleRepoChooser,

    /// Add single repository
    AddSingleRepository(PathBuf),

    /// Remove single repository
    RemoveSingleRepository(PathBuf),

    // === Directory Chooser Enhanced ===
    /// Show directory chooser with mode
    ShowDirectoryChooserWithMode(crate::app::state::DirectoryChooserMode),

    /// Directories selected (multi-select)
    DirectoriesSelected(Vec<String>),

    /// Cancel directory chooser and return
    CancelDirectoryChooser,

    // === Update Operations ===
    /// Trigger update check
    TriggerUpdateCheck,

    /// Update check completed
    UpdateCheckCompleted(
        Box<std::result::Result<crate::update::UpdateCheckResult, crate::error::UpdateError>>,
    ),

    /// Dismiss update notification
    DismissUpdateNotification,

    /// Ignore update version
    IgnoreUpdateVersion(String),

    /// Repository deleted result
    RepositoryDeleted {
        /// Repository path
        repo_path: PathBuf,
        /// Repository name
        repo_name: String,
        /// Success flag
        success: bool,
    },
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
