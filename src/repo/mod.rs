//! Repository module

pub mod discover;
pub mod filter;
pub mod source;
pub mod status;
pub mod types;

pub use discover::discover_repositories;
pub use filter::{filter_repos_fuzzy, filter_repos_simple};
pub use source::RepoSource;
pub use status::check_git_status;
pub use types::{GitStatus, Repository};
