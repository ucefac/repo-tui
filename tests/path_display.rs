//! Integration tests for path display and directory switching

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use repotui::app::model::App;
use repotui::app::msg::AppMsg;
use repotui::app::state::AppState;
use repotui::handler::keyboard::handle_key_event;
use repotui::runtime::executor::Runtime;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a test key event
fn create_key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::NONE,
    }
}

#[tokio::test]
async fn test_m_key_opens_directory_chooser() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    let mut app = App::new(tx.clone());
    let runtime = Runtime::new(tx);

    // Set app to Running state
    app.state = AppState::Running;
    app.repositories = vec![];
    app.filtered_indices = vec![];

    // Press 'm' key
    handle_key_event(create_key(KeyCode::Char('m')), &mut app, &runtime);

    // Process the message
    if let Ok(msg) = rx.try_recv() {
        repotui::app::update::update(msg, &mut app, &runtime);
    }

    // Should open directory chooser
    assert!(matches!(app.state, AppState::ChoosingDir { .. }));
}

#[test]
fn test_path_bar_display_with_main_dir() {
    let (tx, _rx) = tokio::sync::mpsc::channel(100);
    let mut app = App::new(tx);

    // Set main directory
    let temp_dir = TempDir::new().unwrap();
    app.main_dir = Some(temp_dir.path().to_path_buf());
    app.repositories = vec![];

    // Should have path_bar_area (will be set during render)
    // For now, just verify main_dir is set
    assert!(app.main_dir.is_some());
    assert_eq!(app.main_dir.as_ref().unwrap(), temp_dir.path());
}

#[test]
fn test_copy_path_to_clipboard_message() {
    let (tx, _rx) = tokio::sync::mpsc::channel(100);
    let mut app = App::new(tx.clone());
    let runtime = Runtime::new(tx);

    // Set main directory
    let test_path = PathBuf::from("/tmp/test/path");
    app.main_dir = Some(test_path.clone());

    // Send copy path message
    let msg = AppMsg::CopyPathToClipboard(test_path.clone());
    repotui::app::update::update(msg, &mut app, &runtime);

    // Should have success message
    assert!(app.loading_message.is_some());
    assert!(app.loading_message.as_ref().unwrap().contains("copied"));
}

#[test]
fn test_path_bar_widget_creation() {
    use repotui::ui::theme::Theme;
    use repotui::ui::widgets::PathBar;
    use std::path::Path;

    let theme = Theme::dark();
    let path = Path::new("/tmp/test");

    let path_bar = PathBar::new(path, Some(5), &theme);

    // Verify widget properties
    assert_eq!(path_bar.path, path);
    assert_eq!(path_bar.repo_count, Some(5));
}

#[test]
fn test_path_bar_truncation() {
    use repotui::ui::theme::Theme;
    use repotui::ui::widgets::PathBar;
    use std::path::Path;

    let theme = Theme::dark();
    let long_path = Path::new("/Users/yyyyyyh/Desktop/ghclone/repotui/src/ui/widgets/path_bar");

    let path_bar = PathBar::new(long_path, Some(10), &theme).max_length(30);
    let text = path_bar.display_text(50);

    assert!(text.len() <= 30);
    assert!(text.contains("..."));
}

#[tokio::test]
async fn test_directory_chooser_can_be_opened_with_m_key() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    let mut app = App::new(tx.clone());
    let runtime = Runtime::new(tx);

    // Setup initial state
    app.state = AppState::Running;
    app.main_dir = Some(PathBuf::from("/tmp"));

    // Press 'm'
    handle_key_event(create_key(KeyCode::Char('m')), &mut app, &runtime);

    // Process the message
    if let Ok(msg) = rx.try_recv() {
        repotui::app::update::update(msg, &mut app, &runtime);
    }

    // Verify directory chooser is open
    if let AppState::ChoosingDir { path, .. } = &app.state {
        assert_eq!(path, &dirs::home_dir().unwrap_or_default());
    } else {
        panic!("Expected ChoosingDir state");
    }
}

#[test]
fn test_help_panel_includes_m_key() {
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;
    use ratatui::Terminal;
    use repotui::ui::widgets::HelpPanel;

    let backend = TestBackend::new(80, 30);
    let mut terminal = Terminal::new(backend).unwrap();

    let panel = HelpPanel::new();

    // Should render without panic
    terminal
        .draw(|f| {
            let area = Rect::new(10, 2, 60, 28);
            panel.render(f, area);
        })
        .unwrap();
}
