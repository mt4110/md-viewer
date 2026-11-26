use reqwest::{StatusCode, redirect};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, time::Duration};

const LATEST_API: &str = "https://api.github.com/repos/mt4110/md-viewer/releases/latest";
pub const RELEASES_PAGE: &str = "https://github.com/mt4110/md-viewer/releases";
const MAX_RESPONSE: usize = 512 * 1024;

#[derive(Debug, PartialEq, Serialize)]
#[serde(
    tag = "status",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum UpdateCheck {
    Release {
        version: String,
        comparison: VersionComparison,
        has_installer: bool,
    },
    NoRelease,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum VersionComparison {
    Newer,
    Current,
    Older,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UpdateError {
    Unavailable,
    RateLimited,
    InvalidRelease,
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    state: String,
}

pub async fn check(current: &Version) -> Result<UpdateCheck, UpdateError> {
    let client = reqwest::Client::builder()
        .https_only(true)
        .redirect(redirect::Policy::none())
        .timeout(Duration::from_secs(8))
        .user_agent("md-viewer")
        .build()
        .map_err(|_| UpdateError::Unavailable)?;
    let mut response = client
        .get(LATEST_API)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|_| UpdateError::Unavailable)?;
    match response.status() {
        StatusCode::NOT_FOUND => return Ok(UpdateCheck::NoRelease),
        StatusCode::TOO_MANY_REQUESTS => return Err(UpdateError::RateLimited),
        StatusCode::FORBIDDEN
            if response.headers().contains_key("retry-after")
                || response
                    .headers()
                    .get("x-ratelimit-remaining")
                    .is_some_and(|v| v == "0") =>
        {
            return Err(UpdateError::RateLimited);
        }
        StatusCode::OK => {}
        _ => return Err(UpdateError::Unavailable),
    }
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| UpdateError::Unavailable)?
    {
        if chunk.len() > MAX_RESPONSE - body.len() {
            return Err(UpdateError::InvalidRelease);
        }
        body.extend_from_slice(&chunk);
    }
    compare_release(current, &body, std::env::consts::OS)
}

fn compare_release(current: &Version, body: &[u8], os: &str) -> Result<UpdateCheck, UpdateError> {
    let release: Release = serde_json::from_slice(body).map_err(|_| UpdateError::InvalidRelease)?;
    let version = Version::parse(
        release
            .tag_name
            .strip_prefix('v')
            .unwrap_or(&release.tag_name),
    )
    .map_err(|_| UpdateError::InvalidRelease)?;
    if release.draft || release.prerelease || !version.pre.is_empty() {
        return Err(UpdateError::InvalidRelease);
    }
    let comparison = match version.cmp_precedence(current) {
        Ordering::Greater => VersionComparison::Newer,
        Ordering::Equal => VersionComparison::Current,
        Ordering::Less => VersionComparison::Older,
    };
    let extensions: &[&str] = match os {
        "macos" => &[".dmg"],
        "windows" => &[".exe", ".msi", ".msix", ".msixbundle"],
        "linux" => &[".appimage", ".deb", ".rpm"],
        _ => &[],
    };
    let has_installer = release.assets.iter().any(|asset| {
        let name = asset.name.to_ascii_lowercase();
        asset.state == "uploaded" && extensions.iter().any(|ext| name.ends_with(ext))
    });
    Ok(UpdateCheck::Release {
        version: version.to_string(),
        comparison,
        has_installer,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions_and_detects_installers_for_the_current_os() {
        let body = br#"{
            "tag_name":"v0.10.0","draft":false,"prerelease":false,
            "assets":[
                {"name":"md-viewer.exe","state":"uploaded"},
                {"name":"md-viewer.dmg","state":"open"},
                {"name":"md-viewer-source.tar.gz","state":"uploaded"}
            ]
        }"#;
        for (current, os, comparison, has_installer) in [
            ("0.9.0", "windows", VersionComparison::Newer, true),
            ("0.10.0+dev", "macos", VersionComparison::Current, false),
            ("0.11.0", "macos", VersionComparison::Older, false),
        ] {
            assert_eq!(
                compare_release(&Version::parse(current).unwrap(), body, os),
                Ok(UpdateCheck::Release {
                    version: "0.10.0".into(),
                    comparison,
                    has_installer,
                }),
            );
        }
    }

    #[test]
    fn rejects_malformed_or_nonstable_release_metadata() {
        for (tag, draft, prerelease) in [
            ("nightly", false, false),
            ("v0.2.0-beta.1", false, false),
            ("v0.2.0", true, false),
            ("v0.2.0", false, true),
        ] {
            let body = serde_json::json!({
                "tag_name": tag, "draft": draft, "prerelease": prerelease, "assets": []
            });
            assert_eq!(
                compare_release(&Version::new(0, 1, 0), body.to_string().as_bytes(), "macos"),
                Err(UpdateError::InvalidRelease)
            );
        }
    }
}
