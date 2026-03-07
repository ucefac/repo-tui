//! Search benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use repotui::repo::{filter_repos_fuzzy, filter_repos_simple, Repository};
use std::path::PathBuf;

fn bench_search_exact(c: &mut Criterion) {
    let repos = create_test_repos(1000);

    c.bench_function("search_exact_1000", |b| {
        b.iter(|| filter_repos_simple(black_box(&repos), black_box("repo-500")))
    });
}

fn bench_search_prefix(c: &mut Criterion) {
    let repos = create_test_repos(1000);

    c.bench_function("search_prefix_1000", |b| {
        b.iter(|| filter_repos_simple(black_box(&repos), black_box("repo-5")))
    });
}

fn bench_search_fuzzy(c: &mut Criterion) {
    let repos = create_test_repos(1000);

    c.bench_function("search_fuzzy_1000", |b| {
        b.iter(|| filter_repos_fuzzy(black_box(&repos), black_box("rpo50")))
    });
}

fn bench_search_fuzzy_subsequence(c: &mut Criterion) {
    let repos = create_test_repos(1000);

    c.bench_function("search_fuzzy_subsequence_1000", |b| {
        b.iter(|| filter_repos_fuzzy(black_box(&repos), black_box("fbreact")))
    });
}

fn bench_search_fuzzy_short(c: &mut Criterion) {
    let repos = create_test_repos(1000);

    c.bench_function("search_fuzzy_short_1000", |b| {
        b.iter(|| filter_repos_fuzzy(black_box(&repos), black_box("re")))
    });
}

fn bench_search_fuzzy_long(c: &mut Criterion) {
    let repos = create_test_repos(1000);

    c.bench_function("search_fuzzy_long_1000", |b| {
        b.iter(|| filter_repos_fuzzy(black_box(&repos), black_box("repository-test-project")))
    });
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
    bench_search_fuzzy,
    bench_search_fuzzy_subsequence,
    bench_search_fuzzy_short,
    bench_search_fuzzy_long,
);
criterion_main!(benches);
