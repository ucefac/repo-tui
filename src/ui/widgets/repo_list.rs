//! Repository list widget
//!
//! Displays a scrollable list of Git repositories with virtual list optimization.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem};

use crate::repo::Repository;
use crate::ui::layout::{truncate_middle, DisplayMode};
use crate::ui::theme::Theme;
use std::collections::HashSet;

/// Repository list widget
#[derive(Debug, Clone)]
pub struct RepoList<'a> {
    /// Repositories to display
    pub repositories: &'a [Repository],
    /// Filtered indices (indices into repositories)
    pub filtered_indices: &'a [usize],
    /// Currently selected index (into filtered_indices)
    pub selected_index: Option<usize>,
    /// Scroll offset
    pub scroll_offset: usize,
    /// Terminal height (for calculating visible count)
    pub visible_height: u16,
    /// Terminal width (for responsive layout)
    pub area_width: u16,
    /// Theme
    pub theme: &'a Theme,
    /// Show git status
    pub show_git_status: bool,
    /// Show branch name
    pub show_branch: bool,
    /// Total count (for display)
    pub total_count: usize,
    /// Set of favorite repository indices
    pub favorites: HashSet<usize>,
    /// Selection mode flag
    pub selection_mode: bool,
    /// Set of selected repository indices
    pub selected: HashSet<usize>,
}

impl<'a> RepoList<'a> {
    /// Create a new repository list
    pub fn new(
        repositories: &'a [Repository],
        filtered_indices: &'a [usize],
        theme: &'a Theme,
    ) -> Self {
        Self {
            repositories,
            filtered_indices,
            selected_index: None,
            scroll_offset: 0,
            visible_height: 10,
            area_width: 80,
            theme,
            show_git_status: true,
            show_branch: true,
            total_count: repositories.len(),
            favorites: HashSet::new(),
            selection_mode: false,
            selected: HashSet::new(),
        }
    }

    /// Set favorites
    pub fn favorites(mut self, favorites: HashSet<usize>) -> Self {
        self.favorites = favorites;
        self
    }

    /// Set area width for responsive layout
    pub fn area_width(mut self, width: u16) -> Self {
        self.area_width = width;
        self
    }

    /// Set selected index
    pub fn selected_index(mut self, index: Option<usize>) -> Self {
        self.selected_index = index;
        self
    }

    /// Set scroll offset
    pub fn scroll_offset(mut self, offset: usize) -> Self {
        self.scroll_offset = offset;
        self
    }

    /// Set visible height
    pub fn visible_height(mut self, height: u16) -> Self {
        self.visible_height = height;
        self
    }

    /// Set show git status
    pub fn show_git_status(mut self, show: bool) -> Self {
        self.show_git_status = show;
        self
    }

    /// Set show branch
    pub fn show_branch(mut self, show: bool) -> Self {
        self.show_branch = show;
        self
    }

    /// Set total count
    pub fn total_count(mut self, count: usize) -> Self {
        self.total_count = count;
        self
    }

    /// Set selection mode
    pub fn selection_mode(mut self, mode: bool) -> Self {
        self.selection_mode = mode;
        self
    }

    /// Set selected indices
    pub fn selected(mut self, selected: HashSet<usize>) -> Self {
        self.selected = selected;
        self
    }

    /// Calculate visible range
    fn visible_range(&self) -> (usize, usize) {
        let start = self.scroll_offset;
        let visible_count = self.visible_count();
        let end = (start + visible_count).min(self.filtered_indices.len());
        (start, end)
    }

    /// Get display mode based on terminal width
    fn display_mode(&self) -> DisplayMode {
        if self.area_width < 60 {
            DisplayMode::Compact
        } else if self.area_width < 100 {
            DisplayMode::Medium
        } else {
            DisplayMode::Large
        }
    }

    /// Calculate how many items can be visible
    fn visible_count(&self) -> usize {
        // Reserve space for borders (top+bottom = 2)
        // Title is inside the border, doesn't reduce visible items
        self.visible_height.saturating_sub(2) as usize
    }

