//! Title bar widget
//!
//! Displays the application title and current view mode at the top of the main UI.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders};

use crate::app::state::ViewMode;
use crate::ui::theme::Theme;

/// Title bar widget
#[derive(Debug, Clone)]
pub struct TitleBar<'a> {
    /// Current view mode
    view_mode: &'a ViewMode,
    /// Theme reference
    theme: &'a Theme,
    /// Selection mode flag (for batch operations)
    selection_mode: bool,
    /// Number of selected repositories
    selected_count: usize,
}

impl<'a> TitleBar<'a> {
    /// Create a new title bar
    pub fn new(view_mode: &'a ViewMode, theme: &'a Theme) -> Self {
        Self {
            view_mode,
            theme,
            selection_mode: false,
            selected_count: 0,
        }
    }

    /// Set selection mode info
    pub fn selection_info(mut self, selected_count: usize) -> Self {
        self.selection_mode = true;
        self.selected_count = selected_count;
        self
    }

    /// Get the title text based on view mode
    fn get_title_text(&self) -> String {
        if self.selection_mode {
            format!("repotui — 多选模式 (已选 {} 个)", self.selected_count)
        } else {
            let view_text = match self.view_mode {
                ViewMode::All => "全部视图",
                ViewMode::Favorites => "收藏夹",
                ViewMode::Recent => "最近视图",
            };
            format!("repotui — {}", view_text)
        }
    }
}

impl<'a> Widget for TitleBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Get title text
        let title_text = self.get_title_text();

        // Create block with styled border and title
        let block = Block::default()
            .title(title_text)
            .borders(Borders::ALL)
            .border_style(self.theme.focused_border_style())
            .title_style(self.theme.title_style())
            .style(Style::default().bg(self.theme.colors.title_bg.into()));

        Widget::render(block, area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;

    #[test]
    fn test_title_bar_all_view() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();
        let view_mode = ViewMode::All;

        terminal
            .draw(|f| {
                let area = f.area();
                let title = TitleBar::new(&view_mode, &theme);
                f.render_widget(title, area);
            })
            .unwrap();
    }

    #[test]
    fn test_title_bar_favorites_view() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();
        let view_mode = ViewMode::Favorites;

        terminal
            .draw(|f| {
                let area = f.area();
                let title = TitleBar::new(&view_mode, &theme);
                f.render_widget(title, area);
            })
            .unwrap();
    }

    #[test]
    fn test_title_bar_recent_view() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();
        let view_mode = ViewMode::Recent;

        terminal
            .draw(|f| {
                let area = f.area();
                let title = TitleBar::new(&view_mode, &theme);
                f.render_widget(title, area);
            })
            .unwrap();
    }

    #[test]
    fn test_title_bar_selection_mode() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();
        let view_mode = ViewMode::All;

        terminal
            .draw(|f| {
                let area = f.area();
                let title = TitleBar::new(&view_mode, &theme).selection_info(3);
                f.render_widget(title, area);
            })
            .unwrap();
    }
}
