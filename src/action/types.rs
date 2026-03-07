//! Action types

use serde::{Deserialize, Serialize};

/// Available actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Action {
    /// Open in Claude Code
    CdAndCloud,

    /// Open in WebStorm
    OpenWebStorm,

    /// Open in VS Code
    OpenVsCode,

    /// Open in Finder/Explorer
    OpenFileManager,
}

impl Action {
    /// Get action shortcut key
    pub fn shortcut(&self) -> char {
        match self {
            Action::CdAndCloud => '1',
            Action::OpenWebStorm => '2',
            Action::OpenVsCode => '3',
            Action::OpenFileManager => '4',
        }
    }

    /// Get action description
    pub fn description(&self) -> &'static str {
        match self {
            Action::CdAndCloud => "Open in Claude Code",
            Action::OpenWebStorm => "Open in WebStorm",
            Action::OpenVsCode => "Open in VS Code",
            Action::OpenFileManager => "Open in Finder/Explorer",
        }
    }

    /// Get all available actions
    pub fn all() -> Vec<Self> {
        vec![
            Action::CdAndCloud,
            Action::OpenWebStorm,
            Action::OpenVsCode,
            Action::OpenFileManager,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_shortcut() {
        assert_eq!(Action::CdAndCloud.shortcut(), '1');
        assert_eq!(Action::OpenWebStorm.shortcut(), '2');
        assert_eq!(Action::OpenVsCode.shortcut(), '3');
        assert_eq!(Action::OpenFileManager.shortcut(), '4');
    }

    #[test]
    fn test_action_description() {
        assert!(Action::CdAndCloud.description().contains("Claude Code"));
        assert!(Action::OpenWebStorm.description().contains("WebStorm"));
    }

    #[test]
    fn test_action_all() {
        let actions = Action::all();
        assert_eq!(actions.len(), 4);
    }
}
