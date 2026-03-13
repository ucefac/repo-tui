//! Generic directory chooser widget
//!
//! Supports two modes:
//! 1. SelectMainDirectory - Select main directories (any directory)
//! 2. AddSingleRepository - Select single Git repository (validates .git exists)

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use std::path::PathBuf;

use crate::app::state::{DirectoryChooserMode, ReturnTarget};
use crate::ui::theme::Theme;

/// Directory chooser state
#[derive(Debug, Clone)]
pub struct DirectoryChooserState {
    /// Current path
    pub current_path: PathBuf,
    /// Directory entries
    pub entries: Vec<String>,
    /// Selected index
    pub selected_index: usize,
    /// Scroll offset
    pub scroll_offset: usize,
    /// Chooser mode
    pub mode: DirectoryChooserMode,
}

impl DirectoryChooserState {
    /// Create new state
    pub fn new(initial_path: PathBuf, mode: DirectoryChooserMode) -> Self {
        Self {
            current_path: initial_path,
            entries: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            mode,
        }
    }

    /// Get title based on mode
    pub fn title(&self) -> &'static str {
        match &self.mode {
            DirectoryChooserMode::SelectMainDirectory { .. } => "Select Main Directory",
            DirectoryChooserMode::AddSingleRepository => "Add Single Repository",
        }
    }

    /// Get icon based on mode
    pub fn icon(&self) -> &'static str {
        match &self.mode {
            DirectoryChooserMode::SelectMainDirectory { .. } => "📁",
            DirectoryChooserMode::AddSingleRepository => "📦",
        }
    }

    /// Get subtitle based on mode
    pub fn subtitle(&self) -> &'static str {
        match &self.mode {
            DirectoryChooserMode::SelectMainDirectory { return_to, .. } => match return_to {
                ReturnTarget::ManagingDirs => {
                    "Select a folder to serve as root for multiple git repositories"
                }
                ReturnTarget::Running => {
                    "Select a git-managed folder to add to the repository list"
                }
            },
            DirectoryChooserMode::AddSingleRepository => {
                "Select a git-managed folder to add to the repository list"
            }
        }
    }

    /// Get help text based on mode
    pub fn help_text(&self) -> &'static str {
        match &self.mode {
            DirectoryChooserMode::SelectMainDirectory { allow_multiple, .. } => {
                if *allow_multiple {
                    "↑↓ navigate   ← back   → enter   SPACE toggle   Enter confirm   Esc cancel"
                } else {
                    "↑↓ navigate   ← back   → enter   SPACE select   Esc cancel"
                }
            }
            DirectoryChooserMode::AddSingleRepository => {
                "↑↓ navigate   ← back   → enter   SPACE select repo   Esc cancel"
            }
        }
    }
}

/// Directory chooser widget
pub struct DirectoryChooser<'a> {
    /// State
    pub state: &'a DirectoryChooserState,
    /// Theme
    pub theme: &'a Theme,
    /// Visible height
    pub visible_height: u16,
    /// Git repo count (optional)
    pub git_repo_count: Option<usize>,
    /// Selected paths (for multi-select mode)
    pub selected_paths: Option<&'a std::collections::HashSet<PathBuf>>,
}

impl<'a> DirectoryChooser<'a> {
    /// Create new chooser
    pub fn new(state: &'a DirectoryChooserState, theme: &'a Theme) -> Self {
        Self {
            state,
            theme,
            visible_height: 10,
            git_repo_count: None,
            selected_paths: None,
        }
    }

    /// Set visible height
    pub fn visible_height(mut self, height: u16) -> Self {
        self.visible_height = height;
        self
    }

    /// Set git repo count
    pub fn git_repo_count(mut self, count: usize) -> Self {
        self.git_repo_count = Some(count);
        self
    }

    /// Set selected paths (for multi-select)
    pub fn selected_paths(mut self, paths: &'a std::collections::HashSet<PathBuf>) -> Self {
        self.selected_paths = Some(paths);
        self
    }
}

impl<'a> Widget for DirectoryChooser<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let show_selection_indicator = matches!(
            self.state.mode,
            DirectoryChooserMode::SelectMainDirectory {
                allow_multiple: true,
                ..
            }
        );

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title + Subtitle
                Constraint::Length(3), // Current path
                Constraint::Length(2), // Stats
                Constraint::Min(5),    // Directory list
                Constraint::Length(1), // Spacer
                Constraint::Length(1), // Help
            ])
            .split(area);

        self.render_title(chunks[0], buf);
        self.render_current_path(chunks[1], buf);
        self.render_stats(chunks[2], buf);
        self.render_directory_list(chunks[3], buf, show_selection_indicator);
        self.render_help(chunks[5], buf);
    }
}

