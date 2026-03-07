use crate::ui::theme::{ColorPalette, ColorRgb, Theme};

/// Dracula Theme - Popular dark theme
/// https://draculatheme.com/
pub fn dracula_theme() -> Theme {
    Theme {
        name: "dracula".to_string(),
        colors: ColorPalette {
            background: ColorRgb {
                r: 40,
                g: 42,
                b: 54,
            },
            foreground: ColorRgb {
                r: 248,
                g: 248,
                b: 242,
            },
            primary: ColorRgb {
                r: 139,
                g: 233,
                b: 253,
            },
            secondary: ColorRgb {
                r: 189,
                g: 147,
                b: 249,
            },
            success: ColorRgb {
                r: 80,
                g: 250,
                b: 123,
            },
            warning: ColorRgb {
                r: 241,
                g: 250,
                b: 140,
            },
            error: ColorRgb {
                r: 255,
                g: 85,
                b: 85,
            },
            border: ColorRgb {
                r: 62,
                g: 72,
                b: 136,
            },
            selected_bg: ColorRgb {
                r: 98,
                g: 114,
                b: 164,
            },
            selected_fg: ColorRgb {
                r: 248,
                g: 248,
                b: 242,
            },
            text_muted: ColorRgb {
                r: 98,
                g: 114,
                b: 164,
            },
            border_focused: ColorRgb {
                r: 189,
                g: 147,
                b: 249,
            },
        },
    }
}
