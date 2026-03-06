//! Application model

use ratatui::widgets::ListState;
use std::path::PathBuf;
use tokio::sync::mpsc;

use crate::app::msg::AppMsg;
use crate::app::state::AppState;
use crate::config::Config;
use crate::repo::Repository;

/// Application model
pub struct App {
    /// Application configuration
    pub config: Option<Config>,

    /// Main directory path
    pub main_dir: Option<PathBuf>,

    /// All discovered repositories
    pub repositories: Vec<Repository>,

    /// Filtered repository indices (after search)
    pub filtered_indices: Vec<usize>,

    /// Search query
    pub search_query: String,

    /// Pending search query (for debounce)
    pub pending_search: Option<String>,

    /// List state for Ratatui
    pub list_state: ListState,

    /// Scroll offset for virtual list
    pub scroll_offset: usize,

    /// Application state
    pub state: AppState,

    /// Loading flag
    pub loading: bool,

    /// Loading message
    pub loading_message: Option<String>,

    /// Error message
    pub error_message: Option<String>,

    /// Selected repository (for action menu)
    pub selected_repo: Option<Repository>,

    /// Message sender for async operations
    pub msg_tx: mpsc::Sender<AppMsg>,
}

impl App {
    /// Create a new application instance
    pub fn new(msg_tx: mpsc::Sender<AppMsg>) -> Self {
        Self {
            config: None,
            main_dir: None,
            repositories: Vec::new(),
            filtered_indices: Vec::new(),
            search_query: String::new(),
            pending_search: None,
            list_state: ListState::default(),
            scroll_offset: 0,
            state: AppState::Loading {
                message: "Initializing...".to_string(),
            },
            loading: true,
            loading_message: Some("Loading configuration...".to_string()),
            error_message: None,
            selected_repo: None,
            msg_tx,
        }
    }

    /// Get the currently selected repository index
    pub fn selected_index(&self) -> Option<usize> {
        self.list_state.selected()
    }

    /// Set the selected index
    pub fn set_selected_index(&mut self, index: Option<usize>) {
        self.list_state.select(index);
    }

    /// Get the currently selected repository
    pub fn selected_repository(&self) -> Option<&Repository> {
        if let Some(idx) = self.selected_index() {
            self.filtered_indices
                .get(idx)
                .and_then(|&i| self.repositories.get(i))
        } else {
            None
        }
    }

    /// Get visible count based on terminal height
    pub fn visible_count(&self, terminal_height: u16) -> usize {
        // Subtract borders and search box
        terminal_height.saturating_sub(6) as usize
    }

    /// Update scroll offset to keep selected item visible
    pub fn update_scroll_offset(&mut self, terminal_height: u16) {
        let visible_count = self.visible_count(terminal_height);
        if visible_count == 0 {
            return;
        }

        let selected = self.selected_index().unwrap_or(0);
        let current_offset = self.scroll_offset;

        // Scroll down if selected is below visible area
        if selected >= current_offset + visible_count {
            self.scroll_offset = selected - visible_count + 1;
        }
        // Scroll up if selected is above visible area
        else if selected < current_offset {
            self.scroll_offset = selected;
        }

        // Note: ListState in ratatui 0.29 doesn't have set_offset
        // The offset is managed internally by the List widget
    }

