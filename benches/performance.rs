//! Performance benchmarks for Phase 3
//!
//! Comprehensive benchmarks covering:
//! - Rendering performance (virtual list)
//! - Search filtering
//! - Git status cache operations
//! - Batch Git status detection

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ratatui::layout::Rect;
use repotui::git::cache::StatusCache;
use repotui::repo::types::Repository;
use std::path::PathBuf;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Create mock repositories for benchmarking
fn create_mock_repos(count: usize) -> Vec<Repository> {
    (0..count)
        .map(|i| Repository {
            name: format!("test_repo_{}", i),
            path: PathBuf::from(format!("/test/path/repo_{}", i)),
            last_modified: None,
            is_dirty: i % 3 == 0,
            branch: Some("main".to_string()),
        })
        .collect()
}

/// Benchmark: Render repositories with virtual list
fn bench_render_repo_list(c: &mut Criterion) {
    let mut group = c.benchmark_group("rendering");

    for repo_count in [100, 500, 1000].iter() {
        let repos = create_mock_repos(*repo_count);

        group.throughput(Throughput::Elements(*repo_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_repos_render", repo_count)),
            &repos,
            |b, repos| {
                b.iter(|| {
                    // Benchmark filtering + prep for render
                    let filtered: Vec<usize> = repos
                        .iter()
                        .enumerate()
                        .filter(|(_, repo)| repo.name.contains("repo"))
                        .map(|(i, _)| i)
                        .collect();
                    black_box(filtered.len());
                })
            },
        );
    }

    group.finish();
}

/// Benchmark: Search filtering performance
fn bench_search_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("search");

    for repo_count in [100, 500, 1000].iter() {
        let repos = create_mock_repos(*repo_count);

        group.throughput(Throughput::Elements(*repo_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("filter_{}_repos", repo_count)),
            &repos,
            |b, repos| {
                b.iter(|| {
                    let query = "test";
                    let filtered: Vec<usize> = repos
                        .iter()
                        .enumerate()
                        .filter(|(_, repo)| repo.name.to_lowercase().contains(query))
                        .map(|(i, _)| i)
                        .collect();
                    black_box(filtered);
                })
            },
        );
    }

    group.finish();
}

/// Benchmark: Git status cache hit
fn bench_git_status_cache_hit(c: &mut Criterion) {
    let mut group = c.benchmark_group("git_cache");

    let cache = StatusCache::new(300); // 5 minute TTL
    let path = PathBuf::from("/test/path");

    // Pre-populate cache
    cache.insert(
        path.clone(),
        repotui::repo::types::GitStatus {
            is_dirty: false,
            branch: Some("main".to_string()),
            ahead: None,
            behind: None,
        },
    );

    group.throughput(Throughput::Elements(1));
    group.bench_function("cache_hit", |b| {
        b.iter(|| {
            let status = cache.get(&path);
            black_box(status);
        })
    });

    group.finish();
}

/// Benchmark: Git status cache miss
fn bench_git_status_cache_miss(c: &mut Criterion) {
    let mut group = c.benchmark_group("git_cache");

    let cache = StatusCache::new(300); // 5 minute TTL
    let path = PathBuf::from("/test/path/miss");

    group.throughput(Throughput::Elements(1));
    group.bench_function("cache_miss", |b| {
        b.iter(|| {
            let status = cache.get(&path);
            black_box(status);
        })
    });

    group.finish();
}

/// Benchmark: Git status cache insert
fn bench_git_status_cache_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("git_cache");

    group.throughput(Throughput::Elements(1));
    group.bench_function("cache_insert", |b| {
        b.iter(|| {
            let cache = StatusCache::new(300);
            let path = PathBuf::from("/test/path/insert");
            cache.insert(
                path.clone(),
                repotui::repo::types::GitStatus {
                    is_dirty: true,
                    branch: Some("feature".to_string()),
                    ahead: Some(2),
                    behind: Some(1),
                },
            );
            black_box(cache.len());
        })
    });

    group.finish();
}

/// Benchmark: Batch Git status detection
fn bench_batch_git_status(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_git");

    for repo_count in [10, 50, 100].iter() {
        let repos = create_mock_repos(*repo_count);

        group.throughput(Throughput::Elements(*repo_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("batch_{}_repos", repo_count)),
            &repos,
            |b, repos| {
                let rt = Runtime::new().unwrap();
                b.iter(|| {
                    rt.block_on(async {
                        // Simulate batch detection with async delay
                        let futures: Vec<_> = repos
                            .iter()
                            .map(|_repo| tokio::time::sleep(Duration::from_millis(5)))
                            .collect();
                        let _ = futures::future::join_all(futures).await;
                    });
                })
            },
        );
    }

    group.finish();
}

/// Benchmark: Cache TTL expiry check
fn bench_cache_ttl_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("git_cache");

    let cache = StatusCache::new(1); // 1 second TTL for fast expiry test
    let path = PathBuf::from("/test/path/expiry");

    cache.insert(
        path.clone(),
        repotui::repo::types::GitStatus {
            is_dirty: false,
            branch: Some("main".to_string()),
            ahead: None,
            behind: None,
        },
    );

    group.throughput(Throughput::Elements(1));
    group.bench_function("ttl_check", |b| {
        b.iter(|| {
            let status = cache.get(&path);
            black_box(status.is_some());
        })
    });

    group.finish();
}

/// Benchmark: Cache cleanup
fn bench_cache_cleanup(c: &mut Criterion) {
    use criterion::BatchSize;

    let mut group = c.benchmark_group("git_cache");

    group.bench_function("cleanup", |b| {
        b.iter_batched(
            || {
                let cache = StatusCache::new(1); // 1 second TTL

                // Insert 100 items
                for i in 0..100 {
                    let path = PathBuf::from(format!("/test/path/{}", i));
                    cache.insert(
                        path,
                        repotui::repo::types::GitStatus {
                            is_dirty: i % 2 == 0,
                            branch: Some("main".to_string()),
                            ahead: None,
                            behind: None,
                        },
                    );
                }

                // Wait for expiry
                std::thread::sleep(Duration::from_secs(2));
                cache
            },
            |cache| {
                // Cleanup
                black_box(cache.cleanup());
                black_box(cache.len());
            },
            BatchSize::PerIteration,
        )
    });

    group.finish();
}

/// Benchmark: Theme switching
fn bench_theme_switch(c: &mut Criterion) {
    use repotui::ui::theme::Theme;

    let mut group = c.benchmark_group("ui");

    group.throughput(Throughput::Elements(1));
    group.bench_function("theme_switch", |b| {
        b.iter(|| {
            let dark = Theme::dark();
            let light = dark.toggle();
            black_box(light.name);
        })
    });

    group.finish();
}

/// Benchmark: Responsive layout calculation
fn bench_layout_calc(c: &mut Criterion) {
    use repotui::ui::layout::{calculate_main_layout, calculate_repo_list_row};

    let mut group = c.benchmark_group("ui");
    let area = Rect::new(0, 0, 120, 30);

    group.throughput(Throughput::Elements(1));
    group.bench_function("layout_calculation", |b| {
        b.iter(|| {
            let _ = calculate_main_layout(area);
            let _ = calculate_repo_list_row(120);
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_render_repo_list,
    bench_search_filter,
    bench_git_status_cache_hit,
    bench_git_status_cache_miss,
    bench_git_status_cache_insert,
    bench_batch_git_status,
    bench_cache_ttl_check,
    bench_cache_cleanup,
    bench_theme_switch,
    bench_layout_calc
);

criterion_main!(benches);
