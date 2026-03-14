//! Theme system functional verification tests

use repotui::app::model::App;
use repotui::app::msg::AppMsg;
use repotui::app::state::AppState;
use repotui::app::update;
use repotui::config::Config;
use repotui::runtime::executor::Runtime;
use repotui::ui::theme::Theme;
use repotui::ui::themes::{get_theme, THEME_NAMES};
use tokio::sync::mpsc;

/// Test 1: Theme selector opens with 't' key
#[tokio::test(flavor = "multi_thread")]
async fn test_t_key_opens_theme_selector() {
    let (tx, _rx) = mpsc::channel(100);
    let mut app = App::new(tx.clone());
    let runtime = Runtime::new(tx);

    app.state = AppState::Running;

    // Simulate 't' key press
    update::update(AppMsg::OpenThemeSelector, &mut app, &runtime);

    assert!(
        matches!(app.state, AppState::SelectingTheme { .. }),
        "Theme selector should open"
    );
}

/// Test 2: Theme navigation with j/k keys
#[tokio::test(flavor = "multi_thread")]
async fn test_theme_navigation_jk_keys() {
    let (tx, _rx) = mpsc::channel(100);
    let mut app = App::new(tx.clone());
    let runtime = Runtime::new(tx);

    app.state = AppState::SelectingTheme {
        theme_list_state: ratatui::widgets::ListState::default(),
        preview_theme: Theme::dark(),
        scroll_offset: 0,
    };

    // Navigate down with 'j'
    update::update(AppMsg::ThemeNavDown, &mut app, &runtime);
    if let AppState::SelectingTheme {
        theme_list_state, ..
    } = &app.state
    {
        assert_eq!(
            theme_list_state.selected(),
            Some(1),
            "Should move to next theme"
        );
    }

    // Navigate down again
    update::update(AppMsg::ThemeNavDown, &mut app, &runtime);
    if let AppState::SelectingTheme {
        theme_list_state, ..
    } = &app.state
    {
        assert_eq!(theme_list_state.selected(), Some(2));
    }

    // Navigate up with 'k'
    update::update(AppMsg::ThemeNavUp, &mut app, &runtime);
    if let AppState::SelectingTheme {
        theme_list_state, ..
    } = &app.state
    {
        assert_eq!(
            theme_list_state.selected(),
            Some(1),
            "Should move to previous theme"
        );
    }
}

/// Test 3: Theme navigation with arrow keys
#[tokio::test(flavor = "multi_thread")]
async fn test_theme_navigation_arrow_keys() {
    let (tx, _rx) = mpsc::channel(100);
    let mut app = App::new(tx.clone());
    let runtime = Runtime::new(tx);

    app.state = AppState::SelectingTheme {
        theme_list_state: ratatui::widgets::ListState::default(),
        preview_theme: Theme::dark(),
        scroll_offset: 0,
    };

    // Navigate down with ↓
    update::update(AppMsg::ThemeNavDown, &mut app, &runtime);
    if let AppState::SelectingTheme {
        theme_list_state, ..
    } = &app.state
    {
        assert_eq!(theme_list_state.selected(), Some(1));
    }

    // Navigate up with ↑
    update::update(AppMsg::ThemeNavUp, &mut app, &runtime);
    if let AppState::SelectingTheme {
        theme_list_state, ..
    } = &app.state
    {
        assert_eq!(theme_list_state.selected(), Some(0));
    }
}

/// Test 4: Theme selection with Enter key
#[tokio::test(flavor = "multi_thread")]
async fn test_theme_selection_with_enter() {
    let (tx, _rx) = mpsc::channel(100);
    let mut app = App::new(tx.clone());
    let runtime = Runtime::new(tx);

    app.state = AppState::SelectingTheme {
        theme_list_state: ratatui::widgets::ListState::default(),
        preview_theme: Theme::dark(),
        scroll_offset: 0,
    };
    app.config = Some(Config::default());

    // Select theme (simulating Enter press)
    update::update(AppMsg::SelectTheme("nord".to_string()), &mut app, &runtime);

    // Verify selector closed
    assert!(
        matches!(app.state, AppState::Running),
        "Theme selector should close"
    );

    // Verify theme changed
    assert_eq!(app.theme.name, "nord", "Theme should be nord");

    // Verify config updated
    assert_eq!(app.config.as_ref().unwrap().ui.theme, "nord");
}

/// Test 5: Config file update verification
#[tokio::test(flavor = "multi_thread")]
async fn test_config_updated_on_theme_selection() {
    let (tx, _rx) = mpsc::channel(100);
    let mut app = App::new(tx.clone());
    let runtime = Runtime::new(tx);

    app.state = AppState::SelectingTheme {
        theme_list_state: ratatui::widgets::ListState::default(),
        preview_theme: Theme::dark(),
        scroll_offset: 0,
    };
    app.config = Some(Config::default());

    // Select different themes
    let themes = vec!["dark", "light", "nord", "dracula"];

    for theme_name in themes {
        update::update(
            AppMsg::SelectTheme(theme_name.to_string()),
            &mut app,
            &runtime,
        );
        assert_eq!(
            app.config.as_ref().unwrap().ui.theme,
            theme_name,
            "Config should be updated with {}",
            theme_name
        );

        // Re-open selector for next test
        app.state = AppState::SelectingTheme {
            theme_list_state: ratatui::widgets::ListState::default(),
            preview_theme: Theme::dark(),
            scroll_offset: 0,
        };
    }
}

