//! Git repository clone operations
//!
//! Provides URL parsing, folder name generation, and clone functionality.

use crate::error::CloneError;
use crate::repo::types::Repository;
use std::path::{Path, PathBuf};

/// Parsed Git URL information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedGitUrl {
    /// Domain (e.g., "github.com")
    pub domain: String,
    /// Owner/Organization (e.g., "farion1231")
    pub owner: String,
    /// Repository name (e.g., "cc-switch")
    pub repo: String,
    /// Original URL
    pub original_url: String,
}

/// Parse a Git URL into its components
///
/// Supported formats:
/// - HTTPS: `https://github.com/owner/repo` or `https://github.com/owner/repo.git`
/// - HTTPS with auth: `https://user:pass@github.com/owner/repo.git`
/// - SSH shorthand: `git@github.com:owner/repo.git`
/// - SSH full: `ssh://git@github.com/owner/repo.git`
/// - Git protocol: `git://github.com/owner/repo.git`
///
/// # Examples
/// ```
/// use repo_tui::repo::clone::parse_git_url;
///
/// let parsed = parse_git_url("https://github.com/farion1231/cc-switch").unwrap();
/// assert_eq!(parsed.domain, "github.com");
/// assert_eq!(parsed.owner, "farion1231");
/// assert_eq!(parsed.repo, "cc-switch");
/// ```
pub fn parse_git_url(url: &str) -> Result<ParsedGitUrl, CloneError> {
    // Remove trailing .git if present
    let url = url.trim().trim_end_matches(".git");

    if url.is_empty() {
        return Err(CloneError::InvalidUrl("Empty URL".to_string()));
    }

    // Try HTTPS format: https://github.com/owner/repo
    if let Some(parsed) = parse_https_url(url) {
        return Ok(parsed);
    }

    // Try SSH shorthand: git@github.com:owner/repo
    if let Some(parsed) = parse_ssh_shorthand(url) {
        return Ok(parsed);
    }

    // Try SSH full format: ssh://git@github.com/owner/repo
    if let Some(parsed) = parse_ssh_full(url) {
        return Ok(parsed);
    }

    // Try Git protocol: git://github.com/owner/repo
    if let Some(parsed) = parse_git_protocol(url) {
        return Ok(parsed);
    }

    Err(CloneError::InvalidFormat)
}

/// Parse HTTPS URL format
fn parse_https_url(url: &str) -> Option<ParsedGitUrl> {
    // Remove protocol prefix
    let without_scheme = url.strip_prefix("https://")?;
    let without_scheme = without_scheme
        .strip_prefix("http://")
        .unwrap_or(without_scheme);

    // Remove auth info if present (user:pass@host)
    let host_and_path = if let Some(at_pos) = without_scheme.find('@') {
        &without_scheme[at_pos + 1..]
    } else {
        without_scheme
    };

    // Split domain and path
    let first_slash = host_and_path.find('/')?;
    let domain = &host_and_path[..first_slash];
    let path = &host_and_path[first_slash + 1..];

    // Remove port if present (e.g., github.com:8443)
    let domain = domain.split(':').next()?;

    parse_path_components(url, domain, path)
}

/// Parse SSH shorthand format: git@github.com:owner/repo
fn parse_ssh_shorthand(url: &str) -> Option<ParsedGitUrl> {
    // Format: git@github.com:owner/repo
    let without_prefix = url.strip_prefix("git@")?;

    let colon_pos = without_prefix.find(':')?;
    let domain = &without_prefix[..colon_pos];
    let path = &without_prefix[colon_pos + 1..];

    parse_path_components(url, domain, path)
}

/// Parse SSH full format: ssh://git@github.com/owner/repo
fn parse_ssh_full(url: &str) -> Option<ParsedGitUrl> {
    let without_scheme = url.strip_prefix("ssh://")?;

    // Remove auth info
    let host_and_path = if let Some(at_pos) = without_scheme.find('@') {
        &without_scheme[at_pos + 1..]
    } else {
        without_scheme
    };

    let first_slash = host_and_path.find('/')?;
    let domain = &host_and_path[..first_slash];
    let path = &host_and_path[first_slash + 1..];

    parse_path_components(url, domain, path)
}

