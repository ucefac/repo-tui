//! Clone dialog widget
//!
//! Provides UI for the Git clone feature with multiple stages:
//! - URL input with main directory selection
//! - Confirm replace dialog
//! - Progress display
//! - Error display

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use std::path::PathBuf;

use crate::app::state::{CloneStage, CloneState};
use crate::ui::theme::Theme;

/// Clone dialog widget
pub struct CloneDialog<'a> {
    /// Clone state
    state: &'a CloneState,
    /// Theme reference
    theme: &'a Theme,
    /// Main directories for selection
    main_dirs: &'a [(PathBuf, String)], // (path, display_name)
    /// Generated folder name preview
    folder_preview: Option<String>,
    /// Validation error message
    validation_error: Option<String>,
}

impl<'a> CloneDialog<'a> {
    /// Create a new clone dialog
    pub fn new(
        state: &'a CloneState,
        theme: &'a Theme,
        main_dirs: &'a [(PathBuf, String)],
    ) -> Self {
        Self {
            state,
            theme,
            main_dirs,
            folder_preview: None,
            validation_error: None,
        }
    }

    /// Set the folder name preview
    pub fn folder_preview(mut self, preview: Option<String>) -> Self {
        self.folder_preview = preview;
        self
    }

    /// Set validation error message
    pub fn validation_error(mut self, error: Option<String>) -> Self {
        self.validation_error = error;
        self
    }

    /// Get the dialog title based on stage
    fn title(&self) -> &'static str {
        match self.state.stage {
            CloneStage::InputUrl => "📥 Clone Repository",
            CloneStage::ConfirmReplace { .. } => "⚠️ Repository Already Exists",
            CloneStage::Executing => "⏳ Cloning Repository",
            CloneStage::Error(_) => "❌ Clone Failed",
        }
    }
}

impl<'a> Widget for CloneDialog<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear the area
        Clear.render(area, buf);

        // Create block with title
        let title = self.title();
        let border_color = match self.state.stage {
            CloneStage::Error(_) => self.theme.colors.error,
            CloneStage::ConfirmReplace { .. } => self.theme.colors.warning,
            _ => self.theme.colors.border_focused,
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color.into()));

        let inner = block.inner(area);
        block.render(area, buf);

        // Render content based on stage
        match &self.state.stage {
            CloneStage::InputUrl => {
                render_input_url(&self, inner, buf);
            }
            CloneStage::ConfirmReplace { existing_path } => {
                render_confirm_replace(&self, existing_path, inner, buf);
            }
            CloneStage::Executing => {
                render_executing(&self, inner, buf);
            }
            CloneStage::Error(error) => {
                render_error(&self, error, inner, buf);
            }
        }
    }
}