impl<'a> DirectoryChooser<'a> {
    fn render_title(&self, area: Rect, buf: &mut Buffer) {
        let title_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Title
                Constraint::Length(1), // Subtitle
            ])
            .split(area);

        let title_text = format!("{} {}", self.state.icon(), self.state.title());

        let title = Paragraph::new(title_text)
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(self.theme.colors.primary.into())
                    .add_modifier(Modifier::BOLD),
            );
        title.render(title_layout[0], buf);

        let subtitle = Paragraph::new(self.state.subtitle())
            .alignment(Alignment::Center)
            .style(Style::default().fg(self.theme.colors.text_muted.into()));
        subtitle.render(title_layout[1], buf);
    }

    fn render_current_path(&self, area: Rect, buf: &mut Buffer) {
        let path_str = self.state.current_path.display().to_string();
        let text = format!("📂 {}", path_str);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border_focused.into()));

        let paragraph = Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Left)
            .style(Style::default().fg(self.theme.colors.foreground.into()));

        paragraph.render(area, buf);
    }

    fn render_stats(&self, area: Rect, buf: &mut Buffer) {
        let stats_text = if let Some(git) = self.git_repo_count {
            format!(
                "📊 {} subdirectories | 🗂️ {} Git repositories",
                self.state.entries.len(),
                git
            )
        } else {
            format!("📊 {} subdirectories", self.state.entries.len())
        };

        let paragraph = Paragraph::new(stats_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(self.theme.colors.text_muted.into()));

        paragraph.render(area, buf);
    }

    fn render_directory_list(&self, area: Rect, buf: &mut Buffer, show_selection: bool) {
        if self.state.entries.is_empty() {
            let empty_text = match &self.state.mode {
                DirectoryChooserMode::SelectMainDirectory { .. } => "(empty directory)",
                DirectoryChooserMode::AddSingleRepository => "(no Git repositories found)",
            };
            let paragraph = Paragraph::new(empty_text)
                .alignment(Alignment::Center)
                .style(Style::default().fg(self.theme.colors.text_muted.into()));
            paragraph.render(area, buf);
            return;
        }

        // Calculate visible range
        let visible_count = area.height.saturating_sub(2) as usize;
        let start = self.state.scroll_offset;
        let end = (start + visible_count).min(self.state.entries.len());

        let items: Vec<ListItem> = self.state.entries[start..end]
            .iter()
            .enumerate()
            .map(|(visible_idx, name)| {
                let absolute_idx = start + visible_idx;
                let is_selected = absolute_idx == self.state.selected_index;

                let mut style = Style::default().fg(self.theme.colors.foreground.into());
                if is_selected {
                    style = style
                        .bg(self.theme.colors.selected_bg.into())
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD);
                }

                let prefix = if is_selected { "▌ " } else { "  " };
                let selection_marker = if show_selection {
                    let entry_path = self.state.current_path.join(name);
                    let is_marked = self
                        .selected_paths
                        .map(|paths| paths.contains(&entry_path))
                        .unwrap_or(false);
                    if is_marked {
                        "[✓] "
                    } else {
                        "[ ] "
                    }
                } else {
                    ""
                };

                ListItem::new(format!("{}{}📁 {}", prefix, selection_marker, name)).style(style)
            })
            .collect();

        let block = Block::default()
            .title(format!(
                " Directories ({}/{}) ",
                end - start,
                self.state.entries.len()
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border.into()));

        let list = List::new(items)
            .block(block)
            .style(Style::default().fg(self.theme.colors.foreground.into()));

        Widget::render(list, area, buf);
    }

    fn render_help(&self, area: Rect, buf: &mut Buffer) {
        let help_text = self.state.help_text();
        let spans = parse_help_message(help_text, self.theme);

        let paragraph = Paragraph::new(Line::from(spans))
            .alignment(Alignment::Left)
            .style(Style::default().fg(self.theme.colors.text_muted.into()));

        paragraph.render(area, buf);
    }
}

