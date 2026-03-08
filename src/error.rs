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
                format!("Command not found: {}", cmd)
            }
            ActionError::CommandNotAllowed(cmd) => {
                format!("Command not allowed: {}", cmd)
            }
            _ => self.to_string(),
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
