//! Unified status bar widget

use ratatui::prelude::*;
use ratatui::widgets::{Padding, Paragraph, Wrap};
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
    pub padding: Padding,
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
            padding: Padding::new(1, 1, 0, 0), // left=1, right=1, top=0, bottom=0
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
        // Fill background (no border)
        buf.set_style(area, Style::default().bg(Color::DarkGray));

        // Apply padding manually
        let inner = Rect::new(
            area.x + self.padding.left,
            area.y + self.padding.top,
            area.width
                .saturating_sub(self.padding.left + self.padding.right),
            area.height
                .saturating_sub(self.padding.top + self.padding.bottom),
        );

        // Check minimum inner height (needs at least 2 rows: status + path)
        let inner_height = inner.height;
        if inner_height < 2 {
            return;
        }

        // Status message
        let status_text = if self.loading {
            format!("⏳ {}", self.status_message)
        } else if self.error {
            format!("⚠️ {}", self.status_message)
        } else {
            self.status_message.to_string()
        };

        // Render status on top row
        let status_paragraph = Paragraph::new(status_text)
            .style(Style::default().fg(self.theme.text_secondary))
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left);
        status_paragraph.render(inner, buf);

        // Path on bottom row (only if inner height >= 2)
        if inner_height >= 2 {
            if let Some(path) = self.path {
                let path_text = format!(
                    "📂 {}{}",
                    path.display(),
                    self.repo_count
                        .map(|c| format!(" ({} repos)", c))
                        .unwrap_or_default()
                );

                let path_paragraph = Paragraph::new(path_text)
                    .style(Style::default().fg(self.theme.text_secondary))
                    .alignment(Alignment::Left);

                let path_area = Rect::new(inner.x, inner.y + inner_height - 1, inner.width, 1);
                path_paragraph.render(path_area, buf);
            }
        }
    }
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
                let long_message = "[j/k]Nav [g/G]Jump [/]Search [Enter]Open [r]Refresh [?]Help [q]Quit [m]ChangeDir";
                let status_bar = StatusBar::new(long_message, &theme)
                    .path(Path::new("/very/long/path/that/might/wrap"));
                f.render_widget(status_bar, f.area());
            })
            .unwrap();
    }
}
