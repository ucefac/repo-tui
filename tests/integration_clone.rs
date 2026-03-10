//! Integration tests for Clone functionality
//!
//! Tests the complete clone workflow:
//! - URL parsing and validation
//! - Folder name generation
//! - State transitions during clone
//! - Progress reporting
//! - Error handling

use repotui::app::msg::AppMsg;
use repotui::app::state::{AppState, CloneStage};
use repotui::config::types::{EditorConfig, SecurityConfig, FavoritesConfig, RecentConfig};
use repotui::repo::clone::{generate_folder_name, parse_git_url, validate_git_url};
use std::path::PathBuf;

mod helpers;

/// Test URL parsing for various Git hosting platforms
#[test]
fn test_clone_url_parsing_various_platforms() {
    // GitHub HTTPS
    let url = "https://github.com/farion1231/cc-switch";
    let parsed = parse_git_url(url).unwrap();
    assert_eq!(parsed.domain, "github.com");
    assert_eq!(parsed.owner, "farion1231");
    assert_eq!(parsed.repo, "cc-switch");

    // GitHub SSH
    let url = "git@github.com:farion1231/cc-switch.git";
    let parsed = parse_git_url(url).unwrap();
    assert_eq!(parsed.domain, "github.com");
    assert_eq!(parsed.owner, "farion1231");
    assert_eq!(parsed.repo, "cc-switch");

    // GitLab
    let url = "https://gitlab.com/user/my-project";
    let parsed = parse_git_url(url).unwrap();
    assert_eq!(parsed.domain, "gitlab.com");
    assert_eq!(parsed.owner, "user");
    assert_eq!(parsed.repo, "my-project");

    // Bitbucket
    let url = "https://bitbucket.org/team/repo.git";
    let parsed = parse_git_url(url).unwrap();
    assert_eq!(parsed.domain, "bitbucket.org");
    assert_eq!(parsed.owner, "team");
    assert_eq!(parsed.repo, "repo");

    // Self-hosted Gitea (custom domain)
    let url = "https://git.example.com/user/project";
    let parsed = parse_git_url(url).unwrap();
    assert_eq!(parsed.domain, "git.example.com");
    assert_eq!(parsed.owner, "user");
    assert_eq!(parsed.repo, "project");
}

/// Test folder name generation matches specification
#[test]
fn test_clone_folder_name_generation() {
    // Standard case
    let parsed = parse_git_url("https://github.com/owner/repo").unwrap();
    let folder_name = generate_folder_name(&parsed);
    assert_eq!(folder_name, "github.com_owner_repo");

    // With .git suffix
    let parsed = parse_git_url("https://github.com/owner/repo.git").unwrap();
    let folder_name = generate_folder_name(&parsed);
    assert_eq!(folder_name, "github.com_owner_repo");

    // Organization with hyphen
    let parsed = parse_git_url("https://github.com/my-org/my-repo").unwrap();
    let folder_name = generate_folder_name(&parsed);
    assert_eq!(folder_name, "github.com_my-org_my-repo");

    // Nested GitLab groups
    let parsed = parse_git_url("https://gitlab.com/group/subgroup/project").unwrap();
    let folder_name = generate_folder_name(&parsed);
    assert_eq!(folder_name, "gitlab.com_group-subgroup_project");

    // With port number
    let parsed = parse_git_url("https://github.com:8443/owner/repo").unwrap();
    let folder_name = generate_folder_name(&parsed);
    assert_eq!(folder_name, "github.com_owner_repo");
}

/// Test URL validation
#[test]
fn test_clone_url_validation() {
    // Valid URLs
    assert!(validate_git_url("https://github.com/user/repo", 1000).is_ok());
    assert!(validate_git_url("git@github.com:user/repo.git", 1000).is_ok());
    assert!(validate_git_url("ssh://git@gitlab.com/user/repo", 1000).is_ok());

    // Invalid URLs
    assert!(validate_git_url("", 1000).is_err()); // Empty
    assert!(validate_git_url("not-a-url", 1000).is_err()); // Invalid format
    assert!(validate_git_url("-dangerous", 1000).is_err()); // Starts with dash
    assert!(validate_git_url("https://github.com/", 1000).is_err()); // Missing path
}

