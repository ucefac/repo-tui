//! Search benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use repotui::repo::Repository;
use std::path::PathBuf;

fn bench_search_exact(c: &mut Criterion) {
    let repos = create_test_repos(1000);

    c.bench_function("search_exact_1000", |b| {
        b.iter(|| filter_repos(black_box(&repos), black_box("repo-500")))
    });
}

fn bench_search_prefix(c: &mut Criterion) {
    let repos = create_test_repos(1000);

    c.bench_function("search_prefix_1000", |b| {
        b.iter(|| filter_repos(black_box(&repos), black_box("repo-5")))
    });
}

fn bench_search_fuzzy(c: &mut Criterion) {
    let repos = create_test_repos(1000);

    c.bench_function("search_fuzzy_1000", |b| {
        b.iter(|| filter_repos(black_box(&repos), black_box("rpo50")))
    });
}

fn filter_repos(repos: &[Repository], query: &str) -> Vec<usize> {
    let query_lower = query.to_lowercase();
    repos
        .iter()
        .enumerate()
        .filter(|(_, repo)| repo.name.to_lowercase().contains(&query_lower))
        .map(|(i, _)| i)
        .collect()
}

fn create_test_repos(count: usize) -> Vec<Repository> {
    (0..count)
        .map(|i| Repository {
            name: format!("repo-{}", i),
            path: PathBuf::from(format!("/tmp/repo-{}", i)),
            last_modified: None,
            is_dirty: false,
            branch: Some("main".to_string()),
        })
        .collect()
}

criterion_group!(
    benches,
    bench_search_exact,
    bench_search_prefix,
    bench_search_fuzzy
);
criterion_main!(benches);
