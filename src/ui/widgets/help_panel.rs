//! Help panel widget
//!
//! Displays keyboard shortcuts and help information.

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Help panel widget
pub struct HelpPanel;

impl HelpPanel {
    /// Create a new help panel
    pub fn new() -> Self {
        Self
    }

    /// Render the help panel
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Clear the area behind the popup
        frame.render_widget(Clear, area);

        // Create help content
        let help_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Navigation",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            self.key_binding("j/↓", "Move down"),
            self.key_binding("k/↑", "Move up"),
            self.key_binding("g", "Go to top"),
            self.key_binding("G", "Go to bottom"),
            self.key_binding("Ctrl+d", "Scroll down half-page"),
            self.key_binding("Ctrl+u", "Scroll up half-page"),
            self.key_binding("Home/End", "Go to first/last"),
            Line::from(""),
            Line::from(Span::styled(
                "Search",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            self.key_binding("/", "Focus search"),
            self.key_binding("Esc", "Clear search / Close panel"),
            self.key_binding("[char]", "Add to search query"),
            self.key_binding("Backspace", "Delete character"),
            Line::from(""),
            Line::from(Span::styled(
                "Actions",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            self.key_binding("Enter/o", "Open action menu"),
            self.key_binding("c/1", "cd + cloud (claude)"),
            self.key_binding("w/2", "Open in WebStorm"),
            self.key_binding("v/3", "Open in VS Code"),
            self.key_binding("f/4", "Open in Finder/Explorer"),
            Line::from(""),
            Line::from(Span::styled(
                "Global",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            self.key_binding("m", "Change main directory"),
            self.key_binding("Shift+F", "Toggle favorite"),
            self.key_binding("f", "Open in Finder/Explorer"),
            self.key_binding("r", "Refresh list"),
            self.key_binding("Ctrl+r", "Show recent repositories"),
            self.key_binding("t", "Open theme selector"),
            self.key_binding("?", "Show this help"),
            self.key_binding("q", "Quit"),
            self.key_binding("Ctrl+c", "Force quit"),
            Line::from(""),
        ];

        let paragraph = Paragraph::new(help_text)
            .block(
                Block::default()
                    .title(" Keyboard Shortcuts ")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .style(Style::default().bg(Color::Black)),
            )
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, area);
    }

    /// Create a key binding line
    fn key_binding<'a>(&self, key: &'a str, desc: &'a str) -> Line<'a> {
        Line::from(vec![
            Span::styled(format!(" {:<12} ", key), Style::default().fg(Color::Yellow)),
            Span::raw(desc),
        ])
    }
}

impl Default for HelpPanel {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate centered popup rectangle
pub fn centered_help_popup(area: Rect) -> Rect {
    // Fixed size for help panel
    let width = 60.min(area.width.saturating_sub(4));
    let height = 28.min(area.height.saturating_sub(4));

    let x = (area.width - width) / 2;
    let y = (area.height - height) / 2;

    Rect::new(x, y, width, height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_panel_creation() {
        let panel = HelpPanel::new();
        // Basic creation test
        let _ = panel;
    }

    #[test]
    fn test_centered_help_popup() {
        let area = Rect::new(0, 0, 100, 50);
        let popup = centered_help_popup(area);

        assert!(popup.width <= 60);
        assert!(popup.height <= 28);
        assert_eq!(popup.x, (100 - popup.width) / 2);
        assert_eq!(popup.y, (50 - popup.height) / 2);
    }

    #[test]
    fn test_centered_help_popup_small_area() {
        let area = Rect::new(0, 0, 40, 20);
        let popup = centered_help_popup(area);

        assert_eq!(popup.width, 36); // 40 - 4
        assert_eq!(popup.height, 16); // 20 - 4
    }
}
