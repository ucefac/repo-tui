use crate::ui::theme::{ColorPalette, ColorRgb, Theme};

/// Catppuccin Mocha Theme - Soothing dark theme
/// https://github.com/catppuccin/catppuccin
pub fn catppuccin_mocha_theme() -> Theme {
    Theme {
        name: "catppuccin_mocha".to_string(),
        colors: ColorPalette {
            background: ColorRgb {
                r: 30,
                g: 30,
                b: 46,
            },
            foreground: ColorRgb {
                r: 205,
                g: 214,
                b: 244,
            },
            primary: ColorRgb {
                r: 137,
                g: 180,
                b: 250,
            },
            secondary: ColorRgb {
                r: 203,
                g: 166,
                b: 247,
            },
            success: ColorRgb {
                r: 166,
                g: 227,
                b: 161,
            },
            warning: ColorRgb {
                r: 250,
                g: 179,
                b: 135,
            },
            error: ColorRgb {
                r: 243,
                g: 139,
                b: 168,
            },
            border: ColorRgb {
                r: 49,
                g: 50,
                b: 68,
            },
            selected_bg: ColorRgb {
                r: 88,
                g: 91,
                b: 112,
            },
            selected_fg: ColorRgb {
                r: 205,
                g: 214,
                b: 244,
            },
            text_muted: ColorRgb {
                r: 108,
                g: 112,
                b: 134,
            },
            border_focused: ColorRgb {
                r: 137,
                g: 180,
                b: 250,
            },
        },
    }
}
