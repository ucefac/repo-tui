//! Built-in theme definitions

mod catppuccin_mocha;
mod dark;
mod dracula;
mod gruvbox_dark;
mod light;
mod nord;
mod tokyo_night;

pub use catppuccin_mocha::catppuccin_mocha_theme;
pub use dark::dark_theme;
pub use dracula::dracula_theme;
pub use gruvbox_dark::gruvbox_dark_theme;
pub use light::light_theme;
pub use nord::nord_theme;
pub use tokyo_night::tokyo_night_theme;

use crate::ui::theme::Theme;
use rand::seq::SliceRandom;

/// Available theme names
pub const THEME_NAMES: &[&str] = &[
    "🎲 Random (随机)",
    "dark",
    "light",
    "nord",
    "dracula",
    "gruvbox_dark",
    "tokyo_night",
    "catppuccin_mocha",
];

/// Get theme by name
pub fn get_theme(name: &str) -> Option<Theme> {
    match name {
        "dark" => Some(dark_theme()),
        "light" => Some(light_theme()),
        "nord" => Some(nord_theme()),
        "dracula" => Some(dracula_theme()),
        "gruvbox_dark" => Some(gruvbox_dark_theme()),
        "tokyo_night" => Some(tokyo_night_theme()),
        "catppuccin_mocha" => Some(catppuccin_mocha_theme()),
        _ => None,
    }
}

/// Get default theme name
pub fn default_theme_name() -> &'static str {
    "dark"
}

/// Get a random theme (excluding the "random" option itself)
pub fn get_random_theme() -> Theme {
    let mut rng = rand::thread_rng();
    // Exclude "random" option (first element)
    let real_themes = &THEME_NAMES[1..];

    real_themes
        .choose(&mut rng)
        .and_then(|&name| get_theme(name))
        .unwrap_or_else(dark_theme)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_themes() {
        // Skip "🎲 Random (随机)" as it's not a real theme
        for &name in &THEME_NAMES[1..] {
            let theme = get_theme(name);
            assert!(theme.is_some(), "Theme {} should exist", name);
            assert_eq!(theme.unwrap().name, name);
        }
    }

    #[test]
    fn test_get_invalid_theme() {
        assert!(get_theme("invalid_theme").is_none());
    }

    #[test]
    fn test_get_random_theme() {
        let theme = get_random_theme();
        // Should always return a valid theme
        assert_ne!(theme.name, "🎲 Random (随机)");
        assert_ne!(theme.name, "random");
        // Should be one of the real themes
        let real_themes = &THEME_NAMES[1..];
        assert!(real_themes.contains(&theme.name.as_str()));
    }

    #[test]
    fn test_get_random_theme_variety() {
        let themes: Vec<_> = (0..20).map(|_| get_random_theme().name).collect();
        assert!(
            themes
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len()
                >= 2
        );
    }
}