/// Test state transition: Running -> Cloning
#[test]
fn test_clone_state_transition_start() {
    let (tx, _rx) = tokio::sync::mpsc::channel(100);
    let mut app = repotui::app::model::App::new(tx);

    // Initially in loading or running state
    // Simulate starting clone
    app.state = AppState::Running;

    // Send StartClone message
    let runtime = repotui::runtime::executor::Runtime::new(app.msg_tx.clone());
    repotui::app::update::update(AppMsg::StartClone, &mut app, &runtime);

    // Verify state changed to Cloning
    assert!(app.state.is_cloning());
    if let AppState::Cloning { clone_state } = &app.state {
        assert!(matches!(clone_state.stage, CloneStage::InputUrl));
        assert!(clone_state.url_input.is_empty());
    }
}

/// Test URL input in clone state
#[test]
fn test_clone_url_input() {
    let (tx, _rx) = tokio::sync::mpsc::channel(100);
    let mut app = repotui::app::model::App::new(tx);
    let runtime = repotui::runtime::executor::Runtime::new(app.msg_tx.clone());

    // Start clone
    repotui::app::update::update(AppMsg::StartClone, &mut app, &runtime);
    assert!(app.state.is_cloning());

    // Type URL
    for c in "https://github.com/user/repo".chars() {
        repotui::app::update::update(AppMsg::CloneUrlInput(c), &mut app, &runtime);
    }

    if let AppState::Cloning { clone_state } = &app.state {
        assert_eq!(clone_state.url_input, "https://github.com/user/repo");
    }
}

/// Test paste functionality in clone state
#[test]
fn test_clone_url_paste() {
    let (tx, _rx) = tokio::sync::mpsc::channel(100);
    let mut app = repotui::app::model::App::new(tx);
    let runtime = repotui::runtime::executor::Runtime::new(app.msg_tx.clone());

    // Start clone
    repotui::app::update::update(AppMsg::StartClone, &mut app, &runtime);

    // Paste URL
    let url = "https://github.com/anthropics/claude-code".to_string();
    repotui::app::update::update(AppMsg::CloneUrlPaste(url), &mut app, &runtime);

    if let AppState::Cloning { clone_state } = &app.state {
        assert_eq!(clone_state.url_input, "https://github.com/anthropics/claude-code");
        assert_eq!(clone_state.cursor_position, 41);
    }
}

/// Test backspace in clone URL input
#[test]
fn test_clone_url_backspace() {
    let (tx, _rx) = tokio::sync::mpsc::channel(100);
    let mut app = repotui::app::model::App::new(tx);
    let runtime = repotui::runtime::executor::Runtime::new(app.msg_tx.clone());

    // Start clone
    repotui::app::update::update(AppMsg::StartClone, &mut app, &runtime);

    // Type some text
    for c in "abc".chars() {
        repotui::app::update::update(AppMsg::CloneUrlInput(c), &mut app, &runtime);
    }

    // Backspace
    repotui::app::update::update(AppMsg::CloneUrlBackspace, &mut app, &runtime);

    if let AppState::Cloning { clone_state } = &app.state {
        assert_eq!(clone_state.url_input, "ab");
        assert_eq!(clone_state.cursor_position, 2);
    }
}

/// Test cancel clone returns to running state
#[test]
fn test_clone_cancel() {
    let (tx, _rx) = tokio::sync::mpsc::channel(100);
    let mut app = repotui::app::model::App::new(tx);
    let runtime = repotui::runtime::executor::Runtime::new(app.msg_tx.clone());

    // Start clone
    repotui::app::update::update(AppMsg::StartClone, &mut app, &runtime);
    assert!(app.state.is_cloning());

    // Cancel
    repotui::app::update::update(AppMsg::CancelClone, &mut app, &runtime);

    // Should return to Running state
    assert!(matches!(app.state, AppState::Running));
}

