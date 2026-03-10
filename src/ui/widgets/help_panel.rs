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
pub struct HelpPanel {
    pub scroll_offset: usize,
}

impl HelpPanel {
    /// Create a new help panel
    pub fn new() -> Self {
        Self { scroll_offset: 0 }
    }

    /// Render the help panel with scroll support
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Clear the area behind the popup
        frame.render_widget(Clear, area);

        // Create help content
        let help_text = vec![
            Line::from(Span::styled(
                "Navigation",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("  ↓/↑         - Move down/up (cyclic)"),
            Line::from("  Home/End    - Go to first/last"),
            Line::from(""),
            Line::from(Span::styled(
                "Search",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("  /           - Focus search"),
            Line::from("  Esc         - Exit search / Close panel"),
            Line::from("  [char]      - Add to query"),
            Line::from("  Backspace   - Delete"),
            Line::from(""),
            Line::from(Span::styled(
                "Actions",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("  1           - Claude Code"),
            Line::from("  2           - WebStorm"),
            Line::from("  3           - VS Code"),
            Line::from("  4           - Finder/Explorer"),
            Line::from("  5           - IntelliJ IDEA"),
            Line::from("  6           - OpenCode"),
            Line::from("  7           - LazyGit"),
            Line::from(""),
            Line::from(Span::styled(
                "Selection",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("  v           - Toggle selection mode"),
            Line::from("  SPACE       - Toggle selection"),
            Line::from("  Ctrl+A      - Select all"),
            Line::from(""),
            Line::from(Span::styled(
                "Favorites",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("  f           - Toggle favorite"),
            Line::from("  Ctrl+F      - Toggle favorites view"),
            Line::from(""),
            Line::from(Span::styled(
                "View Modes",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("  Ctrl+F      - Toggle favorites view"),
            Line::from("  Ctrl+R      - Toggle recent view"),
            Line::from(""),
            Line::from(Span::styled(
                "Global",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("  a           - Add single repository"),
            Line::from("  c           - Clone repository"),
            Line::from("  m           - Change main directory"),
            Line::from("  r           - Refresh list"),
            Line::from("  t           - Theme selector"),
            Line::from("  U           - Check for updates"),
            Line::from("  ?           - Show this help"),
            Line::from("  Ctrl+C      - Force quit"),
            Line::from(""),
        ];

        let total_lines = help_text.len();
        let visible_height = area.height.saturating_sub(2) as usize;
        let max_scroll = total_lines.saturating_sub(visible_height);

        let scroll_offset = self.scroll_offset.min(max_scroll);
        let visible_end = (scroll_offset + visible_height).min(total_lines);

        let visible_text: Vec<Line> = help_text[scroll_offset..visible_end].to_vec();

        let mut final_text = visible_text;

        if scroll_offset > 0 {
            final_text.insert(
                0,
                Line::from(Span::styled("▲", Style::default().fg(Color::Gray))),
            );
        }
        if visible_end < total_lines {
            final_text.push(Line::from(Span::styled(
                "▼",
                Style::default().fg(Color::Gray),
            )));
        }

        let paragraph = Paragraph::new(final_text)
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
