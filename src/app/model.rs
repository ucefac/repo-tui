//! Application model

use ratatui::widgets::ListState;
use std::collections::HashSet;
use std::path::PathBuf;
use tokio::sync::mpsc;

use crate::app::msg::AppMsg;
use crate::app::state::{AppState, ViewMode};
use crate::config::Config;
use crate::favorites::FavoritesStore;
use crate::git::cache::StatusCache;
use crate::git::scheduler::GitStatusScheduler;
use crate::recent::RecentStore;
use crate::repo::Repository;
use crate::ui::Theme;
use std::sync::Arc;

/// Main directory info for UI
#[derive(Debug, Clone)]
pub struct MainDirectoryInfo {
    /// Directory path
    pub path: PathBuf,
    /// Display name
    pub display_name: String,
    /// Enabled state
    pub enabled: bool,
    /// Repository count
    pub repo_count: usize,
}

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

    /// Search box focus flag (true when search is active)
    pub search_active: bool,

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

    /// Path bar click area (for mouse events)
    pub path_bar_area: Option<ratatui::layout::Rect>,

    /// Git status cache
    pub git_cache: Arc<StatusCache>,

    /// Git status scheduler
    pub git_scheduler: Option<Arc<GitStatusScheduler>>,

    /// Current UI theme
    pub theme: Theme,

    /// Favorites store
    pub favorites: FavoritesStore,

    /// Recent repositories store
    pub recent: RecentStore,

    /// Current view mode
    pub view_mode: ViewMode,

    /// Selection mode flag (for batch operations)
    pub selection_mode: bool,

    /// Selected repository indices (for batch operations)
    pub selected_indices: HashSet<usize>,

    /// Main directories list
    pub main_directories: Vec<MainDirectoryInfo>,

    /// Single repositories list
    pub single_repositories: Vec<PathBuf>,

    /// Active main directory index
    pub active_main_dir_index: Option<usize>,

    /// Move target directories (for Shift+M move operation)
    pub move_target_dirs: Vec<(usize, String, usize)>,

    /// Update check status
    pub update_status: crate::update::UpdateStatus,

    /// Available update information
    pub available_update: Option<crate::update::UpdateInfo>,

    /// Update notification dismissed
    pub update_notification_dismissed: bool,

    /// Terminal needs reinitialization (after external TUI exit)
    pub needs_terminal_reinit: bool,
}

impl App {
    /// Create a new application instance
    pub fn new(msg_tx: mpsc::Sender<AppMsg>) -> Self {
        let git_cache = Arc::new(StatusCache::default_cache());
        let scheduler = GitStatusScheduler::new(Arc::clone(&git_cache), msg_tx.clone());
        let git_scheduler = Some(Arc::new(scheduler));

        Self {
            config: None,
            main_dir: None,
            repositories: Vec::new(),
            filtered_indices: Vec::new(),
            search_query: String::new(),
            pending_search: None,
            search_active: false,
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
            path_bar_area: None,
            git_cache,
            git_scheduler,
            theme: Theme::dark(),
            favorites: FavoritesStore::new(),
            recent: RecentStore::new(),
            view_mode: ViewMode::All,
            selection_mode: false,
            selected_indices: HashSet::new(),
            main_directories: Vec::new(),
            single_repositories: Vec::new(),
            active_main_dir_index: None,
            move_target_dirs: Vec::new(),
            update_status: crate::update::UpdateStatus::NeverChecked,
            available_update: None,
            update_notification_dismissed: false,
            needs_terminal_reinit: false,
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

    /// Filter repositories based on search query using fuzzy search
    pub fn apply_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_indices = (0..self.repositories.len()).collect();
        } else {
            // Use fuzzy search with scoring
            let results = crate::repo::filter_repos_fuzzy(&self.repositories, &self.search_query);
            self.filtered_indices = results.into_iter().map(|(idx, _)| idx).collect();
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

    /// Filter repositories based on view mode
    pub fn filter_by_view_mode(&mut self) {
        match self.view_mode {
            ViewMode::All => {
                // Show all repositories (respect search)
                self.apply_filter();
            }
            ViewMode::Favorites => {
                // Show only favorited repositories
                if self.search_query.is_empty() {
                    // No search: filter by favorites only
                    self.filtered_indices = self
                        .repositories
                        .iter()
                        .enumerate()
                        .filter(|(_, repo)| self.favorites.contains(&repo.path))
                        .map(|(idx, _)| idx)
                        .collect();
                } else {
                    // Search active: filter by both search and favorites
                    let search_results =
                        crate::repo::filter_repos_fuzzy(&self.repositories, &self.search_query);
                    self.filtered_indices = search_results
                        .into_iter()
                        .filter_map(|(idx, _)| {
                            if self.favorites.contains(&self.repositories[idx].path) {
                                Some(idx)
                            } else {
                                None
                            }
                        })
                        .collect();
                }
            }
            ViewMode::Recent => {
                // Show only recently opened repositories
                if self.search_query.is_empty() {
                    // No search: filter by recent only
                    self.filtered_indices = self
                        .repositories
                        .iter()
                        .enumerate()
                        .filter(|(_, repo)| self.recent.contains(&repo.path))
                        .map(|(idx, _)| idx)
                        .collect();
                } else {
                    // Search active: filter by both search and recent
                    let search_results =
                        crate::repo::filter_repos_fuzzy(&self.repositories, &self.search_query);
                    self.filtered_indices = search_results
                        .into_iter()
                        .filter_map(|(idx, _)| {
                            if self.recent.contains(&self.repositories[idx].path) {
                                Some(idx)
                            } else {
                                None
                            }
                        })
                        .collect();
                }
            }
        }

        // Reset selection
        if !self.filtered_indices.is_empty() {
            self.set_selected_index(Some(0));
        } else {
            self.set_selected_index(None);
        }
        self.scroll_offset = 0;
    }

    /// Toggle view mode between All and Favorites
    pub fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::All => ViewMode::Favorites,
            ViewMode::Favorites => ViewMode::Recent,
            ViewMode::Recent => ViewMode::All,
        };
        self.filter_by_view_mode();
    }

