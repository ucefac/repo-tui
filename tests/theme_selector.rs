//! Theme selector integration tests

use repotui::app::model::App;
use repotui::app::msg::AppMsg;
use repotui::app::state::AppState;
use repotui::app::update;
use repotui::config;
use repotui::runtime::executor::Runtime;
use repotui::ui::theme::Theme;
use repotui::ui::themes::THEME_NAMES;
use tokio::sync::mpsc;

#[tokio::test(flavor = "multi_thread")]
async fn test_theme_selector_opens() {
    let (tx, _rx) = mpsc::channel(100);
    let mut app = App::new(tx.clone());
    let runtime = Runtime::new(tx);

    app.state = AppState::Running;

    // Open theme selector
    update::update(AppMsg::OpenThemeSelector, &mut app, &runtime);

    // Verify state changed to SelectingTheme
    assert!(matches!(app.state, AppState::SelectingTheme { .. }));
    assert!(app.state.is_modal());
    assert_eq!(app.state.priority(), 3);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_theme_selector_closes() {
    let (tx, _rx) = mpsc::channel(100);
    let mut app = App::new(tx.clone());
    let runtime = Runtime::new(tx);

    app.state = AppState::SelectingTheme {
        theme_list_state: ratatui::widgets::ListState::default(),
        preview_theme: Theme::dark(),
    };

    // Close theme selector
    update::update(AppMsg::CloseThemeSelector, &mut app, &runtime);

    // Verify state changed back to Running
    assert!(matches!(app.state, AppState::Running));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_theme_navigation() {
    let (tx, _rx) = mpsc::channel(100);
    let mut app = App::new(tx.clone());
    let runtime = Runtime::new(tx);

    app.state = AppState::SelectingTheme {
        theme_list_state: ratatui::widgets::ListState::default(),
        preview_theme: Theme::dark(),
    };

    // Navigate down
    update::update(AppMsg::ThemeNavDown, &mut app, &runtime);
    if let AppState::SelectingTheme {
        theme_list_state, ..
    } = &app.state
    {
        assert_eq!(theme_list_state.selected(), Some(1));
    }

    // Navigate down again
    update::update(AppMsg::ThemeNavDown, &mut app, &runtime);
    if let AppState::SelectingTheme {
        theme_list_state, ..
    } = &app.state
    {
        assert_eq!(theme_list_state.selected(), Some(2));
    }

    // Navigate up
    update::update(AppMsg::ThemeNavUp, &mut app, &runtime);
    if let AppState::SelectingTheme {
        theme_list_state, ..
    } = &app.state
    {
        assert_eq!(theme_list_state.selected(), Some(1));
    }

    // Navigate to wrap around (from position 1, go down enough to cycle back)
    // With 7 themes: 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 0 (6 moves to wrap)
    for _ in 0..(THEME_NAMES.len() - 1) {
        update::update(AppMsg::ThemeNavDown, &mut app, &runtime);
    }
    if let AppState::SelectingTheme {
        theme_list_state, ..
    } = &app.state
    {
        // Should wrap around to first item
        assert_eq!(theme_list_state.selected(), Some(0));
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_theme_selection() {
    let (tx, _rx) = mpsc::channel(100);
    let mut app = App::new(tx.clone());
    let runtime = Runtime::new(tx);

    app.state = AppState::SelectingTheme {
        theme_list_state: ratatui::widgets::ListState::default(),
        preview_theme: Theme::dark(),
    };
    app.config = Some(config::Config::default());

    // Select a theme
    update::update(AppMsg::SelectTheme("nord".to_string()), &mut app, &runtime);

    // Verify state changed back to Running
    assert!(matches!(app.state, AppState::Running));
    // Verify theme was changed
    assert_eq!(app.theme.name, "nord");
    // Verify config was updated
    assert_eq!(app.config.as_ref().unwrap().ui.theme, "nord");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_theme_selector_state_creation() {
    let (tx, _rx) = mpsc::channel(100);
    let mut app = App::new(tx.clone());
    let runtime = Runtime::new(tx);

    app.state = AppState::Running;

    // Open theme selector
    update::update(AppMsg::OpenThemeSelector, &mut app, &runtime);

    // Verify theme list state is initialized
    // Without config, defaults to index 1 (dark)
    if let AppState::SelectingTheme {
        theme_list_state, ..
    } = &app.state
    {
        assert_eq!(theme_list_state.selected(), Some(1));
    } else {
        panic!("State should be SelectingTheme");
    }
}

#[test]
fn test_theme_list_state_mut() {
    let mut state = AppState::SelectingTheme {
        theme_list_state: ratatui::widgets::ListState::default(),
        preview_theme: Theme::dark(),
    };

    // Should get mutable reference
    assert!(state.theme_list_state_mut().is_some());

    // Change to different state
    state = AppState::Running;

    // Should get None
    assert!(state.theme_list_state_mut().is_none());
}