/// Parse Git protocol format: git://github.com/owner/repo
fn parse_git_protocol(url: &str) -> Option<ParsedGitUrl> {
    let without_scheme = url.strip_prefix("git://")?;

    let first_slash = without_scheme.find('/')?;
    let domain = &without_scheme[..first_slash];
    let path = &without_scheme[first_slash + 1..];

    parse_path_components(url, domain, path)
}

/// Parse owner and repo from path component
fn parse_path_components(original_url: &str, domain: &str, path: &str) -> Option<ParsedGitUrl> {
    // Remove leading/trailing slashes
    let path = path.trim_matches('/');

    if path.is_empty() {
        return None;
    }

    // Split path into components
    let components: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    if components.len() < 2 {
        return None;
    }

    // Last component is repo name, everything before is owner (handles nested groups)
    let repo = components.last()?.to_string();
    let owner = components[..components.len() - 1].join("-");

    Some(ParsedGitUrl {
        domain: domain.to_string(),
        owner,
        repo,
        original_url: original_url.to_string(),
    })
}

/// Strip common TLD suffixes from domain name
///
/// Examples:
/// - "github.com" -> "github"
/// - "gitlab.org" -> "gitlab"
/// - "bitbucket.io" -> "bitbucket"
fn strip_tld(domain: &str) -> String {
    // List of common TLDs to strip
    // IMPORTANT: Longer TLDs must come first to match before shorter ones
    // e.g., ".co.uk" must come before ".uk" or ".co"
    const TLDS: &[&str] = &[
        ".com.cn", ".com.au", ".co.uk", ".co.jp", ".com", ".org", ".net", ".io", ".co", ".dev",
        ".app", ".info", ".biz", ".us", ".uk", ".eu", ".de", ".fr", ".jp", ".cn", ".ru", ".in",
        ".au", ".br", ".mx",
    ];

    let domain_lower = domain.to_lowercase();
    for tld in TLDS {
        if let Some(stripped) = domain_lower.strip_suffix(tld) {
            return stripped.to_string();
        }
    }
    domain_lower
}

/// Generate folder name from parsed URL
///
/// Format: `{domain}_{owner}_{repo}` (domain without TLD)
///
/// # Examples
/// ```
/// use repo_tui::repo::clone::{parse_git_url, generate_folder_name};
///
/// let parsed = parse_git_url("https://github.com/farion1231/cc-switch").unwrap();
/// let folder_name = generate_folder_name(&parsed);
/// assert_eq!(folder_name, "github_farion1231_cc-switch");
/// ```
pub fn generate_folder_name(parsed: &ParsedGitUrl) -> String {
    format!(
        "{}_{}_{}",
        sanitize(&strip_tld(&parsed.domain)),
        sanitize(&parsed.owner),
        sanitize(&parsed.repo)
    )
}

/// Sanitize a string for use as a folder name
///
/// Replaces invalid characters with underscores while preserving Unicode alphanumeric chars.
fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Validate that a URL is a valid Git repository URL
///
/// # Arguments
/// * `url` - The URL to validate
/// * `max_length` - Maximum allowed URL length
///
/// # Returns
/// * `Ok(())` if the URL is valid
/// * `Err(CloneError)` if the URL is invalid
pub fn validate_git_url(url: &str, max_length: usize) -> Result<(), CloneError> {
    // Length check
    if url.len() > max_length {
        return Err(CloneError::UrlTooLong(max_length));
    }

    // URL cannot start with '-' (prevent git command injection)
    if url.trim().starts_with('-') {
        return Err(CloneError::InvalidCharacters);
    }

    // Try to parse the URL
    parse_git_url(url)?;

    Ok(())
}

