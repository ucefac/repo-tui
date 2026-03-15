//! Batch operations integration tests

use std::path::PathBuf;
use tokio::sync::mpsc;

use repo_tui::app::model::App;
use repo_tui::app::msg::AppMsg;
use repo_tui::repo::{RepoSource, Repository};

fn create_test_repo(name: &str, path: &str) -> Repository {
    Repository {
        name: name.to_string(),
        path: PathBuf::from(path),
        last_modified: None,
        is_dirty: false,
        branch: Some("main".to_string()),
        is_git_repo: true,
        source: RepoSource::Standalone,
    }
}

fn create_test_app_with_repos() -> (App, mpsc::Receiver<AppMsg>) {
    let (tx, rx) = mpsc::channel(100);
    let mut app = App::new(tx);

    // Add test repositories
    app.repositories = vec![
        create_test_repo("repo1", "/tmp/repo1"),
        create_test_repo("repo2", "/tmp/repo2"),
        create_test_repo("repo3", "/tmp/repo3"),
    ];
    app.filtered_indices = vec![0, 1, 2];
    app.set_selected_index(Some(0));

    (app, rx)
}

#[test]
fn test_toggle_selection_mode() {
    let (mut app, _rx) = create_test_app_with_repos();

    assert!(!app.selection_mode);
    app.toggle_selection_mode();
    assert!(app.selection_mode);
    app.toggle_selection_mode();
    assert!(!app.selection_mode);
}

#[test]
fn test_toggle_selection() {
    let (mut app, _rx) = create_test_app_with_repos();
    app.set_selected_index(Some(0));

    // Initially no selections
    assert_eq!(app.selected_count(), 0);

    // Toggle first repo
    app.toggle_selection();
    assert_eq!(app.selected_count(), 1);
    assert!(app.selected_indices.contains(&0));

    // Toggle first repo again (deselect)
    app.toggle_selection();
    assert_eq!(app.selected_count(), 0);
}

#[test]
fn test_toggle_selection_multiple() {
    let (mut app, _rx) = create_test_app_with_repos();

    // Select all three repos
    app.set_selected_index(Some(0));
    app.toggle_selection();

    app.set_selected_index(Some(1));
    app.toggle_selection();

    app.set_selected_index(Some(2));
    app.toggle_selection();

    assert_eq!(app.selected_count(), 3);
    assert!(app.selected_indices.contains(&0));
    assert!(app.selected_indices.contains(&1));
    assert!(app.selected_indices.contains(&2));
}

#[test]
fn test_select_all() {
    let (mut app, _rx) = create_test_app_with_repos();

    app.select_all();

    assert_eq!(app.selected_count(), 3);
    assert_eq!(app.selected_indices.len(), 3);
}

#[test]
fn test_clear_selection() {
    let (mut app, _rx) = create_test_app_with_repos();

    // Select all
    app.select_all();
    assert_eq!(app.selected_count(), 3);

    // Clear selection
    app.clear_selection();
    assert_eq!(app.selected_count(), 0);
    assert!(app.selected_indices.is_empty());
}

#[test]
fn test_get_selected_repos() {
    let (mut app, _rx) = create_test_app_with_repos();

    // Select repos 0 and 2
    app.selected_indices.insert(0);
    app.selected_indices.insert(2);

    let selected = app.get_selected_repos();
    assert_eq!(selected.len(), 2);

    // Collect names and check (order doesn't matter with HashSet)
    let names: std::collections::HashSet<&str> = selected.iter().map(|r| r.name.as_str()).collect();
    assert!(names.contains("repo1"));
    assert!(names.contains("repo3"));
}

#[test]
fn test_is_current_selected() {
    let (mut app, _rx) = create_test_app_with_repos();
    app.set_selected_index(Some(0));

    // Initially not selected
    assert!(!app.is_current_selected());

    // Select current repo
    app.toggle_selection();
    assert!(app.is_current_selected());

    // Move to next repo
    app.set_selected_index(Some(1));
    assert!(!app.is_current_selected());
}

#[test]
fn test_selected_indices_unique() {
    let (mut app, _rx) = create_test_app_with_repos();
    app.set_selected_index(Some(0));

    // Try to select same repo multiple times
    app.toggle_selection();
    app.toggle_selection();
    app.toggle_selection();

    // Should only be selected once
    assert_eq!(app.selected_count(), 1);
    assert!(app.selected_indices.contains(&0));
}

#[test]
fn test_selection_with_filtered_indices() {
    let (mut app, _rx) = create_test_app_with_repos();

    // Filter to only first two repos
    app.filtered_indices = vec![0, 1];

    app.select_all();

    // Should only select filtered repos
    assert_eq!(app.selected_count(), 2);
    assert!(app.selected_indices.contains(&0));
    assert!(app.selected_indices.contains(&1));
    assert!(!app.selected_indices.contains(&2));
}

#[test]
fn test_selection_mode_persistence() {
    let (mut app, _rx) = create_test_app_with_repos();

    // Enable selection mode
    app.toggle_selection_mode();
    assert!(app.selection_mode);

    // Make some selections
    app.toggle_selection();
    app.set_selected_index(Some(1));
    app.toggle_selection();

    // Selection mode should still be active
    assert!(app.selection_mode);
    assert_eq!(app.selected_count(), 2);
}

#[test]
fn test_batch_result_empty() {
    use repo_tui::action::batch::BatchResult;

    let result = BatchResult::new(0);
    assert_eq!(result.total, 0);
    assert_eq!(result.success, 0);
    assert_eq!(result.failed, 0);
    assert!(result.errors.is_empty());
}

#[test]
fn test_batch_result_success_rate() {
    use repo_tui::action::batch::BatchResult;

    let mut result = BatchResult::new(10);
    result.success = 8;
    result.failed = 2;

    assert!((result.success_rate() - 0.8).abs() < f64::EPSILON);
}

#[test]
fn test_batch_result_all_succeeded() {
    use repo_tui::action::batch::BatchResult;

    let mut result = BatchResult::new(5);
    result.success = 5;

    assert!(result.all_succeeded());

    result.failed = 1;
    assert!(!result.all_succeeded());
}

#[tokio::test]
async fn test_batch_execute_empty_list() {
    use repo_tui::action::{execute_batch, Action};

    let repos: Vec<Repository> = vec![];
    let result = execute_batch(&Action::OpenFileManager, repos, 5).await;

    assert_eq!(result.total, 0);
    assert_eq!(result.success, 0);
    assert_eq!(result.failed, 0);
}

#[tokio::test]
async fn test_batch_execute_concurrency() {
    use repo_tui::action::{execute_batch, Action};

    // Create test repos
    let repos: Vec<Repository> = (0..5)
        .map(|i| {
            create_test_repo(
                &format!("test_repo_{}", i),
                &format!("/tmp/test_repo_{}", i),
            )
        })
        .collect();

    // Execute with concurrency limit of 2
    let result = execute_batch(&Action::OpenFileManager, repos, 2).await;

    // All should fail since paths don't exist, but function should complete
    assert_eq!(result.total, 5);
}
