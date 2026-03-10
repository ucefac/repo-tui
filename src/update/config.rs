//! Update configuration

use serde::{Deserialize, Serialize};

/// Auto-update configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    /// Whether auto-check is enabled
    #[serde(default = "default_true")]
    pub auto_check_enabled: bool,
    /// Check interval in hours (default: 24)
    #[serde(default = "default_check_interval_hours")]
    pub check_interval_hours: u64,
    /// Ignored version (user chose to skip this version)
    #[serde(default)]
    pub ignored_version: Option<String>,
    /// Last check time
    #[serde(default)]
    pub last_checked_at: Option<chrono::DateTime<chrono::Local>>,
    /// GitHub repository owner
    #[serde(default = "default_owner")]
    pub github_owner: String,
    /// GitHub repository name
    #[serde(default = "default_repo")]
    pub github_repo: String,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            auto_check_enabled: true,
            check_interval_hours: 24,
            ignored_version: None,
            last_checked_at: None,
            github_owner: default_owner(),
            github_repo: default_repo(),
        }
    }
}

impl UpdateConfig {
    /// Check if enough time has passed since last check
    pub fn should_check(&self) -> bool {
        if !self.auto_check_enabled {
            return false;
        }

        match self.last_checked_at {
            None => true,
            Some(last) => {
                let elapsed = chrono::Local::now() - last;
                let interval_hours = self.check_interval_hours as i64;
                elapsed.num_hours() >= interval_hours
            }
        }
    }

    /// Check if a version is ignored
    pub fn is_version_ignored(&self, version: &str) -> bool {
        self.ignored_version
            .as_ref()
            .map(|v| v == version)
            .unwrap_or(false)
    }
}

fn default_true() -> bool {
    true
}

fn default_check_interval_hours() -> u64 {
    24
}

fn default_owner() -> String {
    "yyyyyyh".to_string()
}

fn default_repo() -> String {
    "ghclone".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = UpdateConfig::default();
        assert!(config.auto_check_enabled);
        assert_eq!(config.check_interval_hours, 24);
        assert!(config.ignored_version.is_none());
        assert!(config.last_checked_at.is_none());
    }

    #[test]
    fn test_should_check_when_never_checked() {
        let config = UpdateConfig::default();
        assert!(config.should_check());
    }

    #[test]
    fn test_should_not_check_when_disabled() {
        let config = UpdateConfig {
            auto_check_enabled: false,
            ..Default::default()
        };
        assert!(!config.should_check());
    }

    #[test]
    fn test_is_version_ignored() {
        let config = UpdateConfig {
            ignored_version: Some("v1.0.0".to_string()),
            ..Default::default()
        };
        assert!(config.is_version_ignored("v1.0.0"));
        assert!(!config.is_version_ignored("v1.0.1"));
    }
}
