use crate::ui::theme::{ColorPalette, ColorRgb, Theme};

pub fn dark_theme() -> Theme {
    Theme {
        name: "dark".to_string(),
        colors: ColorPalette {
            primary: ColorRgb {
                r: 88,
                g: 166,
                b: 255,
            },
            secondary: ColorRgb {
                r: 139,
                g: 92,
                b: 246,
            },
            success: ColorRgb {
                r: 63,
                g: 185,
                b: 80,
            },
            warning: ColorRgb {
                r: 210,
                g: 153,
                b: 34,
            },
            error: ColorRgb {
                r: 248,
                g: 81,
                b: 73,
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
                r: 56,
                g: 139,
                b: 253,
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
