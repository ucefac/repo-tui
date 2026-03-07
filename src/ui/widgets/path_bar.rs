//! Path bar widget
//!
//! Displays the main directory path in the status bar area.
//! Supports clicking to copy path to clipboard.

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use std::path::Path;

use crate::ui::theme::Theme;

/// Path bar widget state
#[derive(Debug, Clone)]
pub struct PathBar<'a> {
    /// Main directory path
    pub path: &'a Path,
    /// Repository count (optional)
    pub repo_count: Option<usize>,
    /// Theme
    pub theme: &'a Theme,
    /// Truncate path if too long
    pub truncate: bool,
    /// Max display length (0 = no limit)
    pub max_length: usize,
}

impl<'a> PathBar<'a> {
    /// Create a new path bar
    pub fn new(path: &'a Path, repo_count: Option<usize>, theme: &'a Theme) -> Self {
        Self {
            path,
            repo_count,
            theme,
            truncate: true,
            max_length: 0,
        }
    }

    /// Set max display length
    pub fn max_length(mut self, length: usize) -> Self {
        self.max_length = length;
        self
    }

    /// Enable/disable truncation
    pub fn truncate(mut self, truncate: bool) -> Self {
        self.truncate = truncate;
        self
    }

    /// Get the display text
    pub fn display_text(&self, available_width: usize) -> String {
        let mut text = format!(" 📂 {}", self.path.display());

        if let Some(count) = self.repo_count {
            text.push_str(&format!(" ({} repos)", count));
        }

        // Truncate if needed
        if self.truncate && self.max_length > 0 && text.len() > self.max_length {
            text = truncate_path(&text, self.max_length);
        } else if self.truncate && available_width > 0 && text.len() > available_width {
            text = truncate_path(&text, available_width);
        }

        text
    }
}

impl<'a> Widget for PathBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = self.display_text(area.width as usize);

        let paragraph = Paragraph::new(text).style(
            Style::default()
                .fg(self.theme.text_secondary)
                .bg(Color::DarkGray),
        );

        paragraph.render(area, buf);
    }
}

/// Truncate path with ellipsis in the middle
fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len || max_len < 5 {
        return path.to_string();
    }

    // Reserve space for " ... "
    let available = max_len - 4;
    let start_len = available / 2;
    let end_len = available - start_len;

    format!(
        "{}...{}",
        &path[..start_len],
        &path[path.len().saturating_sub(end_len)..]
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;

    fn create_theme() -> Theme {
        Theme::dark()
    }

    #[test]
    fn test_path_bar_simple() {
        let backend = TestBackend::new(80, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = create_theme();

        terminal
            .draw(|f| {
                let path_bar = PathBar::new(Path::new("/home/user/projects"), Some(5), &theme);
                f.render_widget(path_bar, f.area());
            })
            .unwrap();
    }

    #[test]
    fn test_truncate_path_short() {
        let path = "/home/user";
        assert_eq!(truncate_path(path, 20), "/home/user");
    }

    #[test]
    fn test_truncate_path_long() {
        let path = "/Users/yyyyyyh/Desktop/ghclone/repotui/src/ui/widgets";
        let truncated = truncate_path(path, 30);

        assert!(truncated.len() <= 30);
        assert!(truncated.contains("..."));
        assert!(truncated.starts_with("/Users/"));
        assert!(truncated.ends_with("widgets"));
    }

    #[test]
    fn test_truncate_path_very_short() {
        let path = "/home/user";
        assert_eq!(truncate_path(path, 3), "/home/user"); // No truncation if too short
    }

    #[test]
    fn test_path_bar_display_text() {
        let theme = create_theme();
        let path_bar = PathBar::new(Path::new("/tmp/test"), Some(3), &theme);

        let text = path_bar.display_text(50);
        assert!(text.contains("📂"));
        assert!(text.contains("/tmp/test"));
        assert!(text.contains("3 repos"));
    }

    #[test]
    fn test_path_bar_display_text_truncate() {
        let theme = create_theme();
        let long_path = "/Users/yyyyyyh/Desktop/ghclone/repotui/src/ui/widgets/path_bar";
        let path_bar = PathBar::new(Path::new(long_path), Some(10), &theme).max_length(30);

        let text = path_bar.display_text(50);
        assert!(text.len() <= 30);
        assert!(text.contains("..."));
    }

    #[test]
    fn test_path_bar_no_repo_count() {
        let theme = create_theme();
        let path_bar = PathBar::new(Path::new("/tmp"), None, &theme);

        let text = path_bar.display_text(50);
        assert!(!text.contains("repos"));
    }
}
