//! UI theme configuration

use crate::constants;
use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

/// RGB color representation for serialization
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

/// UI Theme
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub colors: ColorPalette,
}

impl Theme {
    /// Create dark theme
    pub fn dark() -> Self {
        Self {
            name: "dark".to_string(),
            colors: ColorPalette {
                primary: ColorRgb {
                    r: constants::ui::dark::PRIMARY.r,
                    g: constants::ui::dark::PRIMARY.g,
                    b: constants::ui::dark::PRIMARY.b,
                },
                secondary: ColorRgb {
                    r: 139,
                    g: 92,
                    b: 246,
                },
                success: ColorRgb {
                    r: constants::ui::dark::SUCCESS.r,
                    g: constants::ui::dark::SUCCESS.g,
                    b: constants::ui::dark::SUCCESS.b,
                },
                warning: ColorRgb {
                    r: constants::ui::dark::WARNING.r,
                    g: constants::ui::dark::WARNING.g,
                    b: constants::ui::dark::WARNING.b,
                },
                error: ColorRgb {
                    r: constants::ui::dark::ERROR.r,
                    g: constants::ui::dark::ERROR.g,
                    b: constants::ui::dark::ERROR.b,
                },
                background: ColorRgb { r: 9, g: 9, b: 11 },
                foreground: ColorRgb {
                    r: 248,
                    g: 248,
                    b: 242,
                },
                border: ColorRgb {
                    r: 63,
                    g: 63,
                    b: 70,
                },
                selected_bg: ColorRgb {
                    r: constants::ui::dark::SELECTED_BG.r,
                    g: constants::ui::dark::SELECTED_BG.g,
                    b: constants::ui::dark::SELECTED_BG.b,
                },
                selected_fg: ColorRgb {
                    r: 255,
                    g: 255,
                    b: 255,
                },
                text_muted: ColorRgb {
                    r: 107,
                    g: 107,
                    b: 107,
                },
                border_focused: ColorRgb {
                    r: 56,
                    g: 189,
                    b: 248,
                },
            },
        }
    }

    /// Create light theme
    pub fn light() -> Self {
        Self {
            name: "light".to_string(),
            colors: ColorPalette {
                primary: ColorRgb {
                    r: constants::ui::light::PRIMARY.r,
                    g: constants::ui::light::PRIMARY.g,
                    b: constants::ui::light::PRIMARY.b,
                },
                secondary: ColorRgb {
                    r: 126,
                    g: 34,
                    b: 206,
                },
                success: ColorRgb {
                    r: constants::ui::light::SUCCESS.r,
                    g: constants::ui::light::SUCCESS.g,
                    b: constants::ui::light::SUCCESS.b,
                },
                warning: ColorRgb {
                    r: constants::ui::light::WARNING.r,
                    g: constants::ui::light::WARNING.g,
                    b: constants::ui::light::WARNING.b,
                },
                error: ColorRgb {
                    r: constants::ui::light::ERROR.r,
                    g: constants::ui::light::ERROR.g,
                    b: constants::ui::light::ERROR.b,
                },
                background: ColorRgb {
                    r: 255,
                    g: 255,
                    b: 255,
                },
                foreground: ColorRgb { r: 9, g: 9, b: 11 },
                border: ColorRgb {
                    r: 209,
                    g: 213,
                    b: 219,
                },
                selected_bg: ColorRgb {
                    r: constants::ui::light::SELECTED_BG.r,
                    g: constants::ui::light::SELECTED_BG.g,
                    b: constants::ui::light::SELECTED_BG.b,
                },
                selected_fg: ColorRgb {
                    r: 255,
                    g: 255,
                    b: 255,
                },
                text_muted: ColorRgb {
                    r: 156,
                    g: 163,
                    b: 175,
                },
                border_focused: ColorRgb {
                    r: 37,
                    g: 99,
                    b: 235,
                },
            },
        }
    }

    /// Create theme from config
    pub fn from_config(theme_name: &str) -> Self {
        match theme_name {
            "light" => Self::light(),
            _ => Self::dark(),
        }
    }

    /// Toggle between dark and light theme
    pub fn toggle(&self) -> Self {
        if self.name == "dark" {
            Self::light()
        } else {
            Self::dark()
        }
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
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
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
    fn test_styles() {
        let theme = Theme::dark();

        let selected = theme.selected_style();
        assert!(selected.add_modifier == Modifier::BOLD);

        let focused = theme.focused_border_style();
        assert_eq!(focused.fg, Some(theme.colors.border_focused.into()));
    }
}
