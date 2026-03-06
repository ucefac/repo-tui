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
pub const MIN_TERMINAL_WIDTH: u16 = 60;
pub const MIN_TERMINAL_HEIGHT: u16 = 20;

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
        use ratatui::style::Color;

        pub const PRIMARY: Color = Color::Rgb(88, 166, 255);
        pub const SUCCESS: Color = Color::Rgb(63, 185, 80);
        pub const WARNING: Color = Color::Rgb(210, 153, 34);
        pub const ERROR: Color = Color::Rgb(248, 81, 73);
        pub const SELECTED_BG: Color = Color::Rgb(56, 139, 253);
        pub const BORDER_FOCUSED: Color = Color::Cyan;
        pub const BORDER_NORMAL: Color = Color::DarkGray;
        pub const TEXT_PRIMARY: Color = Color::White;
        pub const TEXT_SECONDARY: Color = Color::Gray;
    }

    /// Light theme colors
    pub mod light {
        use ratatui::style::Color;

        pub const PRIMARY: Color = Color::Rgb(9, 105, 218);
        pub const SUCCESS: Color = Color::Rgb(26, 127, 55);
        pub const WARNING: Color = Color::Rgb(154, 103, 0);
        pub const ERROR: Color = Color::Rgb(209, 36, 47);
        pub const SELECTED_BG: Color = Color::Rgb(9, 105, 218);
        pub const BORDER_FOCUSED: Color = Color::Blue;
        pub const BORDER_NORMAL: Color = Color::DarkGray;
        pub const TEXT_PRIMARY: Color = Color::Black;
        pub const TEXT_SECONDARY: Color = Color::Gray;
    }
}

/// Security configuration
pub mod security {
    /// Default: deny symlinks
    pub const DEFAULT_ALLOW_SYMLINKS: bool = false;

    /// Maximum search depth
    pub const DEFAULT_MAX_SEARCH_DEPTH: usize = 2;
}

/// Help text
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
