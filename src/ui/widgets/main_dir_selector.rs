//! Main directory selector widget
//!
//! Displays a list of main directories for selection during repository move operations.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use std::path::PathBuf;

use crate::ui::theme::Theme;

/// Main directory selector widget
pub struct MainDirSelector<'a> {
    /// Main directories list: (index, path_string, repo_count)
    pub main_dirs: &'a [(usize, String, usize)],
    /// Selected index
    pub selected_index: usize,
    /// Theme
    pub theme: &'a Theme,
    /// Repository name being moved (optional, for confirmation display)
    pub repo_name: Option<&'a str>,
    /// Conflict exists flag
    pub conflict_exists: bool,
    /// Target path (for confirmation display)
    pub target_path: Option<&'a PathBuf>,
}

impl<'a> MainDirSelector<'a> {
    /// Create new main directory selector
    pub fn new(
        main_dirs: &'a [(usize, String, usize)],
        selected_index: usize,
        theme: &'a Theme,
    ) -> Self {
        Self {
            main_dirs,
            selected_index,
            theme,
            repo_name: None,
            conflict_exists: false,
            target_path: None,
        }
    }

    /// Set repository name for confirmation display
    pub fn repo_name(mut self, repo_name: Option<&'a str>) -> Self {
        self.repo_name = repo_name;
        self
    }

    /// Set conflict flag
    pub fn conflict_exists(mut self, conflict: bool) -> Self {
        self.conflict_exists = conflict;
        self
    }

    /// Set target path
    pub fn target_path(mut self, target_path: Option<&'a PathBuf>) -> Self {
        self.target_path = target_path;
        self
    }
}

impl Widget for MainDirSelector<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let main_dir_count = self.main_dirs.len();

        // Create main block with title
        let title = format!("Select Target Directory ({main_dir_count} available)");
        let block = Block::default()
            .title(title.as_str())
            .borders(Borders::ALL)
            .border_style(self.theme.focused_border_style());

        let inner = block.inner(area);

        // Render block first
        block.render(area, buf);

        // Check if we have content to render
        if inner.height < 3 || inner.width < 10 {
            return;
        }

        // Build directory list items
        let items: Vec<ListItem> = self
            .main_dirs
            .iter()
            .enumerate()
            .map(|(idx, (_index, path, repo_count))| {
                let is_selected = idx == self.selected_index;

                // Format: ▌ path/to/dir (N repos)
                let display_text = if is_selected {
                    format!("▌ {} ({} repos)", path, repo_count)
                } else {
                    format!("  {} ({} repos)", path, repo_count)
                };

                let style = if is_selected {
                    self.theme.selected_style()
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(display_text).style(style)
            })
            .collect();

        // Create list widget
        let list = List::new(items);

        // Render list
        ratatui::prelude::Widget::render(list, inner, buf);

        // Render confirmation info if available
        let mut lines_to_reserve = 1; // Help text
        if let Some(_repo_name) = self.repo_name {
            lines_to_reserve += 2; // Repo name and target path
            if self.conflict_exists {
                lines_to_reserve += 2; // Conflict warning and empty line
            }
        }

        let available_height = inner.height.saturating_sub(lines_to_reserve);

        // Render confirmation info
        if let Some(repo_name) = self.repo_name {
            if let Some(target_path) = self.target_path {
                let confirm_y = inner.y + available_height;

                // Repo name
                let repo_line = format!("仓库：{}", repo_name);
                let repo_para = Paragraph::new(repo_line)
                    .style(self.theme.primary_text_style())
                    .alignment(Alignment::Center);
                let repo_area = Rect::new(inner.x, confirm_y, inner.width, 1);
                repo_para.render(repo_area, buf);

                // Target path
                let target_y = confirm_y + 1;
                let target_line = format!("目标：{}", target_path.display());
                let target_para = Paragraph::new(target_line)
                    .style(self.theme.primary_text_style())
                    .alignment(Alignment::Center);
                let target_area = Rect::new(inner.x, target_y, inner.width, 1);
                target_para.render(target_area, buf);

                // Conflict warning
                if self.conflict_exists {
                    let warning_y = target_y + 1;
                    let warning_line = "⚠️  目标目录已存在同名仓库！";
                    let warning_para = Paragraph::new(warning_line)
                        .style(Style::default().fg(Color::Yellow))
                        .alignment(Alignment::Center);
                    let warning_area = Rect::new(inner.x, warning_y, inner.width, 1);
                    warning_para.render(warning_area, buf);
                }
            }
        }

        // Render help text at bottom
        let help_text = "↑↓ navigate   Enter confirm   Esc cancel";
        let help = Paragraph::new(help_text)
            .style(self.theme.primary_text_style())
            .alignment(Alignment::Center);

        // Calculate help position (bottom of inner area)
        let help_y = inner.y + inner.height.saturating_sub(1);
        let help_area = Rect::new(inner.x, help_y, inner.width, 1);
        help.render(help_area, buf);
    }
}

/// Calculate centered rectangle for main directory selector
pub fn centered_main_dir_selector_rect(area: Rect, _min_width: u16, _min_height: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::Theme;

    #[test]
    fn test_main_dir_selector_creation() {
        let theme = Theme::dark();
        let main_dirs = vec![
            (0, "/Users/test/Dev".to_string(), 5),
            (1, "/Users/test/Projects".to_string(), 3),
        ];
        let selector = MainDirSelector::new(&main_dirs, 0, &theme);

        assert_eq!(selector.main_dirs.len(), 2);
        assert_eq!(selector.selected_index, 0);
    }

    #[test]
    fn test_centered_rect_calculation() {
        let area = Rect::new(0, 0, 80, 24);
        let rect = centered_main_dir_selector_rect(area, 60, 15);

        assert!(rect.width >= 60);
        assert!(rect.height >= 15);
        assert!(rect.x > 0);
        assert!(rect.y > 0);
    }
}
