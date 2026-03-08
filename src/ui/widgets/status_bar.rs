//! Unified status bar widget

use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use std::path::Path;

use crate::ui::theme::Theme;

/// Unified status bar widget combining status message and path display
pub struct StatusBar<'a> {
    pub status_message: &'a str,
    pub path: Option<&'a Path>,
    pub repo_count: Option<usize>,
    pub theme: &'a Theme,
    pub loading: bool,
    pub error: bool,
}

impl<'a> StatusBar<'a> {
    /// Create a new status bar
    pub fn new(status_message: &'a str, theme: &'a Theme) -> Self {
        Self {
            status_message,
            path: None,
            repo_count: None,
            theme,
            loading: false,
            error: false,
        }
    }

    /// Set the path to display
    pub fn path(mut self, path: &'a Path) -> Self {
        self.path = Some(path);
        self
    }

    /// Set repository count
    pub fn repo_count(mut self, count: usize) -> Self {
        self.repo_count = Some(count);
        self
    }

    /// Set loading state
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// Set error state
    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }
}

impl<'a> Widget for StatusBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Check minimum height (needs at least 2 rows: status + path)
        if area.height < 2 {
            return;
        }

        // Status message with highlighted key hints
        let status_spans = if self.loading {
            vec![
                Span::styled("⏳ ", Style::default().fg(self.theme.colors.primary.into())),
                Span::raw(self.status_message),
            ]
        } else if self.error {
            vec![
                Span::styled("⚠️ ", Style::default().fg(self.theme.colors.error.into())),
                Span::raw(self.status_message),
            ]
        } else {
            parse_status_message(self.status_message, self.theme)
        };

        // Render status on top row
        let status_paragraph = Paragraph::new(Line::from(status_spans))
            .style(Style::default().fg(self.theme.colors.text_muted.into()))
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left);
        status_paragraph.render(area, buf);

        // Path on bottom row
        if let Some(path) = self.path {
            let path_text = format!(
                "📂 {}{}",
                path.display(),
                self.repo_count
                    .map(|c| format!(" ({} repos)", c))
                    .unwrap_or_default()
            );

            let path_paragraph = Paragraph::new(path_text)
                .style(Style::default().fg(self.theme.colors.text_muted.into()))
                .alignment(Alignment::Left);

            let path_area = Rect::new(area.x, area.y + area.height - 1, area.width, 1);
            path_paragraph.render(path_area, buf);
        }
    }
}

/// Parse status message and apply theme color highlight to key hints
/// Format: "↑↓ navigate   g/G jump   ENTER open"
fn parse_status_message<'a>(message: &'a str, theme: &'a Theme) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    let segments: Vec<&'a str> = message.split("   ").collect();

    for (i, segment) in segments.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("   "));
        }

        if let Some(space_pos) = segment.find(' ') {
            let keys: String = segment[..space_pos].to_string();
            let desc: String = segment[space_pos..].to_string();

            spans.push(Span::styled(
                keys,
                Style::default().fg(theme.colors.primary.into()),
            ));
            spans.push(Span::raw(desc));
        } else {
            spans.push(Span::raw(segment.to_string()));
        }
    }

    spans
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use std::path::Path;

    #[test]
    fn test_status_bar_render() {
        let backend = TestBackend::new(80, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();

        terminal
            .draw(|f| {
                let status_bar = StatusBar::new("Test", &theme)
                    .path(Path::new("/tmp"))
                    .repo_count(5);
                f.render_widget(status_bar, f.area());
            })
            .unwrap();
    }

    #[test]
    fn test_status_bar_loading() {
        let backend = TestBackend::new(80, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();

        terminal
            .draw(|f| {
                let status_bar = StatusBar::new("Loading...", &theme)
                    .loading(true)
                    .path(Path::new("/tmp"));
                f.render_widget(status_bar, f.area());
            })
            .unwrap();
    }

    #[test]
    fn test_status_bar_error() {
        let backend = TestBackend::new(80, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();

        terminal
            .draw(|f| {
                let status_bar = StatusBar::new("Error occurred", &theme)
                    .error(true)
                    .path(Path::new("/tmp"));
                f.render_widget(status_bar, f.area());
            })
            .unwrap();
    }

    #[test]
    fn test_status_bar_wrap() {
        let backend = TestBackend::new(40, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();

        terminal
            .draw(|f| {
                let long_message = "[↑/↓]Nav [g/G]Jump [/]Search [Enter]Open [r]Refresh [?]Help [Ctrl+C]Quit [m]ChangeDir";
                let status_bar = StatusBar::new(long_message, &theme)
                    .path(Path::new("/very/long/path/that/might/wrap"));
                f.render_widget(status_bar, f.area());
            })
            .unwrap();
    }
}
