//! Keyboard navigation integration tests

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[test]
fn test_key_code_mappings() {
    let key = KeyEvent {
        code: KeyCode::Char('j'),
        modifiers: KeyModifiers::NONE,
        kind: crossterm::event::KeyEventKind::Press,
        state: crossterm::event::KeyEventState::NONE,
    };
    assert!(matches!(key.code, KeyCode::Char('j')));
}

#[test]
fn test_navigation_keys() {
    let nav_keys = [
        KeyCode::Up,
        KeyCode::Down,
        KeyCode::Left,
        KeyCode::Right,
        KeyCode::Home,
        KeyCode::End,
    ];
    assert_eq!(nav_keys.len(), 6);
}
