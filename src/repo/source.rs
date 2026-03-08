//! Repository source types

use std::path::PathBuf;

/// Repository source type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RepoSource {
    /// From main directory scan
    MainDirectory {
        /// Main directory index
        dir_index: usize,
        /// Main directory path (for generating display name)
        dir_path: PathBuf,
    },
    /// Standalone repository
    Standalone,
}

impl RepoSource {
    /// Get scope name (for display_name)
    pub fn scope(&self) -> String {
        match self {
            RepoSource::MainDirectory { dir_path, .. } => dir_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            RepoSource::Standalone => "stand".to_string(),
        }
    }

    /// Check if from specific main directory
    pub fn is_from_main_dir(&self, dir_index: usize) -> bool {
        matches!(self, RepoSource::MainDirectory { dir_index: idx, .. } if *idx == dir_index)
    }

    /// Check if this is a standalone repository
    pub fn is_standalone(&self) -> bool {
        matches!(self, RepoSource::Standalone)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_source_scope() {
        let source = RepoSource::MainDirectory {
            dir_index: 0,
            dir_path: PathBuf::from("/home/user/work"),
        };
        assert_eq!(source.scope(), "work");

        let standalone = RepoSource::Standalone;
        assert_eq!(standalone.scope(), "stand");
    }

    #[test]
    fn test_repo_source_is_from_main_dir() {
        let source = RepoSource::MainDirectory {
            dir_index: 1,
            dir_path: PathBuf::from("/home/user/work"),
        };
        assert!(source.is_from_main_dir(1));
        assert!(!source.is_from_main_dir(0));

        let standalone = RepoSource::Standalone;
        assert!(!standalone.is_from_main_dir(0));
    }

    #[test]
    fn test_repo_source_is_standalone() {
        let source = RepoSource::MainDirectory {
            dir_index: 0,
            dir_path: PathBuf::from("/home/user/work"),
        };
        assert!(!source.is_standalone());

        let standalone = RepoSource::Standalone;
        assert!(standalone.is_standalone());
    }
}
