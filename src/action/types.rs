//! Action types

use serde::{Deserialize, Serialize};

/// Available actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    /// cd into repo + start claude
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
            Action::CdAndCloud => 'c',
            Action::OpenWebStorm => 'w',
            Action::OpenVsCode => 'v',
            Action::OpenFileManager => 'f',
        }
    }

    /// Get action description
    pub fn description(&self) -> &'static str {
        match self {
            Action::CdAndCloud => "cd + cloud (claude)",
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
        assert_eq!(Action::CdAndCloud.shortcut(), 'c');
        assert_eq!(Action::OpenWebStorm.shortcut(), 'w');
    }

    #[test]
    fn test_action_description() {
        assert!(Action::CdAndCloud.description().contains("claude"));
        assert!(Action::OpenWebStorm.description().contains("WebStorm"));
    }

    #[test]
    fn test_action_all() {
        let actions = Action::all();
        assert_eq!(actions.len(), 4);
    }
}
