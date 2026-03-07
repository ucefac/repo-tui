//! Theme selector integration tests

use repotui::app::model::App;
use repotui::app::msg::AppMsg;
use repotui::app::state::AppState;
use repotui::app::update;
use repotui::config;
use repotui::runtime::executor::Runtime;
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
    };

    // Navigate down
    update::update(AppMsg::ThemeNavDown, &mut app, &runtime);
    if let AppState::SelectingTheme { theme_list_state } = &app.state {
        assert_eq!(theme_list_state.selected(), Some(1));
    }

    // Navigate down again
    update::update(AppMsg::ThemeNavDown, &mut app, &runtime);
    if let AppState::SelectingTheme { theme_list_state } = &app.state {
        assert_eq!(theme_list_state.selected(), Some(2));
    }

    // Navigate up
    update::update(AppMsg::ThemeNavUp, &mut app, &runtime);
    if let AppState::SelectingTheme { theme_list_state } = &app.state {
        assert_eq!(theme_list_state.selected(), Some(1));
    }

    // Navigate to boundary
    for _ in 0..10 {
        update::update(AppMsg::ThemeNavDown, &mut app, &runtime);
    }
    if let AppState::SelectingTheme { theme_list_state } = &app.state {
        // Should be at last item
        assert_eq!(theme_list_state.selected(), Some(THEME_NAMES.len() - 1));
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_theme_selection() {
    let (tx, _rx) = mpsc::channel(100);
    let mut app = App::new(tx.clone());
    let runtime = Runtime::new(tx);

    app.state = AppState::SelectingTheme {
        theme_list_state: ratatui::widgets::ListState::default(),
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
    if let AppState::SelectingTheme { theme_list_state } = &app.state {
        assert_eq!(theme_list_state.selected(), Some(0));
    } else {
        panic!("State should be SelectingTheme");
    }
}

#[test]
fn test_theme_list_state_mut() {
    let mut state = AppState::SelectingTheme {
        theme_list_state: ratatui::widgets::ListState::default(),
    };

    // Should get mutable reference
    assert!(state.theme_list_state_mut().is_some());

    // Change to different state
    state = AppState::Running;

    // Should get None
    assert!(state.theme_list_state_mut().is_none());
}