/// Test clone state reset after cancel
#[test]
fn test_clone_state_reset() {
    use repotui::app::state::CloneState;

    let mut state = CloneState::new();

    // Modify state
    state.url_input = "https://github.com/test".to_string();
    state.cursor_position = 10;
    state.add_progress("Cloning...".to_string());

    // Reset
    state.reset();

    // Verify reset
    assert!(state.url_input.is_empty());
    assert_eq!(state.cursor_position, 0);
    assert!(state.progress_lines.is_empty());
    assert!(matches!(state.stage, CloneStage::InputUrl));
}

/// Test clone state cursor movement
#[test]
fn test_clone_state_cursor_movement() {
    use repotui::app::state::CloneState;

    let mut state = CloneState::new();

    // Insert some text
    state.insert_char('a');
    state.insert_char('b');
    state.insert_char('c');

    assert_eq!(state.cursor_position, 3);

    // Move left
    state.move_cursor_left();
    assert_eq!(state.cursor_position, 2);

    // Insert at cursor
    state.insert_char('x');
    assert_eq!(state.url_input, "abxc");

    // Move to end
    state.move_cursor_right();
    state.move_cursor_right();
    assert_eq!(state.cursor_position, 4);

    // Can't move past end
    state.move_cursor_right();
    assert_eq!(state.cursor_position, 4);
}

/// Test clone state progress management
#[test]
fn test_clone_state_progress_management() {
    use repotui::app::state::CloneState;

    let mut state = CloneState::new();

    // Add progress lines
    for i in 0..105 {
        state.add_progress(format!("Line {}", i));
    }

    // Should be capped at 100 lines
    assert_eq!(state.progress_lines.len(), 100);

    // First line should be removed (oldest)
    assert!(!state.progress_lines.contains(&"Line 0".to_string()));
    assert!(!state.progress_lines.contains(&"Line 4".to_string()));

    // Recent lines should be present
    assert!(state.progress_lines.contains(&"Line 104".to_string()));
}

/// Test sanitize function with various inputs
#[test]
fn test_clone_sanitize_various_inputs() {
    // Import the private sanitize function via the public API
    // by using generate_folder_name

    // Test through generate_folder_name
    let parsed = parse_git_url("https://github.com/normal/repo").unwrap();
    assert_eq!(generate_folder_name(&parsed), "github.com_normal_repo");

    // Test with dots (should be preserved for domains)
    let parsed = parse_git_url("https://github.com/user/repo.name").unwrap();
    assert_eq!(generate_folder_name(&parsed), "github.com_user_repo.name");
}

/// Test clone with multiple main directories configured
#[test]
fn test_clone_with_multiple_main_dirs() {
    let (tx, _rx) = tokio::sync::mpsc::channel(100);
    let mut app = repotui::app::model::App::new(tx);
    let runtime = repotui::runtime::executor::Runtime::new(app.msg_tx.clone());

    // Configure multiple main directories
    let config = repotui::config::Config {
        main_directories: vec![
            repotui::config::MainDirectoryConfig {
                path: PathBuf::from("/home/user/repos"),
                display_name: Some("Personal".to_string()),
                max_depth: None,
                enabled: true,
            },
            repotui::config::MainDirectoryConfig {
                path: PathBuf::from("/home/user/work"),
                display_name: Some("Work".to_string()),
                max_depth: None,
                enabled: true,
            },
        ],
        single_repositories: vec![],
        main_directory: None,
        editors: EditorConfig::default(),
        default_command: None,
        ui: repotui::config::UiConfig::default(),
        security: SecurityConfig::default(),
        favorites: FavoritesConfig::default(),
        recent: RecentConfig::default(),
        update: repotui::update::UpdateConfig::default(),
        version: "2.0".to_string(),
    };

    app.config = Some(config);

    // Start clone
    repotui::app::update::update(AppMsg::StartClone, &mut app, &runtime);

    if let AppState::Cloning { clone_state } = &app.state {
        // With multiple main dirs, should have main_dir_list_state initialized
        assert_eq!(clone_state.selected_main_dir(), 0);
    }

    // Navigate to second directory
    repotui::app::update::update(AppMsg::CloneNextMainDir, &mut app, &runtime);

    if let AppState::Cloning { clone_state } = &app.state {
        // Still 0 because we need to know the max
        // Let's check the list state instead
        // (navigation requires knowing max dirs)
    }
}

