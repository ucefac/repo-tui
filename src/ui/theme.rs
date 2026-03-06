//! UI theme configuration

use crate::constants;
use ratatui::style::{Color, Modifier, Style};

/// UI Theme
#[derive(Debug, Clone)]
pub struct Theme {
    pub primary: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub selected_bg: Color,
    pub selected_fg: Color,
    pub border_focused: Color,
    pub border_normal: Color,
    pub border_active: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub cursor: Color,
    pub highlight: Color,
    pub background: Color,
}

impl Theme {
    /// Create dark theme
    pub fn dark() -> Self {
        Self {
            primary: constants::ui::dark::PRIMARY,
            success: constants::ui::dark::SUCCESS,
            warning: constants::ui::dark::WARNING,
            error: constants::ui::dark::ERROR,
            selected_bg: constants::ui::dark::SELECTED_BG,
            selected_fg: Color::White,
            border_focused: constants::ui::dark::BORDER_FOCUSED,
            border_normal: constants::ui::dark::BORDER_NORMAL,
            border_active: constants::ui::dark::PRIMARY,
            text_primary: constants::ui::dark::TEXT_PRIMARY,
            text_secondary: constants::ui::dark::TEXT_SECONDARY,
            text_muted: Color::DarkGray,
            cursor: constants::ui::dark::PRIMARY,
            highlight: constants::ui::dark::PRIMARY,
            background: Color::Black,
        }
    }

    /// Create light theme
    pub fn light() -> Self {
        Self {
            primary: constants::ui::light::PRIMARY,
            success: constants::ui::light::SUCCESS,
            warning: constants::ui::light::WARNING,
            error: constants::ui::light::ERROR,
            selected_bg: constants::ui::light::SELECTED_BG,
            selected_fg: Color::White,
            border_focused: constants::ui::light::BORDER_FOCUSED,
            border_normal: constants::ui::light::BORDER_NORMAL,
            border_active: constants::ui::light::PRIMARY,
            text_primary: constants::ui::light::TEXT_PRIMARY,
            text_secondary: constants::ui::light::TEXT_SECONDARY,
            text_muted: Color::Gray,
            cursor: constants::ui::light::PRIMARY,
            highlight: constants::ui::light::PRIMARY,
            background: Color::White,
        }
    }

    /// Create theme from config
    pub fn from_config(theme_name: &str) -> Self {
        match theme_name {
            "light" => Self::light(),
            _ => Self::dark(),
        }
    }

    /// Get selected style
    pub fn selected_style(&self) -> Style {
        Style::default()
            .fg(self.selected_fg)
            .bg(self.selected_bg)
            .add_modifier(Modifier::BOLD)
    }

    /// Get focused border style
    pub fn focused_border_style(&self) -> Style {
        Style::default().fg(self.border_focused)
    }

    /// Get normal border style
    pub fn normal_border_style(&self) -> Style {
        Style::default().fg(self.border_normal)
    }

    /// Get primary text style
    pub fn primary_text_style(&self) -> Style {
        Style::default().fg(self.text_primary)
    }

    /// Get secondary text style
    pub fn secondary_text_style(&self) -> Style {
        Style::default().fg(self.text_secondary)
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
        assert_eq!(theme.primary, constants::ui::dark::PRIMARY);
    }

    #[test]
    fn test_light_theme() {
        let theme = Theme::light();
        assert_eq!(theme.primary, constants::ui::light::PRIMARY);
    }

    #[test]
    fn test_from_config() {
        let dark = Theme::from_config("dark");
        assert_eq!(dark.background, Color::Black);

        let light = Theme::from_config("light");
        assert_eq!(light.background, Color::White);

        let default = Theme::from_config("unknown");
        assert_eq!(default.background, Color::Black);
    }

    #[test]
    fn test_styles() {
        let theme = Theme::dark();

        let selected = theme.selected_style();
        assert!(selected.add_modifier == Modifier::BOLD);

        let focused = theme.focused_border_style();
        assert_eq!(focused.fg, Some(constants::ui::dark::BORDER_FOCUSED));
    }
}
