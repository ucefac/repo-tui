//! Unified error types for repotui

use std::path::PathBuf;
use thiserror::Error;

/// Application-level errors
#[derive(Error, Debug, Clone)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Repository error: {0}")]
    Repo(#[from] RepoError),

    #[error("Action error: {0}")]
    Action(#[from] ActionError),

    #[error("Clone error: {0}")]
    Clone(#[from] CloneError),

    #[error("Terminal error: {0}")]
    Terminal(String),

    #[error("IO error: {0}")]
    Io(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, AppError>;

/// App-specific result type with AppError
pub type AppResult<T> = Result<T>;

/// Configuration errors
#[derive(Error, Debug, Clone)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    NotFound(PathBuf),

    #[error("Configuration parse error: {0}")]
    ParseError(String),

    #[error("Configuration serialize error: {0}")]
    SerializeError(String),

    #[error(
        "Configuration version too new: {current} (max supported: {max_supported}). {message}"
    )]
    VersionTooNew {
        current: String,
        max_supported: String,
        message: String,
    },

    #[error("Unsupported configuration version: {0}")]
    UnsupportedVersion(String),

    #[error("Main directory not found: {0}")]
    DirectoryNotFound(PathBuf),

    #[error("Path is not a directory: {0}")]
    NotADirectory(PathBuf),

    #[error("No read permission for directory: {0}")]
    NoReadPermission(PathBuf),

    #[error("Directory outside home directory: {0}")]
    DirectoryOutsideHome(PathBuf),

    #[error("Home directory not found")]
    HomeNotFound,

    #[error("Symlink not allowed: {0}")]
    SymlinkNotAllowed(PathBuf),

    #[error("Invalid editor command: {0}")]
    InvalidEditorCommand(String),

    #[error("Editor not found: {0}")]
    EditorNotFound(String),

    #[error("Editor not executable: {0}")]
    EditorNotExecutable(String),

    #[error("Path error: {0}")]
    PathError(String),

    #[error("IO error: {0}")]
    IoError(String),
}

/// Repository errors
#[derive(Error, Debug, Clone)]
pub enum RepoError {
    #[error("Failed to scan directory: {0}")]
    ScanFailed(String),

    #[error("Not a git repository: {0}")]
    NotGitRepo(PathBuf),

    #[error("Git command failed: {0}")]
    GitCommandFailed(String),

    #[error("Git error: {0}")]
    GitError(String),

    #[error("Path error: {0}")]
    PathError(String),
}

/// Action errors (command execution)
#[derive(Error, Debug, Clone)]
pub enum ActionError {
    #[error("Command not found: {0}")]
    CommandNotFound(String),

    #[error("Command not allowed: {0}")]
    CommandNotAllowed(String),

    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Command timed out")]
    Timeout,

    #[error("Path validation failed: {0}")]
    PathValidationFailed(String),

    #[error("Path outside allowed directory: {0}")]
    PathOutsideAllowed(PathBuf),

    #[error("Unsafe path detected: {0}")]
    UnsafePath(String),

    /// Indicates that terminal needs reinitialization after action execution
    /// This is used for interactive commands that take over the terminal
    #[error("Terminal needs reinitialization")]
    TerminalNeedsReinit,

    /// Indicates that the action executed successfully and the app should exit
    /// This is used for external TUI programs that should take over the terminal
    #[error("Exit after execution")]
    ExitAfterExecution,
}

/// Clone operation errors
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CloneError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("URL too long (max {0} characters)")]
    UrlTooLong(usize),

    #[error("Invalid URL format")]
    InvalidFormat,

    #[error("Invalid URL scheme: {0}")]
    InvalidScheme(String),

    #[error("Unsupported Git host: {0}")]
    UnsupportedHost(String),

    #[error("Repository already exists at: {0}")]
    AlreadyExists(PathBuf),

    #[error("Git command failed with code: {0:?}")]
    GitFailed(Option<i32>),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),

    #[error("Disk full")]
    DiskFull,

    #[error("Git not installed")]
    GitNotFound,

    #[error("Operation cancelled by user")]
    Cancelled,

    #[error("Target is not a git repository")]
    NotAGitRepository,

    #[error("Protected path cannot be removed: {0}")]
    ProtectedPath(PathBuf),

    #[error("Path outside allowed directory: {0}")]
    OutsideAllowedDirectory(PathBuf),

    #[error("Invalid characters in URL")]
    InvalidCharacters,

    #[error("Path error: {0}")]
    PathError(String),

    #[error("IO error: {0}")]
    Io(String),
}

/// Update check errors
#[derive(Error, Debug, Clone)]
pub enum UpdateError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Version parse error: {0}")]
    VersionParseError(String),

    #[error("No releases found")]
    NoReleasesFound,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

