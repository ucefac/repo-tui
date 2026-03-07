//! Recent repositories storage and management

mod store;

pub use store::RecentStore;

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Recent entry with timestamp
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecentEntry {
    /// Repository path
    pub path: String,

    /// When the repository was opened (ISO 8601 format)
    pub opened_at: String,
}

impl RecentEntry {
    /// Create a new recent entry with current timestamp
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_string_lossy().to_string(),
            opened_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Create from path and timestamp
    pub fn from_parts(path: String, opened_at: String) -> Self {
        Self { path, opened_at }
    }

    /// Parse timestamp from ISO 8601 string
    pub fn parsed_timestamp(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        chrono::DateTime::parse_from_rfc3339(&self.opened_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_recent_entry_new() {
        let path = PathBuf::from("/home/user/repo");
        let entry = RecentEntry::new(&path);

        assert_eq!(entry.path, "/home/user/repo");
        assert!(!entry.opened_at.is_empty());
        assert!(entry.parsed_timestamp().is_some());
    }

    #[test]
    fn test_recent_entry_from_parts() {
        let entry = RecentEntry::from_parts(
            "/home/user/repo".to_string(),
            "2026-03-07T10:00:00Z".to_string(),
        );

        assert_eq!(entry.path, "/home/user/repo");
        assert_eq!(entry.opened_at, "2026-03-07T10:00:00Z");
        assert!(entry.parsed_timestamp().is_some());
    }

    #[test]
    fn test_recent_entry_parse_invalid() {
        let entry = RecentEntry::from_parts("/home/user/repo".to_string(), "invalid".to_string());

        assert!(entry.parsed_timestamp().is_none());
    }
}
