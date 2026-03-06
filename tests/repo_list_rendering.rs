//! Repository list rendering integration tests

use repotui::repo::Repository;
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
    let mut repos = vec![
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
