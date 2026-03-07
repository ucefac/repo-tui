//! Theme selector widget
//!
//! Provides a UI for selecting and previewing themes.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use crate::ui::theme::Theme;

/// Theme selector widget state
#[derive(Debug, Clone)]
pub struct ThemeSelector<'a> {
    /// List of theme names
    pub themes: &'a [&'a str],
    /// Selected index
    pub selected_index: usize,
    /// Currently active theme (for preview comparison)
    pub current_theme: &'a Theme,
    /// Preview theme (what would be applied if selected)
    pub preview_theme: Theme,
    /// Title
    pub title: &'a str,
}

impl<'a> ThemeSelector<'a> {
    /// Create a new theme selector
    pub fn new(
        themes: &'a [&'a str],
        selected_index: usize,
        current_theme: &'a Theme,
        preview_theme: Theme,
    ) -> Self {
        Self {
            themes,
            selected_index,
            current_theme,
            preview_theme,
            title: "Select Theme",
        }
    }

    /// Set the title
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    /// Get the currently selected theme name
    pub fn selected(&self) -> Option<&str> {
        self.themes.get(self.selected_index).copied()
    }

    /// Select next theme
    pub fn next(&mut self) {
        if !self.themes.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.themes.len();
        }
    }

    /// Select previous theme
    pub fn previous(&mut self) {
        if !self.themes.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.themes.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }
}

impl<'a> Widget for ThemeSelector<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create vertical layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(7), // Theme preview
                Constraint::Min(5),    // Theme list
                Constraint::Length(3), // Help text
            ])
            .split(area);

        // Render title
        render_title(chunks[0], buf, &self.preview_theme);

        // Render theme preview
        render_preview(
            chunks[1],
            buf,
            self.themes,
            self.selected_index,
            &self.preview_theme,
            self.current_theme,
        );

        // Render theme list
        render_theme_list(
            chunks[2],
            buf,
            self.themes,
            self.selected_index,
            &self.preview_theme,
        );

        // Render help text
        render_help(chunks[3], buf, &self.preview_theme);
    }
}

/// Render title section
fn render_title(area: Rect, buf: &mut Buffer, theme: &Theme) {
    let title = Paragraph::new("🎨 Theme Selector")
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(theme.colors.primary.into())
                .add_modifier(Modifier::BOLD),
        );
    title.render(area, buf);
}

