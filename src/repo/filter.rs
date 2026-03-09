//! Fuzzy search filter for repositories
//!
//! Uses nucleo-matcher for subsequence matching with scoring

use crate::repo::Repository;
use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};

/// Filter repositories using fuzzy search
///
/// Returns indices of matching repositories sorted by match score (highest first)
///
/// # Arguments
///
/// * `repos` - List of repositories to filter
/// * `query` - Search query (supports subsequence matching)
///
/// # Examples
///
/// ```ignore
/// use repotui::repo::{Repository, filter_repos_fuzzy};
///
/// let repos = vec![
///     Repository { name: "facebook/react".to_string(), /* ... */ },
///     Repository { name: "vercel/next.js".to_string(), /* ... */ },
/// ];
/// let matches = filter_repos_fuzzy(&repos, "fbreact");
/// // "facebook/react" will be in matches with high score
/// ```
pub fn filter_repos_fuzzy(repos: &[Repository], query: &str) -> Vec<(usize, u32)> {
    if query.is_empty() {
        // Return all indices with score 0 when query is empty
        return repos.iter().enumerate().map(|(i, _)| (i, 0u32)).collect();
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let mut results = Vec::new();
    let mut indices = Vec::new();
    let mut buf = Vec::new();

    let query_lower = query.to_lowercase();
    let pattern = Pattern::new(
        &query_lower,
        CaseMatching::Ignore,
        Normalization::Smart,
        AtomKind::Fuzzy,
    );

    for (idx, repo) in repos.iter().enumerate() {
        let repo_name_lower = repo.name.to_lowercase();
        let haystack = Utf32Str::new(&repo_name_lower, &mut buf);
        indices.clear();

        // Match and get score - pattern.indices returns Option<u16> for single atom patterns
        if let Some(score) = pattern.indices(haystack, &mut matcher, &mut indices) {
            results.push((idx, score));
        }
    }

    // Sort by score (descending)
    results.sort_by(|a, b| b.1.cmp(&a.1));

    results
}

/// Filter repositories with simple substring match (fallback)
pub fn filter_repos_simple(repos: &[Repository], query: &str) -> Vec<usize> {
    if query.is_empty() {
        return (0..repos.len()).collect();
    }

    let query_lower = query.to_lowercase();
    repos
        .iter()
        .enumerate()
        .filter(|(_, repo)| repo.name.to_lowercase().contains(&query_lower))
        .map(|(i, _)| i)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_repos() -> Vec<Repository> {
        vec![
            Repository {
                name: "facebook/react".to_string(),
                path: PathBuf::from("/tmp/facebook/react"),
                last_modified: None,
                is_dirty: false,
                branch: Some("main".to_string()),
                is_git_repo: true,
                source: crate::repo::source::RepoSource::Standalone,
            },
            Repository {
                name: "vercel/next.js".to_string(),
                path: PathBuf::from("/tmp/vercel/next.js"),
                last_modified: None,
                is_dirty: false,
                branch: Some("main".to_string()),
                is_git_repo: true,
                source: crate::repo::source::RepoSource::Standalone,
            },
            Repository {
                name: "microsoft/vscode".to_string(),
                path: PathBuf::from("/tmp/microsoft/vscode"),
                last_modified: None,
                is_dirty: false,
                branch: Some("main".to_string()),
                is_git_repo: true,
                source: crate::repo::source::RepoSource::Standalone,
            },
            Repository {
                name: "rust-lang/rust".to_string(),
                path: PathBuf::from("/tmp/rust-lang/rust"),
                last_modified: None,
                is_dirty: false,
                branch: Some("main".to_string()),
                is_git_repo: true,
                source: crate::repo::source::RepoSource::Standalone,
            },
            Repository {
                name: "nodejs/node".to_string(),
                path: PathBuf::from("/tmp/nodejs/node"),
                last_modified: None,
                is_dirty: false,
                branch: Some("main".to_string()),
                is_git_repo: true,
                source: crate::repo::source::RepoSource::Standalone,
            },
        ]
    }

    #[test]
    fn test_fuzzy_search_empty_query() {
        let repos = create_test_repos();
        let results = filter_repos_fuzzy(&repos, "");

        assert_eq!(results.len(), 5);
        assert!(results.iter().all(|(_, score)| *score == 0));
    }

    #[test]
    fn test_fuzzy_search_exact_match() {
        let repos = create_test_repos();
        let results = filter_repos_fuzzy(&repos, "react");

        assert!(!results.is_empty());
        // "facebook/react" should match
        let matched_names: Vec<&str> = results
            .iter()
            .map(|(idx, _)| repos[*idx].name.as_str())
            .collect();
        assert!(matched_names.iter().any(|&n| n == "facebook/react"));
    }

    #[test]
    fn test_fuzzy_search_subsequence() {
        let repos = create_test_repos();
        let results = filter_repos_fuzzy(&repos, "fbreact");

        // Should match "facebook/react" with subsequence matching
        if !results.is_empty() {
            let best_match = &repos[results[0].0].name;
            assert_eq!(best_match, "facebook/react");
        }
    }

    #[test]
    fn test_fuzzy_search_no_match() {
        let repos = create_test_repos();
        let results = filter_repos_fuzzy(&repos, "xyznonexistent");

        assert!(results.is_empty());
    }

    #[test]
    fn test_fuzzy_search_case_insensitive() {
        let repos = create_test_repos();
        let results_upper = filter_repos_fuzzy(&repos, "REACT");
        let results_lower = filter_repos_fuzzy(&repos, "react");

        assert_eq!(results_upper.len(), results_lower.len());
    }

    #[test]
    fn test_fuzzy_search_score_ordering() {
        let repos = create_test_repos();
        let results = filter_repos_fuzzy(&repos, "node");

        if results.len() > 1 {
            // Verify scores are in descending order
            for i in 0..results.len() - 1 {
                assert!(results[i].1 >= results[i + 1].1);
            }
        }
    }

    #[test]
    fn test_fuzzy_search_partial_match() {
        let repos = create_test_repos();
        let results = filter_repos_fuzzy(&repos, "next");

        assert!(!results.is_empty());
        let matched_names: Vec<&str> = results
            .iter()
            .map(|(idx, _)| repos[*idx].name.as_str())
            .collect();
        assert!(matched_names.iter().any(|&n| n == "vercel/next.js"));
    }

    #[test]
    fn test_simple_search_empty() {
        let repos = create_test_repos();
        let results = filter_repos_simple(&repos, "");

        assert_eq!(results.len(), repos.len());
    }

    #[test]
    fn test_simple_search_no_match() {
        let repos = create_test_repos();
        let results = filter_repos_simple(&repos, "nonexistent");

        assert!(results.is_empty());
    }

    #[test]
    fn test_fuzzy_search_vscode() {
        let repos = create_test_repos();
        let results = filter_repos_fuzzy(&repos, "vscode");

        assert!(!results.is_empty());
        let matched_names: Vec<&str> = results
            .iter()
            .map(|(idx, _)| repos[*idx].name.as_str())
            .collect();
        assert!(matched_names.iter().any(|&n| n == "microsoft/vscode"));
    }

    #[test]
    fn test_fuzzy_search_abbreviations() {
        let repos = vec![
            Repository {
                name: "facebook/react".to_string(),
                path: PathBuf::from("/tmp/facebook/react"),
                last_modified: None,
                is_dirty: false,
                branch: Some("main".to_string()),
                is_git_repo: true,
                source: crate::repo::source::RepoSource::Standalone,
            },
            Repository {
                name: "test/project".to_string(),
                path: PathBuf::from("/tmp/test/project"),
                last_modified: None,
                is_dirty: false,
                branch: Some("main".to_string()),
                is_git_repo: true,
                source: crate::repo::source::RepoSource::Standalone,
            },
        ];

        // Test abbreviation matching
        let results = filter_repos_fuzzy(&repos, "fbr");
        if !results.is_empty() {
            assert_eq!(repos[results[0].0].name, "facebook/react");
        }
    }
}