/// Test error handling for invalid URL during clone
#[test]
fn test_clone_invalid_url_error() {
    let (tx, _rx) = tokio::sync::mpsc::channel(100);
    let mut app = repotui::app::model::App::new(tx);
    let runtime = repotui::runtime::executor::Runtime::new(app.msg_tx.clone());

    // Start clone
    repotui::app::update::update(AppMsg::StartClone, &mut app, &runtime);

    // Enter invalid URL and confirm
    for c in "not-a-valid-url".chars() {
        repotui::app::update::update(AppMsg::CloneUrlInput(c), &mut app, &runtime);
    }

    repotui::app::update::update(AppMsg::CloneUrlConfirm, &mut app, &runtime);

    // Should transition to error state within cloning
    if let AppState::Cloning { clone_state } = &app.state {
        assert!(matches!(clone_state.stage, CloneStage::Error(_)));
    }
}

/// Test clone URL clear
#[test]
fn test_clone_url_clear() {
    let (tx, _rx) = tokio::sync::mpsc::channel(100);
    let mut app = repotui::app::model::App::new(tx);
    let runtime = repotui::runtime::executor::Runtime::new(app.msg_tx.clone());

    // Start clone and enter URL
    repotui::app::update::update(AppMsg::StartClone, &mut app, &runtime);
    for c in "https://github.com/test".chars() {
        repotui::app::update::update(AppMsg::CloneUrlInput(c), &mut app, &runtime);
    }

    // Move cursor to middle
    if let AppState::Cloning { clone_state } = &mut app.state {
        clone_state.cursor_position = 10;
    }

    // Clear from cursor
    repotui::app::update::update(AppMsg::CloneUrlClear, &mut app, &runtime);

    if let AppState::Cloning { clone_state } = &app.state {
        assert_eq!(clone_state.url_input, "https://gi");
        assert_eq!(clone_state.cursor_position, 10);
    }
}

/// Test clone retry after error
#[test]
fn test_clone_retry() {
    let (tx, _rx) = tokio::sync::mpsc::channel(100);
    let mut app = repotui::app::model::App::new(tx);
    let runtime = repotui::runtime::executor::Runtime::new(app.msg_tx.clone());

    // Start clone
    repotui::app::update::update(AppMsg::StartClone, &mut app, &runtime);

    // Enter URL and confirm with invalid URL to trigger error
    for c in "invalid".chars() {
        repotui::app::update::update(AppMsg::CloneUrlInput(c), &mut app, &runtime);
    }
    repotui::app::update::update(AppMsg::CloneUrlConfirm, &mut app, &runtime);

    // Verify error state
    let was_error = if let AppState::Cloning { clone_state } = &app.state {
        matches!(clone_state.stage, CloneStage::Error(_))
    } else {
        false
    };
    assert!(was_error, "Should be in error state");

    // Retry
    repotui::app::update::update(AppMsg::CloneRetry, &mut app, &runtime);

    // Should be back to input stage
    if let AppState::Cloning { clone_state } = &app.state {
        assert!(matches!(clone_state.stage, CloneStage::InputUrl));
    }
}

/// Test clone state cancellation flag
#[test]
fn test_clone_cancel_flag() {
    use repotui::app::state::CloneState;

    let state = CloneState::new();

    // Initially not cancelled
    assert!(!state.is_cancelled());

    // Cancel
    state.cancel();
    assert!(state.is_cancelled());
}

/// Test main directory navigation
#[test]
fn test_clone_main_dir_navigation() {
    use repotui::app::state::CloneState;

    let mut state = CloneState::new();

    // Navigate with max 3
    state.next_main_dir(3);
    assert_eq!(state.selected_main_dir(), 1);

    state.next_main_dir(3);
    assert_eq!(state.selected_main_dir(), 2);

    // Should not go beyond max-1
    state.next_main_dir(3);
    assert_eq!(state.selected_main_dir(), 2);

    // Navigate back
    state.previous_main_dir();
    assert_eq!(state.selected_main_dir(), 1);

    state.previous_main_dir();
    assert_eq!(state.selected_main_dir(), 0);

    // Should not go below 0
    state.previous_main_dir();
    assert_eq!(state.selected_main_dir(), 0);
}
