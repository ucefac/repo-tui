//! Application state enum

use std::path::PathBuf;

pub use crate::repo::clone::ParsedGitUrl;
use ratatui::widgets::ListState;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// Clone operation stage
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloneStage {
    /// Input URL and select main directory
    InputUrl,
    /// Confirm replacing existing folder
    ConfirmReplace {
        /// Path to existing folder
        existing_path: PathBuf,
    },
    /// Executing git clone
    Executing,
    /// Error state with error details
    Error(crate::error::CloneError),
}

/// Clone state
#[derive(Debug, Clone)]
pub struct CloneState {
    /// URL input buffer
    pub url_input: String,
    /// Cursor position in URL input
    pub cursor_position: usize,
    /// Parsed URL info
    pub parsed_url: Option<ParsedGitUrl>,
    /// Selected main directory index
    pub target_main_dir: Option<usize>,
    /// Current stage
    pub stage: CloneStage,
    /// Progress lines for display
    pub progress_lines: Vec<String>,
    /// Main directory list state for selection
    pub main_dir_list_state: ListState,
    /// Cancel flag for async operation
    pub cancel_flag: Arc<AtomicBool>,
    /// Validation error for URL input (real-time validation)
    pub validation_error: Option<crate::error::CloneError>,
}

impl PartialEq for CloneState {
    fn eq(&self, other: &Self) -> bool {
        self.url_input == other.url_input
            && self.cursor_position == other.cursor_position
            && self.parsed_url == other.parsed_url
            && self.target_main_dir == other.target_main_dir
            && self.stage == other.stage
            && self.progress_lines == other.progress_lines
            && self.selected_main_dir() == other.selected_main_dir()
            && self.validation_error == other.validation_error
        // Note: cancel_flag and main_dir_list_state are not compared
    }
}

impl Eq for CloneState {}

impl CloneState {
    /// Create a new clone state
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            url_input: String::new(),
            cursor_position: 0,
            parsed_url: None,
            target_main_dir: None,
            stage: CloneStage::InputUrl,
            progress_lines: Vec::new(),
            main_dir_list_state: list_state,
            cancel_flag: Arc::new(AtomicBool::new(false)),
            validation_error: None,
        }
    }

    /// Reset to initial state
    pub fn reset(&mut self) {
        self.url_input.clear();
        self.cursor_position = 0;
        self.parsed_url = None;
        self.target_main_dir = None;
        self.stage = CloneStage::InputUrl;
        self.progress_lines.clear();
        self.main_dir_list_state.select(Some(0));
        self.cancel_flag
            .store(false, std::sync::atomic::Ordering::Relaxed);
        self.validation_error = None;
    }

    /// Insert character at cursor position
    pub fn insert_char(&mut self, c: char) {
        self.url_input.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    /// Delete character before cursor
    pub fn backspace(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.url_input.remove(self.cursor_position);
        }
    }

    /// Delete character at cursor
    pub fn delete(&mut self) {
        if self.cursor_position < self.url_input.len() {
            self.url_input.remove(self.cursor_position);
        }
    }

    /// Clear input from cursor to end
    pub fn clear_from_cursor(&mut self) {
        self.url_input.truncate(self.cursor_position);
    }

    /// Paste text at cursor position
    pub fn paste(&mut self, text: &str) {
        self.url_input.insert_str(self.cursor_position, text);
        self.cursor_position += text.len();
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.url_input.len() {
            self.cursor_position += 1;
        }
    }

    /// Navigate to next main directory
    pub fn next_main_dir(&mut self, max: usize) {
        let current = self.main_dir_list_state.selected().unwrap_or(0);
        if current < max.saturating_sub(1) {
            self.main_dir_list_state.select(Some(current + 1));
        }
    }

    /// Navigate to previous main directory
    pub fn previous_main_dir(&mut self) {
        let current = self.main_dir_list_state.selected().unwrap_or(0);
        if current > 0 {
            self.main_dir_list_state.select(Some(current - 1));
        }
    }

    /// Get selected main directory index
    pub fn selected_main_dir(&self) -> usize {
        self.main_dir_list_state.selected().unwrap_or(0)
    }

    /// Set selected main directory
    pub fn set_selected_main_dir(&mut self, index: usize) {
        self.main_dir_list_state.select(Some(index));
    }

    /// Add progress line
    pub fn add_progress(&mut self, line: String) {
        self.progress_lines.push(line);
        // Keep only last 100 lines to prevent memory issues
        if self.progress_lines.len() > 100 {
            self.progress_lines.remove(0);
        }
    }

    /// Clear progress lines
    pub fn clear_progress(&mut self) {
        self.progress_lines.clear();
    }

    /// Check if operation should be cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancel_flag.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Request cancellation
    pub fn cancel(&self) {
        self.cancel_flag
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Default for CloneState {
    fn default() -> Self {
        Self::new()
    }
}

/// Directory chooser mode
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectoryChooserMode {
    /// Select main directory (allows adding/updating main directories)
    SelectMainDirectory {
        /// Allow multiple selection
        allow_multiple: bool,
        /// Edit mode (replace existing or add new)
        edit_mode: bool,
        /// Return target for subtitle display
        return_to: ReturnTarget,
    },
    /// Add single repository (validates .git exists)
    AddSingleRepository,
}

impl Default for DirectoryChooserMode {
    fn default() -> Self {
        DirectoryChooserMode::SelectMainDirectory {
            allow_multiple: false,
            edit_mode: false,
            return_to: ReturnTarget::Running,
        }
    }
}

/// Return target for directory chooser
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ReturnTarget {
    /// Return to running state (main repo list)
    #[default]
    Running,
    /// Return to main directory manager
    ManagingDirs,
}

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

        /// Chooser mode
        mode: DirectoryChooserMode,

        /// Return to state after selection
        return_to: ReturnTarget,
    },

    /// Managing main directories
    ManagingDirs {
        /// List state for main directories
        list_state: ratatui::widgets::ListState,
        /// Selected directory index
        selected_dir_index: usize,
        /// Editing state
        editing: Option<MainDirEdit>,
        /// Delete confirmation state
        confirming_delete: bool,
    },

    /// Confirming repository deletion
    ConfirmingDeleteRepo {
        /// Repository index in filtered list
        repo_index: usize,
        /// Repository path for deletion
        repo_path: PathBuf,
        /// Repository name for display
        repo_name: String,
    },

    /// Showing help panel
    ShowingHelp {
        /// Scroll offset for viewport
        scroll_offset: usize,
    },

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

    /// Cloning repository
    Cloning {
        /// Clone state
        clone_state: CloneState,
    },

    /// Choosing move target directory
    ChoosingMoveTarget {
        /// Target main directories
        targets: Vec<PathBuf>,
        /// Selected index
        selected_index: usize,
        /// Current repository path
        current_repo_path: PathBuf,
        /// Current main directory index (for skipping same directory)
        current_main_dir_index: Option<usize>,
    },
}

