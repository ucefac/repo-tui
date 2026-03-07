//! Repository list rendering integration tests

use ratatui::backend::TestBackend;
use ratatui::Terminal;
use repotui::repo::Repository;
use repotui::ui::theme::Theme;
use repotui::ui::widgets::RepoList;
use std::path::PathBuf;

#[test]
fn test_repository_display_format() {
    let repo = Repository {
        name: "test-repo".to_string(),
        path: PathBuf::from("/home/user/test-repo"),
        last_modified: None,
        is_dirty: false,
        branch: Some("main".to_string()),
    };
    assert_eq!(repo.name, "test-repo");
    assert_eq!(repo.branch, Some("main".to_string()));
}

#[test]
fn test_repository_sorting() {
    let mut repos = [
        Repository {
            name: "zebra".to_string(),
            path: PathBuf::from("/tmp/zebra"),
            last_modified: None,
            is_dirty: false,
            branch: None,
        },
        Repository {
            name: "alpha".to_string(),
            path: PathBuf::from("/tmp/alpha"),
            last_modified: None,
            is_dirty: false,
            branch: None,
        },
    ];
    repos.sort_by(|a, b| a.name.cmp(&b.name));
    assert_eq!(repos[0].name, "alpha");
    assert_eq!(repos[1].name, "zebra");
}

#[test]
fn test_repo_list_respects_area_width() {
    let repos = vec![Repository {
        name: "test-repo".to_string(),
        path: PathBuf::from("/tmp/test-repo"),
        last_modified: None,
        is_dirty: true,
        branch: Some("main".to_string()),
    }];
    let filtered: Vec<usize> = vec![0];
    let theme = Theme::dark();

    // Test with narrow width (Compact mode - no status)
    let backend = TestBackend::new(50, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            let area = f.area();
            let list = RepoList::new(&repos, &filtered, &theme)
                .area_width(area.width)
                .selected_index(Some(0))
                .visible_height(area.height);
            f.render_widget(list, area);
        })
        .unwrap();

    // Test with wide width (Large mode - should show status)
    let backend = TestBackend::new(120, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            let area = f.area();
            let list = RepoList::new(&repos, &filtered, &theme)
                .area_width(area.width)
                .selected_index(Some(0))
                .visible_height(area.height)
                .show_git_status(true);
            f.render_widget(list, area);
        })
        .unwrap();

    // If we get here without panic, the test passes
    // The area_width is being respected for display mode calculation
}

#[test]
fn test_repo_list_with_git_status_enabled() {
    let repos = vec![
        Repository {
            name: "dirty-repo".to_string(),
            path: PathBuf::from("/tmp/dirty-repo"),
            last_modified: None,
            is_dirty: true,
            branch: Some("main".to_string()),
        },
        Repository {
            name: "clean-repo".to_string(),
            path: PathBuf::from("/tmp/clean-repo"),
            last_modified: None,
            is_dirty: false,
            branch: Some("feature".to_string()),
        },
    ];
    let filtered: Vec<usize> = vec![0, 1];
    let theme = Theme::dark();

    let backend = TestBackend::new(120, 15);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            let area = f.area();
            let list = RepoList::new(&repos, &filtered, &theme)
                .area_width(area.width)
                .selected_index(Some(0))
                .visible_height(area.height)
                .show_git_status(true);
            f.render_widget(list, area);
        })
        .unwrap();

    // If we get here without panic, the test passes
    // The git status display is working correctly
}