    /// Toggle favorite for current repository
    pub fn toggle_favorite(&mut self) {
        if let Some(repo) = self.selected_repository() {
            let path = repo.path.clone();
            self.favorites.toggle(&path);
            // Re-filter if in favorites view
            if self.view_mode == ViewMode::Favorites {
                self.filter_by_view_mode();
            }
        }
    }

    /// Check if current repository is favorited
    pub fn is_current_favorited(&self) -> bool {
        if let Some(repo) = self.selected_repository() {
            self.favorites.contains(&repo.path)
        } else {
            false
        }
    }

    /// Get the current view mode
    pub fn get_view_mode(&self) -> &ViewMode {
        &self.view_mode
    }

    /// Toggle selection mode
    pub fn toggle_selection_mode(&mut self) {
        self.selection_mode = !self.selection_mode;
    }

    /// Toggle selection for current repository
    pub fn toggle_selection(&mut self) {
        if let Some(idx) = self.selected_index() {
            if self.selected_indices.contains(&idx) {
                self.selected_indices.remove(&idx);
            } else {
                self.selected_indices.insert(idx);
            }
        }
    }

    /// Select all filtered repositories
    pub fn select_all(&mut self) {
        self.selected_indices = self.filtered_indices.iter().copied().collect();
    }

    /// Clear all selections
    pub fn clear_selection(&mut self) {
        self.selected_indices.clear();
    }

    /// Get selected repositories
    pub fn get_selected_repos(&self) -> Vec<&Repository> {
        self.selected_indices
            .iter()
            .filter_map(|&idx| self.repositories.get(idx))
            .collect()
    }

    /// Get selected count
    pub fn selected_count(&self) -> usize {
        self.selected_indices.len()
    }

