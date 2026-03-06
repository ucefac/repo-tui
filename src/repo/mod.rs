//! Repository module

pub mod discover;
pub mod status;
pub mod types;

pub use discover::discover_repositories;
pub use status::check_git_status;
pub use types::{GitStatus, Repository};
