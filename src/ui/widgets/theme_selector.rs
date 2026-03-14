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
    /// Scroll offset for list
    pub scroll_offset: usize,
    /// Visible height for scroll calculation
    pub visible_height: u16,
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
            scroll_offset: 0,
            visible_height: 10,
        }
    }

    /// Set the title
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    /// Set scroll offset
    pub fn scroll_offset(mut self, offset: usize) -> Self {
        self.scroll_offset = offset;
        self
    }

    /// Set visible height for scroll calculation
    pub fn visible_height(mut self, height: u16) -> Self {
        self.visible_height = height;
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

    /// Update scroll offset to ensure selected item is visible
    pub fn update_scroll(&mut self) {
        // Calculate visible count from the actual layout
        // Title: 2, Preview: 7, Help: 1, Border: 2, Total: 12
        let visible_count = self.visible_height.saturating_sub(12) as usize;
        if visible_count == 0 {
            return;
        }

        let selected = self.selected_index;
        let current_offset = self.scroll_offset;

        // Scroll down if selected is below visible area (use > not >=)
        if selected > current_offset + visible_count - 1 {
            self.scroll_offset = selected.saturating_sub(visible_count - 1);
        }
        // Scroll up if selected is above visible area
        else if selected < current_offset {
            self.scroll_offset = selected;
        }
    }
}

impl<'a> Widget for ThemeSelector<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        // Update scroll offset first
        self.update_scroll();

        // Create outer block with focused border for visual distinction
        let outer_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.preview_theme.colors.border_focused.into()))
            .style(Style::default().bg(self.preview_theme.colors.background.into()));

        // Get inner area for content (must be done before rendering the block)
        let inner_area = outer_block.inner(area);

        // Render outer block
        outer_block.render(area, buf);

        // Create vertical layout inside the bordered area
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Title (reduced from 3)
                Constraint::Length(7), // Theme preview
                Constraint::Min(5),    // Theme list
                Constraint::Length(1), // Help text (reduced from 3)
            ])
            .split(inner_area);

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
            self.scroll_offset,
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
        // Check if random option is selected
        if theme_name.contains("Random") {
            // Show random hint instead of color samples
            let preview_block = Block::default()
                .title(" Preview: 🎲 随机 ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(preview_theme.colors.border.into()));

            let lines = vec![
                Line::from("🎲 随机主题"),
                Line::from(""),
                Line::from("将随机选择一个主题"),
                Line::from("每次启动时变化"),
            ];

            let preview_text = Paragraph::new(lines)
                .block(preview_block)
                .alignment(Alignment::Center)
                .style(Style::default().fg(preview_theme.colors.foreground.into()));

            preview_text.render(area, buf);
            return;
        }

        // Normal theme preview
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
    scroll_offset: usize,
) {
    if themes.is_empty() {
        let empty_text = "(no themes available)";
        let paragraph = Paragraph::new(empty_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme.colors.text_muted.into()));
        paragraph.render(area, buf);
        return;
    }

    // Calculate visible range
    let visible_count = area.height.saturating_sub(2) as usize;
    let start = scroll_offset;
    let end = (start + visible_count).min(themes.len());

    let items: Vec<ListItem> = themes[start..end]
        .iter()
        .enumerate()
        .map(|(visible_idx, &name)| {
            let absolute_idx = start + visible_idx;
            let is_selected = absolute_idx == selected_index;

            let mut style = Style::default().fg(theme.colors.foreground.into());

            if is_selected {
                style = style
                    .bg(theme.colors.selected_bg.into())
                    .fg(theme.colors.selected_fg.into())
                    .add_modifier(Modifier::BOLD);
            }

            let prefix = if is_selected { "▶ " } else { "  " };
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

/// Render help text - 重构后，符合 UI 设计规范
/// 参考: StatusBar 组件设计 (无背景色、左对齐)
fn render_help(area: Rect, buf: &mut Buffer, theme: &Theme) {
    // 使用 Line + Span 构建帮助文本，按键使用 primary 色高亮
    let help_line = Line::from(vec![
        Span::styled("↑↓", Style::default().fg(theme.colors.primary.into())),
        Span::styled(
            " navigate  ",
            Style::default().fg(theme.colors.text_muted.into()),
        ),
        Span::styled("ENTER", Style::default().fg(theme.colors.primary.into())),
        Span::styled(
            " select  ",
            Style::default().fg(theme.colors.text_muted.into()),
        ),
        Span::styled("ESC", Style::default().fg(theme.colors.primary.into())),
        Span::styled(
            " cancel",
            Style::default().fg(theme.colors.text_muted.into()),
        ),
    ]);

    let paragraph = Paragraph::new(help_line).alignment(Alignment::Left);
    // ❌ 移除 .bg(Color::DarkGray) - 硬编码背景色
    // ❌ 移除 .block(...) - 过时边框设计

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

        // Index 1 is "dark" (index 0 is "🎲 Random (随机)")
        let selector = ThemeSelector::new(themes, 1, &current_theme, preview_theme);
        assert_eq!(selector.selected(), Some("dark"));
    }

    #[test]
    fn test_theme_selector_next() {
        let themes = THEME_NAMES;
        let current_theme = Theme::dark();
        let preview_theme = Theme::new("nord");

        // Start at index 1 ("dark")
        let mut selector = ThemeSelector::new(themes, 1, &current_theme, preview_theme);
        assert_eq!(selector.selected(), Some("dark"));

        selector.next();
        assert_eq!(selector.selected(), Some("light"));
    }

    #[test]
    fn test_theme_selector_previous() {
        let themes = THEME_NAMES;
        let current_theme = Theme::dark();
        let preview_theme = Theme::new("nord");

        // Start at index 1 ("dark")
        let mut selector = ThemeSelector::new(themes, 1, &current_theme, preview_theme);
        assert_eq!(selector.selected(), Some("dark"));

        selector.previous();
        assert_eq!(
            selector.selected(),
            Some("🎲 Random (随机)"),
            "Should wrap around to first item (random)"
        );
    }

    #[test]
    fn test_theme_selector_wrap() {
        let themes = THEME_NAMES;
        let current_theme = Theme::dark();
        let preview_theme = Theme::new("nord");

        // Start at last theme
        let mut selector =
            ThemeSelector::new(themes, themes.len() - 1, &current_theme, preview_theme);

        selector.next();
        assert_eq!(
            selector.selected(),
            Some("🎲 Random (随机)"),
            "Should wrap around to first item (random)"
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
