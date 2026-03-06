//! Repository list widget
//!
//! Displays a scrollable list of Git repositories with virtual list optimization.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem};

use crate::repo::Repository;
use crate::ui::theme::Theme;

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
    /// Theme
    pub theme: &'a Theme,
    /// Show git status
    pub show_git_status: bool,
    /// Show branch name
    pub show_branch: bool,
    /// Total count (for display)
    pub total_count: usize,
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
            theme,
            show_git_status: true,
            show_branch: true,
            total_count: repositories.len(),
        }
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

    /// Calculate visible range
    fn visible_range(&self) -> (usize, usize) {
        let start = self.scroll_offset;
        let visible_count = self.visible_count();
        let end = (start + visible_count).min(self.filtered_indices.len());
        (start, end)
    }

    /// Calculate how many items can be visible
    fn visible_count(&self) -> usize {
        // Reserve space for borders
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
            self.scroll_offset = selected.saturating_sub(visible_count - 1);
        }
        // Scroll up if selected is above visible area
        else if selected < self.scroll_offset {
            self.scroll_offset = selected;
        }
    }
}

impl<'a> Widget for RepoList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (start, end) = self.visible_range();

        let items: Vec<ListItem> = self.filtered_indices[start..end]
            .iter()
            .enumerate()
            .map(|(visible_idx, &repo_idx)| {
                let repo = &self.repositories[repo_idx];
                let absolute_idx = start + visible_idx;
                let is_selected = self.selected_index == Some(absolute_idx);

                let content =
                    format_repo_item(repo, is_selected, self.show_git_status, self.show_branch);

                let mut style = Style::default().fg(self.theme.text_primary);
                if is_selected {
                    style = style.bg(self.theme.selected_bg).fg(Color::White);
                }

                ListItem::new(content).style(style)
            })
            .collect();

        let title = format!(
            " Repositories ({}/{}) ",
            self.filtered_indices.len(),
            self.total_count
        );

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border_normal));

        let list = List::new(items)
            .block(block)
            .style(Style::default().fg(self.theme.text_primary));

        Widget::render(list, area, buf);
    }
}

/// Format a repository item for display
fn format_repo_item(
    repo: &Repository,
    is_selected: bool,
    show_git_status: bool,
    show_branch: bool,
) -> String {
    let prefix = if is_selected { "▌ " } else { "  " };

    let status_icon = if show_git_status {
        if repo.is_dirty {
            "● "
        } else {
            "✓ "
        }
    } else {
        ""
    };

    let branch_info = if show_branch {
        repo.branch
            .as_ref()
            .map(|b| format!("({})", b))
            .unwrap_or_default()
    } else {
        String::new()
    };

    if branch_info.is_empty() {
        format!("{}{}{}", prefix, status_icon, repo.name)
    } else {
        format!("{}{}{} {}", prefix, status_icon, repo.name, branch_info)
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
            },
            Repository {
                name: "repo2".to_string(),
                path: PathBuf::from("/tmp/repo2"),
                last_modified: None,
                is_dirty: true,
                branch: Some("feature".to_string()),
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
        let repo = Repository {
            name: "test-repo".to_string(),
            path: PathBuf::from("/tmp/test-repo"),
            last_modified: None,
            is_dirty: true,
            branch: Some("main".to_string()),
        };

        let formatted = format_repo_item(&repo, true, true, true);
        assert!(formatted.contains("test-repo"));
        assert!(formatted.contains("main"));
        assert!(formatted.contains("●"));
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
}
