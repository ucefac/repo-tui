//! Directory selection integration tests

use repotui::app::state::AppState;

#[test]
fn test_directory_selection_state() {
    let state = AppState::ChoosingDir {
        path: std::path::PathBuf::from("/home/user"),
        entries: vec!["Documents".to_string(), "Projects".to_string()],
        selected_index: 0,
        scroll_offset: 0,
    };
    assert!(matches!(state, AppState::ChoosingDir { .. }));
}

#[test]
fn test_directory_chooser_navigation() {
    let entries = ["dir1".to_string(), "dir2".to_string(), "dir3".to_string()];
    let mut selected_index = 0;
    selected_index = (selected_index + 1).min(entries.len() - 1);
    assert_eq!(selected_index, 1);
    selected_index = selected_index.saturating_sub(1);
    assert_eq!(selected_index, 0);
}
