//! Theme system performance benchmarks
//!
//! Benchmarks for theme switching performance:
//! - Theme loading time
//! - Theme switching latency
//! - Color palette application

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use repotui::ui::themes::{get_theme, THEME_NAMES};
use repotui::ui::Theme;

/// Benchmark: Theme loading from name
fn bench_theme_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("theme_loading");

    for &theme_name in THEME_NAMES.iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(theme_name),
            &theme_name,
            |b, &name| {
                b.iter(|| {
                    let theme = get_theme(black_box(name));
                    assert!(theme.is_some());
                    theme
                })
            },
        );
    }
    group.finish();
}

/// Benchmark: Theme switching (create new theme from current)
fn bench_theme_switching(c: &mut Criterion) {
    let mut group = c.benchmark_group("theme_switching");

    let mut current_theme = Theme::new("dark");

    group.bench_function(BenchmarkId::from_parameter("switch_theme"), |b| {
        b.iter(|| {
            // Simulate theme cycling
            let next = current_theme.next();
            current_theme = next;
            black_box(&current_theme.name);
        })
    });

    group.finish();
}

/// Benchmark: Theme name iteration
fn bench_theme_iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("theme_iteration");

    group.bench_function(BenchmarkId::from_parameter("iterate_all_themes"), |b| {
        b.iter(|| {
            let mut count = 0;
            for &name in THEME_NAMES.iter() {
                let theme = get_theme(name);
                assert!(theme.is_some());
                count += 1;
            }
            black_box(count);
        })
    });

    group.finish();
}

/// Benchmark: Theme creation with invalid name (fallback to dark)
fn bench_theme_invalid_name(c: &mut Criterion) {
    let mut group = c.benchmark_group("theme_invalid");

    group.bench_function(BenchmarkId::from_parameter("invalid_theme_name"), |b| {
        b.iter(|| {
            let theme = Theme::new(black_box("invalid_theme_name"));
            assert_eq!(theme.name, "dark");
            black_box(theme);
        })
    });

    group.finish();
}

/// Benchmark: Theme color access
fn bench_theme_color_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("theme_colors");

    let theme = Theme::new("nord");

    group.bench_function(BenchmarkId::from_parameter("access_background"), |b| {
        b.iter(|| {
            let bg = black_box(&theme.colors.background);
            black_box((bg.r, bg.g, bg.b));
        })
    });

    group.bench_function(BenchmarkId::from_parameter("access_foreground"), |b| {
        b.iter(|| {
            let fg = black_box(&theme.colors.foreground);
            black_box((fg.r, fg.g, fg.b));
        })
    });

    group.bench_function(BenchmarkId::from_parameter("access_border_focused"), |b| {
        b.iter(|| {
            let border = black_box(&theme.colors.border_focused);
            black_box((border.r, border.g, border.b));
        })
    });

    group.finish();
}

/// Benchmark: Theme toggle (dark <-> light)
fn bench_theme_toggle(c: &mut Criterion) {
    let mut group = c.benchmark_group("theme_toggle");

    let dark = Theme::dark();
    let light = Theme::light();

    group.bench_function(BenchmarkId::from_parameter("toggle_dark_to_light"), |b| {
        b.iter(|| {
            let toggled = dark.toggle();
            assert_eq!(toggled.name, "light");
            black_box(toggled);
        })
    });

    group.bench_function(BenchmarkId::from_parameter("toggle_light_to_dark"), |b| {
        b.iter(|| {
            let toggled = light.toggle();
            assert_eq!(toggled.name, "dark");
            black_box(toggled);
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_theme_loading,
    bench_theme_switching,
    bench_theme_iteration,
    bench_theme_invalid_name,
    bench_theme_color_access,
    bench_theme_toggle,
);

criterion_main!(benches);
