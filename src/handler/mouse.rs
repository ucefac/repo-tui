//! Mouse event handler

use crate::app::model::App;
use crate::app::msg::AppMsg;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

/// Handle mouse event
pub fn handle_mouse_event(event: MouseEvent, app: &App) -> Option<AppMsg> {
    if let MouseEventKind::Down(MouseButton::Left) = event.kind {
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
    None
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
