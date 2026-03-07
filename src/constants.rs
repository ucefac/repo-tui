//! Application constants

/// Application name
#[allow(dead_code)]
pub const APP_NAME: &str = "repotui";

/// Configuration directory name
pub const CONFIG_DIR_NAME: &str = "repotui";

/// Configuration file name
pub const CONFIG_FILE_NAME: &str = "config.toml";

/// Configuration version
pub const CONFIG_VERSION: &str = "1.0";

/// Minimum terminal dimensions
pub const MIN_TERMINAL_WIDTH: u16 = 80;
pub const MIN_TERMINAL_HEIGHT: u16 = 24;

/// Search debounce duration (milliseconds)
pub const SEARCH_DEBOUNCE_MS: u64 = 100;

/// Git status cache TTL (seconds)
#[allow(dead_code)]
pub const GIT_STATUS_CACHE_TTL_SECS: u64 = 300;

/// Git status cache max size
#[allow(dead_code)]
pub const GIT_STATUS_CACHE_MAX_SIZE: usize = 1000;

/// Command execution timeout (seconds)
#[allow(dead_code)]
pub const COMMAND_TIMEOUT_SECS: u64 = 5;

/// Allowed commands whitelist
pub const ALLOWED_COMMANDS: &[&str] = &["claude", "cursor", "cline"];

/// Allowed editors whitelist
pub const ALLOWED_EDITORS: &[&str] = &[
    "code",
    "code-insiders",
    "webstorm",
    "idea",
    "pycharm",
    "vim",
    "nvim",
    "emacs",
    "nano",
];

/// Shell special characters that need escaping
#[allow(dead_code)]
pub const SHELL_SPECIAL_CHARS: &[char] = &[
    ' ', '\'', '"', '\\', '`', '$', '|', '&', ';', '<', '>', '(', ')', '{', '}', '[', ']', '*',
    '?', '~', '!', '#', '%', '@',
];

/// RGB color representation for constants
#[derive(Debug, Clone, Copy)]
pub struct ColorRgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<ColorRgb> for ratatui::style::Color {
    fn from(rgb: ColorRgb) -> Self {
        ratatui::style::Color::Rgb(rgb.r, rgb.g, rgb.b)
    }
}

/// Default UI configuration
pub mod ui {

    /// Default theme
    pub const DEFAULT_THEME: &str = "dark";

    /// Show git status by default
    #[allow(dead_code)]
    pub const DEFAULT_SHOW_GIT_STATUS: bool = true;

    /// Show branch by default
    #[allow(dead_code)]
    pub const DEFAULT_SHOW_BRANCH: bool = true;

    /// Dark theme colors
    pub mod dark {
        use crate::constants::ColorRgb;

        pub const PRIMARY: ColorRgb = ColorRgb {
            r: 88,
            g: 166,
            b: 255,
        };
        pub const SUCCESS: ColorRgb = ColorRgb {
            r: 63,
            g: 185,
            b: 80,
        };
        pub const WARNING: ColorRgb = ColorRgb {
            r: 210,
            g: 153,
            b: 34,
        };
        pub const ERROR: ColorRgb = ColorRgb {
            r: 248,
            g: 81,
            b: 73,
        };
        pub const SELECTED_BG: ColorRgb = ColorRgb {
            r: 56,
            g: 139,
            b: 253,
        };
        #[allow(dead_code)]
        pub const BORDER_FOCUSED: ColorRgb = ColorRgb {
            r: 56,
            g: 189,
            b: 248,
        };
        #[allow(dead_code)]
        pub const BORDER_NORMAL: ColorRgb = ColorRgb {
            r: 107,
            g: 107,
            b: 107,
        };
        #[allow(dead_code)]
        pub const TEXT_PRIMARY: ColorRgb = ColorRgb {
            r: 248,
            g: 248,
            b: 242,
        };
        #[allow(dead_code)]
        pub const TEXT_SECONDARY: ColorRgb = ColorRgb {
            r: 156,
            g: 163,
            b: 175,
        };
    }

    /// Light theme colors
    pub mod light {
        use crate::constants::ColorRgb;

        pub const PRIMARY: ColorRgb = ColorRgb {
            r: 9,
            g: 105,
            b: 218,
        };
        pub const SUCCESS: ColorRgb = ColorRgb {
            r: 26,
            g: 127,
            b: 55,
        };
        pub const WARNING: ColorRgb = ColorRgb {
            r: 154,
            g: 103,
            b: 0,
        };
        pub const ERROR: ColorRgb = ColorRgb {
            r: 209,
            g: 36,
            b: 47,
        };
        pub const SELECTED_BG: ColorRgb = ColorRgb {
            r: 9,
            g: 105,
            b: 218,
        };
        #[allow(dead_code)]
        pub const BORDER_FOCUSED: ColorRgb = ColorRgb {
            r: 37,
            g: 99,
            b: 235,
        };
        #[allow(dead_code)]
        pub const BORDER_NORMAL: ColorRgb = ColorRgb {
            r: 209,
            g: 213,
            b: 219,
        };
        #[allow(dead_code)]
        pub const TEXT_PRIMARY: ColorRgb = ColorRgb { r: 9, g: 9, b: 11 };
        #[allow(dead_code)]
        pub const TEXT_SECONDARY: ColorRgb = ColorRgb {
            r: 107,
            g: 107,
            b: 107,
        };
    }
}

/// Security configuration
pub mod security {
    /// Default: deny symlinks
    pub const DEFAULT_ALLOW_SYMLINKS: bool = false;

    /// Maximum search depth
    pub const DEFAULT_MAX_SEARCH_DEPTH: usize = 2;
}

/// Help text (deprecated: use HelpPanel widget instead)
#[allow(dead_code)]
pub const HELP_TEXT: &str = r#"Keyboard Shortcuts

Navigation
  j/↓     Move down
  k/↑     Move up
  g       Go to top
  G       Go to bottom
  Ctrl+d  Scroll down half-page
  Ctrl+u  Scroll up half-page

Search
  /       Focus search
  Esc     Clear search / Close panel
  [char]  Add to search query (when focused)

Actions
  Enter   Open action menu
  o       Open action menu
  c       cd + cloud (claude)
  w       Open in WebStorm
  v       Open in VS Code
  f       Open in Finder/Explorer
  r       Refresh list
  ?       Show this help
  q       Quit

State Priority
Higher priority states intercept all key events:
ActionMenu > Help > ChoosingDir > Searching > Running
"#;
