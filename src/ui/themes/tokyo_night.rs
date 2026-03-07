use crate::ui::theme::{ColorPalette, ColorRgb, Theme};

/// Tokyo Night Theme - Modern dark theme
/// https://github.com/enkia/tokyo-night
pub fn tokyo_night_theme() -> Theme {
    Theme {
        name: "tokyo_night".to_string(),
        colors: ColorPalette {
            background: ColorRgb {
                r: 26,
                g: 27,
                b: 38,
            },
            foreground: ColorRgb {
                r: 192,
                g: 202,
                b: 245,
            },
            primary: ColorRgb {
                r: 122,
                g: 162,
                b: 247,
            },
            secondary: ColorRgb {
                r: 187,
                g: 154,
                b: 247,
            },
            success: ColorRgb {
                r: 158,
                g: 206,
                b: 106,
            },
            warning: ColorRgb {
                r: 224,
                g: 175,
                b: 104,
            },
            error: ColorRgb {
                r: 247,
                g: 118,
                b: 142,
            },
            border: ColorRgb {
                r: 41,
                g: 46,
                b: 66,
            },
            selected_bg: ColorRgb {
                r: 51,
                g: 59,
                b: 91,
            },
            selected_fg: ColorRgb {
                r: 192,
                g: 202,
                b: 245,
            },
            text_muted: ColorRgb {
                r: 86,
                g: 95,
                b: 137,
            },
            border_focused: ColorRgb {
                r: 122,
                g: 162,
                b: 247,
            },
        },
    }
}