    /// Check if current repository is selected
    pub fn is_current_selected(&self) -> bool {
        if let Some(idx) = self.selected_index() {
            self.selected_indices.contains(&idx)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod favorites_tests {
    use super::*;
    use crate::repo::Repository;
    use tempfile::TempDir;

    fn create_test_app_with_favorites() -> (App, TempDir) {
        let (tx, _rx) = mpsc::channel(100);
        let mut app = App::new(tx);

        let temp_dir = TempDir::new().unwrap();
        let repo1 = Repository {
            name: "repo1".to_string(),
            path: temp_dir.path().join("repo1"),
            last_modified: None,
            is_dirty: false,
            branch: Some("main".to_string()),
            is_git_repo: true,
            source: crate::repo::source::RepoSource::Standalone,
        };
        let repo2 = Repository {
            name: "repo2".to_string(),
            path: temp_dir.path().join("repo2"),
            last_modified: None,
            is_dirty: true,
            branch: Some("feature".to_string()),
            is_git_repo: true,
            source: crate::repo::source::RepoSource::Standalone,
        };
        app.repositories = vec![repo1, repo2];
        app.filtered_indices = vec![0, 1];
        app.set_selected_index(Some(0));

        (app, temp_dir)
    }

    #[test]
    fn test_app_favorites_new() {
        let (tx, _rx) = mpsc::channel(100);
        let app = App::new(tx);
        assert!(app.favorites.is_empty());
        assert_eq!(app.view_mode, ViewMode::All);
    }

    #[test]
    fn test_toggle_favorite() {
        let (mut app, _temp) = create_test_app_with_favorites();
        let repo_path = app.repositories[0].path.clone();

        // Initially not favorited
        assert!(!app.favorites.contains(&repo_path));

        // Toggle to favorite
        app.toggle_favorite();
        assert!(app.favorites.contains(&repo_path));

        // Toggle to remove
        app.toggle_favorite();
        assert!(!app.favorites.contains(&repo_path));
    }

    #[test]
    fn test_is_current_favorited() {
        let (mut app, _temp) = create_test_app_with_favorites();

        // Initially not favorited
        assert!(!app.is_current_favorited());

        // Add to favorites
        let repo_path = app.repositories[0].path.clone();
        app.favorites.add(&repo_path);
        assert!(app.is_current_favorited());
    }

    #[test]
    fn test_toggle_view_mode() {
        let (mut app, _temp) = create_test_app_with_favorites();

        // Start in All mode
        assert_eq!(app.view_mode, ViewMode::All);

        // Toggle to Favorites
        app.toggle_view_mode();
        assert_eq!(app.view_mode, ViewMode::Favorites);

        // Toggle to Recent
        app.toggle_view_mode();
        assert_eq!(app.view_mode, ViewMode::Recent);

        // Toggle back to All
        app.toggle_view_mode();
        assert_eq!(app.view_mode, ViewMode::All);
    }

    #[test]
    fn test_filter_by_view_mode_all() {
        let (mut app, _temp) = create_test_app_with_favorites();

        // Add one to favorites
        let repo_path = app.repositories[0].path.clone();
        app.favorites.add(&repo_path);

        // Switch to All mode - should show all
        app.view_mode = ViewMode::All;
        app.filter_by_view_mode();
        assert_eq!(app.filtered_indices.len(), 2);
    }

    #[test]
    fn test_filter_by_view_mode_favorites() {
        let (mut app, _temp) = create_test_app_with_favorites();

        // Add one to favorites
        let repo_path = app.repositories[0].path.clone();
        app.favorites.add(&repo_path);

        // Switch to Favorites mode - should show only favorites
        app.view_mode = ViewMode::Favorites;
        app.filter_by_view_mode();
        assert_eq!(app.filtered_indices.len(), 1);
        assert_eq!(app.repositories[app.filtered_indices[0]].name, "repo1");
    }

    #[test]
    fn test_filter_by_view_mode_favorites_empty() {
        let (mut app, _temp) = create_test_app_with_favorites();

        // No favorites
        app.view_mode = ViewMode::Favorites;
        app.filter_by_view_mode();
        assert_eq!(app.filtered_indices.len(), 0);
    }

    #[test]
    fn test_filter_favorites_with_search() {
        let (mut app, _temp) = create_test_app_with_favorites();

        // Add both to favorites
        app.favorites.add(&app.repositories[0].path);
        app.favorites.add(&app.repositories[1].path);

        // Search for repo1
        app.search_query = "repo1".to_string();
        app.view_mode = ViewMode::Favorites;
        app.filter_by_view_mode();

        assert_eq!(app.filtered_indices.len(), 1);
        assert_eq!(app.repositories[app.filtered_indices[0]].name, "repo1");
    }

    #[test]
    fn test_get_view_mode() {
        let (app, _temp) = create_test_app_with_favorites();
        assert_eq!(*app.get_view_mode(), ViewMode::All);
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
            is_git_repo: true,
            source: crate::repo::source::RepoSource::Standalone,
        };
        let repo2 = Repository {
            name: "repo2".to_string(),
            path: temp_dir.path().join("repo2"),
            last_modified: None,
            is_dirty: true,
            branch: Some("feature".to_string()),
            is_git_repo: true,
            source: crate::repo::source::RepoSource::Standalone,
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
            is_git_repo: true,
            source: crate::repo::source::RepoSource::Standalone,
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
    fn test_toggle_selection() {
        let (mut app, _temp) = create_test_app();
        app.set_selected_index(Some(0));

        // Initially no selections
        assert_eq!(app.selected_indices.len(), 0);

        app.toggle_selection();
        assert_eq!(app.selected_indices.len(), 1);
        assert!(app.selected_indices.contains(&0));

        app.toggle_selection();
        assert_eq!(app.selected_indices.len(), 0);
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

    #[test]
    fn test_selection_mode_toggle() {
        let (mut app, _temp) = create_test_app();

        app.select_all();
        assert_eq!(app.selected_indices.len(), 2);
        assert!(app.selected_indices.contains(&0));
        assert!(app.selected_indices.contains(&1));
    }

    #[test]
    fn test_clear_selection() {
        let (mut app, _temp) = create_test_app();

        app.select_all();
        assert_eq!(app.selected_indices.len(), 2);

        app.clear_selection();
        assert_eq!(app.selected_indices.len(), 0);
    }

    #[test]
    fn test_get_selected_repos() {
        let (mut app, _temp) = create_test_app();

        app.selected_indices.insert(0);
        app.selected_indices.insert(1);

        let selected = app.get_selected_repos();
        assert_eq!(selected.len(), 2);

        // Collect names and check (order doesn't matter with HashSet)
        let names: HashSet<&str> = selected.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains("repo1"));
        assert!(names.contains("repo2"));
    }

    #[test]
    fn test_selected_count() {
        let (mut app, _temp) = create_test_app();

        assert_eq!(app.selected_count(), 0);

        app.selected_indices.insert(0);
        assert_eq!(app.selected_count(), 1);

        app.selected_indices.insert(1);
        assert_eq!(app.selected_count(), 2);
    }

    #[test]
    fn test_is_current_selected() {
        let (mut app, _temp) = create_test_app();
        app.set_selected_index(Some(0));

        assert!(!app.is_current_selected());

        app.selected_indices.insert(0);
        assert!(app.is_current_selected());
    }
}