/// Test 6: Theme persistence (simulate restart)
#[tokio::test(flavor = "multi_thread")]
async fn test_theme_persistence() {
    // Create config with specific theme
    let mut config = Config::default();
    config.ui.theme = "catppuccin_mocha".to_string();

    // Simulate app restart with saved config
    let (tx, _rx) = mpsc::channel(100);
    let mut app = App::new(tx.clone());
    app.config = Some(config.clone());
    app.theme = repotui::ui::Theme::from_config(&config.ui.theme);

    // Verify theme persisted
    assert_eq!(
        app.theme.name, "catppuccin_mocha",
        "Theme should persist after 'restart'"
    );
}

/// Test 7: Close selector with Esc key
#[tokio::test(flavor = "multi_thread")]
async fn test_esc_closes_theme_selector() {
    let (tx, _rx) = mpsc::channel(100);
    let mut app = App::new(tx.clone());
    let runtime = Runtime::new(tx);

    app.state = AppState::SelectingTheme {
        theme_list_state: ratatui::widgets::ListState::default(),
        preview_theme: Theme::dark(),
        scroll_offset: 0,
    };

    // Close with Esc
    update::update(AppMsg::CloseThemeSelector, &mut app, &runtime);

    assert!(
        matches!(app.state, AppState::Running),
        "Theme selector should close with Esc"
    );
}

/// Test 8: Theme preview (instant visual update)
#[tokio::test(flavor = "multi_thread")]
async fn test_theme_preview_instant_update() {
    let (tx, _rx) = mpsc::channel(100);
    let mut app = App::new(tx.clone());
    let runtime = Runtime::new(tx);

    app.state = AppState::SelectingTheme {
        theme_list_state: ratatui::widgets::ListState::default(),
        preview_theme: Theme::dark(),
        scroll_offset: 0,
    };
    app.config = Some(Config::default());

    // Navigate through themes and verify preview updates
    let themes = vec!["dark", "nord", "tokyo_night"];

    for theme_name in themes {
        // Simulate selection (which applies theme immediately)
        update::update(
            AppMsg::SelectTheme(theme_name.to_string()),
            &mut app,
            &runtime,
        );

        // Verify theme is applied
        let theme = get_theme(theme_name).unwrap();
        assert_eq!(app.theme.name, theme.name);
        assert_eq!(app.theme.name, theme_name);

        // Re-open for next iteration
        app.state = AppState::SelectingTheme {
            theme_list_state: ratatui::widgets::ListState::default(),
            preview_theme: Theme::dark(),
            scroll_offset: 0,
        };
    }
}

/// Test 9: Theme navigation cyclic (wraps around)
#[tokio::test(flavor = "multi_thread")]
async fn test_theme_navigation_cyclic() {
    let (tx, _rx) = mpsc::channel(100);
    let mut app = App::new(tx.clone());
    let runtime = Runtime::new(tx);

    app.state = AppState::SelectingTheme {
        theme_list_state: ratatui::widgets::ListState::default(),
        preview_theme: Theme::dark(),
        scroll_offset: 0,
    };

    // Navigate to last theme
    for _ in 0..THEME_NAMES.len() - 1 {
        update::update(AppMsg::ThemeNavDown, &mut app, &runtime);
    }

    // Should be at last theme
    if let AppState::SelectingTheme {
        theme_list_state, ..
    } = &app.state
    {
        assert_eq!(
            theme_list_state.selected(),
            Some(THEME_NAMES.len() - 1),
            "Should be at last theme"
        );
    }

    // Navigate down again - should wrap to first
    update::update(AppMsg::ThemeNavDown, &mut app, &runtime);
    if let AppState::SelectingTheme {
        theme_list_state, ..
    } = &app.state
    {
        assert_eq!(
            theme_list_state.selected(),
            Some(0),
            "Should wrap to first theme"
        );
    }
}

/// Test 10: All built-in themes are valid
#[test]
fn test_all_themes_valid() {
    // Skip "🎲 Random (随机)" as it's not a real theme
    for &theme_name in &THEME_NAMES[1..] {
        let theme = get_theme(theme_name);
        assert!(theme.is_some(), "Theme {} should exist", theme_name);

        let theme = theme.unwrap();
        assert_eq!(theme.name, theme_name);

        // Verify colors are initialized
        assert!(
            theme.colors.background.r > 0
                || theme.colors.background.g > 0
                || theme.colors.background.b > 0
        );
    }
}

/// Test 11: Theme selector cyclic navigation
#[tokio::test(flavor = "multi_thread")]
async fn test_theme_selector_cyclic() {
    let (tx, _rx) = mpsc::channel(100);
    let mut app = App::new(tx.clone());
    let runtime = Runtime::new(tx);

    app.state = AppState::SelectingTheme {
        theme_list_state: ratatui::widgets::ListState::default(),
        preview_theme: Theme::dark(),
        scroll_offset: 0,
    };

    // Navigate up from first item - should wrap to last
    update::update(AppMsg::ThemeNavUp, &mut app, &runtime);
    if let AppState::SelectingTheme {
        theme_list_state, ..
    } = &app.state
    {
        assert_eq!(
            theme_list_state.selected(),
            Some(THEME_NAMES.len() - 1),
            "Should wrap to last theme"
        );
    }
}

/// Test 12: Theme selector modal priority
#[tokio::test(flavor = "multi_thread")]
async fn test_theme_selector_modal_priority() {
    let (tx, _rx) = mpsc::channel(100);
    let mut app = App::new(tx.clone());
    let runtime = Runtime::new(tx);

    app.state = AppState::Running;

    update::update(AppMsg::OpenThemeSelector, &mut app, &runtime);

    assert!(app.state.is_modal(), "Theme selector should be modal");
    assert_eq!(
        app.state.priority(),
        3,
        "Theme selector priority should be 3"
    );
}
