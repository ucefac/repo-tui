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

/// Available theme names
pub const THEME_NAMES: &[&str] = &[
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_themes() {
        for &name in THEME_NAMES {
            let theme = get_theme(name);
            assert!(theme.is_some(), "Theme {} should exist", name);
            assert_eq!(theme.unwrap().name, name);
        }
    }

    #[test]
    fn test_get_invalid_theme() {
        assert!(get_theme("invalid_theme").is_none());
    }
}
