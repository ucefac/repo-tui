use crate::ui::theme::{ColorPalette, ColorRgb, Theme};

/// Nord Theme - Arctic North Blue
/// https://www.nordtheme.com/
pub fn nord_theme() -> Theme {
    Theme {
        name: "nord".to_string(),
        colors: ColorPalette {
            background: ColorRgb {
                r: 47,
                g: 52,
                b: 64,
            },
            foreground: ColorRgb {
                r: 216,
                g: 222,
                b: 233,
            },
            primary: ColorRgb {
                r: 136,
                g: 192,
                b: 208,
            },
            secondary: ColorRgb {
                r: 129,
                g: 162,
                b: 190,
            },
            success: ColorRgb {
                r: 163,
                g: 190,
                b: 140,
            },
            warning: ColorRgb {
                r: 235,
                g: 203,
                b: 139,
            },
            error: ColorRgb {
                r: 191,
                g: 97,
                b: 106,
            },
            border: ColorRgb {
                r: 67,
                g: 76,
                b: 94,
            },
            selected_bg: ColorRgb {
                r: 67,
                g: 76,
                b: 94,
            },
            selected_fg: ColorRgb {
                r: 216,
                g: 222,
                b: 233,
            },
            text_muted: ColorRgb {
                r: 94,
                g: 109,
                b: 133,
            },
            border_focused: ColorRgb {
                r: 136,
                g: 192,
                b: 208,
            },
            title_fg: ColorRgb {
                r: 216,
                g: 222,
                b: 233,
            },
            title_bg: ColorRgb {
                r: 47,
                g: 52,
                b: 64,
            },
        },
    }
}
