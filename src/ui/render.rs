//! Main UI rendering

use crate::app::model::App;
use crate::app::state::AppState;
use crate::ui::theme::Theme;
use crate::ui::widgets::{
    centered_help_popup, centered_popup, centered_rect, CloneDialog,
    DirectoryChooser, DirectoryChooserState, HelpPanel, MainDirManager, RepoList, SearchBox,
    ThemeSelector, TitleBar,
};
use ratatui::prelude::*;
use ratatui::widgets::{Clear, Paragraph};
use std::path::PathBuf;

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
            ref mode,
            return_to: _,
        } => {
            render_directory_chooser(
                frame,
                area,
                path,
                entries,
                selected_index,
                scroll_offset,
                mode,
                &theme,
            );
        }
        AppState::ManagingDirs { .. } => {
            render_main_ui(frame, area, app, &theme);
            render_main_dir_manager(frame, area, app, &theme);
        }
        AppState::Running => {
            render_main_ui(frame, area, app, &theme);
        }
        AppState::ShowingHelp { scroll_offset } => {
            render_main_ui(frame, area, app, &theme);
            render_help(frame, area, scroll_offset, &theme);
        }
        AppState::SelectingTheme { .. } => {
            render_main_ui(frame, area, app, &theme);
            render_theme_selector(frame, area, app, &theme);
        }
        AppState::Cloning { .. } => {
            // Phase 1 placeholder - full CloneDialog UI in Phase 2
            render_main_ui(frame, area, app, &theme);
            render_clone_dialog(frame, area, app, &theme);
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
    // Create layout with title bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title bar
            Constraint::Length(3), // Search box
            Constraint::Min(5),    // Repository list
            Constraint::Length(1), // Action hints
            Constraint::Length(2), // Status bar (2 rows: status + path)
        ])
        .split(area);

    // Render title bar
    let title_bar = TitleBar::new(&app.view_mode, theme, &app.update_status)
        .update_dismissed(app.update_notification_dismissed);
    let title_bar = if app.selection_mode {
        title_bar.selection_info(app.selected_count())
    } else {
        title_bar
    };
    frame.render_widget(title_bar, chunks[0]);

    // Render search box using component
    let is_search_focused = app.search_active;
    let search_box = SearchBox::new(&app.search_query, theme, is_search_focused);
    frame.render_widget(search_box, chunks[1]);

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
        .visible_height(chunks[2].height)
        .show_git_status(app.config.as_ref().is_some_and(|c| c.ui.show_git_status))
        .favorites(favorites_set)
        .selection_mode(app.selection_mode)
        .selected(app.selected_indices.clone())
        .area_width(chunks[2].width);
    frame.render_widget(repo_list, chunks[2]);

    // Render action hints
    render_action_hints(frame, chunks[3], app, theme);

    // Render status bar (with path bar)
    render_status_bar_with_path(frame, app, chunks[4], theme);
}

/// Render status bar with path bar
fn render_status_bar_with_path(frame: &mut Frame, app: &mut App, area: Rect, theme: &Theme) {
    use crate::ui::widgets::StatusBar;

    // Store click area for mouse events before any borrows
    app.path_bar_area = Some(area);

    let status_text = if app.loading {
        app.loading_message.as_deref().unwrap_or("Loading...")
    } else if let Some(ref error) = app.error_message {
        error
    } else {
        "↑↓ navigate   / search   r refresh   ? help   Ctrl+C quit"
    };

    let mut status_bar = StatusBar::new(status_text, theme)
        .loading(app.loading)
        .error(app.error_message.is_some());

    // Display selected repository path, fall back to main_dir if no repository selected
    // Clone the path to avoid borrow checker issues
    let path_to_display = if let Some(repo) = app.selected_repository() {
        Some(repo.path.clone())
    } else {
        app.main_dir.clone()
    };

    if let Some(ref path) = path_to_display {
        status_bar = status_bar.path(path).repo_count(app.repositories.len());
    }

    frame.render_widget(status_bar, area);
}

/// Render action hints bar
fn render_action_hints(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    use ratatui::text::{Line, Span};

    // Only show in Running state and when search is not active
    if app.search_active || !matches!(app.state, AppState::Running) {
        return;
    }

    // Don't show when repository list is empty
    if app.repositories.is_empty() || app.filtered_indices.is_empty() {
        return;
    }

    let hints: Vec<(char, &str)> = vec![
        ('1', "Claude Code"),
        ('2', "WebStorm"),
        ('3', "VS Code"),
        ('4', "Finder"),
        ('5', "IntelliJ"),
        ('6', "OpenCode"),
    ];

    // Build styled spans with key hints highlighted (same style as status bar)
    let mut spans = Vec::new();
    for (i, (key, desc)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("   "));
        }
        // Format: [1] Claude Code - highlight "[1]" with primary color
        let key_hint = format!("[{}]", key);
        spans.push(Span::styled(
            key_hint,
            Style::default().fg(theme.colors.primary.into()),
        ));
        spans.push(Span::raw(format!(" {}", desc)));
    }

    let paragraph = Paragraph::new(Line::from(spans))
        .style(Style::default().fg(theme.colors.text_muted.into()))
        .alignment(Alignment::Left);

    frame.render_widget(paragraph, area);
}