/// Validate that a target path is safe for clone operation
///
/// # Arguments
/// * `path` - The target path to validate
/// * `allowed_dirs` - List of allowed parent directories
///
/// # Returns
/// * `Ok(())` if the path is valid
/// * `Err(CloneError)` if the path is invalid
pub fn validate_clone_target(path: &Path, allowed_dirs: &[PathBuf]) -> Result<(), CloneError> {
    use std::fs;

    // Check if path exists
    if path.exists() {
        // If it's a file, error
        if path.is_file() {
            return Err(CloneError::AlreadyExists(path.to_path_buf()));
        }

        // If it's a directory, check if empty
        if path.is_dir() {
            let entries = fs::read_dir(path).map_err(|e| CloneError::Io(e.to_string()))?;
            if entries.count() > 0 {
                return Err(CloneError::AlreadyExists(path.to_path_buf()));
            }
        }
    }

    // Get canonical parent
    let parent = path
        .parent()
        .ok_or_else(|| CloneError::PathError("No parent directory".to_string()))?;

    let canonical_parent = parent
        .canonicalize()
        .map_err(|_| CloneError::PathError("Failed to canonicalize path".to_string()))?;

    // Check if parent is within allowed directories
    let in_allowed = allowed_dirs
        .iter()
        .any(|allowed| canonical_parent.starts_with(allowed));

    if !in_allowed {
        return Err(CloneError::OutsideAllowedDirectory(path.to_path_buf()));
    }

    Ok(())
}

/// Validate that an existing folder can be replaced
///
/// # Arguments
/// * `path` - Path to existing folder
/// * `allowed_dirs` - List of allowed parent directories
///
/// # Returns
/// * `Ok(())` if the folder can be replaced
/// * `Err(CloneError)` if the folder cannot be replaced
pub fn validate_folder_replace(path: &Path, allowed_dirs: &[PathBuf]) -> Result<(), CloneError> {
    // Path must exist and be a directory
    if !path.exists() || !path.is_dir() {
        return Err(CloneError::NotAGitRepository);
    }

    // Get canonical path
    let canonical = path
        .canonicalize()
        .map_err(|e| CloneError::Io(e.to_string()))?;

    // Check if within allowed directories
    let in_allowed = allowed_dirs
        .iter()
        .any(|allowed| canonical.starts_with(allowed));

    if !in_allowed {
        return Err(CloneError::OutsideAllowedDirectory(path.to_path_buf()));
    }

    // Verify it's a git repository (has .git directory or file)
    let git_path = canonical.join(".git");
    if !git_path.exists() {
        return Err(CloneError::NotAGitRepository);
    }

    // Check for protected paths (home directory, root)
    let home = dirs::home_dir().ok_or_else(|| CloneError::Io("Home not found".to_string()))?;
    if canonical == home || canonical == Path::new("/") {
        return Err(CloneError::ProtectedPath(path.to_path_buf()));
    }

    Ok(())
}

