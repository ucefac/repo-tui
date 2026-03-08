//! UI theme configuration

use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

/// RGB color representation for serialization
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct ColorRgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<ColorRgb> for Color {
    fn from(rgb: ColorRgb) -> Self {
        Color::Rgb(rgb.r, rgb.g, rgb.b)
    }
}

/// Color palette for a theme
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ColorPalette {
    pub primary: ColorRgb,
    pub secondary: ColorRgb,
    pub success: ColorRgb,
    pub warning: ColorRgb,
    pub error: ColorRgb,
    pub background: ColorRgb,
    pub foreground: ColorRgb,
    pub border: ColorRgb,
    pub selected_bg: ColorRgb,
    pub selected_fg: ColorRgb,
    pub text_muted: ColorRgb,
    pub border_focused: ColorRgb,
    pub title_fg: ColorRgb,
    pub title_bg: ColorRgb,
}

/// UI Theme
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    pub name: String,
    pub colors: ColorPalette,
}

impl Theme {
    /// Create theme from name
    pub fn new(name: &str) -> Self {
        // Handle "🎲 Random (随机)" option
        if name.contains("Random") || name == "random" {
            return crate::ui::themes::get_random_theme();
        }

        crate::ui::themes::get_theme(name).unwrap_or_else(Self::dark)
    }

    /// Get dark theme (fallback)
    pub fn dark() -> Self {
        crate::ui::themes::dark_theme()
    }

    /// Get light theme
    pub fn light() -> Self {
        crate::ui::themes::light_theme()
    }

    /// Get default theme
    pub fn default_theme() -> Self {
        Self::new(crate::ui::themes::default_theme_name())
    }

    /// Create theme from config (backward compatible)
    pub fn from_config(theme_name: &str) -> Self {
        Self::new(theme_name)
    }

    /// Toggle between dark and light theme (backward compatible)
    pub fn toggle(&self) -> Self {
        if self.name == "dark" {
            Self::light()
        } else {
            Self::dark()
        }
    }

    /// Get next theme in rotation
    pub fn next(&self) -> Self {
        let themes = crate::ui::themes::THEME_NAMES;
        let current_idx = themes.iter().position(|&t| t == self.name).unwrap_or(0);
        let next_idx = (current_idx + 1) % themes.len();
        Self::new(themes[next_idx])
    }

    /// Get all available theme names
    pub fn available_themes() -> Vec<&'static str> {
        crate::ui::themes::THEME_NAMES.to_vec()
    }

    /// Get selected style
    pub fn selected_style(&self) -> Style {
        Style::default()
            .fg(self.colors.selected_fg.into())
            .bg(self.colors.selected_bg.into())
            .add_modifier(Modifier::BOLD)
    }

    /// Get focused border style
    pub fn focused_border_style(&self) -> Style {
        Style::default().fg(self.colors.border_focused.into())
    }

    /// Get normal border style
    pub fn normal_border_style(&self) -> Style {
        Style::default().fg(self.colors.border.into())
    }

    /// Get primary text style
    pub fn primary_text_style(&self) -> Style {
        Style::default().fg(self.colors.foreground.into())
    }

    /// Get secondary text style
    pub fn secondary_text_style(&self) -> Style {
        Style::default().fg(self.colors.text_muted.into())
    }

    /// Get primary color style
    pub fn primary_style(&self) -> Style {
        Style::default().fg(self.colors.primary.into())
    }

    /// Get success color style
    pub fn success_style(&self) -> Style {
        Style::default().fg(self.colors.success.into())
    }

    /// Get warning color style
    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.colors.warning.into())
    }

    /// Get error color style
    pub fn error_style(&self) -> Style {
        Style::default().fg(self.colors.error.into())
    }

    /// Get title style
    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.colors.title_fg.into())
            .bg(self.colors.title_bg.into())
            .add_modifier(Modifier::BOLD)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::default_theme()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dark_theme() {
        let theme = Theme::dark();
        assert_eq!(theme.name, "dark");
    }

    #[test]
    fn test_light_theme() {
        let theme = Theme::light();
        assert_eq!(theme.name, "light");
    }

    #[test]
    fn test_theme_new() {
        let dark = Theme::new("dark");
        assert_eq!(dark.name, "dark");

        let nord = Theme::new("nord");
        assert_eq!(nord.name, "nord");

        let invalid = Theme::new("invalid");
        assert_eq!(invalid.name, "dark");
    }

    #[test]
    fn test_from_config() {
        let dark = Theme::from_config("dark");
        assert_eq!(dark.name, "dark");

        let light = Theme::from_config("light");
        assert_eq!(light.name, "light");

        let default = Theme::from_config("unknown");
        assert_eq!(default.name, "dark");
    }

    #[test]
    fn test_toggle_theme() {
        let dark = Theme::dark();
        let light = dark.toggle();
        assert_eq!(light.name, "light");

        let dark_again = light.toggle();
        assert_eq!(dark_again.name, "dark");
    }

    #[test]
    fn test_theme_next() {
        // First theme is now "🎲 Random (随机)", second is "dark"
        let dark = Theme::new("dark");
        let next = dark.next();
        assert_eq!(next.name, "light");

        let last = Theme::new("catppuccin_mocha");
        let wrapped = last.next();
        // Now wraps to first theme which is random
        // When Theme::new() is called with "🎲 Random (随机)", it returns a random real theme
        // So we just check it's not the same as the input
        assert_ne!(wrapped.name, "catppuccin_mocha");
    }

    #[test]
    fn test_available_themes() {
        let themes = Theme::available_themes();
        assert_eq!(themes.len(), 8); // Now includes "🎲 Random (随机)"
        assert!(themes.contains(&"🎲 Random (随机)"));
        assert!(themes.contains(&"dark"));
        assert!(themes.contains(&"nord"));
        assert!(themes.contains(&"catppuccin_mocha"));
    }

    #[test]
    fn test_styles() {
        let theme = Theme::dark();

        let selected = theme.selected_style();
        assert!(selected.add_modifier == Modifier::BOLD);

        let focused = theme.focused_border_style();
        assert_eq!(focused.fg, Some(theme.colors.border_focused.into()));
    }
}
