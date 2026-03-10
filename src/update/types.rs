//! Update checking types

use serde::Deserialize;
use std::time::SystemTime;

/// Update check status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateStatus {
    /// Never checked
    NeverChecked,
    /// Currently checking
    Checking,
    /// Up to date
    UpToDate,
    /// Update available
    UpdateAvailable { version: String },
    /// Check failed
    CheckFailed { error: String },
}

impl Default for UpdateStatus {
    fn default() -> Self {
        Self::NeverChecked
    }
}

/// Update information from GitHub
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateInfo {
    /// Latest version tag (e.g., "v0.2.0")
    pub tag_name: String,
    /// Release page URL
    pub html_url: String,
    /// Release time
    pub published_at: String,
    /// Release notes
    pub body: Option<String>,
}

impl UpdateInfo {
    /// Get version string without 'v' prefix
    pub fn version(&self) -> &str {
        self.tag_name.trim_start_matches('v')
    }
}

/// Version comparison result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionComparison {
    /// Current version is newer or equal
    CurrentIsNewerOrEqual,
    /// Update available
    UpdateAvailable,
    /// Cannot compare (invalid format)
    Incomparable,
}

/// Update check result
#[derive(Debug, Clone)]
pub struct UpdateCheckResult {
    pub status: UpdateStatus,
    pub info: Option<UpdateInfo>,
    pub checked_at: SystemTime,
}

impl UpdateCheckResult {
    /// Create a new result indicating up to date
    pub fn up_to_date(checked_at: SystemTime) -> Self {
        Self {
            status: UpdateStatus::UpToDate,
            info: None,
            checked_at,
        }
    }

    /// Create a new result with available update
    pub fn update_available(info: UpdateInfo, checked_at: SystemTime) -> Self {
        let version = info.tag_name.clone();
        Self {
            status: UpdateStatus::UpdateAvailable { version },
            info: Some(info),
            checked_at,
        }
    }

    /// Create a new result with error
    pub fn error(error: impl ToString, checked_at: SystemTime) -> Self {
        Self {
            status: UpdateStatus::CheckFailed {
                error: error.to_string(),
            },
            info: None,
            checked_at,
        }
    }
}