/// Create a Repository instance for a newly cloned repository
///
/// # Arguments
/// * `path` - Path to the cloned repository
/// * `dir_index` - Index of the main directory (if applicable)
///
/// # Returns
/// * `Repository` instance
pub fn repository_from_clone(path: PathBuf, dir_index: Option<usize>) -> Repository {
    use crate::repo::source::RepoSource;

    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let last_modified = path.metadata().ok().and_then(|m| m.modified().ok());

    let source = if let Some(idx) = dir_index {
        RepoSource::MainDirectory {
            dir_index: idx,
            dir_path: path.parent().unwrap_or(&path).to_path_buf(),
        }
    } else {
        RepoSource::Standalone
    };

    Repository {
        name,
        path,
        last_modified,
        is_dirty: false,
        branch: None,
        is_git_repo: true,
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_https_url() {
        let parsed = parse_git_url("https://github.com/farion1231/cc-switch").unwrap();
        assert_eq!(parsed.domain, "github.com");
        assert_eq!(parsed.owner, "farion1231");
        assert_eq!(parsed.repo, "cc-switch");
    }

    #[test]
    fn test_parse_https_with_git_suffix() {
        let parsed = parse_git_url("https://github.com/farion1231/cc-switch.git").unwrap();
        assert_eq!(parsed.domain, "github.com");
        assert_eq!(parsed.owner, "farion1231");
        assert_eq!(parsed.repo, "cc-switch");
    }

    #[test]
    fn test_parse_ssh_shorthand() {
        let parsed = parse_git_url("git@github.com:farion1231/cc-switch.git").unwrap();
        assert_eq!(parsed.domain, "github.com");
        assert_eq!(parsed.owner, "farion1231");
        assert_eq!(parsed.repo, "cc-switch");
    }

    #[test]
    fn test_parse_gitlab_nested() {
        let parsed = parse_git_url("https://gitlab.com/group/subgroup/project.git").unwrap();
        assert_eq!(parsed.domain, "gitlab.com");
        assert_eq!(parsed.owner, "group-subgroup");
        assert_eq!(parsed.repo, "project");
    }

    #[test]
    fn test_parse_ssh_full() {
        let parsed = parse_git_url("ssh://git@github.com/farion1231/cc-switch.git").unwrap();
        assert_eq!(parsed.domain, "github.com");
        assert_eq!(parsed.owner, "farion1231");
        assert_eq!(parsed.repo, "cc-switch");
    }

    #[test]
    fn test_parse_with_port() {
        let parsed = parse_git_url("https://github.com:8443/farion1231/cc-switch.git").unwrap();
        assert_eq!(parsed.domain, "github.com");
        assert_eq!(parsed.owner, "farion1231");
        assert_eq!(parsed.repo, "cc-switch");
    }

    #[test]
    fn test_generate_folder_name() {
        let parsed = ParsedGitUrl {
            domain: "github.com".to_string(),
            owner: "farion1231".to_string(),
            repo: "cc-switch".to_string(),
            original_url: "https://github.com/farion1231/cc-switch".to_string(),
        };
        assert_eq!(generate_folder_name(&parsed), "github_farion1231_cc-switch");
    }

    #[test]
    fn test_generate_folder_name_gitlab() {
        let parsed = ParsedGitUrl {
            domain: "gitlab.org".to_string(),
            owner: "group".to_string(),
            repo: "project".to_string(),
            original_url: "https://gitlab.org/group/project".to_string(),
        };
        assert_eq!(generate_folder_name(&parsed), "gitlab_group_project");
    }

    #[test]
    fn test_generate_folder_name_bitbucket() {
        let parsed = ParsedGitUrl {
            domain: "bitbucket.org".to_string(),
            owner: "team".to_string(),
            repo: "project".to_string(),
            original_url: "https://bitbucket.org/team/project".to_string(),
        };
        assert_eq!(generate_folder_name(&parsed), "bitbucket_team_project");
    }

    #[test]
    fn test_sanitize_special_chars() {
        assert_eq!(sanitize("hello/world"), "hello_world");
        assert_eq!(sanitize("hello\\world"), "hello_world");
        assert_eq!(sanitize("hello world"), "hello_world");
        assert_eq!(sanitize("hello-world"), "hello-world");
    }

    #[test]
    fn test_sanitize_unicode() {
        // Unicode alphanumeric chars should be preserved
        assert_eq!(sanitize("项目"), "项目");
        assert_eq!(sanitize("test项目"), "test项目");
    }

    #[test]
    fn test_validate_git_url_empty() {
        assert!(validate_git_url("", 1000).is_err());
    }

    #[test]
    fn test_validate_git_url_too_long() {
        let long_url = "a".repeat(1001);
        assert!(matches!(
            validate_git_url(&long_url, 1000),
            Err(CloneError::UrlTooLong(1000))
        ));
    }

    #[test]
    fn test_validate_git_url_starts_with_dash() {
        assert!(validate_git_url("-evil-flag", 1000).is_err());
    }

    #[test]
    fn test_parse_invalid_url() {
        assert!(parse_git_url("not-a-url").is_err());
        assert!(parse_git_url("").is_err());
    }

    #[test]
    fn test_parse_git_protocol() {
        let parsed = parse_git_url("git://github.com/owner/repo.git").unwrap();
        assert_eq!(parsed.domain, "github.com");
        assert_eq!(parsed.owner, "owner");
        assert_eq!(parsed.repo, "repo");
    }

    #[test]
    fn test_parse_bitbucket_url() {
        let parsed = parse_git_url("https://bitbucket.org/team/project.git").unwrap();
        assert_eq!(parsed.domain, "bitbucket.org");
        assert_eq!(parsed.owner, "team");
        assert_eq!(parsed.repo, "project");
    }

    #[test]
    fn test_parse_azure_devops_url() {
        let parsed = parse_git_url("https://dev.azure.com/org/project/_git/repo").unwrap();
        assert_eq!(parsed.domain, "dev.azure.com");
        assert_eq!(parsed.owner, "org-project-_git");
        assert_eq!(parsed.repo, "repo");
    }

    #[test]
    fn test_parse_with_trailing_slash() {
        let parsed = parse_git_url("https://github.com/owner/repo/").unwrap();
        assert_eq!(parsed.repo, "repo");
    }

    #[test]
    fn test_generate_folder_name_special_chars() {
        let parsed = ParsedGitUrl {
            domain: "github.com".to_string(),
            owner: "user/name".to_string(),
            repo: "repo@name".to_string(),
            original_url: "https://github.com/user/name/repo@name".to_string(),
        };
        assert_eq!(generate_folder_name(&parsed), "github_user_name_repo_name");
    }

    #[test]
    fn test_generate_folder_name_unicode() {
        let parsed = ParsedGitUrl {
            domain: "github.com".to_string(),
            owner: "用户".to_string(),
            repo: "项目".to_string(),
            original_url: "https://github.com/用户/项目".to_string(),
        };
        assert_eq!(generate_folder_name(&parsed), "github_用户_项目");
    }

    #[test]
    fn test_validate_git_url_whitespace_only() {
        assert!(validate_git_url("   ", 1000).is_err());
        assert!(validate_git_url("\t\n", 1000).is_err());
    }

    #[test]
    fn test_validate_git_url_with_whitespace_prefix() {
        // Should fail because it starts with whitespace then dash
        assert!(validate_git_url("  -evil", 1000).is_err());
    }

    #[test]
    fn test_strip_tld() {
        assert_eq!(strip_tld("github.com"), "github");
        assert_eq!(strip_tld("gitlab.org"), "gitlab");
        assert_eq!(strip_tld("bitbucket.io"), "bitbucket");
        assert_eq!(strip_tld("my-site.net"), "my-site");
        assert_eq!(strip_tld("example.co.uk"), "example");
        assert_eq!(strip_tld("no-tld"), "no-tld");
        assert_eq!(strip_tld("GitHub.COM"), "github"); // Case insensitive
    }

    #[test]
    fn test_sanitize_preserve_dots() {
        assert_eq!(sanitize(".github.io"), ".github.io");
        assert_eq!(sanitize("repo.name"), "repo.name");
    }

    #[test]
    fn test_sanitize_multiple_special_chars() {
        assert_eq!(sanitize("a/b\\c d"), "a_b_c_d");
    }

    #[test]
    fn test_parse_deeply_nested_gitlab() {
        // GitLab supports nested groups
        let parsed = parse_git_url("https://gitlab.com/a/b/c/d/e/repo.git").unwrap();
        assert_eq!(parsed.domain, "gitlab.com");
        assert_eq!(parsed.owner, "a-b-c-d-e");
        assert_eq!(parsed.repo, "repo");
    }

    #[test]
    fn test_parse_ssh_without_git_suffix() {
        let parsed = parse_git_url("git@github.com:owner/repo").unwrap();
        assert_eq!(parsed.repo, "repo");
    }

    #[test]
    fn test_parse_single_path_component() {
        // Should fail - need at least owner/repo
        assert!(parse_git_url("https://github.com/owner").is_err());
    }

    #[test]
    fn test_parse_too_many_slashes() {
        // Edge case: multiple slashes should be handled
        let parsed = parse_git_url("https://github.com///owner///repo///").unwrap();
        assert_eq!(parsed.owner, "owner");
        assert_eq!(parsed.repo, "repo");
    }

    #[test]
    fn test_validate_clone_target_nonexistent_path() {
        // Create a temporary path that doesn't exist
        // Use canonical temp dir to avoid symlink issues on macOS
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("this_should_not_exist_for_testing_12345");
        let allowed_dirs = vec![temp_dir.canonicalize().unwrap_or(temp_dir.clone())];

        // Should succeed for non-existent path within allowed dir
        let result = validate_clone_target(&temp_path, &allowed_dirs);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_clone_target_outside_allowed() {
        use std::path::PathBuf;

        let temp_path = PathBuf::from("/etc/passwd/repo");
        let allowed_dirs = vec![PathBuf::from("/home")];

        let result = validate_clone_target(&temp_path, &allowed_dirs);
        assert!(result.is_err());
    }
}
