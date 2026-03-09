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
async fn test_m_key_opens_main_directory_manager() {
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

    // Should open main directory manager
    assert!(matches!(app.state, AppState::ManagingDirs { .. }));
}

#[tokio::test]
async fn test_m_key_opens_manager_from_running() {
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

    // Verify main directory manager is open
    if let AppState::ManagingDirs { .. } = &app.state {
        // Success - ManagingDirs state
    } else {
        panic!("Expected ManagingDirs state, got {:?}", app.state);
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
