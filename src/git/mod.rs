//! Git operations module
//!
//! Provides asynchronous Git status detection with TTL caching.

pub mod cache;
pub mod scheduler;
pub mod status;

pub use cache::{CachedGitStatus, StatusCache};
pub use scheduler::GitStatusScheduler;
pub use status::{check_git_status, is_git_repo, GitError};
