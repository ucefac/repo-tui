//! RepoSource functionality tests
//!
//! Tests for the RepoSource enum and related functionality

use std::path::PathBuf;

/// Repository source type (mock for testing)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RepoSource {
    /// From main directory scan
    MainDirectory {
        /// Main directory index
        dir_index: usize,
        /// Main directory path
        dir_path: PathBuf,
    },
    /// Independently added repository
    Single,
}

impl RepoSource {
    /// Get the scope name for display
    pub fn scope(&self) -> String {
        match self {
            RepoSource::MainDirectory { dir_path, .. } => dir_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            RepoSource::Single => "single".to_string(),
        }
    }

    /// Check if from a specific main directory
    pub fn is_from_main_dir(&self, dir_index: usize) -> bool {
        matches!(self, RepoSource::MainDirectory { dir_index: idx, .. } if *idx == dir_index)
    }

    /// Get the display name format: @scope/repo-name
    pub fn display_name(&self, repo_name: &str) -> String {
        format!("@{}/{}", self.scope(), repo_name)
    }
}

/// Match repositories by source scope
pub fn match_by_scope(repos: &[(String, RepoSource)], scope: &str) -> Vec<String> {
    repos
        .iter()
        .filter(|(_, source)| source.scope() == scope)
        .map(|(name, _)| name.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_directory_scope_extraction() {
        // Arrange: Create MainDirectory sources with different paths
        let work_source = RepoSource::MainDirectory {
            dir_index: 0,
            dir_path: PathBuf::from("/home/user/work"),
        };

        let personal_source = RepoSource::MainDirectory {
            dir_index: 1,
            dir_path: PathBuf::from("/home/user/personal"),
        };

        let company_source = RepoSource::MainDirectory {
            dir_index: 2,
            dir_path: PathBuf::from("/home/user/company/projects"),
        };

        // Act & Assert: Verify scope extraction
        assert_eq!(work_source.scope(), "work");
        assert_eq!(personal_source.scope(), "personal");
        assert_eq!(company_source.scope(), "projects"); // Last component
    }

    #[test]
    fn test_single_source_scope() {
        // Arrange
        let single_source = RepoSource::Single;

        // Act & Assert
        assert_eq!(single_source.scope(), "single");
    }

    #[test]
    fn test_is_from_main_dir() {
        // Arrange
        let source_0 = RepoSource::MainDirectory {
            dir_index: 0,
            dir_path: PathBuf::from("/home/user/work"),
        };

        let source_1 = RepoSource::MainDirectory {
            dir_index: 1,
            dir_path: PathBuf::from("/home/user/personal"),
        };

        let single = RepoSource::Single;

        // Act & Assert
        assert!(source_0.is_from_main_dir(0));
        assert!(!source_0.is_from_main_dir(1));
        assert!(source_1.is_from_main_dir(1));
        assert!(!source_1.is_from_main_dir(0));
        assert!(!single.is_from_main_dir(0)); // Single is never from a main dir
        assert!(!single.is_from_main_dir(1));
    }

    #[test]
    fn test_display_name_format() {
        // Arrange
        let work_source = RepoSource::MainDirectory {
            dir_index: 0,
            dir_path: PathBuf::from("/home/user/work"),
        };

        let single_source = RepoSource::Single;

        // Act & Assert
        assert_eq!(work_source.display_name("my-app"), "@work/my-app");
        assert_eq!(
            work_source.display_name("react-project"),
            "@work/react-project"
        );
        assert_eq!(
            single_source.display_name("standalone-repo"),
            "@single/standalone-repo"
        );
    }

    #[test]
    fn test_scope_with_special_characters() {
        // Arrange: Paths with special characters in directory names
        let source = RepoSource::MainDirectory {
            dir_index: 0,
            dir_path: PathBuf::from("/home/user/my-projects_v2.0"),
        };

        // Act & Assert
        assert_eq!(source.scope(), "my-projects_v2.0");
    }

    #[test]
    fn test_scope_with_unicode() {
        // Arrange: Paths with unicode characters
        let source = RepoSource::MainDirectory {
            dir_index: 0,
            dir_path: PathBuf::from("/home/user/项目"),
        };

        // Act & Assert
        assert_eq!(source.scope(), "项目");
    }

    #[test]
    fn test_empty_path_scope() {
        // Arrange: Edge case - empty or root path
        let source = RepoSource::MainDirectory {
            dir_index: 0,
            dir_path: PathBuf::from("/"),
        };

        // Act & Assert: Should return "unknown" for root
        assert_eq!(source.scope(), "unknown");
    }

    #[test]
    fn test_match_by_scope() {
        // Arrange: Create repositories with different sources
        let repos = vec![
            (
                "app1".to_string(),
                RepoSource::MainDirectory {
                    dir_index: 0,
                    dir_path: PathBuf::from("/home/user/work"),
                },
            ),
            (
                "app2".to_string(),
                RepoSource::MainDirectory {
                    dir_index: 0,
                    dir_path: PathBuf::from("/home/user/work"),
                },
            ),
            (
                "dotfiles".to_string(),
                RepoSource::MainDirectory {
                    dir_index: 1,
                    dir_path: PathBuf::from("/home/user/personal"),
                },
            ),
            ("standalone".to_string(), RepoSource::Single),
        ];

        // Act: Match by scope
        let work_repos = match_by_scope(&repos, "work");
        let personal_repos = match_by_scope(&repos, "personal");
        let single_repos = match_by_scope(&repos, "single");

        // Assert
        assert_eq!(work_repos.len(), 2);
        assert!(work_repos.contains(&"app1".to_string()));
        assert!(work_repos.contains(&"app2".to_string()));

        assert_eq!(personal_repos.len(), 1);
        assert!(personal_repos.contains(&"dotfiles".to_string()));

        assert_eq!(single_repos.len(), 1);
        assert!(single_repos.contains(&"standalone".to_string()));
    }

    #[test]
    fn test_scope_with_absolute_paths() {
        // Arrange: Various absolute paths
        let test_cases = vec![
            ("/home/user/projects", "projects"),
            ("/work/repos", "repos"),
            ("/opt/external", "external"),
            ("/var/www", "www"),
        ];

        for (path_str, expected_scope) in test_cases {
            let source = RepoSource::MainDirectory {
                dir_index: 0,
                dir_path: PathBuf::from(path_str),
            };

            assert_eq!(
                source.scope(),
                expected_scope,
                "Path {} should have scope {}",
                path_str,
                expected_scope
            );
        }
    }

    #[test]
    fn test_repo_source_equality() {
        // Arrange
        let source1 = RepoSource::MainDirectory {
            dir_index: 0,
            dir_path: PathBuf::from("/home/user/work"),
        };

        let source2 = RepoSource::MainDirectory {
            dir_index: 0,
            dir_path: PathBuf::from("/home/user/work"),
        };

        let source3 = RepoSource::MainDirectory {
            dir_index: 1,
            dir_path: PathBuf::from("/home/user/personal"),
        };

        let single1 = RepoSource::Single;
        let single2 = RepoSource::Single;

        // Act & Assert
        assert_eq!(source1, source2);
        assert_ne!(source1, source3);
        assert_eq!(single1, single2);
        assert_ne!(source1, single1);
    }

    #[test]
    fn test_scope_with_dot_directories() {
        // Arrange: Hidden directories
        let source = RepoSource::MainDirectory {
            dir_index: 0,
            dir_path: PathBuf::from("/home/user/.hidden-projects"),
        };

        // Act & Assert
        assert_eq!(source.scope(), ".hidden-projects");
    }

    #[test]
    fn test_scope_with_numbers() {
        // Arrange: Directory names with numbers
        let source = RepoSource::MainDirectory {
            dir_index: 0,
            dir_path: PathBuf::from("/home/user/projects2024"),
        };

        // Act & Assert
        assert_eq!(source.scope(), "projects2024");
    }

    #[test]
    fn test_nested_path_scope() {
        // Arrange: Deeply nested paths
        let source = RepoSource::MainDirectory {
            dir_index: 0,
            dir_path: PathBuf::from("/home/user/company/department/team/projects"),
        };

        // Act & Assert: Should only get the last component
        assert_eq!(source.scope(), "projects");
    }

    #[test]
    fn test_display_name_with_special_chars() {
        // Arrange
        let source = RepoSource::MainDirectory {
            dir_index: 0,
            dir_path: PathBuf::from("/home/user/work"),
        };

        // Act & Assert: Test various repo names
        assert_eq!(source.display_name("my-app"), "@work/my-app");
        assert_eq!(source.display_name("my_app"), "@work/my_app");
        assert_eq!(source.display_name("MyApp"), "@work/MyApp");
        assert_eq!(source.display_name("my-app.v2"), "@work/my-app.v2");
    }

    #[test]
    fn test_source_cloning() {
        // Arrange
        let original = RepoSource::MainDirectory {
            dir_index: 0,
            dir_path: PathBuf::from("/home/user/work"),
        };

        // Act
        let cloned = original.clone();

        // Assert
        assert_eq!(original, cloned);
        assert_eq!(original.scope(), cloned.scope());
    }

    #[test]
    fn test_large_dir_index() {
        // Arrange: Test with large index values
        let source = RepoSource::MainDirectory {
            dir_index: 999,
            dir_path: PathBuf::from("/home/user/work"),
        };

        // Act & Assert
        assert!(source.is_from_main_dir(999));
        assert!(!source.is_from_main_dir(0));
        assert!(!source.is_from_main_dir(998));
    }
}