/// Render help panel
fn render_help(frame: &mut Frame, area: Rect, scroll_offset: usize, _theme: &Theme) {
    let popup_area = centered_help_popup(area);

    // Render help panel widget (includes Clear widget internally)
    let mut panel = HelpPanel::new();
    panel.scroll_offset = scroll_offset;
    panel.render(frame, popup_area);
}

/// Render directory chooser using component
#[allow(clippy::too_many_arguments)]
fn render_directory_chooser(
    frame: &mut Frame,
    area: Rect,
    path: &std::path::Path,
    entries: &[String],
    selected_index: usize,
    scroll_offset: usize,
    mode: &crate::app::state::DirectoryChooserMode,
    theme: &Theme,
) {
    let popup_area = centered_rect(80, 80, area);

    // Clear background for modal
    frame.render_widget(Clear, popup_area);

    // Create state for the chooser
    let state = DirectoryChooserState {
        current_path: path.to_path_buf(),
        entries: entries.to_vec(),
        selected_index,
        scroll_offset,
        mode: mode.clone(),
    };

    // Use DirectoryChooser component with scroll support
    let chooser =
        DirectoryChooser::new(&state, theme).visible_height(popup_area.height.saturating_sub(10));

    frame.render_widget(chooser, popup_area);
}

/// Render main directory manager
fn render_main_dir_manager(frame: &mut Frame, area: Rect, app: &mut App, theme: &Theme) {
    // Check if we're in delete confirmation mode
    let is_confirming = if let AppState::ManagingDirs { confirming_delete, .. } = &app.state {
        *confirming_delete
    } else {
        false
    };

    // Full screen display (not popup/modal)
    // Clear the entire area
    frame.render_widget(Clear, area);

    if let AppState::ManagingDirs {
        selected_dir_index,
        editing,
        ..
    } = &app.state
    {
        let manager = MainDirManager::new(&app.main_directories, *selected_dir_index, theme);

        // If editing, add editing state
        let manager = if let Some(edit) = editing {
            manager.editing(edit.index.unwrap_or(0), &edit.display_name)
        } else {
            manager
        };

        frame.render_widget(manager, area);
    }

    // If confirming delete, render confirmation dialog as overlay on top
    if is_confirming {
        render_delete_confirmation_dialog(frame, area, app, theme);
    }
}

/// Render delete confirmation dialog
fn render_delete_confirmation_dialog(
    frame: &mut Frame,
    area: Rect,
    app: &mut App,
    theme: &Theme,
) {
    // Create a centered popup for the confirmation dialog
    let popup_area = centered_popup(50, 30, area);

    // Clear background
    frame.render_widget(Clear, popup_area);

    // Get the directory name being deleted
    let dir_name = if let AppState::ManagingDirs { selected_dir_index, .. } = &app.state {
        app.main_directories
            .get(*selected_dir_index)
            .map(|d| d.display_name.clone())
            .unwrap_or_else(|| "Unknown".to_string())
    } else {
        "Unknown".to_string()
    };

    // Build dialog content
    let text = format!(
        "⚠️  Delete Main Directory\n\n\"{}\"\n\nThis will remove the directory from the list.\nRepositories will not be deleted.\n\n[y] Confirm  [n] Cancel",
        dir_name
    );

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true })
        .style(
            Style::default()
                .fg(theme.colors.foreground.into())
                .bg(theme.colors.background.into()),
        )
        .block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .border_style(Style::default().fg(theme.colors.error.into()))
                .title("Confirm Delete"),
        );

    frame.render_widget(paragraph, popup_area);
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

/// Render clone dialog using the CloneDialog component
fn render_clone_dialog(frame: &mut Frame, area: Rect, app: &mut App, theme: &Theme) {
    use crate::ui::widgets::clone_dialog_rect;

    let popup_area = clone_dialog_rect(area);

    // Get clone state
    let clone_state = match app.state.clone_state() {
        Some(state) => state,
        None => return,
    };

    // Build main directories list for display
    let main_dirs: Vec<(PathBuf, String)> = app
        .main_directories
        .iter()
        .filter(|d| d.enabled)
        .map(|d| (d.path.clone(), d.display_name.clone()))
        .collect();

    // Generate folder preview if URL is parsed
    let folder_preview = clone_state.parsed_url.as_ref().map(|parsed| {
        let folder_name = crate::repo::clone::generate_folder_name(parsed);
        if let Some(target_idx) = clone_state.target_main_dir {
            if let Some((path, _)) = main_dirs.get(target_idx) {
                format!("{}/{}", path.display(), folder_name)
            } else {
                folder_name
            }
        } else if let Some((path, _)) = main_dirs.first() {
            format!("{}/{}", path.display(), folder_name)
        } else {
            folder_name
        }
    });

    // Create and render the dialog
    let dialog = CloneDialog::new(clone_state, theme, &main_dirs)
        .folder_preview(folder_preview)
        .validation_error(None); // TODO: Add validation error from state

    frame.render_widget(dialog, popup_area);
}
