//! Asks GitHub for the latest release, at launch and once a day, and offers it when it's newer than this
//! build. Updating stays manual: the app opens the release page.

use std::time::Duration;

use serde::Deserialize;

const LATEST_RELEASE_URL: &str = "https://api.github.com/repos/ArefMozafari/Nasib/releases/latest";
const TAG_PREFIX: &str = "v";
const VERSION_SEPARATOR: char = '.';
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);
/// GitHub's API refuses requests without a user agent.
const USER_AGENT: &str = "Nasib";
pub const CHECK_INTERVAL_SECONDS: u32 = 24 * 60 * 60;
/// This build's version.
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    pub version: String,
    pub page: String,
}

/// The fields of GitHub's latest-release response that the checker reads.
#[derive(Deserialize)]
struct LatestReleaseResponse {
    tag_name: String,
    html_url: String,
}

fn components(version: &str) -> Vec<u64> {
    version
        .split(VERSION_SEPARATOR)
        .map(|part| part.parse().unwrap_or(0))
        .collect()
}

/// Compares dotted version numbers component by component; a missing component counts as zero.
pub fn is_newer(candidate: &str, current: &str) -> bool {
    let candidate = components(candidate);
    let current = components(current);
    for index in 0..candidate.len().max(current.len()) {
        let candidate_part = candidate.get(index).copied().unwrap_or(0);
        let current_part = current.get(index).copied().unwrap_or(0);
        if candidate_part != current_part {
            return candidate_part > current_part;
        }
    }
    false
}

/// Reads the version and release page from GitHub's latest-release response; `None` for anything else.
pub fn parse_release(body: &str) -> Option<Release> {
    let response: LatestReleaseResponse = serde_json::from_str(body).ok()?;
    let version = response
        .tag_name
        .strip_prefix(TAG_PREFIX)
        .unwrap_or(&response.tag_name)
        .to_string();
    Some(Release {
        version,
        page: response.html_url,
    })
}

/// One notification per new version, so a relaunch doesn't notify again.
pub fn should_notify(version: &str, notified_version: Option<&str>) -> bool {
    notified_version != Some(version)
}

/// Fetches the latest release when it's newer than this build. Blocks, so call it off the main thread.
/// Stays quiet on failure: a filtered network often can't reach GitHub, and the next check retries.
pub fn check() -> Option<Release> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(REQUEST_TIMEOUT))
        .build()
        .into();
    let body = agent
        .get(LATEST_RELEASE_URL)
        .header("User-Agent", USER_AGENT)
        .call()
        .ok()?
        .body_mut()
        .read_to_string()
        .ok()?;
    parse_release(&body).filter(|release| is_newer(&release.version, CURRENT_VERSION))
}
