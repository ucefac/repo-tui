//! Directory chooser widget
//!
//! Provides a file browser for selecting the main directory.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use std::path::Path;

use crate::ui::theme::Theme;

/// Directory chooser widget state
#[derive(Debug, Clone)]
pub struct DirChooser<'a> {
    /// Current directory path
    pub current_path: &'a Path,
    /// Directory entries (names only)
    pub entries: &'a [String],
    /// Selected index
    pub selected_index: usize,
    /// Scroll offset for viewport tracking
    pub scroll_offset: usize,
    /// Visible height (for calculating visible count)
    pub visible_height: u16,
    /// Theme
    pub theme: &'a Theme,
    /// Title
    pub title: &'a str,
    /// Git repo count in current directory (optional)
    pub git_repo_count: Option<usize>,
}

impl<'a> DirChooser<'a> {
    /// Create a new directory chooser
    pub fn new(
        current_path: &'a Path,
        entries: &'a [String],
        selected_index: usize,
        scroll_offset: usize,
        theme: &'a Theme,
    ) -> Self {
        Self {
            current_path,
            entries,
            selected_index,
            scroll_offset,
            visible_height: 10,
            theme,
            title: "Select Main Directory",
            git_repo_count: None,
        }
    }

    /// Set the title
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    /// Set git repo count
    pub fn git_repo_count(mut self, count: usize) -> Self {
        self.git_repo_count = Some(count);
        self
    }

    /// Set visible height
    pub fn visible_height(mut self, height: u16) -> Self {
        self.visible_height = height;
        self
    }
}

impl<'a> Widget for DirChooser<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create vertical layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Current path
                Constraint::Length(2), // Stats
                Constraint::Min(5),    // Directory list
                Constraint::Length(1), // Spacer
                Constraint::Length(3), // Help text
            ])
            .split(area);

        // Render title
        render_title(chunks[0], buf, self.theme);

        // Render current path
        render_current_path(chunks[1], buf, self.current_path, self.theme);

        // Render stats
        render_stats(
            chunks[2],
            buf,
            self.entries.len(),
            self.git_repo_count,
            self.theme,
        );

        // Render directory list
        render_directory_list(
            chunks[3],
            buf,
            self.entries,
            self.selected_index,
            self.scroll_offset,
            chunks[3].height,
            self.theme,
        );

        // Render help text
        render_help(chunks[5], buf, self.theme);
    }
}

/// Render title section
fn render_title(area: Rect, buf: &mut Buffer, theme: &Theme) {
    let title = Paragraph::new("📁 Select Main Directory")
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        );
    title.render(area, buf);
}

/// Render current path section
fn render_current_path(area: Rect, buf: &mut Buffer, path: &Path, theme: &Theme) {
    let path_str = path.display().to_string();
    let text = format!("📂 {}", path_str);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border_focused));

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Left)
        .style(Style::default().fg(theme.text_primary));

    paragraph.render(area, buf);
}

/// Render statistics
fn render_stats(
    area: Rect,
    buf: &mut Buffer,
    dir_count: usize,
    git_count: Option<usize>,
    theme: &Theme,
) {
    let stats_text = if let Some(git) = git_count {
        format!(
            "📊 {} subdirectories | 🗂️ {} Git repositories",
            dir_count, git
        )
    } else {
        format!("📊 {} subdirectories", dir_count)
    };

    let paragraph = Paragraph::new(stats_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.text_secondary));

    paragraph.render(area, buf);
}

/// Render directory list
fn render_directory_list(
    area: Rect,
    buf: &mut Buffer,
    entries: &[String],
    selected_index: usize,
    scroll_offset: usize,
    visible_height: u16,
    theme: &Theme,
) {
    if entries.is_empty() {
        let empty_text = "(empty directory)";
        let paragraph = Paragraph::new(empty_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme.text_secondary));
        paragraph.render(area, buf);
        return;
    }

    // Calculate visible range
    let visible_count = visible_height.saturating_sub(2) as usize;
    let start = scroll_offset;
    let end = (start + visible_count).min(entries.len());

    let items: Vec<ListItem> = entries[start..end]
        .iter()
        .enumerate()
        .map(|(visible_idx, name)| {
            let absolute_idx = start + visible_idx;
            let mut style = Style::default().fg(theme.text_primary);
            if absolute_idx == selected_index {
                style = style
                    .bg(theme.selected_bg)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD);
            }
            let prefix = if absolute_idx == selected_index {
                "▌ 📁 "
            } else {
                "  📁 "
            };
            ListItem::new(format!("{}{}", prefix, name)).style(style)
        })
        .collect();

    let block = Block::default()
        .title(format!(" Directories ({}/{}) ", end - start, entries.len()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border_normal));

    let list = List::new(items)
        .block(block)
        .style(Style::default().fg(theme.text_primary));

    Widget::render(list, area, buf);
}

/// Render help text
fn render_help(area: Rect, buf: &mut Buffer, theme: &Theme) {
    let help_text = "[j/k] Navigate  [Enter] Select  [←] Go Back  [q] Cancel";

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border_normal));

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(theme.text_secondary)
                .bg(Color::DarkGray),
        );

    paragraph.render(area, buf);
}

/// Create centered rectangle for popup
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;

    #[test]
    fn test_dir_chooser_empty() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();

        terminal
            .draw(|f| {
                let area = f.area();
                let chooser = DirChooser::new(Path::new("/tmp"), &[], 0, 0, &theme);
                f.render_widget(chooser, area);
            })
            .unwrap();
    }

    #[test]
    fn test_dir_chooser_with_entries() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();

        let entries = vec![
            "Documents".to_string(),
            "Projects".to_string(),
            "Downloads".to_string(),
        ];

        terminal
            .draw(|f| {
                let area = f.area();
                let chooser = DirChooser::new(Path::new("/home/user"), &entries, 1, 0, &theme)
                    .git_repo_count(5);
                f.render_widget(chooser, area);
            })
            .unwrap();
    }
}
