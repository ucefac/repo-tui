//! Main directory manager widget
//!
//! Displays and manages the list of main directories.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};

use crate::app::model::MainDirectoryInfo;
use crate::ui::theme::Theme;

/// Main directory manager widget
pub struct MainDirManager<'a> {
    /// Directories to display
    pub directories: &'a [MainDirectoryInfo],
    /// Selected index
    pub selected_index: usize,
    /// Theme
    pub theme: &'a Theme,
    /// Editing index (if any)
    pub editing_index: Option<usize>,
    /// Editing name (if editing)
    pub editing_name: &'a str,
}

impl<'a> MainDirManager<'a> {
    /// Create new manager
    pub fn new(
        directories: &'a [MainDirectoryInfo],
        selected_index: usize,
        theme: &'a Theme,
    ) -> Self {
        Self {
            directories,
            selected_index,
            theme,
            editing_index: None,
            editing_name: "",
        }
    }

    /// Set editing state
    pub fn editing(mut self, index: usize, name: &'a str) -> Self {
        self.editing_index = Some(index);
        self.editing_name = name;
        self
    }
}

impl<'a> Widget for MainDirManager<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear background
        Clear.render(area, buf);

        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(5),    // Directory list
                Constraint::Length(1), // Help
            ])
            .split(area);

        self.render_title(chunks[0], buf);
        self.render_directory_list(chunks[1], buf);
        self.render_help(chunks[2], buf);
    }
}

impl<'a> MainDirManager<'a> {
    fn render_title(&self, area: Rect, buf: &mut Buffer) {
        let title_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Title
                Constraint::Length(1), // Subtitle
            ])
            .split(area);

        let title = Paragraph::new("🏠 Manage Main Directories")
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(self.theme.colors.primary.into())
                    .add_modifier(Modifier::BOLD),
            );
        title.render(title_layout[0], buf);

        let subtitle = Paragraph::new("Manage root directories that store multiple repositories")
            .alignment(Alignment::Center)
            .style(Style::default().fg(self.theme.colors.text_muted.into()));
        subtitle.render(title_layout[1], buf);
    }

    fn render_directory_list(&self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = self
            .directories
            .iter()
            .enumerate()
            .map(|(i, dir)| {
                let is_selected = i == self.selected_index;
                let is_editing = self.editing_index == Some(i);

                // Build display text
                let enabled_icon = if dir.enabled { "✓" } else { "✗" };
                let display_text = if is_editing {
                    format!(
                        "► {} {} (editing: {})",
                        enabled_icon, dir.display_name, self.editing_name
                    )
                } else {
                    format!(
                        "{} {} ({} repos)",
                        enabled_icon, dir.display_name, dir.repo_count
                    )
                };

                let style = if is_selected {
                    Style::default()
                        .bg(self.theme.colors.selected_bg.into())
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.theme.colors.foreground.into())
                };

                ListItem::new(display_text).style(style)
            })
            .collect();

        let block = Block::default()
            .title(format!(" Directories ({}) ", self.directories.len()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border.into()));

        let list = List::new(items).block(block);
        ratatui::prelude::Widget::render(list, area, buf);
    }

    fn render_help(&self, area: Rect, buf: &mut Buffer) {
        let help_text = if self.editing_index.is_some() {
            "Enter confirm   Esc cancel   Type to edit"
        } else {
            "↑↓ navigate   a add   d delete   e edit   SPACE toggle   Esc close"
        };

        // Parse help text with color highlighting
        let spans = parse_help_text(help_text, self.theme);

        let paragraph = Paragraph::new(Line::from(spans))
            .alignment(Alignment::Left)
            .style(Style::default().fg(self.theme.colors.text_muted.into()));

        paragraph.render(area, buf);
    }
}

/// Parse help text and highlight keys
fn parse_help_text<'a>(text: &'a str, theme: &'a Theme) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    let parts: Vec<&'a str> = text.split("   ").collect();

    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("   "));
        }

        if let Some(space_pos) = part.find(' ') {
            let key = &part[..space_pos];
            let desc = &part[space_pos..];

            spans.push(Span::styled(
                key,
                Style::default().fg(theme.colors.primary.into()),
            ));
            spans.push(Span::raw(desc));
        } else {
            spans.push(Span::raw(part.to_string()));
        }
    }

    spans
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
    use std::path::PathBuf;

    fn create_test_directories() -> Vec<MainDirectoryInfo> {
        vec![
            MainDirectoryInfo {
                path: PathBuf::from("/home/user/Projects"),
                display_name: "Projects".to_string(),
                enabled: true,
                repo_count: 12,
            },
            MainDirectoryInfo {
                path: PathBuf::from("/home/user/work"),
                display_name: "work".to_string(),
                enabled: true,
                repo_count: 8,
            },
            MainDirectoryInfo {
                path: PathBuf::from("/home/user/experiments"),
                display_name: "experiments".to_string(),
                enabled: false,
                repo_count: 5,
            },
        ]
    }

    #[test]
    fn test_main_dir_manager_render() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();
        let dirs = create_test_directories();

        terminal
            .draw(|f| {
                let area = f.area();
                let manager = MainDirManager::new(&dirs, 0, &theme);
                f.render_widget(manager, area);
            })
            .unwrap();
    }

    #[test]
    fn test_main_dir_manager_with_selection() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();
        let dirs = create_test_directories();

        terminal
            .draw(|f| {
                let area = f.area();
                let manager = MainDirManager::new(&dirs, 1, &theme);
                f.render_widget(manager, area);
            })
            .unwrap();
    }

    #[test]
    fn test_main_dir_manager_editing() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();
        let dirs = create_test_directories();

        terminal
            .draw(|f| {
                let area = f.area();
                let manager = MainDirManager::new(&dirs, 0, &theme).editing(0, "My Projects");
                f.render_widget(manager, area);
            })
            .unwrap();
    }

    #[test]
    fn test_main_dir_manager_empty() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();
        let dirs: Vec<MainDirectoryInfo> = vec![];

        terminal
            .draw(|f| {
                let area = f.area();
                let manager = MainDirManager::new(&dirs, 0, &theme);
                f.render_widget(manager, area);
            })
            .unwrap();
    }
}
