//! Search filtering integration tests

use repo_tui::app::model::App;
use repo_tui::repo::{RepoSource, Repository};
use std::path::PathBuf;
use tokio::sync::mpsc;

#[test]
fn test_filter_case_insensitive() {
    let (tx, _rx) = mpsc::channel(100);
    let mut app = App::new(tx);
    app.repositories = vec![
        Repository {
            name: "MyProject".to_string(),
            path: PathBuf::from("/tmp/myproject"),
            last_modified: None,
            is_dirty: false,
            branch: None,
            is_git_repo: true,
            source: RepoSource::Standalone,
        },
        Repository {
            name: "ANOTHER-REPO".to_string(),
            path: PathBuf::from("/tmp/another"),
            last_modified: None,
            is_dirty: false,
            branch: None,
            is_git_repo: true,
            source: RepoSource::Standalone,
        },
    ];
    app.search_query = "my".to_string();
    app.apply_filter();
    assert_eq!(app.filtered_indices.len(), 1);
    assert_eq!(app.repositories[app.filtered_indices[0]].name, "MyProject");
}

#[test]
fn test_filter_empty_query() {
    let (tx, _rx) = mpsc::channel(100);
    let mut app = App::new(tx);
    app.repositories = vec![
        Repository {
            name: "repo1".to_string(),
            path: PathBuf::from("/tmp/repo1"),
            last_modified: None,
            is_dirty: false,
            branch: None,
            is_git_repo: true,
            source: RepoSource::Standalone,
        },
        Repository {
            name: "repo2".to_string(),
            path: PathBuf::from("/tmp/repo2"),
            last_modified: None,
            is_dirty: false,
            branch: None,
            is_git_repo: true,
            source: RepoSource::Standalone,
        },
    ];
    app.search_query = "".to_string();
    app.apply_filter();
    assert_eq!(app.filtered_indices.len(), 2);
}