impl AppError {
    /// Get user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            AppError::Config(ConfigError::DirectoryNotFound(path)) => {
                format!(
                    "Main directory not found: {}\nPlease select a valid directory",
                    path.display()
                )
            }
            AppError::Config(ConfigError::NoReadPermission(path)) => {
                format!(
                    "No permission to read directory: {}\nPlease check directory permissions",
                    path.display()
                )
            }
            AppError::Action(ActionError::CommandNotFound(cmd)) => {
                format!(
                    "Command '{}' not found\nPlease ensure it is installed and in PATH",
                    cmd
                )
            }
            AppError::Action(ActionError::CommandNotAllowed(cmd)) => {
                format!(
                    "Command '{}' is not in the allowed list\nPlease check security configuration",
                    cmd
                )
            }
            AppError::Repo(RepoError::NotGitRepo(path)) => {
                format!("Not a git repository: {}", path.display())
            }
            AppError::Clone(clone_err) => clone_err.user_message(),
            _ => self.to_string(),
        }
    }

    /// Get error severity
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            AppError::Config(ConfigError::NotFound(_)) => ErrorSeverity::Info,
            AppError::Config(ConfigError::DirectoryNotFound(_)) => ErrorSeverity::Warning,
            AppError::Config(ConfigError::NoReadPermission(_)) => ErrorSeverity::Error,
            AppError::Config(ConfigError::HomeNotFound) => ErrorSeverity::Error,
            AppError::Action(ActionError::CommandNotFound(_)) => ErrorSeverity::Warning,
            AppError::Action(ActionError::CommandNotAllowed(_)) => ErrorSeverity::Error,
            AppError::Action(ActionError::ExecutionFailed(_)) => ErrorSeverity::Error,
            AppError::Repo(RepoError::NotGitRepo(_)) => ErrorSeverity::Info,
            AppError::Clone(clone_err) => clone_err.severity(),
            _ => ErrorSeverity::Info,
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        ConfigError::IoError(e.to_string())
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(e: toml::de::Error) -> Self {
        ConfigError::ParseError(e.to_string())
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(e: toml::ser::Error) -> Self {
        ConfigError::SerializeError(e.to_string())
    }
}

impl ConfigError {
    /// Get user-friendly message
    pub fn user_message(&self) -> String {
        match self {
            ConfigError::DirectoryNotFound(path) => {
                format!("Directory not found: {}", path.display())
            }
            ConfigError::NoReadPermission(path) => {
                format!("Permission denied: {}", path.display())
            }
            ConfigError::HomeNotFound => "Home directory not found".to_string(),
            ConfigError::NotFound(path) => {
                format!("Config file not found: {}", path.display())
            }
            _ => self.to_string(),
        }
    }
}

impl RepoError {
    pub fn user_message(&self) -> String {
        match self {
            RepoError::NotGitRepo(path) => {
                format!("Not a git repository: {}", path.display())
            }
            _ => self.to_string(),
        }
    }
}

impl ActionError {
    pub fn user_message(&self) -> String {
        match self {
            ActionError::CommandNotFound(cmd) => {
                match cmd.as_str() {
                    "code" => "默认情况下，VS Code 安装后不会自动添加 code 命令到终端。\n你需要手动配置：\n1. 打开 VS Code\n2. 按 Cmd+Shift+P 打开命令面板\n3. 输入并选择 Shell Command: Install 'code' command in PATH\n4. 这会将 code 命令添加到 /usr/local/bin".to_string(),
                    "webstorm" => "WebStorm 未安装".to_string(),
                    "idea" => "IDEA 未安装".to_string(),
                    "opencode" => "OpenCode 未安装\n终端执行命令：brew install anomalyco/tap/opencode".to_string(),
                    "claude" => "Claude Code 未安装\n终端执行命令：brew install --cask claude-code".to_string(),
                    _ => format!("Command not found: {}", cmd),
                }
            }
            ActionError::CommandNotAllowed(cmd) => {
                format!("Command not allowed: {}", cmd)
            }
            ActionError::TerminalNeedsReinit => {
                // This is not an error, just a signal - return empty string
                String::new()
            }
            _ => self.to_string(),
        }
    }
}

impl CloneError {
    /// Get user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            CloneError::InvalidUrl(url) => {
                format!("Invalid repository URL: {}", url)
            }
            CloneError::UrlTooLong(max) => {
                format!("URL too long (max {} characters)", max)
            }
            CloneError::InvalidFormat => {
                "Invalid URL format. Example: https://github.com/owner/repo".to_string()
            }
            CloneError::InvalidScheme(scheme) => {
                format!("Unsupported URL scheme: {}. Use https:// or git@", scheme)
            }
            CloneError::UnsupportedHost(host) => {
                format!("Unsupported Git host: {}", host)
            }
            CloneError::AlreadyExists(path) => {
                format!("Repository already exists at: {}", path.display())
            }
            CloneError::GitFailed(code) => {
                format!("Git clone failed (code: {:?}). Check the URL and your permissions", code)
            }
            CloneError::Network(msg) => {
                format!("Network error: {}. Check your connection", msg)
            }
            CloneError::PermissionDenied(path) => {
                format!("Permission denied: {}. Check directory permissions", path.display())
            }
            CloneError::DiskFull => {
                "Not enough disk space".to_string()
            }
            CloneError::GitNotFound => {
                "Git not found. Please install Git".to_string()
            }
            CloneError::Cancelled => {
                "Clone operation cancelled".to_string()
            }
            CloneError::NotAGitRepository => {
                "Target is not a Git repository".to_string()
            }
            CloneError::ProtectedPath(path) => {
                format!("Cannot remove protected path: {}", path.display())
            }
            CloneError::OutsideAllowedDirectory(path) => {
                format!("Path outside allowed directory: {}", path.display())
            }
            _ => self.to_string(),
        }
    }

    /// Get error severity
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            CloneError::Cancelled => ErrorSeverity::Info,
            CloneError::InvalidUrl(_) | CloneError::InvalidFormat => ErrorSeverity::Warning,
            CloneError::AlreadyExists(_) => ErrorSeverity::Warning,
            CloneError::Network(_) => ErrorSeverity::Warning,
            _ => ErrorSeverity::Error,
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_user_message() {
        let err = AppError::Config(ConfigError::DirectoryNotFound(PathBuf::from("/test")));
        assert!(err.user_message().contains("Main directory not found"));
    }

    #[test]
    fn test_error_severity() {
        let err = AppError::Config(ConfigError::NotFound(PathBuf::from("/test")));
        assert_eq!(err.severity(), ErrorSeverity::Info);

        let err = AppError::Config(ConfigError::NoReadPermission(PathBuf::from("/test")));
        assert_eq!(err.severity(), ErrorSeverity::Error);
    }
}
