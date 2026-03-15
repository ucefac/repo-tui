//! Mouse event handler

use crate::app::model::App;
use crate::app::msg::AppMsg;
use crate::app::state::AppState;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

/// Handle mouse event
pub fn handle_mouse_event(event: MouseEvent, app: &App) -> Option<AppMsg> {
    match event.kind {
        // Existing click handling
        MouseEventKind::Down(MouseButton::Left) => {
            // Check if clicked on path bar
            if let Some(path_area) = app.path_bar_area {
                if event.column >= path_area.x
                    && event.column < path_area.x + path_area.width
                    && event.row >= path_area.y
                    && event.row < path_area.y + path_area.height
                {
                    if let Some(ref path) = app.main_dir {
                        return Some(AppMsg::CopyPathToClipboard(path.clone()));
                    }
                }
            }
        }

        // Scroll wheel up - navigate up/previous
        MouseEventKind::ScrollUp => {
            return Some(get_scroll_up_message(app));
        }

        // Scroll wheel down - navigate down/next
        MouseEventKind::ScrollDown => {
            return Some(get_scroll_down_message(app));
        }

        _ => {}
    }
    None
}

/// Get the message to send for scroll up event based on current state
fn get_scroll_up_message(app: &App) -> AppMsg {
    match &app.state {
        AppState::Running => AppMsg::PreviousRepo,
        AppState::ChoosingDir { .. } => AppMsg::DirectoryNavUp,
        AppState::SelectingTheme { .. } => AppMsg::ThemeNavUp,
        AppState::ManagingDirs { .. } => AppMsg::MainDirNavUp,
        AppState::ShowingHelp { .. } => AppMsg::ScrollUp,
        AppState::SelectingMoveTarget { .. } => AppMsg::MoveTargetNavUp,
        AppState::Cloning { .. } => AppMsg::ClonePreviousMainDir,
        AppState::Loading { .. } | AppState::Error { .. } | AppState::Quit => AppMsg::PreviousRepo,
        AppState::ConfirmingDeleteRepo { .. } => AppMsg::PreviousRepo,
    }
}

/// Get the message to send for scroll down event based on current state
fn get_scroll_down_message(app: &App) -> AppMsg {
    match &app.state {
        AppState::Running => AppMsg::NextRepo,
        AppState::ChoosingDir { .. } => AppMsg::DirectoryNavDown,
        AppState::SelectingTheme { .. } => AppMsg::ThemeNavDown,
        AppState::ManagingDirs { .. } => AppMsg::MainDirNavDown,
        AppState::ShowingHelp { .. } => AppMsg::ScrollDown,
        AppState::SelectingMoveTarget { .. } => AppMsg::MoveTargetNavDown,
        AppState::Cloning { .. } => AppMsg::CloneNextMainDir,
        AppState::Loading { .. } | AppState::Error { .. } | AppState::Quit => AppMsg::NextRepo,
        AppState::ConfirmingDeleteRepo { .. } => AppMsg::NextRepo,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    #[test]
    fn test_mouse_click_on_path_bar() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx);

        // Setup path bar area
        app.path_bar_area = Some(Rect::new(0, 20, 80, 1));
        app.main_dir = Some(std::path::PathBuf::from("/tmp/test"));

        let event = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 20,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };

        let msg = handle_mouse_event(event, &app);
        assert!(matches!(msg, Some(AppMsg::CopyPathToClipboard(_))));
    }

    #[test]
    fn test_mouse_click_outside_path_bar() {
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let mut app = App::new(tx);

        app.path_bar_area = Some(Rect::new(0, 20, 80, 1));

        let event = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 10, // Outside path bar
            modifiers: crossterm::event::KeyModifiers::NONE,
        };

        let msg = handle_mouse_event(event, &app);
        assert!(msg.is_none());
    }
}
