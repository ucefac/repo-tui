//! Startup benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use repotui::repo;
use std::path::PathBuf;
use tempfile::TempDir;

fn bench_repository_discovery(c: &mut Criterion) {
    // Create mock repositories
    let temp_dir = create_mock_repos(100);

    c.bench_function("discover_100_repos", |b| {
        b.iter(|| repo::discover_repositories(black_box(temp_dir.path())))
    });

    let temp_dir = create_mock_repos(1000);

    c.bench_function("discover_1000_repos", |b| {
        b.iter(|| repo::discover_repositories(black_box(temp_dir.path())))
    });
}

fn bench_search_filter(c: &mut Criterion) {
    let repos: Vec<repo::Repository> = (0..1000)
        .map(|i| repo::Repository {
            name: format!("repo-{}", i),
            path: PathBuf::from(format!("/tmp/repo-{}", i)),
            last_modified: None,
            is_dirty: false,
            branch: Some("main".to_string()),
        })
        .collect();

    c.bench_function("filter_1000_repos", |b| {
        b.iter(|| filter_repos(black_box(&repos), black_box("repo-5")))
    });
}

fn filter_repos(repos: &[repo::Repository], query: &str) -> Vec<usize> {
    let query_lower = query.to_lowercase();
    repos
        .iter()
        .enumerate()
        .filter(|(_, repo)| repo.name.to_lowercase().contains(&query_lower))
        .map(|(i, _)| i)
        .collect()
}

fn create_mock_repos(count: usize) -> TempDir {
    use std::fs;

    let temp_dir = TempDir::new().unwrap();

    for i in 0..count {
        let repo_path = temp_dir.path().join(format!("repo-{}", i));
        fs::create_dir(&repo_path).unwrap();
        fs::create_dir(repo_path.join(".git")).unwrap();
    }

    temp_dir
}

criterion_group!(benches, bench_repository_discovery, bench_search_filter);
criterion_main!(benches);
