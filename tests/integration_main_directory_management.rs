//! Main directory management integration tests
//!
//! Tests for the main directory management flow

use std::collections::HashSet;
use std::path::PathBuf;
use tempfile::TempDir;

/// Represents the main directory management state
#[derive(Debug, Clone)]
struct MainDirectoryManager {
    directories: Vec<MainDirectory>,
    selected_index: usize,
}

#[derive(Debug, Clone)]
struct MainDirectory {
    path: PathBuf,
    display_name: String,
    enabled: bool,
    repo_count: usize,
}

impl MainDirectoryManager {
    fn new() -> Self {
        Self {
            directories: Vec::new(),
            selected_index: 0,
        }
    }

    fn add_directory(&mut self, path: PathBuf) -> Result<(), ManagementError> {
        // Check for duplicates
        if self.directories.iter().any(|d| d.path == path) {
            return Err(ManagementError::DuplicatePath);
        }

        let display_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        self.directories.push(MainDirectory {
            path,
            display_name,
            enabled: true,
            repo_count: 0,
        });

        Ok(())
    }

    fn remove_directory(&mut self, index: usize) -> Result<(), ManagementError> {
        if self.directories.len() <= 1 {
            return Err(ManagementError::CannotRemoveLast);
        }

        if index >= self.directories.len() {
            return Err(ManagementError::InvalidIndex);
        }

        self.directories.remove(index);

        // Adjust selected index if necessary
        if self.selected_index >= self.directories.len() {
            self.selected_index = self.directories.len().saturating_sub(1);
        }

        Ok(())
    }

