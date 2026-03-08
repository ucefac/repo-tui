use crate::ui::theme::{ColorPalette, ColorRgb, Theme};

pub fn light_theme() -> Theme {
    Theme {
        name: "light".to_string(),
        colors: ColorPalette {
            primary: ColorRgb {
                r: 9,
                g: 105,
                b: 218,
            },
            secondary: ColorRgb {
                r: 126,
                g: 34,
                b: 206,
            },
            success: ColorRgb {
                r: 26,
                g: 127,
                b: 55,
            },
            warning: ColorRgb {
                r: 154,
                g: 103,
                b: 0,
            },
            error: ColorRgb {
                r: 209,
                g: 36,
                b: 47,
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
                r: 9,
                g: 105,
                b: 218,
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
            title_fg: ColorRgb { r: 9, g: 9, b: 11 },
            title_bg: ColorRgb {
                r: 255,
                g: 255,
                b: 255,
            },
        },
    }
}
