//! Search box widget
//!
//! Provides a text input field for search queries.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::ui::theme::Theme;

/// Search box widget
#[derive(Debug, Clone)]
pub struct SearchBox<'a> {
    /// Current search query
    query: &'a str,
    /// Theme reference
    theme: &'a Theme,
    /// Whether the search box is focused
    focused: bool,
    /// Placeholder text
    placeholder: &'a str,
}

impl<'a> SearchBox<'a> {
    /// Create a new search box
    pub fn new(query: &'a str, theme: &'a Theme, focused: bool) -> Self {
        Self {
            query,
            theme,
            focused,
            placeholder: "Type to search...",
        }
    }

    /// Set the placeholder text
    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }
}

impl<'a> Widget for SearchBox<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Determine border color based on focus
        let border_color = if self.focused {
            self.theme.colors.border_focused
        } else {
            self.theme.colors.border
        };

        // Create block with title
        let title = if self.focused {
            "🔍 Search (active)"
        } else {
            "🔍 Search"
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color.into()));

        // Determine text to display
        let display_text = if self.query.is_empty() {
            self.placeholder
        } else {
            self.query
        };

        // Determine text style
        let text_style = if self.query.is_empty() {
            Style::default().fg(self.theme.colors.text_muted.into())
        } else {
            Style::default().fg(self.theme.colors.foreground.into())
        };

        // Add cursor indicator if focused and has query
        let final_text = if self.focused && !self.query.is_empty() {
            format!("{}▌", display_text)
        } else {
            display_text.to_string()
        };

        let paragraph = Paragraph::new(final_text).block(block).style(text_style);

        Widget::render(paragraph, area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;

    #[test]
    fn test_search_box_empty() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();

        terminal
            .draw(|f| {
                let area = f.area();
                let search = SearchBox::new("", &theme, false);
                f.render_widget(search, area);
            })
            .unwrap();
    }

    #[test]
    fn test_search_box_with_query() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();

        terminal
            .draw(|f| {
                let area = f.area();
                let search = SearchBox::new("test query", &theme, true);
                f.render_widget(search, area);
            })
            .unwrap();
    }

    #[test]
    fn test_search_box_placeholder() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();

        terminal
            .draw(|f| {
                let area = f.area();
                let search = SearchBox::new("", &theme, false).placeholder("Custom placeholder...");
                f.render_widget(search, area);
            })
            .unwrap();
    }
}