    /// Filter repositories based on search query
    pub fn apply_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_indices = (0..self.repositories.len()).collect();
        } else {
            let query_lower = self.search_query.to_lowercase();
            self.filtered_indices = self
                .repositories
                .iter()
                .enumerate()
                .filter(|(_, repo)| repo.name.to_lowercase().contains(&query_lower))
                .map(|(i, _)| i)
                .collect();
        }

        // Reset selection
        if !self.filtered_indices.is_empty() {
            self.set_selected_index(Some(0));
        } else {
            self.set_selected_index(None);
        }
        self.scroll_offset = 0;
    }

    /// Check if repositories are loaded
    pub fn has_repositories(&self) -> bool {
        !self.repositories.is_empty()
    }

    /// Get repository count
    pub fn repository_count(&self) -> usize {
        self.repositories.len()
    }

    /// Get filtered count
    pub fn filtered_count(&self) -> usize {
        self.filtered_indices.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::Repository;
    use tempfile::TempDir;

    fn create_test_app() -> (App, TempDir) {
        let (tx, _rx) = mpsc::channel(100);
        let mut app = App::new(tx);

        let temp_dir = TempDir::new().unwrap();
        let repo1 = Repository {
            name: "repo1".to_string(),
            path: temp_dir.path().join("repo1"),
            last_modified: None,
            is_dirty: false,
            branch: Some("main".to_string()),
        };
        let repo2 = Repository {
            name: "repo2".to_string(),
            path: temp_dir.path().join("repo2"),
            last_modified: None,
            is_dirty: true,
            branch: Some("feature".to_string()),
        };
        app.repositories = vec![repo1, repo2];
        app.filtered_indices = vec![0, 1];
        app.set_selected_index(Some(0));

        (app, temp_dir)
    }

    #[test]
    fn test_app_new() {
        let (tx, _rx) = mpsc::channel(100);
        let app = App::new(tx);
        assert!(app.loading);
        assert!(app.error_message.is_none());
    }

    #[test]
    fn test_apply_filter() {
        let (mut app, _temp) = create_test_app();

        app.search_query = "repo1".to_string();
        app.apply_filter();

        assert_eq!(app.filtered_indices.len(), 1);
    }

    #[test]
    fn test_apply_filter_empty() {
        let (mut app, _temp) = create_test_app();

        app.search_query = "nonexistent".to_string();
        app.apply_filter();

        assert_eq!(app.filtered_indices.len(), 0);
    }

    #[test]
    fn test_selected_repository() {
        let (app, _temp) = create_test_app();

        let repo = app.selected_repository();
        assert!(repo.is_some());
        assert_eq!(repo.unwrap().name, "repo1");
    }

    #[test]
    fn test_selected_repository_none() {
        let (tx, _rx) = mpsc::channel(100);
        let app = App::new(tx);

        assert!(app.selected_repository().is_none());
    }

    #[test]
    fn test_apply_filter_case_insensitive() {
        let (mut app, _temp) = create_test_app();

        app.search_query = "REPO1".to_string();
        app.apply_filter();

        assert_eq!(app.filtered_indices.len(), 1);
        assert_eq!(app.repositories[app.filtered_indices[0]].name, "repo1");
    }

    #[test]
    fn test_apply_filter_clear() {
        let (mut app, _temp) = create_test_app();

        // First filter
        app.search_query = "repo1".to_string();
        app.apply_filter();
        assert_eq!(app.filtered_indices.len(), 1);

        // Clear filter
        app.search_query.clear();
        app.apply_filter();
        assert_eq!(app.filtered_indices.len(), 2);
    }

    #[test]
    fn test_visible_count() {
        let (app, _temp) = create_test_app();

        let count = app.visible_count(30);
        assert_eq!(count, 24); // 30 - 6 = 24

        let count_small = app.visible_count(10);
        assert_eq!(count_small, 4); // 10 - 6 = 4
    }

    #[test]
    fn test_update_scroll_offset() {
        let (mut app, _temp) = create_test_app();

        // Add more repos to test scrolling
        app.repositories.push(Repository {
            name: "repo3".to_string(),
            path: PathBuf::from("/tmp/repo3"),
            last_modified: None,
            is_dirty: false,
            branch: Some("main".to_string()),
        });
        app.apply_filter();

        // Test scroll down
        app.set_selected_index(Some(0));
        app.update_scroll_offset(10);
        assert_eq!(app.scroll_offset, 0);

        // Select item beyond visible area
        app.set_selected_index(Some(5));
        app.update_scroll_offset(10);
        // Should have scrolled
        assert!(app.scroll_offset > 0);

        // Scroll back up
        app.set_selected_index(Some(0));
        app.update_scroll_offset(10);
        assert_eq!(app.scroll_offset, 0);
    }

    #[test]
    fn test_has_repositories() {
        let (app, _temp) = create_test_app();
        assert!(app.has_repositories());

        let (tx, _rx) = mpsc::channel(100);
        let empty_app = App::new(tx);
        assert!(!empty_app.has_repositories());
    }

    #[test]
    fn test_repository_count() {
        let (app, _temp) = create_test_app();
        assert_eq!(app.repository_count(), 2);
    }

    #[test]
    fn test_filtered_count() {
        let (mut app, _temp) = create_test_app();

        assert_eq!(app.filtered_count(), 2);

        app.search_query = "repo1".to_string();
        app.apply_filter();
        assert_eq!(app.filtered_count(), 1);
    }

    #[test]
    fn test_selected_index() {
        let (mut app, _temp) = create_test_app();

        assert_eq!(app.selected_index(), Some(0));

        app.set_selected_index(Some(1));
        assert_eq!(app.selected_index(), Some(1));

        app.set_selected_index(None);
        assert_eq!(app.selected_index(), None);
    }
}
