//! Title bar widget
//!
//! Displays the application title and current view mode at the top of the main UI.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders};

use crate::app::state::ViewMode;
use crate::ui::theme::Theme;
use crate::update::UpdateStatus;

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
    /// Update status
    update_status: &'a UpdateStatus,
    /// Update notification dismissed
    update_dismissed: bool,
}

impl<'a> TitleBar<'a> {
    /// Create a new title bar
    pub fn new(view_mode: &'a ViewMode, theme: &'a Theme, update_status: &'a UpdateStatus) -> Self {
        Self {
            view_mode,
            theme,
            selection_mode: false,
            selected_count: 0,
            update_status,
            update_dismissed: false,
        }
    }

    /// Set selection mode info
    pub fn selection_info(mut self, selected_count: usize) -> Self {
        self.selection_mode = true;
        self.selected_count = selected_count;
        self
    }

    /// Set update dismissed state
    pub fn update_dismissed(mut self, dismissed: bool) -> Self {
        self.update_dismissed = dismissed;
        self
    }

    /// Get the title text based on view mode
    fn get_title_text(&self) -> String {
        if self.selection_mode {
            format!("repo-tui — 多选模式 (已选 {} 个)", self.selected_count)
        } else {
            let view_text = match self.view_mode {
                ViewMode::All => "全部视图",
                ViewMode::Favorites => "收藏夹",
                ViewMode::Recent => "最近视图",
            };
            format!("repo-tui — {}", view_text)
        }
    }

    /// Get update indicator text
    fn get_update_text(&self) -> Option<String> {
        if self.update_dismissed {
            return None;
        }
        match self.update_status {
            UpdateStatus::UpdateAvailable { version } => Some(format!("⬆ {}", version)),
            _ => None,
        }
    }
}

impl<'a> Widget for TitleBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Get title text
        let title_text = self.get_title_text();
        let update_text = self.get_update_text();

        // Create block with styled border
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.focused_border_style())
            .style(Style::default().bg(self.theme.colors.title_bg.into()));

        // If there's an update available, add it to the right side
        if let Some(update_text) = update_text {
            // Create title with left and right parts
            let left_title = Line::from(title_text).left_aligned();
            let right_title = Line::from(Span::styled(
                update_text,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ))
            .right_aligned();
            let block = block.title_top(left_title).title_top(right_title);
            Widget::render(block, area, buf);
        } else {
            let title = Line::from(title_text).left_aligned();
            let block = block.title_top(title);
            Widget::render(block, area, buf);
        }
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
        let update_status = UpdateStatus::NeverChecked;

        terminal
            .draw(|f| {
                let area = f.area();
                let title = TitleBar::new(&view_mode, &theme, &update_status);
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
        let update_status = UpdateStatus::NeverChecked;

        terminal
            .draw(|f| {
                let area = f.area();
                let title = TitleBar::new(&view_mode, &theme, &update_status);
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
        let update_status = UpdateStatus::NeverChecked;

        terminal
            .draw(|f| {
                let area = f.area();
                let title = TitleBar::new(&view_mode, &theme, &update_status);
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
        let update_status = UpdateStatus::NeverChecked;

        terminal
            .draw(|f| {
                let area = f.area();
                let title = TitleBar::new(&view_mode, &theme, &update_status).selection_info(3);
                f.render_widget(title, area);
            })
            .unwrap();
    }

    #[test]
    fn test_title_bar_update_available() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();
        let view_mode = ViewMode::All;
        let update_status = UpdateStatus::UpdateAvailable {
            version: "v0.2.0".to_string(),
        };

        terminal
            .draw(|f| {
                let area = f.area();
                let title = TitleBar::new(&view_mode, &theme, &update_status);
                f.render_widget(title, area);
            })
            .unwrap();
    }
}
