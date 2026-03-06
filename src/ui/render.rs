//! Main UI rendering

use crate::app::model::App;
use crate::app::state::AppState;
use crate::ui::theme::Theme;
use crate::ui::widgets::{
    centered_help_popup, centered_popup, centered_rect, ActionMenu, DirChooser, HelpPanel,
    RepoList, SearchBox,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

/// Render the application UI
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Check minimum terminal size
    if area.width < crate::constants::MIN_TERMINAL_WIDTH
        || area.height < crate::constants::MIN_TERMINAL_HEIGHT
    {
        render_size_warning(frame, area, &Theme::dark());
        return;
    }

    // Get theme
    let theme = Theme::from_config(
        app.config
            .as_ref()
            .map(|c| c.ui.theme.as_str())
            .unwrap_or("dark"),
    );

    // Render based on state
    match &app.state {
        AppState::Loading { message } => {
            render_loading(frame, area, message, &theme);
        }
        AppState::Error { message } => {
            render_error(frame, area, message, &theme);
        }
        AppState::ChoosingDir {
            path,
            entries,
            selected_index,
            scroll_offset,
        } => {
            render_directory_chooser(
                frame,
                area,
                path,
                entries,
                *selected_index,
                *scroll_offset,
                &theme,
            );
        }
        AppState::Running | AppState::Searching => {
            render_main_ui(frame, area, app, &theme);
        }
        AppState::ShowingActions { repo } => {
            render_main_ui(frame, area, app, &theme);
            render_action_menu(frame, area, repo, &theme);
        }
        AppState::ShowingHelp => {
            render_main_ui(frame, area, app, &theme);
            render_help(frame, area, &theme);
        }
        AppState::Quit => {
            // Don't render anything when quitting
        }
    }
}

/// Render size warning
fn render_size_warning(frame: &mut Frame, area: Rect, _theme: &Theme) {
    let text = format!(
        "Terminal too small!\n\nMinimum size: {}x{}\nCurrent size: {}x{}\n\nPlease resize your terminal.",
        crate::constants::MIN_TERMINAL_WIDTH,
        crate::constants::MIN_TERMINAL_HEIGHT,
        area.width,
        area.height
    );

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow));

    frame.render_widget(paragraph, area);
}

/// Render loading state
fn render_loading(frame: &mut Frame, area: Rect, message: &str, theme: &Theme) {
    let loading_text = format!("⏳ {}", message);
    let paragraph = Paragraph::new(loading_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.primary));

    frame.render_widget(paragraph, area);
}

/// Render error state
fn render_error(frame: &mut Frame, area: Rect, message: &str, theme: &Theme) {
    let error_text = format!("❌ Error\n\n{}", message);
    let paragraph = Paragraph::new(error_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.error));

    frame.render_widget(paragraph, area);
}

/// Render main UI
fn render_main_ui(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search box
            Constraint::Min(5),    // Repository list
            Constraint::Length(3), // Status bar
        ])
        .split(area);

    // Render search box using component
    let is_search_focused = matches!(app.state, AppState::Searching);
    let search_box = SearchBox::new(&app.search_query, theme, is_search_focused);
    frame.render_widget(search_box, chunks[0]);

    // Render repository list using component
    let repo_list = RepoList::new(&app.repositories, &app.filtered_indices, theme)
        .selected_index(app.selected_index())
        .scroll_offset(app.scroll_offset)
        .visible_height(chunks[1].height)
        .show_git_status(app.config.as_ref().is_some_and(|c| c.ui.show_git_status));
    frame.render_widget(repo_list, chunks[1]);

    // Render status bar
    render_status_bar(frame, app, chunks[2], theme);
}

/// Render status bar
fn render_status_bar(frame: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let status_text = if app.loading {
        format!(
            " ⏳ {}",
            app.loading_message.as_deref().unwrap_or("Loading...")
        )
    } else if let Some(ref error) = app.error_message {
        format!(" ⚠️ {}", error)
    } else {
        " [j/k] Navigate  [g/G] Jump  [/] Search  [Enter] Open  [r] Refresh  [?] Help  [q] Quit "
            .to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border_normal));

    let paragraph = Paragraph::new(status_text).block(block).style(
        Style::default()
            .fg(theme.text_secondary)
            .bg(Color::DarkGray),
    );

    frame.render_widget(paragraph, area);
}

/// Render action menu
fn render_action_menu(
    frame: &mut Frame,
    area: Rect,
    repo: &crate::repo::Repository,
    _theme: &Theme,
) {
    // Create centered popup
    let popup_area = centered_popup(50, 50, area);

    // Render action menu widget (includes Clear widget internally)
    let menu = ActionMenu::new(repo, 0);
    menu.render(frame, popup_area);
}

/// Render help panel
fn render_help(frame: &mut Frame, area: Rect, _theme: &Theme) {
    let popup_area = centered_help_popup(area);

    // Render help panel widget (includes Clear widget internally)
    let panel = HelpPanel::new();
    panel.render(frame, popup_area);
}

/// Render directory chooser using component
fn render_directory_chooser(
    frame: &mut Frame,
    area: Rect,
    path: &std::path::Path,
    entries: &[String],
    selected_index: usize,
    scroll_offset: usize,
    theme: &Theme,
) {
    let popup_area = centered_rect(80, 80, area);

    // Clear background for modal
    frame.render_widget(Clear, popup_area);

    // Use DirChooser component with scroll support
    let chooser = DirChooser::new(path, entries, selected_index, scroll_offset, theme)
        .visible_height(popup_area.height.saturating_sub(10)); // Reserve space for title, path, stats, help
    frame.render_widget(chooser, popup_area);
}
