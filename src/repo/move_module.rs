//! Repository move operations

use std::path::{Path, PathBuf};
use std::fs;
use std::io;

use crate::error::MoveError;
use crate::repo::Repository;

/// Move a repository to a target main directory
///
/// # Arguments
/// * `repo` - Repository to move
/// * `target_main_dir` - Target main directory path
///
/// # Returns
/// * `Ok(PathBuf)` - New repository path after successful move
/// * `Err(MoveError)` - Error if move failed
pub fn move_repository(repo: &Repository, target_main_dir: &Path) -> Result<PathBuf, MoveError> {
    let repo_name = &repo.name;
    let target_path = target_main_dir.join(repo_name);

    // Check if target already exists
    if target_path.exists() {
        return Err(MoveError::TargetAlreadyExists(target_path.clone()));
    }

    // Perform the move operation
    // First try rename (works for same filesystem)
    // If that fails with InvalidCrossDeviceLink, fall back to copy+delete
    match fs::rename(&repo.path, &target_path) {
        Ok(()) => Ok(target_path),
        Err(e) if e.kind() == io::ErrorKind::CrossesDevices => {
            // Cross-device move: copy then delete
            move_cross_device(&repo.path, &target_path)
        }
        Err(e) => {
            match e.kind() {
                io::ErrorKind::PermissionDenied => {
                    Err(MoveError::PermissionDenied(repo.path.clone()))
                }
                _ => Err(MoveError::Io(e.to_string())),
            }
        }
    }
}

/// Move a directory by copying and then deleting (for cross-device moves)
fn move_cross_device(from: &Path, to: &Path) -> Result<PathBuf, MoveError> {
    // Create target directory
    if let Err(e) = fs::create_dir_all(to.parent().unwrap()) {
        return Err(MoveError::Io(e.to_string()));
    }

    // Copy the directory recursively
    if let Err(e) = copy_dir_recursive(from, to) {
        // Clean up partial copy on failure
        let _ = fs::remove_dir_all(to);
        return Err(MoveError::Io(e.to_string()));
    }

    // Remove original after successful copy
    if let Err(e) = fs::remove_dir_all(from) {
        // Copy succeeded but cleanup failed - this is a warning case
        // Return success anyway as data is safe
        tracing::warn!("Failed to remove original directory after copy: {}", e);
    }

    Ok(to.to_path_buf())
}

/// Recursively copy a directory
fn copy_dir_recursive(from: &Path, to: &Path) -> io::Result<()> {
    // Create target directory
    fs::create_dir_all(to)?;

    // Copy all entries
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let from_path = entry.path();
        let to_path = to.join(entry.file_name());

        if from_path.is_dir() {
            copy_dir_recursive(&from_path, &to_path)?;
        } else {
            fs::copy(&from_path, &to_path)?;
        }
    }

    Ok(())
}

/// Check if a repository can be moved to a target directory
///
/// # Arguments
/// * `repo` - Repository to check
/// * `target_main_dir` - Target main directory path
///
/// # Returns
/// * `Ok(())` - Can be moved
/// * `Err(MoveError)` - Cannot be moved with reason
pub fn check_move_feasible(repo: &Repository, target_main_dir: &Path) -> Result<(), MoveError> {
    let target_path = target_main_dir.join(&repo.name);

    // Check if target already exists
    if target_path.exists() {
        return Err(MoveError::TargetAlreadyExists(target_path));
    }

    // Check if we have write permission on target directory
    // We check the parent directory since target doesn't exist yet
    if let Some(parent) = target_main_dir.parent() {
        if !parent.exists() {
            return Err(MoveError::PathError(format!(
                "Target parent directory does not exist: {}",
                parent.display()
            )));
        }
    }

    Ok(())
}

/// Get the main directory index for a repository
///
/// # Arguments
/// * `repo_path` - Repository path
/// * `main_dirs` - List of main directories
///
/// # Returns
/// * `Some(usize)` - Index of main directory containing the repo
/// * `None` - Repository not in any main directory
pub fn find_repo_main_dir_index(repo_path: &Path, main_dirs: &[PathBuf]) -> Option<usize> {
    for (idx, main_dir) in main_dirs.iter().enumerate() {
        if repo_path.starts_with(main_dir) {
            return Some(idx);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_move_repository_same_filesystem() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        let target_dir = temp_dir.path().join("target");
        let repo_path = source_dir.join("test-repo");

        fs::create_dir_all(&source_dir).unwrap();
        fs::create_dir_all(&target_dir).unwrap();
        fs::create_dir_all(&repo_path).unwrap();
        fs::write(repo_path.join("test.txt"), "test").unwrap();

        let repo = Repository::from_path(repo_path.clone());
        let result = move_repository(&repo, &target_dir);

        assert!(result.is_ok());
        assert!(target_dir.join("test-repo").exists());
        assert!(!repo_path.exists());
    }

    #[test]
    fn test_check_move_feasible() {
        let temp_dir = TempDir::new().unwrap();
        let target_dir = temp_dir.path().join("target");
        let repo_path = temp_dir.path().join("test-repo");

        fs::create_dir_all(&target_dir).unwrap();
        fs::create_dir_all(&repo_path).unwrap();

        let repo = Repository::from_path(repo_path);

        // Should be feasible
        assert!(check_move_feasible(&repo, &target_dir).is_ok());

        // Create conflicting target
        let conflict_path = target_dir.join("test-repo");
        fs::create_dir_all(&conflict_path).unwrap();

        // Should fail with TargetAlreadyExists
        assert!(matches!(
            check_move_feasible(&repo, &target_dir),
            Err(MoveError::TargetAlreadyExists(_))
        ));
    }

    #[test]
    fn test_find_repo_main_dir_index() {
        let main_dirs = vec![
            PathBuf::from("/Users/test/Projects"),
            PathBuf::from("/Users/test/Work"),
        ];

        let repo_in_projects = PathBuf::from("/Users/test/Projects/my-repo");
        let repo_in_work = PathBuf::from("/Users/test/Work/my-repo");
        let repo_standalone = PathBuf::from("/tmp/my-repo");

        assert_eq!(find_repo_main_dir_index(&repo_in_projects, &main_dirs), Some(0));
        assert_eq!(find_repo_main_dir_index(&repo_in_work, &main_dirs), Some(1));
        assert_eq!(find_repo_main_dir_index(&repo_standalone, &main_dirs), None);
    }
}
