//! Application constants

/// Application name
#[allow(dead_code)]
pub const APP_NAME: &str = "repotui";

/// Configuration directory name
pub const CONFIG_DIR_NAME: &str = "repotui";

/// Configuration file name
pub const CONFIG_FILE_NAME: &str = "config.toml";

/// Configuration version
pub const CONFIG_VERSION: &str = "2.0";

/// Maximum supported config version
pub const MAX_SUPPORTED_VERSION: &str = "2.0";

/// Minimum supported config version (for future use)
#[allow(dead_code)]
pub const MIN_SUPPORTED_VERSION: &str = "1.0";

/// Minimum terminal dimensions
pub const MIN_TERMINAL_WIDTH: u16 = 80;
pub const MIN_TERMINAL_HEIGHT: u16 = 25;

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

/// Maximum URL length for clone operations
pub const MAX_URL_LENGTH: usize = 2048;

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

/// Security configuration
pub mod security {
    /// Default: deny symlinks
    pub const DEFAULT_ALLOW_SYMLINKS: bool = false;

    /// Maximum search depth
    pub const DEFAULT_MAX_SEARCH_DEPTH: usize = 2;
}