/// Main directory edit state
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MainDirEdit {
    /// Directory index being edited (None for new)
    pub index: Option<usize>,
    /// Current path
    pub path: std::path::PathBuf,
    /// Display name
    pub display_name: String,
    /// Enabled state
    pub enabled: bool,
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
            AppState::Cloning { .. } => 6,
            AppState::ShowingHelp { .. } => 4,
            AppState::ManagingDirs { .. } => 4,
            AppState::ConfirmingDeleteRepo { .. } => 5,
            AppState::ChoosingMoveTarget { .. } => 5,
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
            AppState::ShowingHelp { .. }
                | AppState::ChoosingDir { .. }
                | AppState::SelectingTheme { .. }
                | AppState::ManagingDirs { .. }
                | AppState::Cloning { .. }
                | AppState::ConfirmingDeleteRepo { .. }
                | AppState::ChoosingMoveTarget { .. }
        )
    }

    /// Check if application is running normally
    pub fn is_running(&self) -> bool {
        matches!(self, AppState::Running)
    }

    /// Check if application is in cloning state
    pub fn is_cloning(&self) -> bool {
        matches!(self, AppState::Cloning { .. })
    }

    /// Get mutable reference to clone state if in Cloning state
    pub fn clone_state_mut(&mut self) -> Option<&mut CloneState> {
        if let AppState::Cloning { clone_state } = self {
            Some(clone_state)
        } else {
            None
        }
    }

    /// Get reference to clone state if in Cloning state
    pub fn clone_state(&self) -> Option<&CloneState> {
        if let AppState::Cloning { clone_state } = self {
            Some(clone_state)
        } else {
            None
        }
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
        assert_eq!(AppState::ShowingHelp { scroll_offset: 0 }.priority(), 4);
        assert_eq!(AppState::Running.priority(), 1);
    }

    #[test]
    fn test_is_modal() {
        assert!(AppState::ShowingHelp { scroll_offset: 0 }.is_modal());
        assert!(!AppState::Running.is_modal());
    }

    #[test]
    fn test_is_running() {
        assert!(AppState::Running.is_running());
        assert!(!AppState::ShowingHelp { scroll_offset: 0 }.is_running());
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