/// Render URL input stage
fn render_input_url(dialog: &CloneDialog, area: Rect, buf: &mut Buffer) {
    // Split area into sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // URL input label + field
            Constraint::Length(2), // Spacing
            Constraint::Min(5),    // Main directory selection or info
            Constraint::Length(1), // Help text
        ])
        .split(area);

    // URL input section
    let url_label = Paragraph::new("Enter Git repository URL:");
    url_label.render(chunks[0], buf);

    // URL input box with cursor
    let url_display = if dialog.state.url_input.is_empty() {
        format!("{}", "▌")
    } else {
        // Insert cursor at cursor_position
        let pos = dialog
            .state
            .cursor_position
            .min(dialog.state.url_input.len());
        let before = &dialog.state.url_input[..pos];
        let after = &dialog.state.url_input[pos..];
        format!("{}▌{}", before, after)
    };

    let border_color = if dialog.validation_error.is_some() {
        dialog.theme.colors.error
    } else {
        dialog.theme.colors.border_focused
    };

    let url_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color.into()));

    let url_paragraph = Paragraph::new(url_display)
        .block(url_block)
        .style(Style::default().fg(dialog.theme.colors.foreground.into()));

    url_paragraph.render(chunks[0], buf);

    // Validation error or folder preview
    if let Some(error) = &dialog.validation_error {
        let error_text = Paragraph::new(error.as_str())
            .style(Style::default().fg(dialog.theme.colors.error.into()));
        error_text.render(chunks[1], buf);
    } else if let Some(preview) = &dialog.folder_preview {
        let preview_text = format!("Folder name: {}", preview);
        let preview_para = Paragraph::new(preview_text)
            .style(Style::default().fg(dialog.theme.colors.text_muted.into()));
        preview_para.render(chunks[1], buf);
    }

    // Main directory selection
    if dialog.main_dirs.len() > 1 {
        // Multiple main directories - show selection list
        let dir_label = Paragraph::new("Target directory:");
        dir_label.render(chunks[2], buf);

        let selected_index = dialog.state.selected_main_dir();
        let items: Vec<ListItem> = dialog
            .main_dirs
            .iter()
            .enumerate()
            .map(|(idx, (_, name))| {
                let icon = if idx == selected_index { "▌ " } else { "  " };
                let text = format!("{}{}", icon, name);
                let style = if idx == selected_index {
                    Style::default()
                        .fg(dialog.theme.colors.selected_fg.into())
                        .bg(dialog.theme.colors.selected_bg.into())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(dialog.theme.colors.foreground.into())
                };
                ListItem::new(text).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(dialog.theme.selected_style());

        Widget::render(list, chunks[2], buf);
    } else if let Some((path, _)) = dialog.main_dirs.first() {
        // Single main directory - just show it
        let dir_text = format!("Target directory: {}", path.display());
        let dir_para = Paragraph::new(dir_text)
            .style(Style::default().fg(dialog.theme.colors.text_muted.into()));
        dir_para.render(chunks[2], buf);
    }

    // Help text
    let help_text = if dialog.main_dirs.len() > 1 {
        "[Enter] Confirm  [Esc] Cancel  [↑↓] Select directory"
    } else {
        "[Enter] Confirm  [Esc] Cancel"
    };
    let help = Paragraph::new(help_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(dialog.theme.colors.text_muted.into()));
    help.render(chunks[3], buf);
}

/// Render confirm replace stage
fn render_confirm_replace(
    dialog: &CloneDialog,
    existing_path: &PathBuf,
    area: Rect,
    buf: &mut Buffer,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Warning icon + message
            Constraint::Length(2), // Path display
            Constraint::Length(2), // Question
            Constraint::Length(3), // Options
            Constraint::Length(1), // Help
        ])
        .split(area);

    // Warning message
    let warning = Paragraph::new("⚠️  Folder already exists:")
        .style(Style::default().fg(dialog.theme.colors.warning.into()));
    warning.render(chunks[0], buf);

    // Path
    let path_text = format!("   {}", existing_path.display());
    let path_para =
        Paragraph::new(path_text).style(Style::default().fg(dialog.theme.colors.foreground.into()));
    path_para.render(chunks[1], buf);

    // Question
    let question = Paragraph::new("Do you want to remove it and re-clone?")
        .style(Style::default().fg(dialog.theme.colors.foreground.into()));
    question.render(chunks[2], buf);

    // Options
    let options_text = "[Y] Yes, remove and re-clone\n[N] No, cancel";
    let options =
        Paragraph::new(options_text).style(Style::default().fg(dialog.theme.colors.primary.into()));
    options.render(chunks[3], buf);

    // Help
    let help = Paragraph::new("Press Y to confirm, N or Esc to cancel")
        .alignment(Alignment::Center)
        .style(Style::default().fg(dialog.theme.colors.text_muted.into()));
    help.render(chunks[4], buf);
}

/// Render executing stage
fn render_executing(dialog: &CloneDialog, area: Rect, buf: &mut Buffer) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // URL and target info
            Constraint::Length(2), // Spacing
            Constraint::Min(5),    // Progress output
            Constraint::Length(1), // Help
        ])
        .split(area);

    // URL info
    let url_info = format!("URL: {}", dialog.state.url_input);
    let url_para =
        Paragraph::new(url_info).style(Style::default().fg(dialog.theme.colors.foreground.into()));
    url_para.render(chunks[0], buf);

    // Target info (if available)
    if let Some(preview) = &dialog.folder_preview {
        let target_info = format!("Target: {}", preview);
        let target_para = Paragraph::new(target_info)
            .style(Style::default().fg(dialog.theme.colors.text_muted.into()));
        target_para.render(chunks[1], buf);
    }

    // Progress output
    let progress_text = if dialog.state.progress_lines.is_empty() {
        "Starting...".to_string()
    } else {
        // Show last 20 lines to fit in the area
        let start = dialog.state.progress_lines.len().saturating_sub(20);
        dialog.state.progress_lines[start..].join("\n")
    };

    let progress_block = Block::default()
        .borders(Borders::ALL)
        .title("Progress")
        .border_style(Style::default().fg(dialog.theme.colors.border.into()));

    let progress_para = Paragraph::new(progress_text)
        .block(progress_block)
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(dialog.theme.colors.foreground.into()));

    progress_para.render(chunks[2], buf);

    // Help
    let help = Paragraph::new("[Esc] Cancel")
        .alignment(Alignment::Center)
        .style(Style::default().fg(dialog.theme.colors.text_muted.into()));
    help.render(chunks[3], buf);
}