/// Parse help message and apply theme color highlight to key hints
fn parse_help_message<'a>(message: &'a str, theme: &'a Theme) -> Vec<Span<'a>> {
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
    fn test_dir_chooser_empty_main_dir_mode() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();

        let state = DirectoryChooserState::new(
            PathBuf::from("/tmp"),
            DirectoryChooserMode::SelectMainDirectory {
                allow_multiple: false,
                edit_mode: false,
                return_to: ReturnTarget::Running,
            },
        );

        terminal
            .draw(|f| {
                let area = f.area();
                let chooser = DirectoryChooser::new(&state, &theme);
                f.render_widget(chooser, area);
            })
            .unwrap();
    }

    #[test]
    fn test_dir_chooser_single_repo_mode() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();

        let state = DirectoryChooserState::new(
            PathBuf::from("/home/user"),
            DirectoryChooserMode::AddSingleRepository,
        );

        terminal
            .draw(|f| {
                let area = f.area();
                let chooser = DirectoryChooser::new(&state, &theme);
                f.render_widget(chooser, area);
            })
            .unwrap();
    }

    #[test]
    fn test_dir_chooser_with_entries() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();

        let mut state = DirectoryChooserState::new(
            PathBuf::from("/home/user"),
            DirectoryChooserMode::SelectMainDirectory {
                allow_multiple: false,
                edit_mode: false,
                return_to: ReturnTarget::Running,
            },
        );
        state.entries = vec![
            "Documents".to_string(),
            "Projects".to_string(),
            "Downloads".to_string(),
        ];
        state.selected_index = 1;

        terminal
            .draw(|f| {
                let area = f.area();
                let chooser = DirectoryChooser::new(&state, &theme).git_repo_count(5);
                f.render_widget(chooser, area);
            })
            .unwrap();
    }

    #[test]
    fn test_dir_chooser_multi_select() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::dark();

        let mut state = DirectoryChooserState::new(
            PathBuf::from("/home/user"),
            DirectoryChooserMode::SelectMainDirectory {
                allow_multiple: true,
                edit_mode: false,
                return_to: ReturnTarget::Running,
            },
        );
        state.entries = vec![
            "Documents".to_string(),
            "Projects".to_string(),
            "Downloads".to_string(),
        ];

        let mut selected = std::collections::HashSet::new();
        selected.insert(PathBuf::from("/home/user/Projects"));

        terminal
            .draw(|f| {
                let area = f.area();
                let chooser = DirectoryChooser::new(&state, &theme).selected_paths(&selected);
                f.render_widget(chooser, area);
            })
            .unwrap();
    }

    #[test]
    fn test_state_title() {
        let state = DirectoryChooserState::new(
            PathBuf::from("/tmp"),
            DirectoryChooserMode::SelectMainDirectory {
                allow_multiple: false,
                edit_mode: false,
                return_to: ReturnTarget::Running,
            },
        );
        assert_eq!(state.title(), "Select Main Directory");

        let state2 = DirectoryChooserState::new(
            PathBuf::from("/tmp"),
            DirectoryChooserMode::AddSingleRepository,
        );
        assert_eq!(state2.title(), "Add Single Repository");
    }

    #[test]
    fn test_state_icon() {
        let state = DirectoryChooserState::new(
            PathBuf::from("/tmp"),
            DirectoryChooserMode::SelectMainDirectory {
                allow_multiple: false,
                edit_mode: false,
                return_to: ReturnTarget::Running,
            },
        );
        assert_eq!(state.icon(), "📁");

        let state2 = DirectoryChooserState::new(
            PathBuf::from("/tmp"),
            DirectoryChooserMode::AddSingleRepository,
        );
        assert_eq!(state2.icon(), "📦");
    }

    #[test]
    fn test_state_subtitle() {
        // Test subtitle when opened from Running state (repo list)
        let state_from_running = DirectoryChooserState::new(
            PathBuf::from("/tmp"),
            DirectoryChooserMode::SelectMainDirectory {
                allow_multiple: false,
                edit_mode: false,
                return_to: ReturnTarget::Running,
            },
        );
        assert_eq!(
            state_from_running.subtitle(),
            "Select a git-managed folder to add to the repository list"
        );

        // Test subtitle when opened from ManagingDirs state
        let state_from_managing = DirectoryChooserState::new(
            PathBuf::from("/tmp"),
            DirectoryChooserMode::SelectMainDirectory {
                allow_multiple: false,
                edit_mode: false,
                return_to: ReturnTarget::ManagingDirs,
            },
        );
        assert_eq!(
            state_from_managing.subtitle(),
            "Select a folder to serve as root for multiple git repositories"
        );

        // Test subtitle for AddSingleRepository mode
        let state_single = DirectoryChooserState::new(
            PathBuf::from("/tmp"),
            DirectoryChooserMode::AddSingleRepository,
        );
        assert_eq!(
            state_single.subtitle(),
            "Select a git-managed folder to add to the repository list"
        );
    }
}
