//! Action validators

use crate::action::Action;
use crate::constants::{ALLOWED_COMMANDS, ALLOWED_EDITORS};
use crate::error::{ActionError, AppError, AppResult};

/// Validate action can be executed
pub fn validate_action(action: &Action) -> AppResult<()> {
    match action {
        Action::CdAndCloud => {
            // Check if claude is allowed and available
            if !is_command_allowed("claude") {
                return Err(AppError::Action(ActionError::CommandNotAllowed(
                    "claude".to_string(),
                )));
            }

            // Check if command exists in PATH
            if which::which("claude").is_err() {
                return Err(AppError::Action(ActionError::CommandNotFound(
                    "claude".to_string(),
                )));
            }
        }

        Action::OpenWebStorm => {
            if !is_editor_allowed("webstorm") {
                return Err(AppError::Action(ActionError::CommandNotAllowed(
                    "webstorm".to_string(),
                )));
            }

            if which::which("webstorm").is_err() {
                return Err(AppError::Action(ActionError::CommandNotFound(
                    "webstorm".to_string(),
                )));
            }
        }

        Action::OpenVsCode => {
            if !is_editor_allowed("code") {
                return Err(AppError::Action(ActionError::CommandNotAllowed(
                    "code".to_string(),
                )));
            }

            if which::which("code").is_err() {
                return Err(AppError::Action(ActionError::CommandNotFound(
                    "code".to_string(),
                )));
            }
        }

        Action::OpenIntelliJ => {
            if !is_editor_allowed("idea") {
                return Err(AppError::Action(ActionError::CommandNotAllowed(
                    "idea".to_string(),
                )));
            }

            if which::which("idea").is_err() {
                return Err(AppError::Action(ActionError::CommandNotFound(
                    "idea".to_string(),
                )));
            }
        }

        Action::OpenOpenCode => {
            if !is_command_allowed("opencode") {
                return Err(AppError::Action(ActionError::CommandNotAllowed(
                    "opencode".to_string(),
                )));
            }

            if which::which("opencode").is_err() {
                return Err(AppError::Action(ActionError::CommandNotFound(
                    "opencode".to_string(),
                )));
            }
        }

        Action::OpenFileManager => {
            // File manager, commands are platform-specific and always allowed
            #[cfg(target_os = "macos")]
            if which::which("open").is_err() {
                return Err(AppError::Action(ActionError::CommandNotFound(
                    "open".to_string(),
                )));
            }

            #[cfg(target_os = "linux")]
            if which::which("xdg-open").is_err() {
                return Err(AppError::Action(ActionError::CommandNotFound(
                    "xdg-open".to_string(),
                )));
            }

            #[cfg(target_os = "windows")]
            {
                // Windows explorer is always available
            }
        }

        Action::OpenLazyGit => {
            if !is_command_allowed("lazygit") {
                return Err(AppError::Action(ActionError::CommandNotAllowed(
                    "lazygit".to_string(),
                )));
            }

            if which::which("lazygit").is_err() {
                return Err(AppError::Action(ActionError::CommandNotFound(
                    "lazygit".to_string(),
                )));
            }
        }
    }

    Ok(())
}

/// Check if command is in allowed list
fn is_command_allowed(cmd: &str) -> bool {
    ALLOWED_COMMANDS.contains(&cmd)
}

/// Check if editor is in allowed list
fn is_editor_allowed(editor: &str) -> bool {
    ALLOWED_EDITORS.contains(&editor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_action_cd_cloud() {
        // This test may fail if claude is not installed
        let result = validate_action(&Action::CdAndCloud);
        // We just check the validation logic works
        // The actual result depends on system state
        assert!(
            result.is_ok()
                || matches!(
                    result,
                    Err(AppError::Action(ActionError::CommandNotFound(_)))
                )
        );
    }

    #[test]
    fn test_is_command_allowed() {
        assert!(is_command_allowed("claude"));
        assert!(!is_command_allowed("malicious-command"));
    }

    #[test]
    fn test_is_editor_allowed() {
        assert!(is_editor_allowed("code"));
        assert!(is_editor_allowed("webstorm"));
        assert!(!is_editor_allowed("malicious-editor"));
    }
}
