use crate::ui::theme::{ColorPalette, ColorRgb, Theme};

/// Gruvbox Dark Theme - Retro soft colors
/// https://github.com/morhetz/gruvbox
pub fn gruvbox_dark_theme() -> Theme {
    Theme {
        name: "gruvbox_dark".to_string(),
        colors: ColorPalette {
            background: ColorRgb {
                r: 40,
                g: 40,
                b: 40,
            },
            foreground: ColorRgb {
                r: 235,
                g: 219,
                b: 178,
            },
            primary: ColorRgb {
                r: 131,
                g: 165,
                b: 152,
            },
            secondary: ColorRgb {
                r: 211,
                g: 134,
                b: 155,
            },
            success: ColorRgb {
                r: 152,
                g: 195,
                b: 121,
            },
            warning: ColorRgb {
                r: 254,
                g: 128,
                b: 25,
            },
            error: ColorRgb {
                r: 204,
                g: 36,
                b: 29,
            },
            border: ColorRgb {
                r: 60,
                g: 58,
                b: 50,
            },
            selected_bg: ColorRgb {
                r: 102,
                g: 92,
                b: 84,
            },
            selected_fg: ColorRgb {
                r: 235,
                g: 219,
                b: 178,
            },
            text_muted: ColorRgb {
                r: 146,
                g: 131,
                b: 116,
            },
            border_focused: ColorRgb {
                r: 254,
                g: 128,
                b: 25,
            },
        },
    }
}