    fn navigate_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            // Wrap around to the end
            self.selected_index = self.directories.len().saturating_sub(1);
        }
    }

    fn navigate_down(&mut self) {
        if self.selected_index + 1 < self.directories.len() {
            self.selected_index += 1;
        } else {
            // Wrap around to the beginning
            self.selected_index = 0;
        }
    }

    fn toggle_enabled(&mut self, index: usize) -> Result<(), ManagementError> {
        if let Some(dir) = self.directories.get_mut(index) {
            dir.enabled = !dir.enabled;
            Ok(())
        } else {
            Err(ManagementError::InvalidIndex)
        }
    }

    fn get_enabled_paths(&self) -> Vec<&PathBuf> {
        self.directories
            .iter()
            .filter(|d| d.enabled)
            .map(|d| &d.path)
            .collect()
    }

    fn len(&self) -> usize {
        self.directories.len()
    }

    fn is_empty(&self) -> bool {
        self.directories.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ManagementError {
    DuplicatePath,
    CannotRemoveLast,
    InvalidIndex,
    EmptyPath,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_main_directory() {
        // Arrange
        let mut manager = MainDirectoryManager::new();

        // Act
        manager
            .add_directory(PathBuf::from("/home/user/projects"))
            .unwrap();

        // Assert
        assert_eq!(manager.len(), 1);
        assert_eq!(
            manager.directories[0].path,
            PathBuf::from("/home/user/projects")
        );
        assert_eq!(manager.directories[0].display_name, "projects");
        assert!(manager.directories[0].enabled);
    }

    #[test]
    fn test_add_duplicate_directory_fails() {
        // Arrange
        let mut manager = MainDirectoryManager::new();
        manager
            .add_directory(PathBuf::from("/home/user/projects"))
            .unwrap();

        // Act
        let result = manager.add_directory(PathBuf::from("/home/user/projects"));

        // Assert
        assert!(matches!(result, Err(ManagementError::DuplicatePath)));
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_remove_directory() {
        // Arrange
        let mut manager = MainDirectoryManager::new();
        manager
            .add_directory(PathBuf::from("/home/user/dir1"))
            .unwrap();
        manager
            .add_directory(PathBuf::from("/home/user/dir2"))
            .unwrap();
        manager
            .add_directory(PathBuf::from("/home/user/dir3"))
            .unwrap();

        // Act
        manager.remove_directory(1).unwrap();

        // Assert
        assert_eq!(manager.len(), 2);
        assert_eq!(
            manager.directories[0].path,
            PathBuf::from("/home/user/dir1")
        );
        assert_eq!(
            manager.directories[1].path,
            PathBuf::from("/home/user/dir3")
        );
    }

    #[test]
    fn test_cannot_remove_last_directory() {
        // Arrange
        let mut manager = MainDirectoryManager::new();
        manager
            .add_directory(PathBuf::from("/home/user/projects"))
            .unwrap();

        // Act
        let result = manager.remove_directory(0);

        // Assert
        assert!(matches!(result, Err(ManagementError::CannotRemoveLast)));
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_navigate_up_down() {
        // Arrange
        let mut manager = MainDirectoryManager::new();
        for i in 1..=3 {
            manager
                .add_directory(PathBuf::from(format!("/home/user/dir{}", i)))
                .unwrap();
        }
        manager.selected_index = 1; // Start at middle

        // Act & Assert: Navigate down
        manager.navigate_down();
        assert_eq!(manager.selected_index, 2);

        // Act & Assert: Wrap around from end
        manager.navigate_down();
        assert_eq!(manager.selected_index, 0);

        // Act & Assert: Navigate up
        manager.selected_index = 1;
        manager.navigate_up();
        assert_eq!(manager.selected_index, 0);

        // Act & Assert: Wrap around from beginning
        manager.navigate_up();
        assert_eq!(manager.selected_index, 2);
    }

    #[test]
    fn test_toggle_enabled() {
        // Arrange
        let mut manager = MainDirectoryManager::new();
        manager
            .add_directory(PathBuf::from("/home/user/projects"))
            .unwrap();

        assert!(manager.directories[0].enabled);

        // Act: Toggle off
        manager.toggle_enabled(0).unwrap();

        // Assert
        assert!(!manager.directories[0].enabled);

        // Act: Toggle on
        manager.toggle_enabled(0).unwrap();

        // Assert
        assert!(manager.directories[0].enabled);
    }

    #[test]
    fn test_get_enabled_paths() {
        // Arrange
        let mut manager = MainDirectoryManager::new();
        manager
            .add_directory(PathBuf::from("/home/user/dir1"))
            .unwrap();
        manager
            .add_directory(PathBuf::from("/home/user/dir2"))
            .unwrap();
        manager
            .add_directory(PathBuf::from("/home/user/dir3"))
            .unwrap();

        // Disable the second directory
        manager.toggle_enabled(1).unwrap();

        // Act
        let enabled = manager.get_enabled_paths();

        // Assert
        assert_eq!(enabled.len(), 2);
        assert!(enabled.contains(&&PathBuf::from("/home/user/dir1")));
        assert!(!enabled.contains(&&PathBuf::from("/home/user/dir2")));
        assert!(enabled.contains(&&PathBuf::from("/home/user/dir3")));
    }

    #[test]
    fn test_add_multiple_directories() {
        // Arrange
        let mut manager = MainDirectoryManager::new();

        // Act
        manager
            .add_directory(PathBuf::from("/home/user/work"))
            .unwrap();
        manager
            .add_directory(PathBuf::from("/home/user/personal"))
            .unwrap();
        manager
            .add_directory(PathBuf::from("/home/user/experiments"))
            .unwrap();

        // Assert
        assert_eq!(manager.len(), 3);
        assert_eq!(manager.directories[0].display_name, "work");
        assert_eq!(manager.directories[1].display_name, "personal");
        assert_eq!(manager.directories[2].display_name, "experiments");
    }

    #[test]
    fn test_remove_directory_adjusts_selection() {
        // Arrange
        let mut manager = MainDirectoryManager::new();
        for i in 1..=3 {
            manager
                .add_directory(PathBuf::from(format!("/home/user/dir{}", i)))
                .unwrap();
        }
        manager.selected_index = 2; // Last item

        // Act: Remove last item
        manager.remove_directory(2).unwrap();

        // Assert: Selection should adjust to new last item
        assert_eq!(manager.selected_index, 1);

        // Arrange: Only 2 items left, select first
        manager.selected_index = 0;

        // Act: Remove first item
        manager.remove_directory(0).unwrap();

        // Assert: Selection should stay at 0
        assert_eq!(manager.selected_index, 0);
    }

    #[test]
    fn test_empty_manager() {
        // Arrange
        let manager = MainDirectoryManager::new();

        // Assert
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
        assert!(manager.get_enabled_paths().is_empty());
    }

    #[test]
    fn test_all_directories_disabled() {
        // Arrange
        let mut manager = MainDirectoryManager::new();
        manager
            .add_directory(PathBuf::from("/home/user/dir1"))
            .unwrap();
        manager
            .add_directory(PathBuf::from("/home/user/dir2"))
            .unwrap();

        // Act: Disable all
        manager.toggle_enabled(0).unwrap();
        manager.toggle_enabled(1).unwrap();

        // Assert
        let enabled = manager.get_enabled_paths();
        assert!(enabled.is_empty());
    }

    #[test]
    fn test_directory_name_extraction() {
        // Arrange & Act
        let mut manager = MainDirectoryManager::new();

        // Various path formats
        manager
            .add_directory(PathBuf::from("/home/user/my-projects"))
            .unwrap();
        manager.add_directory(PathBuf::from("/work/repos")).unwrap();
        manager
            .add_directory(PathBuf::from("/opt/external-tools"))
            .unwrap();

        // Assert
        assert_eq!(manager.directories[0].display_name, "my-projects");
        assert_eq!(manager.directories[1].display_name, "repos");
        assert_eq!(manager.directories[2].display_name, "external-tools");
    }

    #[test]
    fn test_cannot_remove_invalid_index() {
        // Arrange
        let mut manager = MainDirectoryManager::new();
        manager
            .add_directory(PathBuf::from("/home/user/dir1"))
            .unwrap();
        manager
            .add_directory(PathBuf::from("/home/user/dir2"))
            .unwrap();

        // Act
        let result = manager.remove_directory(5); // Invalid index

        // Assert
        assert!(matches!(result, Err(ManagementError::InvalidIndex)));
    }

    #[test]
    fn test_add_similar_but_different_paths() {
        // Arrange
        let mut manager = MainDirectoryManager::new();

        // Act: These are different paths
        manager
            .add_directory(PathBuf::from("/home/user/projects"))
            .unwrap();
        manager
            .add_directory(PathBuf::from("/home/user/projects2"))
            .unwrap();
        manager
            .add_directory(PathBuf::from("/home/user/work/projects"))
            .unwrap();

        // Assert: All should be added
        assert_eq!(manager.len(), 3);
    }

    #[test]
    fn test_navigation_in_empty_manager() {
        // Arrange
        let mut manager = MainDirectoryManager::new();

        // Act: Should not panic
        manager.navigate_up();
        manager.navigate_down();

        // Assert
        assert_eq!(manager.selected_index, 0);
    }

    #[test]
    fn test_navigation_in_single_item_manager() {
        // Arrange
        let mut manager = MainDirectoryManager::new();
        manager
            .add_directory(PathBuf::from("/home/user/projects"))
            .unwrap();

        // Act: Should stay at 0
        manager.navigate_up();
        assert_eq!(manager.selected_index, 0);

        manager.navigate_down();
        assert_eq!(manager.selected_index, 0);
    }

    #[test]
    fn test_directory_state_isolation() {
        // Arrange
        let mut manager = MainDirectoryManager::new();
        manager
            .add_directory(PathBuf::from("/home/user/dir1"))
            .unwrap();
        manager
            .add_directory(PathBuf::from("/home/user/dir2"))
            .unwrap();

        // Act: Modify first directory
        manager.directories[0].enabled = false;
        manager.directories[0].repo_count = 5;

        // Assert: Second directory should be unaffected
        assert!(!manager.directories[0].enabled);
        assert!(manager.directories[1].enabled);
        assert_eq!(manager.directories[0].repo_count, 5);
        assert_eq!(manager.directories[1].repo_count, 0);
    }

    #[test]
    fn test_unicode_directory_names() {
        // Arrange
        let mut manager = MainDirectoryManager::new();

        // Act
        manager
            .add_directory(PathBuf::from("/home/user/项目"))
            .unwrap();
        manager
            .add_directory(PathBuf::from("/home/user/プロジェクト"))
            .unwrap();

        // Assert
        assert_eq!(manager.directories[0].display_name, "项目");
        assert_eq!(manager.directories[1].display_name, "プロジェクト");
    }

    #[test]
    fn test_complex_path_scenarios() {
        // Arrange
        let mut manager = MainDirectoryManager::new();

        // Paths with dots, numbers, special chars
        manager
            .add_directory(PathBuf::from("/home/user/.hidden-projects"))
            .unwrap();
        manager
            .add_directory(PathBuf::from("/home/user/projects_v2.0"))
            .unwrap();
        manager
            .add_directory(PathBuf::from("/home/user/test+dev"))
            .unwrap();

        // Assert
        assert_eq!(manager.directories[0].display_name, ".hidden-projects");
        assert_eq!(manager.directories[1].display_name, "projects_v2.0");
        assert_eq!(manager.directories[2].display_name, "test+dev");
    }
}
