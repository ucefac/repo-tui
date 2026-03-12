//! Update checking logic

use crate::error::UpdateError;
use crate::update::types::{UpdateCheckResult, UpdateInfo, VersionComparison};
use std::time::SystemTime;

const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
const REQUEST_TIMEOUT_SECS: u64 = 30;

/// Fetch latest release from GitHub
pub async fn fetch_latest_release(owner: &str, repo: &str) -> Result<UpdateInfo, UpdateError> {
    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .build()
        .map_err(|e| UpdateError::Network(e.to_string()))?;

    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        owner, repo
    );

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| UpdateError::Network(e.to_string()))?;

    match response.status() {
        reqwest::StatusCode::OK => {
            let release: UpdateInfo = response
                .json()
                .await
                .map_err(|e| UpdateError::ApiError(e.to_string()))?;
            Ok(release)
        }
        reqwest::StatusCode::FORBIDDEN => {
            // GitHub API rate limit
            Err(UpdateError::RateLimitExceeded)
        }
        reqwest::StatusCode::NOT_FOUND => Err(UpdateError::NoReleasesFound),
        status => Err(UpdateError::ApiError(format!("HTTP {}", status))),
    }
}

/// Compare two semantic versions
pub fn compare_versions(current: &str, latest: &str) -> VersionComparison {
    // Remove 'v' prefix if present
    let current_clean = current.trim_start_matches('v');
    let latest_clean = latest.trim_start_matches('v');

    let current_ver = match semver::Version::parse(current_clean) {
        Ok(v) => v,
        Err(_) => return VersionComparison::Incomparable,
    };

    let latest_ver = match semver::Version::parse(latest_clean) {
        Ok(v) => v,
        Err(_) => return VersionComparison::Incomparable,
    };

    if latest_ver > current_ver {
        VersionComparison::UpdateAvailable
    } else {
        VersionComparison::CurrentIsNewerOrEqual
    }
}

/// Check for updates
pub async fn check_for_update(owner: &str, repo: &str) -> Result<UpdateCheckResult, UpdateError> {
    let release = fetch_latest_release(owner, repo).await?;
    let current_version = env!("CARGO_PKG_VERSION");

    let comparison = compare_versions(current_version, &release.tag_name);
    let checked_at = SystemTime::now();

    match comparison {
        VersionComparison::UpdateAvailable => {
            Ok(UpdateCheckResult::update_available(release, checked_at))
        }
        VersionComparison::CurrentIsNewerOrEqual => Ok(UpdateCheckResult::up_to_date(checked_at)),
        VersionComparison::Incomparable => Err(UpdateError::VersionParseError(
            "Failed to parse version".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_versions_update_available() {
        assert_eq!(
            compare_versions("0.1.0", "0.2.0"),
            VersionComparison::UpdateAvailable
        );
        assert_eq!(
            compare_versions("0.1.0", "1.0.0"),
            VersionComparison::UpdateAvailable
        );
    }

    #[test]
    fn test_compare_versions_current_newer() {
        assert_eq!(
            compare_versions("0.2.0", "0.1.0"),
            VersionComparison::CurrentIsNewerOrEqual
        );
        assert_eq!(
            compare_versions("1.0.0", "0.9.0"),
            VersionComparison::CurrentIsNewerOrEqual
        );
    }

    #[test]
    fn test_compare_versions_equal() {
        assert_eq!(
            compare_versions("0.1.0", "0.1.0"),
            VersionComparison::CurrentIsNewerOrEqual
        );
    }

    #[test]
    fn test_compare_versions_with_v_prefix() {
        assert_eq!(
            compare_versions("v0.1.0", "v0.2.0"),
            VersionComparison::UpdateAvailable
        );
        assert_eq!(
            compare_versions("0.1.0", "v0.2.0"),
            VersionComparison::UpdateAvailable
        );
        assert_eq!(
            compare_versions("v0.1.0", "0.1.0"),
            VersionComparison::CurrentIsNewerOrEqual
        );
    }

    #[test]
    fn test_compare_versions_prerelease() {
        // Pre-release versions are considered less than the release version
        assert_eq!(
            compare_versions("0.1.0-alpha", "0.1.0"),
            VersionComparison::UpdateAvailable
        );
        assert_eq!(
            compare_versions("0.1.0-beta", "0.1.0"),
            VersionComparison::UpdateAvailable
        );
        assert_eq!(
            compare_versions("0.1.0-alpha", "0.1.0-alpha.1"),
            VersionComparison::UpdateAvailable
        );
    }

    #[test]
    fn test_compare_versions_invalid() {
        assert_eq!(
            compare_versions("not-a-version", "0.1.0"),
            VersionComparison::Incomparable
        );
        assert_eq!(
            compare_versions("0.1.0", "not-a-version"),
            VersionComparison::Incomparable
        );
    }
}
