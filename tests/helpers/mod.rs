//! Test helpers and utilities
//!
//! This module provides common utilities for testing ghclone-tui:
//! - Mock filesystem for creating test repositories
//! - Mock terminal for UI testing
//! - App builders for test scenarios
//! - Configuration test helpers
//! - UI assertions

pub mod config_helper;
pub mod mock_fs;
pub mod mock_terminal;
pub mod ui_assertions;

use repotui::app::model::App;
use repotui::app::msg::AppMsg;
use repotui::config::types::{EditorConfig, FavoritesConfig, RecentConfig, SecurityConfig};
use repotui::config::Config;
use repotui::repo::Repository;
use std::path::PathBuf;
use tokio::sync::mpsc;

/// Create a test app with a message channel
pub fn create_test_app() -> (App, mpsc::Receiver<AppMsg>) {
    let (tx, rx) = mpsc::channel(100);
    let app = App::new(tx);
    (app, rx)
}

/// Create a test app with repositories
pub fn create_test_app_with_repos(repos: Vec<Repository>) -> (App, mpsc::Receiver<AppMsg>) {
    let (mut app, rx) = create_test_app();

    app.repositories = repos;
    app.apply_filter();
    if !app.filtered_indices.is_empty() {
        app.set_selected_index(Some(0));
    }

    (app, rx)
}

/// Create a test app with sample repositories
pub fn create_test_app_with_sample_repos() -> (App, mpsc::Receiver<AppMsg>) {
    let repos = vec![
        Repository {
            name: "react".to_string(),
            path: PathBuf::from("/home/user/repos/react"),
            last_modified: None,
            is_dirty: false,
            branch: Some("main".to_string()),
            is_git_repo: true,
            source: repotui::repo::source::RepoSource::Standalone,
        },
        Repository {
            name: "vue".to_string(),
            path: PathBuf::from("/home/user/repos/vue"),
            last_modified: None,
            is_dirty: true,
            branch: Some("dev".to_string()),
            is_git_repo: true,
            source: repotui::repo::source::RepoSource::Standalone,
        },
        Repository {
            name: "angular".to_string(),
            path: PathBuf::from("/home/user/repos/angular"),
            last_modified: None,
            is_dirty: false,
            branch: Some("main".to_string()),
            is_git_repo: true,
            source: repotui::repo::source::RepoSource::Standalone,
        },
    ];

    create_test_app_with_repos(repos)
}

/// Create a test app with many repositories (for performance testing)
pub fn create_test_app_with_many_repos(count: usize) -> (App, mpsc::Receiver<AppMsg>) {
    let repos: Vec<Repository> = (0..count)
        .map(|i| Repository {
            name: format!("repo{:04}", i),
            path: PathBuf::from(format!("/home/user/repos/repo{:04}", i)),
            last_modified: None,
            is_dirty: i % 3 == 0,
            branch: Some("main".to_string()),
            is_git_repo: true,
            source: repotui::repo::source::RepoSource::Standalone,
        })
        .collect();

    create_test_app_with_repos(repos)
}

/// Create a test configuration
pub fn create_test_config(main_dir: PathBuf) -> Config {
    Config {
        main_directories: vec![repotui::config::MainDirectoryConfig {
            path: main_dir,
            display_name: None,
            max_depth: None,
            enabled: true,
        }],
        single_repositories: vec![],
        main_directory: None,
        editors: EditorConfig {
            webstorm: Some("webstorm".to_string()),
            vscode: Some("code".to_string()),
            others: std::collections::HashMap::new(),
        },
        default_command: Some("claude".to_string()),
        security: SecurityConfig::default(),
        ui: repotui::config::UiConfig::default(),
        favorites: FavoritesConfig::default(),
        recent: RecentConfig::default(),
        update: repotui::update::UpdateConfig::default(),
        version: "2.0".to_string(),
    }
}

/// Collect all messages from the channel (non-blocking)
pub async fn collect_messages(rx: &mut mpsc::Receiver<AppMsg>) -> Vec<AppMsg> {
    let mut messages = Vec::new();
    while let Ok(msg) = rx.try_recv() {
        messages.push(msg);
    }
    messages
}

/// Wait for a specific message type
pub async fn wait_for_message<F>(rx: &mut mpsc::Receiver<AppMsg>, predicate: F) -> Option<AppMsg>
where
    F: Fn(&AppMsg) -> bool,
{
    loop {
        if let Some(msg) = rx.recv().await {
            if predicate(&msg) {
                return Some(msg);
            }
        } else {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_app() {
        let (app, _rx) = create_test_app();
        assert!(app.loading);
        assert!(app.repositories.is_empty());
    }

    #[test]
    fn test_create_test_app_with_sample_repos() {
        let (app, _rx) = create_test_app_with_sample_repos();
        assert_eq!(app.repositories.len(), 3);
        assert_eq!(app.filtered_count(), 3);
    }

    #[test]
    fn test_create_test_app_with_many_repos() {
        let (app, _rx) = create_test_app_with_many_repos(100);
        assert_eq!(app.repositories.len(), 100);
    }
}
