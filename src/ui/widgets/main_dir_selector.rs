//! Main directory selector widget
//!
//! Displays a list of main directories for selection during repository move operations.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use crate::ui::theme::Theme;

/// Main directory selector widget
pub struct MainDirSelector<'a> {
    /// Main directories list: (index, path_string, repo_count)
    pub main_dirs: &'a [(usize, String, usize)],
    /// Selected index
    pub selected_index: usize,
    /// Theme
    pub theme: &'a Theme,
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
        }
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

        // Render help text at bottom
        let help_text = "↑↓ navigate   Enter confirm   Esc cancel";
        let help = Paragraph::new(help_text)
            .style(self.theme.secondary_text_style())
            .alignment(Alignment::Center);

        // Calculate help position (bottom of inner area)
        let help_area = Rect::new(
            inner.x,
            inner.y + inner.height.saturating_sub(1),
            inner.width,
            1,
        );
        help.render(help_area, buf);
    }
}

/// Calculate centered rectangle for main directory selector
pub fn centered_main_dir_selector_rect(area: Rect, min_width: u16, min_height: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Min(min_height),
            Constraint::Percentage(50),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Min(min_width),
            Constraint::Percentage(50),
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
