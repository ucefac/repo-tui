//! Main UI rendering

use crate::app::model::App;
use crate::app::state::AppState;
use crate::ui::theme::Theme;
use crate::ui::widgets::{
    centered_help_popup, centered_popup, centered_rect, ActionMenu, DirChooser, HelpPanel,
    RepoList, SearchBox, ThemeSelector,
};
use ratatui::prelude::*;
use ratatui::widgets::{Clear, Paragraph};

/// Render the application UI
pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    // Check minimum terminal size
    if area.width < crate::constants::MIN_TERMINAL_WIDTH
        || area.height < crate::constants::MIN_TERMINAL_HEIGHT
    {
        render_size_warning(frame, area, &Theme::dark());
        return;
    }

    // Get theme - use the already-applied theme from app state
    // Clone to avoid borrow conflicts with render functions that need &mut App
    let theme = app.theme.clone();

    // Clone state for matching (to avoid borrow conflicts with app)
    let state_clone = app.state.clone();

    // Render based on state
    match state_clone {
        AppState::Loading { ref message } => {
            render_loading(frame, area, message, &theme);
        }
        AppState::Error { ref message } => {
            render_error(frame, area, message, &theme);
        }
        AppState::ChoosingDir {
            ref path,
            ref entries,
            selected_index,
            scroll_offset,
        } => {
            render_directory_chooser(
                frame,
                area,
                path,
                entries,
                selected_index,
                scroll_offset,
                &theme,
            );
        }
        AppState::Running => {
            render_main_ui(frame, area, app, &theme);
        }
        AppState::ShowingActions { ref repo } => {
            render_main_ui(frame, area, app, &theme);
            render_action_menu(frame, area, repo, &theme);
        }
        AppState::ShowingHelp => {
            render_main_ui(frame, area, app, &theme);
            render_help(frame, area, &theme);
        }
        AppState::SelectingTheme { .. } => {
            render_main_ui(frame, area, app, &theme);
            render_theme_selector(frame, area, app, &theme);
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
        .style(Style::default().fg(theme.colors.primary.into()));

    frame.render_widget(paragraph, area);
}

/// Render error state
fn render_error(frame: &mut Frame, area: Rect, message: &str, theme: &Theme) {
    let error_text = format!("❌ Error\n\n{}", message);
    let paragraph = Paragraph::new(error_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.colors.error.into()));

    frame.render_widget(paragraph, area);
}

/// Render main UI
fn render_main_ui(frame: &mut Frame, area: Rect, app: &mut App, theme: &Theme) {
    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search box
            Constraint::Min(5),    // Repository list
            Constraint::Length(2), // Status bar (status + path)
        ])
        .split(area);

    // Render search box using component
    let is_search_focused = app.search_active;
    let search_box = SearchBox::new(&app.search_query, theme, is_search_focused);
    frame.render_widget(search_box, chunks[0]);

    // Render repository list using component
    let favorites_set: std::collections::HashSet<usize> = app
        .favorites
        .get_all()
        .iter()
        .filter_map(|fav_path| {
            app.repositories
                .iter()
                .position(|r| r.path.to_string_lossy() == *fav_path)
        })
        .collect();

    let repo_list = RepoList::new(&app.repositories, &app.filtered_indices, theme)
        .selected_index(app.selected_index())
        .scroll_offset(app.scroll_offset)
        .visible_height(chunks[1].height)
        .show_git_status(app.config.as_ref().is_some_and(|c| c.ui.show_git_status))
        .favorites(favorites_set)
        .selection_mode(app.selection_mode)
        .selected(app.selected_indices.clone())
        .area_width(chunks[1].width);
    frame.render_widget(repo_list, chunks[1]);

    // Render status bar (with path bar)
    render_status_bar_with_path(frame, app, chunks[2], theme);
}

/// Render status bar with path bar
fn render_status_bar_with_path(frame: &mut Frame, app: &mut App, area: Rect, theme: &Theme) {
    use crate::ui::widgets::StatusBar;

    let status_text = if app.loading {
        app.loading_message.as_deref().unwrap_or("Loading...")
    } else if let Some(ref error) = app.error_message {
        error
    } else {
        // Optimized spacing: reduce space between keys and description
        "[j/k]Nav [g/G]Jump [/]Search [Enter]Open [r]Refresh [?]Help [q]Quit [m]ChangeDir"
    };

    let mut status_bar = StatusBar::new(status_text, theme)
        .loading(app.loading)
        .error(app.error_message.is_some());

    if let Some(ref main_dir) = app.main_dir {
        status_bar = status_bar.path(main_dir).repo_count(app.repositories.len());
    }

    // Store click area for mouse events
    app.path_bar_area = Some(area);

    frame.render_widget(status_bar, area);
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

/// Render theme selector
fn render_theme_selector(frame: &mut Frame, area: Rect, app: &mut App, theme: &Theme) {
    use crate::ui::themes::THEME_NAMES;

    let popup_area = centered_rect(60, 55, area);

    // Clear background for modal
    frame.render_widget(Clear, popup_area);

    // Get current theme for comparison
    let current_theme = theme.clone();

    // Get preview theme from app state (stored in SelectingTheme state)
    let preview_theme = if let Some(preview) = app.state.preview_theme() {
        preview.clone()
    } else {
        // Fallback (shouldn't happen)
        Theme::dark()
    };

    // Get theme list state
    if let Some(theme_list_state) = app.state.theme_list_state_mut() {
        let selected_index = theme_list_state.selected().unwrap_or(0);
        let selector =
            ThemeSelector::new(THEME_NAMES, selected_index, &current_theme, preview_theme)
                .title("🎨 Select Theme");

        frame.render_widget(selector, popup_area);
    }
}
