//! Repository module

pub mod clone;
pub mod discover;
pub mod filter;
pub mod source;
pub mod status;
pub mod types;

pub use clone::{
    generate_folder_name, parse_git_url, repository_from_clone, validate_clone_target,
    validate_folder_replace, validate_git_url, ParsedGitUrl,
};
pub use discover::discover_repositories;
pub use filter::{filter_repos_fuzzy, filter_repos_simple};
pub use source::RepoSource;
pub use status::check_git_status;
pub use types::{GitStatus, Repository};
