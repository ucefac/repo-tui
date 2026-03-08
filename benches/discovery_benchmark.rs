//! Repository discovery benchmark tests
//!
//! Performance benchmarks for multi-directory repository discovery

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::path::PathBuf;
use tempfile::TempDir;

/// Create a mock git repository
fn create_mock_repo(parent: &PathBuf, name: &str) -> PathBuf {
    let repo_path = parent.join(name);
    std::fs::create_dir_all(&repo_path).unwrap();
    std::fs::create_dir(repo_path.join(".git")).unwrap();
    repo_path
}

/// Simple repository discovery (for benchmarking)
fn discover_repositories(main_dir: &PathBuf) -> Vec<PathBuf> {
    let mut repos = Vec::new();

    if let Ok(entries) = std::fs::read_dir(main_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() && path.join(".git").exists() {
                repos.push(path);
            }
        }
    }

    repos.sort();
    repos
}

/// Multi-directory discovery (for benchmarking)
fn discover_from_multiple_dirs(main_dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut all_repos = Vec::new();

    for dir in main_dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() && path.join(".git").exists() {
                    all_repos.push(path);
                }
            }
        }
    }

    all_repos.sort();
    all_repos
}

/// Benchmark: Single directory with varying repo counts
fn bench_single_directory_discovery(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_directory_discovery");

    for repo_count in [10, 50, 100, 500].iter() {
        let temp = TempDir::new().unwrap();
        let main_dir = temp.path().join("main");
        std::fs::create_dir(&main_dir).unwrap();

        // Create repos
        for i in 0..*repo_count {
            create_mock_repo(&main_dir, &format!("repo{:04}", i));
        }

        group.bench_with_input(
            BenchmarkId::from_parameter(repo_count),
            &main_dir,
            |b, main_dir| {
                b.iter(|| {
                    let repos = discover_repositories(black_box(main_dir));
                    black_box(repos);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Multiple directories
fn bench_multi_directory_discovery(c: &mut Criterion) {
    let mut group = c.benchmark_group("multi_directory_discovery");

    for dir_count in [2, 5, 10].iter() {
        let temp = TempDir::new().unwrap();
        let mut main_dirs = Vec::new();

        // Create multiple directories with repos
        for d in 0..*dir_count {
            let dir = temp.path().join(format!("dir{}", d));
            std::fs::create_dir(&dir).unwrap();

            // 50 repos per directory
            for i in 0..50 {
                create_mock_repo(&dir, &format!("repo{:04}", i));
            }

            main_dirs.push(dir);
        }

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_dirs", dir_count)),
            &main_dirs,
            |b, dirs| {
                b.iter(|| {
                    let repos = discover_from_multiple_dirs(black_box(dirs));
                    black_box(repos);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Deduplication overhead
fn bench_deduplication(c: &mut Criterion) {
    use std::collections::HashSet;

    fn deduplicate_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
        let mut seen = HashSet::new();
        paths
            .into_iter()
            .filter(|p| seen.insert(p.clone()))
            .collect()
    }

    let mut group = c.benchmark_group("deduplication");

    for count in [100, 500, 1000, 5000].iter() {
        // Create paths with some duplicates
        let mut paths = Vec::new();
        for i in 0..*count {
            paths.push(PathBuf::from(format!(
                "/home/user/repos/repo{}",
                i % (*count / 2)
            )));
        }

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_paths", count)),
            &paths,
            |b, paths| {
                b.iter(|| {
                    let unique = deduplicate_paths(black_box(paths.clone()));
                    black_box(unique);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Sorting overhead
fn bench_sorting(c: &mut Criterion) {
    let mut group = c.benchmark_group("sorting");

    for count in [100, 500, 1000, 5000].iter() {
        let temp = TempDir::new().unwrap();
        let main_dir = temp.path().join("main");
        std::fs::create_dir(&main_dir).unwrap();

        // Create repos in random order
        for i in 0..*count {
            create_mock_repo(&main_dir, &format!("repo{:08}", i));
        }

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_repos", count)),
            &main_dir,
            |b, main_dir| {
                b.iter(|| {
                    let mut repos = discover_repositories(black_box(main_dir));
                    repos.sort(); // Sort again to benchmark sorting
                    black_box(repos);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Full discovery pipeline
fn bench_full_discovery_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_discovery_pipeline");
    group.sample_size(50);

    // Setup: Create realistic multi-directory structure
    let temp = TempDir::new().unwrap();

    let work_dir = temp.path().join("work");
    let personal_dir = temp.path().join("personal");
    let company_dir = temp.path().join("company");

    std::fs::create_dir(&work_dir).unwrap();
    std::fs::create_dir(&personal_dir).unwrap();
    std::fs::create_dir(&company_dir).unwrap();

    // Create repos in each directory
    for i in 0..50 {
        create_mock_repo(&work_dir, &format!("work-repo{:03}", i));
        create_mock_repo(&personal_dir, &format!("personal-repo{:03}", i));
        create_mock_repo(&company_dir, &format!("company-repo{:03}", i));
    }

    let main_dirs = vec![work_dir, personal_dir, company_dir];

    group.bench_function("discover_150_repos_across_3_dirs", |b| {
        b.iter(|| {
            let repos = discover_from_multiple_dirs(black_box(&main_dirs));
            black_box(repos);
        });
    });

    group.finish();
}

/// Benchmark: Directory scanning vs file reading
fn bench_directory_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("directory_operations");

    // Setup
    let temp = TempDir::new().unwrap();
    let main_dir = temp.path().join("main");
    std::fs::create_dir(&main_dir).unwrap();

    for i in 0..100 {
        create_mock_repo(&main_dir, &format!("repo{:03}", i));
    }

    group.bench_function("read_dir_only", |b| {
        b.iter(|| {
            if let Ok(entries) = std::fs::read_dir(black_box(&main_dir)) {
                let count = entries.count();
                black_box(count);
            }
        });
    });

    group.bench_function("read_dir_with_git_check", |b| {
        b.iter(|| {
            let mut count = 0;
            if let Ok(entries) = std::fs::read_dir(black_box(&main_dir)) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_dir() && path.join(".git").exists() {
                        count += 1;
                    }
                }
            }
            black_box(count);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_single_directory_discovery,
    bench_multi_directory_discovery,
    bench_deduplication,
    bench_sorting,
    bench_full_discovery_pipeline,
    bench_directory_operations
);

criterion_main!(benches);