    /// Update scroll offset to ensure selected item is visible
    pub fn update_scroll(&mut self) {
        let visible_count = self.visible_count();
        if visible_count == 0 {
            return;
        }

        let selected = self.selected_index.unwrap_or(0);

        // Scroll down if selected is below visible area
        if selected >= self.scroll_offset + visible_count {
            self.scroll_offset = selected - visible_count + 1;
        }
        // Scroll up if selected is above visible area
        else if selected < self.scroll_offset {
            self.scroll_offset = selected;
        }
    }
}

/// Parameters for formatting a repository item
struct RepoItemParams {
    is_favorite: bool,
    selection_mode: bool,
    is_checked: bool,
}

/// Format a repository item for display with responsive layout
fn format_repo_item(
    repo: &Repository,
    is_selected: bool,
    _show_git_status: bool,
    display_mode: DisplayMode,
    area_width: u16,
    theme: &Theme,
    params: RepoItemParams,
) -> Line<'static> {
    let prefix = if is_selected { "▌ " } else { "  " };
    let mut spans = Vec::new();

    // Add prefix
    spans.push(Span::raw(prefix));

    // Add checkbox in selection mode or favorite marker in normal mode
    if params.selection_mode {
        let checkbox = if params.is_checked { "[✓] " } else { "[ ] " };
        let checkbox_style = if params.is_checked {
            Style::default().fg(theme.colors.success.into())
        } else {
            Style::default().fg(theme.colors.secondary.into())
        };
        spans.push(Span::styled(checkbox, checkbox_style));
    } else {
        // Add favorite marker in normal mode
        if params.is_favorite {
            spans.push(Span::styled(
                "★ ",
                Style::default().fg(theme.colors.primary.into()),
            ));
        } else {
            spans.push(Span::raw("  "));
        }
    }

    // Add repo name
    let max_name_len = display_mode.max_name_length();
    let truncated_name = truncate_middle(&repo.display_name(), max_name_len);
    spans.push(Span::styled(
        truncated_name,
        Style::default().fg(theme.colors.foreground.into()),
    ));

    // Add separator space before branch
    spans.push(Span::raw(" "));

    // Add branch name or "Not a git repo" message for Medium+ modes
    if display_mode.show_branch() {
        spans.push(Span::raw(" "));
        if repo.is_git_repo {
            if let Some(ref branch) = repo.branch {
                let max_branch_len = if area_width < 100 { 20 } else { 30 };
                let truncated_branch = truncate_middle(branch, max_branch_len);
                spans.push(Span::styled(
                    format!("({})", truncated_branch),
                    Style::default().fg(theme.colors.secondary.into()),
                ));
            } else {
                // Git repo but branch not detected yet
                spans.push(Span::styled(
                    "(detecting...)",
                    Style::default().fg(theme.colors.text_muted.into()),
                ));
            }
        } else {
            // Not a git repository
            spans.push(Span::styled(
                "(Not a git repo)",
                Style::default().fg(theme.colors.text_muted.into()),
            ));
        }
    }

    // Add status text for Large+ modes
    if display_mode.show_status() {
        spans.push(Span::raw("  "));
        let status_text = if repo.is_dirty { "Modified" } else { "Clean" };
        let status_color = if repo.is_dirty {
            theme.colors.error.into()
        } else {
            theme.colors.success.into()
        };
        spans.push(Span::styled(status_text, Style::default().fg(status_color)));
    }

    Line::from(spans)
}

