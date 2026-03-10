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

    /// Open in IntelliJ IDEA
    OpenIntelliJ,

    /// Open in OpenCode
    OpenOpenCode,

    /// Open in LazyGit
    OpenLazyGit,
}

impl Action {
    /// Get action shortcut key
    pub fn shortcut(&self) -> char {
        match self {
            Action::CdAndCloud => '1',
            Action::OpenWebStorm => '2',
            Action::OpenVsCode => '3',
            Action::OpenFileManager => '4',
            Action::OpenIntelliJ => '5',
            Action::OpenOpenCode => '6',
            Action::OpenLazyGit => '7',
        }
    }

    /// Get action description
    pub fn description(&self) -> &'static str {
        match self {
            Action::CdAndCloud => "Claude Code",
            Action::OpenWebStorm => "WebStorm",
            Action::OpenVsCode => "VS Code",
            Action::OpenFileManager => "Finder/Explorer",
            Action::OpenIntelliJ => "IntelliJ IDEA",
            Action::OpenOpenCode => "OpenCode",
            Action::OpenLazyGit => "LazyGit",
        }
    }

    /// Get all available actions
    pub fn all() -> Vec<Self> {
        vec![
            Action::CdAndCloud,
            Action::OpenWebStorm,
            Action::OpenVsCode,
            Action::OpenFileManager,
            Action::OpenIntelliJ,
            Action::OpenOpenCode,
            Action::OpenLazyGit,
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
        assert_eq!(Action::OpenIntelliJ.shortcut(), '5');
        assert_eq!(Action::OpenOpenCode.shortcut(), '6');
        assert_eq!(Action::OpenLazyGit.shortcut(), '7');
    }

    #[test]
    fn test_action_description() {
        assert_eq!(Action::CdAndCloud.description(), "Claude Code");
        assert_eq!(Action::OpenWebStorm.description(), "WebStorm");
        assert_eq!(Action::OpenVsCode.description(), "VS Code");
        assert_eq!(Action::OpenFileManager.description(), "Finder/Explorer");
        assert_eq!(Action::OpenIntelliJ.description(), "IntelliJ IDEA");
        assert_eq!(Action::OpenOpenCode.description(), "OpenCode");
        assert_eq!(Action::OpenLazyGit.description(), "LazyGit");
    }

    #[test]
    fn test_action_all() {
        let actions = Action::all();
        assert_eq!(actions.len(), 7);
    }
}