/// Render error stage
fn render_error(
    dialog: &CloneDialog,
    error: &crate::error::CloneError,
    area: Rect,
    buf: &mut Buffer,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Error title
            Constraint::Min(3),    // Error message
            Constraint::Length(2), // Help
        ])
        .split(area);

    // Error title
    let title = Paragraph::new("❌ Failed to clone repository")
        .style(Style::default().fg(dialog.theme.colors.error.into()));
    title.render(chunks[0], buf);

    // Error message
    let error_text = format!("Error: {}", error.user_message());
    let error_para = Paragraph::new(error_text)
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(dialog.theme.colors.foreground.into()));
    error_para.render(chunks[1], buf);

    // Help
    let help = Paragraph::new("[Enter] OK  [R] Retry  [Esc] Cancel")
        .alignment(Alignment::Center)
        .style(Style::default().fg(dialog.theme.colors.text_muted.into()));
    help.render(chunks[2], buf);
}

/// Create centered rectangle for clone dialog
pub fn clone_dialog_rect(area: Rect) -> Rect {
    use ratatui::layout::{Constraint, Direction, Layout};

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(15),
            Constraint::Percentage(70),
            Constraint::Percentage(15),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(15),
            Constraint::Percentage(70),
            Constraint::Percentage(15),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;

    fn create_test_clone_state() -> CloneState {
        CloneState::new()
    }

    #[test]
    fn test_clone_dialog_new() {
        let state = create_test_clone_state();
        let theme = Theme::dark();
        let main_dirs: Vec<(PathBuf, String)> = vec![(
            PathBuf::from("/home/user/projects"),
            "~/projects".to_string(),
        )];

        let dialog = CloneDialog::new(&state, &theme, &main_dirs);
        assert!(dialog.folder_preview.is_none());
        assert!(dialog.validation_error.is_none());
    }

    #[test]
    fn test_clone_dialog_with_preview() {
        let state = create_test_clone_state();
        let theme = Theme::dark();
        let main_dirs: Vec<(PathBuf, String)> = vec![];

        let dialog = CloneDialog::new(&state, &theme, &main_dirs)
            .folder_preview(Some("github_user_repo".to_string()));

        assert_eq!(dialog.folder_preview, Some("github_user_repo".to_string()));
    }

    #[test]
    fn test_clone_dialog_render_input() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let state = create_test_clone_state();
        let theme = Theme::dark();
        let main_dirs: Vec<(PathBuf, String)> = vec![(
            PathBuf::from("/home/user/projects"),
            "~/projects".to_string(),
        )];

        terminal
            .draw(|f| {
                let area = f.area();
                let dialog = CloneDialog::new(&state, &theme, &main_dirs);
                f.render_widget(dialog, area);
            })
            .unwrap();
    }

    #[test]
    fn test_clone_dialog_render_with_error() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut state = create_test_clone_state();
        state.stage = CloneStage::Error(crate::error::CloneError::InvalidUrl("test".to_string()));
        let theme = Theme::dark();
        let main_dirs: Vec<(PathBuf, String)> = vec![];

        terminal
            .draw(|f| {
                let area = f.area();
                let dialog = CloneDialog::new(&state, &theme, &main_dirs);
                f.render_widget(dialog, area);
            })
            .unwrap();
    }

    #[test]
    fn test_clone_dialog_rect() {
        let area = Rect::new(0, 0, 100, 50);
        let rect = clone_dialog_rect(area);

        // Check that the rect is centered and smaller than the original area
        assert!(rect.width < area.width);
        assert!(rect.height < area.height);
        assert!(rect.x > 0);
        assert!(rect.y > 0);
    }
}