impl<'a> Widget for RepoList<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        // Update scroll offset to ensure selected item is visible
        self.update_scroll();

        let (start, end) = self.visible_range();
        let display_mode = self.display_mode();

        let items: Vec<ListItem> = self.filtered_indices[start..end]
            .iter()
            .enumerate()
            .map(|(visible_idx, &repo_idx)| {
                let repo = &self.repositories[repo_idx];
                let absolute_idx = start + visible_idx;
                let is_selected = self.selected_index == Some(absolute_idx);

                let content = format_repo_item(
                    repo,
                    is_selected,
                    self.show_git_status,
                    display_mode,
                    self.area_width,
                    self.theme,
                    RepoItemParams {
                        is_favorite: self.favorites.contains(&repo_idx),
                        selection_mode: self.selection_mode,
                        is_checked: self.selected.contains(&repo_idx),
                    },
                );

                let mut style = Style::default().fg(self.theme.colors.foreground.into());
                if is_selected {
                    style = style
                        .bg(self.theme.colors.selected_bg.into())
                        .fg(Color::White);
                }

                ListItem::new(content).style(style)
            })
            .collect();

        let title = if self.selection_mode {
            format!(
                " Repositories [{} selected] ({}/{}) ",
                self.selected.len(),
                self.filtered_indices.len(),
                self.total_count
            )
        } else {
            format!(
                " Repositories ({}/{}) ",
                self.filtered_indices.len(),
                self.total_count
            )
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border.into()));

        let list = List::new(items)
            .block(block)
            .style(Style::default().fg(self.theme.colors.foreground.into()));

        Widget::render(list, area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use std::path::PathBuf;

    fn create_test_repos() -> Vec<Repository> {
        vec![
            Repository {
                name: "repo1".to_string(),
                path: PathBuf::from("/tmp/repo1"),
                last_modified: None,
                is_dirty: false,
                branch: Some("main".to_string()),
                is_git_repo: true,
                source: crate::repo::source::RepoSource::Standalone,
            },
            Repository {
                name: "repo2".to_string(),
                path: PathBuf::from("/tmp/repo2"),
                last_modified: None,
                is_dirty: true,
                branch: Some("feature".to_string()),
                is_git_repo: true,
                source: crate::repo::source::RepoSource::Standalone,
            },
        ]
    }

    #[test]
    fn test_repo_list_render() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();
        let repos = create_test_repos();
        let filtered: Vec<usize> = vec![0, 1];

        terminal
            .draw(|f| {
                let area = f.area();
                let list = RepoList::new(&repos, &filtered, &theme)
                    .selected_index(Some(0))
                    .visible_height(20);
                f.render_widget(list, area);
            })
            .unwrap();
    }

    #[test]
    fn test_format_repo_item() {
        use crate::ui::layout::DisplayMode;

        let repo = Repository {
            name: "test-repo".to_string(),
            path: PathBuf::from("/tmp/test-repo"),
            last_modified: None,
            is_dirty: true,
            branch: Some("main".to_string()),
            is_git_repo: true,
            source: crate::repo::source::RepoSource::Standalone,
        };

        let theme = Theme::dark();
        let formatted = format_repo_item(
            &repo,
            true,
            true,
            DisplayMode::Large,
            100,
            &theme,
            RepoItemParams {
                is_favorite: false,
                selection_mode: false,
                is_checked: false,
            },
        );

        // Check that the line has spans
        assert!(!formatted.spans.is_empty());
    }

    #[test]
    fn test_visible_range() {
        let theme = Theme::dark();
        let repos = create_test_repos();
        let filtered: Vec<usize> = vec![0, 1];

        let list = RepoList::new(&repos, &filtered, &theme)
            .scroll_offset(0)
            .visible_height(10);

        let (start, end) = list.visible_range();
        assert_eq!(start, 0);
        assert!(end <= 2);
    }

    #[test]
    fn test_format_repo_item_contains_name() {
        use crate::ui::layout::DisplayMode;
        use std::path::PathBuf;

        let repo = Repository {
            name: "test-repo-name".to_string(),
            path: PathBuf::from("/tmp/test-repo-name"),
            last_modified: None,
            is_dirty: true,
            branch: Some("main".to_string()),
            is_git_repo: true,
            source: crate::repo::source::RepoSource::Standalone,
        };

        let theme = Theme::dark();
        let formatted = format_repo_item(
            &repo,
            true,
            true,
            DisplayMode::Large,
            100,
            &theme,
            RepoItemParams {
                is_favorite: false,
                selection_mode: false,
                is_checked: false,
            },
        );

        // 验证 spans 包含仓库名称
        let content: String = formatted.spans.iter().map(|s| s.content.as_ref()).collect();

        assert!(
            content.contains("test-repo-name"),
            "Repository name should be displayed, got: {}",
            content
        );
    }

    #[test]
    fn test_display_mode_compact() {
        let theme = Theme::dark();
        let repos = create_test_repos();
        let filtered: Vec<usize> = vec![0, 1];

        let mut list = RepoList::new(&repos, &filtered, &theme);
        list.area_width = 50;
        assert_eq!(list.display_mode(), DisplayMode::Compact);
    }

    #[test]
    fn test_display_mode_medium() {
        let theme = Theme::dark();
        let repos = create_test_repos();
        let filtered: Vec<usize> = vec![0, 1];

        let mut list = RepoList::new(&repos, &filtered, &theme);
        list.area_width = 80;
        assert_eq!(list.display_mode(), DisplayMode::Medium);
    }

    #[test]
    fn test_display_mode_large() {
        let theme = Theme::dark();
        let repos = create_test_repos();
        let filtered: Vec<usize> = vec![0, 1];

        let mut list = RepoList::new(&repos, &filtered, &theme);
        list.area_width = 120;
        assert_eq!(list.display_mode(), DisplayMode::Large);
    }

    #[test]
    fn test_format_repo_item_compact_mode() {
        let repo = Repository {
            name: "test-repo".to_string(),
            path: PathBuf::from("/tmp/test-repo"),
            last_modified: None,
            is_dirty: true,
            branch: Some("main".to_string()),
            is_git_repo: true,
            source: crate::repo::source::RepoSource::Standalone,
        };

        let theme = Theme::dark();
        let formatted = format_repo_item(
            &repo,
            false,
            true,
            DisplayMode::Compact,
            50,
            &theme,
            RepoItemParams {
                is_favorite: false,
                selection_mode: false,
                is_checked: false,
            },
        );

        let content: String = formatted.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(
            content.contains("test-repo"),
            "Compact mode should show repo name"
        );
        assert!(
            !content.contains("main"),
            "Compact mode should NOT show branch"
        );
    }

    #[test]
    fn test_format_repo_item_medium_mode() {
        let repo = Repository {
            name: "test-repo".to_string(),
            path: PathBuf::from("/tmp/test-repo"),
            last_modified: None,
            is_dirty: true,
            branch: Some("main".to_string()),
            is_git_repo: true,
            source: crate::repo::source::RepoSource::Standalone,
        };

        let theme = Theme::dark();
        let formatted = format_repo_item(
            &repo,
            false,
            true,
            DisplayMode::Medium,
            80,
            &theme,
            RepoItemParams {
                is_favorite: false,
                selection_mode: false,
                is_checked: false,
            },
        );

        let content: String = formatted.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(
            content.contains("test-repo"),
            "Medium mode should show repo name"
        );
        assert!(content.contains("main"), "Medium mode should show branch");
        assert!(
            !content.contains("Modified"),
            "Medium mode should NOT show status"
        );
        assert!(
            !content.contains("Clean"),
            "Medium mode should NOT show status"
        );
    }

    #[test]
    fn test_format_repo_item_large_mode_with_status() {
        let repo = Repository {
            name: "test-repo".to_string(),
            path: PathBuf::from("/tmp/test-repo"),
            last_modified: None,
            is_dirty: true,
            branch: Some("main".to_string()),
            is_git_repo: true,
            source: crate::repo::source::RepoSource::Standalone,
        };

        let theme = Theme::dark();
        let formatted = format_repo_item(
            &repo,
            false,
            true,
            DisplayMode::Large,
            120,
            &theme,
            RepoItemParams {
                is_favorite: false,
                selection_mode: false,
                is_checked: false,
            },
        );

        let content: String = formatted.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(
            content.contains("test-repo"),
            "Large mode should show repo name"
        );
        assert!(content.contains("main"), "Large mode should show branch");
        assert!(
            content.contains("Modified"),
            "Large mode should show status for dirty repo"
        );
    }

    #[test]
    fn test_format_repo_item_non_git() {
        use crate::ui::layout::DisplayMode;

        let repo = Repository {
            name: "non-git-folder".to_string(),
            path: PathBuf::from("/tmp/non-git-folder"),
            last_modified: None,
            is_dirty: false,
            branch: None,
            is_git_repo: false,
            source: crate::repo::source::RepoSource::Standalone,
        };

        let theme = Theme::dark();
        let formatted = format_repo_item(
            &repo,
            false,
            true,
            DisplayMode::Medium,
            80,
            &theme,
            RepoItemParams {
                is_favorite: false,
                selection_mode: false,
                is_checked: false,
            },
        );

        let content: String = formatted.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(content.contains("non-git-folder"), "Should show repo name");
        assert!(
            content.contains("Not a git repo"),
            "Non-git repo should show 'Not a git repo' message, got: {}",
            content
        );
    }
}