/// Render theme preview section
fn render_preview(
    area: Rect,
    buf: &mut Buffer,
    themes: &[&str],
    selected_index: usize,
    preview_theme: &Theme,
    current_theme: &Theme,
) {
    if let Some(theme_name) = themes.get(selected_index) {
        // Create preview block with color samples
        let preview_block = Block::default()
            .title(format!(" Preview: {} ", theme_name))
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(preview_theme.colors.border.into()));

        // Build preview text with color samples
        let mut lines = vec![];

        // Theme name line
        lines.push(Line::from(format!("Theme: {}", theme_name)));

        // Color samples - show primary colors
        let colors_line = Line::from(vec![
            Span::styled(
                " Primary ",
                Style::default()
                    .fg(preview_theme.colors.foreground.into())
                    .bg(preview_theme.colors.primary.into())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                " Success ",
                Style::default()
                    .fg(preview_theme.colors.foreground.into())
                    .bg(preview_theme.colors.success.into())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                " Warning ",
                Style::default()
                    .fg(preview_theme.colors.foreground.into())
                    .bg(preview_theme.colors.warning.into())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                " Error ",
                Style::default()
                    .fg(preview_theme.colors.foreground.into())
                    .bg(preview_theme.colors.error.into())
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        lines.push(colors_line);

        // Show RGB values for selected colors
        let rgb_line = Line::from(format!(
            "Selected: RGB({},{},{}) | Current theme: {}",
            preview_theme.colors.selected_bg.r,
            preview_theme.colors.selected_bg.g,
            preview_theme.colors.selected_bg.b,
            current_theme.name
        ));
        lines.push(rgb_line);

        let preview_text = Paragraph::new(lines)
            .block(preview_block)
            .alignment(Alignment::Center)
            .style(Style::default().fg(preview_theme.colors.foreground.into()));

        preview_text.render(area, buf);
    }
}

/// Render theme list
fn render_theme_list(
    area: Rect,
    buf: &mut Buffer,
    themes: &[&str],
    selected_index: usize,
    theme: &Theme,
) {
    if themes.is_empty() {
        let empty_text = "(no themes available)";
        let paragraph = Paragraph::new(empty_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme.colors.text_muted.into()));
        paragraph.render(area, buf);
        return;
    }

    let items: Vec<ListItem> = themes
        .iter()
        .enumerate()
        .map(|(i, &name)| {
            let mut style = Style::default().fg(theme.colors.foreground.into());

            if i == selected_index {
                style = style
                    .bg(theme.colors.selected_bg.into())
                    .fg(theme.colors.selected_fg.into())
                    .add_modifier(Modifier::BOLD);
            }

            let prefix = if i == selected_index { "▶ " } else { "  " };
            ListItem::new(format!("{}{}", prefix, name)).style(style)
        })
        .collect();

    let block = Block::default()
        .title(format!(
            " Themes ({}/{}) ",
            selected_index + 1,
            themes.len()
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.colors.border.into()));

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(theme.colors.selected_bg.into())
                .fg(theme.colors.selected_fg.into())
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▌ ");

    Widget::render(list, area, buf);
}

/// Render help text
fn render_help(area: Rect, buf: &mut Buffer, theme: &Theme) {
    let help_text = "[j/k/↑/↓] Navigate  [Enter] Select  [Esc/q] Cancel";

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.colors.border.into()));

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(theme.colors.text_muted.into())
                .bg(Color::DarkGray),
        );

    paragraph.render(area, buf);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::themes::THEME_NAMES;
    use ratatui::backend::TestBackend;

    #[test]
    fn test_theme_selector_creation() {
        let themes = THEME_NAMES;
        let current_theme = Theme::dark();
        let preview_theme = Theme::new("nord");

        let selector = ThemeSelector::new(themes, 0, &current_theme, preview_theme);
        assert_eq!(selector.selected(), Some("dark"));
    }

    #[test]
    fn test_theme_selector_next() {
        let themes = THEME_NAMES;
        let current_theme = Theme::dark();
        let preview_theme = Theme::new("nord");

        let mut selector = ThemeSelector::new(themes, 0, &current_theme, preview_theme);
        assert_eq!(selector.selected(), Some("dark"));

        selector.next();
        assert_eq!(selector.selected(), Some("light"));
    }

    #[test]
    fn test_theme_selector_previous() {
        let themes = THEME_NAMES;
        let current_theme = Theme::dark();
        let preview_theme = Theme::new("nord");

        let mut selector = ThemeSelector::new(themes, 0, &current_theme, preview_theme);
        assert_eq!(selector.selected(), Some("dark"));

        selector.previous();
        assert_eq!(
            selector.selected(),
            Some("catppuccin_mocha"),
            "Should wrap around to last item"
        );
    }

    #[test]
    fn test_theme_selector_wrap() {
        let themes = THEME_NAMES;
        let current_theme = Theme::dark();
        let preview_theme = Theme::new("nord");

        let mut selector =
            ThemeSelector::new(themes, themes.len() - 1, &current_theme, preview_theme);

        selector.next();
        assert_eq!(
            selector.selected(),
            Some("dark"),
            "Should wrap around to first item"
        );
    }

    #[test]
    fn test_theme_selector_empty_themes() {
        let current_theme = Theme::dark();
        let preview_theme = Theme::new("nord");
        let empty_themes: &[&str] = &[];

        let selector = ThemeSelector::new(empty_themes, 0, &current_theme, preview_theme);
        assert_eq!(selector.selected(), None);
    }

    #[test]
    fn test_theme_selector_render_empty() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let current_theme = Theme::dark();
        let preview_theme = Theme::new("nord");
        let empty_themes: &[&str] = &[];

        terminal
            .draw(|f| {
                let area = f.area();
                let selector = ThemeSelector::new(empty_themes, 0, &current_theme, preview_theme);
                f.render_widget(selector, area);
            })
            .unwrap();
    }

    #[test]
    fn test_theme_selector_render() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let current_theme = Theme::dark();
        let preview_theme = Theme::new("nord");

        terminal
            .draw(|f| {
                let area = f.area();
                let selector = ThemeSelector::new(THEME_NAMES, 2, &current_theme, preview_theme);
                f.render_widget(selector, area);
            })
            .unwrap();
    }

    #[test]
    fn test_theme_selector_title() {
        let themes = THEME_NAMES;
        let current_theme = Theme::dark();
        let preview_theme = Theme::new("nord");

        let selector =
            ThemeSelector::new(themes, 0, &current_theme, preview_theme).title("Custom Title");
        assert_eq!(selector.title, "Custom Title");
    }
}
