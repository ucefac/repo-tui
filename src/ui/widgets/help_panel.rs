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

        // Build two-column layout
        let left_column = vec![
            Line::from(Span::styled(
                "Navigation",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("  ↓/↑         - Move down/up (cyclic)"),
            Line::from("  Home/End    - Go to first/last"),
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
                "View Modes",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("  Ctrl+F      - Toggle favorites view"),
            Line::from("  Ctrl+R      - Toggle recent view"),
            Line::from(""),
        ];

        let right_column = vec![
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
                "Favorites",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("  f           - Toggle favorite"),
            Line::from("  Ctrl+F      - Toggle favorites view"),
            Line::from(""),
            Line::from(Span::styled(
                "Global",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("  a           - Add repository"),
            Line::from("  c           - Clone repository"),
            Line::from("  m           - Change main directory"),
            Line::from("  t           - Theme selector"),
            Line::from("  U           - Check for updates"),
            Line::from("  ?           - Show this help"),
            Line::from("  Ctrl+C      - Force quit"),
            Line::from(""),
        ];

        // Calculate scroll bounds based on the longer column
        let total_lines = left_column.len().max(right_column.len());
        let visible_height = area.height.saturating_sub(2) as usize;
        let max_scroll = total_lines.saturating_sub(visible_height);

        let scroll_offset = self.scroll_offset.min(max_scroll);

        // Apply scroll offset to both columns
        let visible_end = (scroll_offset + visible_height).min(total_lines);
        let mut left_visible: Vec<Line> = left_column[scroll_offset..visible_end].to_vec();
        let mut right_visible: Vec<Line> = right_column[scroll_offset..visible_end].to_vec();

        // Add scroll indicators
        if scroll_offset > 0 {
            let indicator = Line::from(Span::styled("▲", Style::default().fg(Color::Gray)));
            left_visible.insert(0, indicator.clone());
            right_visible.insert(0, indicator);
        }
        if visible_end < total_lines {
            let indicator = Line::from(Span::styled("▼", Style::default().fg(Color::Gray)));
            left_visible.push(indicator.clone());
            right_visible.push(indicator);
        }

        // Calculate column widths
        let total_width = area.width.saturating_sub(4) as usize; // Account for borders
        let left_width = total_width / 2;

        // Pad lines to ensure equal height for proper alignment
        let max_lines = left_visible.len().max(right_visible.len());
        while left_visible.len() < max_lines {
            left_visible.push(Line::from(""));
        }
        while right_visible.len() < max_lines {
            right_visible.push(Line::from(""));
        }

        // Combine columns side by side
        let combined_lines: Vec<Line> = left_visible
            .into_iter()
            .zip(right_visible.into_iter())
            .map(|(left, right)| {
                let spacer = " ".repeat(left_width.saturating_sub(left.width() as usize) + 2);
                Line::from(vec![
                    Span::raw(left.to_string()),
                    Span::raw(spacer),
                    Span::raw(right.to_string()),
                ])
            })
            .collect();

        let paragraph = Paragraph::new(combined_lines)
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
    // Fixed size for help panel - max 80% of terminal width
    let width = ((area.width as f32 * 0.8) as u16).min(area.width.saturating_sub(4));
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

        // Width should be 80% of terminal width (80) since it's less than (width - 4)
        assert_eq!(popup.width, 80);
        assert!(popup.height <= 28);
        assert_eq!(popup.x, (100 - popup.width) / 2);
        assert_eq!(popup.y, (50 - popup.height) / 2);
    }

    #[test]
    fn test_centered_help_popup_small_area() {
        let area = Rect::new(0, 0, 40, 20);
        let popup = centered_help_popup(area);

        // Width should be 80% of 40 = 32 (since 32 < 36 = 40-4)
        assert_eq!(popup.width, 32); // 80% of 40
        assert_eq!(popup.height, 16); // 20 - 4
    }
}
