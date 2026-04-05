use std::env;
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

use serde::Deserialize;

use crate::error::{KosError, Result};

/// GitHub release metadata (subset).
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    #[allow(dead_code)]
    prerelease: bool,
    published_at: String,
    #[serde(default)]
    assets: Vec<GitHubAsset>,
}

/// GitHub release asset metadata.
#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

/// How kos was installed — determines update strategy.
#[derive(Debug, PartialEq, Eq)]
pub enum InstallMethod {
    /// Installed via Homebrew (macOS). User should run `brew upgrade`.
    Homebrew,
    /// Installed via a Linux system package manager (apt/dpkg, yum/rpm, apk).
    LinuxPackageManager { manager: String },
    /// Direct binary install — self-update is possible.
    DirectBinary,
}

/// Build the HTTP client used for GitHub API and asset downloads.
fn http_client() -> Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .user_agent("kos-updater")
        .build()
        .map_err(|e| KosError::Update {
            message: format!("http client error: {e}"),
        })
}

/// Detect how kos was installed by examining the binary path.
pub fn detect_install_method() -> Result<InstallMethod> {
    let exe = env::current_exe().map_err(|e| KosError::Update {
        message: format!("cannot determine binary path: {e}"),
    })?;

    let path_str = exe.to_string_lossy();

    // macOS Homebrew: path contains /Cellar/ or /homebrew/
    if path_str.contains("/Cellar/") || path_str.contains("/homebrew/") {
        return Ok(InstallMethod::Homebrew);
    }

    // Linux package managers: check if the binary is managed by dpkg, rpm, or apk
    if cfg!(target_os = "linux") {
        if let Some(manager) = detect_linux_package_manager(&path_str) {
            return Ok(InstallMethod::LinuxPackageManager { manager });
        }
    }

    Ok(InstallMethod::DirectBinary)
}

/// Return the brew formula name based on the current binary name.
pub fn brew_formula_name() -> &'static str {
    let is_alpha_binary = env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .is_some_and(|name| name.starts_with("kos-a"));

    if is_alpha_binary {
        "ArcavenAE/tap/kos-a"
    } else {
        "ArcavenAE/tap/kos"
    }
}

/// Check if a Linux package manager owns the binary path.
fn detect_linux_package_manager(binary_path: &str) -> Option<String> {
    if Command::new("dpkg")
        .args(["-S", binary_path])
        .output()
        .is_ok_and(|o| o.status.success())
    {
        return Some("apt".to_string());
    }

    if Command::new("rpm")
        .args(["-qf", binary_path])
        .output()
        .is_ok_and(|o| o.status.success())
    {
        return Some("yum/dnf".to_string());
    }

    if Command::new("apk")
        .args(["info", "--who-owns", binary_path])
        .output()
        .is_ok_and(|o| o.status.success())
    {
        return Some("apk".to_string());
    }

    None
}

/// Determine the expected asset name for the current platform.
fn asset_name() -> Result<String> {
    let os = if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        return Err(KosError::Update {
            message: format!("unsupported OS: {}", env::consts::OS),
        });
    };

    let arch = match env::consts::ARCH {
        "aarch64" => "arm64",
        "x86_64" => "amd64",
        other => {
            return Err(KosError::Update {
                message: format!("unsupported architecture: {other}"),
            });
        }
    };

    Ok(format!("kos-{os}-{arch}"))
}

/// The tag prefix for the current channel, derived from the compile-time channel.
fn channel_tag_prefix() -> String {
    format!("{}-", env!("KOS_CHANNEL"))
}

/// Find a release matching the given criteria.
///
/// If `target_version` is `None`, returns the latest release for the current channel.
/// If `target_version` is `Some(v)`, finds the release whose tag matches `v`
/// (exact match first, then partial/contains match, picking latest if ambiguous).
fn find_release<'a>(
    releases: &'a [GitHubRelease],
    target_version: Option<&str>,
) -> Result<&'a GitHubRelease> {
    let prefix = channel_tag_prefix();
    let channel_releases: Vec<&GitHubRelease> = releases
        .iter()
        .filter(|r| r.tag_name.starts_with(&prefix))
        .collect();

    if channel_releases.is_empty() {
        return Err(KosError::Update {
            message: format!("no releases found for channel '{}'", env!("KOS_CHANNEL")),
        });
    }

    match target_version {
        None => channel_releases
            .into_iter()
            .max_by_key(|r| r.published_at.clone())
            .ok_or_else(|| KosError::Update {
                message: "no releases found".to_string(),
            }),
        Some(version) => {
            // Exact match first
            if let Some(release) = channel_releases.iter().find(|r| r.tag_name == version) {
                return Ok(release);
            }
            // Partial match: user typed a prefix or substring
            let matches: Vec<&&GitHubRelease> = channel_releases
                .iter()
                .filter(|r| r.tag_name.starts_with(version) || r.tag_name.contains(version))
                .collect();
            match matches.len() {
                0 => Err(KosError::Update {
                    message: format!(
                        "no release matching '{version}'. Available: {}",
                        channel_releases
                            .iter()
                            .map(|r| r.tag_name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                }),
                _ => matches
                    .into_iter()
                    .max_by_key(|r| r.published_at.clone())
                    .copied()
                    .ok_or_else(|| KosError::Update {
                        message: "no releases found".to_string(),
                    }),
            }
        }
    }
}

/// Fetch releases from the GitHub API.
fn fetch_releases() -> Result<Vec<GitHubRelease>> {
    let client = http_client()?;
    client
        .get("https://api.github.com/repos/ArcavenAE/kos/releases")
        .send()
        .map_err(|e| KosError::Update {
            message: format!("failed to fetch releases: {e}"),
        })?
        .json()
        .map_err(|e| KosError::Update {
            message: format!("failed to parse releases: {e}"),
        })
}

/// Check for available updates on GitHub.
///
/// If `target_version` is None, finds the latest release for the current channel.
/// If specified, finds the matching release (for upgrade or downgrade).
pub fn check_for_update(target_version: Option<&str>) -> Result<Option<String>> {
    let releases = fetch_releases()?;
    match find_release(&releases, target_version) {
        Ok(release) => Ok(Some(release.tag_name.clone())),
        Err(_) => Ok(None),
    }
}

/// Download and install a release, replacing the current binary.
///
/// If `target_version` is None, installs the latest release.
/// If specified, installs the matching release (upgrade or downgrade).
///
/// Returns the new version tag on success.
pub fn download_and_install(target_version: Option<&str>) -> Result<String> {
    let releases = fetch_releases()?;
    let release = find_release(&releases, target_version)?;

    // Find the right asset for this platform
    let expected_asset = asset_name()?;
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == expected_asset)
        .ok_or_else(|| KosError::Update {
            message: format!(
                "no asset named '{expected_asset}' in release {}. Available: {}",
                release.tag_name,
                release
                    .assets
                    .iter()
                    .map(|a| a.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        })?;

    // Determine current binary path
    let current_exe = env::current_exe().map_err(|e| KosError::Update {
        message: format!("cannot determine binary path: {e}"),
    })?;

    // Resolve symlinks to get the actual binary location
    let current_exe = current_exe.canonicalize().map_err(|e| KosError::Update {
        message: format!("cannot resolve binary path: {e}"),
    })?;

    let parent_dir = current_exe.parent().ok_or_else(|| KosError::Update {
        message: "binary has no parent directory".to_string(),
    })?;

    // Check we can write to the directory
    let test_path = parent_dir.join(".kos-update-test");
    fs::write(&test_path, b"test").map_err(|e| KosError::Update {
        message: format!(
            "cannot write to {}: {e}\n\nTry running with sudo, or move the binary to a user-writable location.",
            parent_dir.display()
        ),
    })?;
    let _ = fs::remove_file(&test_path);

    let exe_name = current_exe
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "kos".to_string());

    let new_path = parent_dir.join(format!("{exe_name}.new"));
    let old_path = parent_dir.join(format!("{exe_name}.old"));

    // Download the asset
    let client = http_client()?;
    println!("Downloading {}...", asset.name);
    let response = client
        .get(&asset.browser_download_url)
        .send()
        .map_err(|e| KosError::Update {
            message: format!("download failed: {e}"),
        })?;

    if !response.status().is_success() {
        return Err(KosError::Update {
            message: format!(
                "download failed with HTTP {}: {}",
                response.status(),
                asset.browser_download_url
            ),
        });
    }

    let bytes = response.bytes().map_err(|e| KosError::Update {
        message: format!("failed to read download body: {e}"),
    })?;

    if bytes.is_empty() {
        return Err(KosError::Update {
            message: "downloaded file is empty".to_string(),
        });
    }

    // Write to .new file
    {
        let mut file = fs::File::create(&new_path).map_err(|e| KosError::Update {
            message: format!("cannot create {}: {e}", new_path.display()),
        })?;
        file.write_all(&bytes).map_err(|e| {
            let _ = fs::remove_file(&new_path);
            KosError::Update {
                message: format!("failed to write {}: {e}", new_path.display()),
            }
        })?;
        file.flush().map_err(|e| {
            let _ = fs::remove_file(&new_path);
            KosError::Update {
                message: format!("failed to flush {}: {e}", new_path.display()),
            }
        })?;
    }

    // Set executable permissions (rwxr-xr-x = 0o755)
    fs::set_permissions(&new_path, fs::Permissions::from_mode(0o755)).map_err(|e| {
        let _ = fs::remove_file(&new_path);
        KosError::Update {
            message: format!("cannot set permissions on {}: {e}", new_path.display()),
        }
    })?;

    // Atomic swap: current -> .old, .new -> current, cleanup .old
    fs::rename(&current_exe, &old_path).map_err(|e| {
        let _ = fs::remove_file(&new_path);
        KosError::Update {
            message: format!(
                "cannot backup current binary to {}: {e}",
                old_path.display()
            ),
        }
    })?;

    if let Err(e) = fs::rename(&new_path, &current_exe) {
        // Rollback: restore the old binary
        let _ = fs::rename(&old_path, &current_exe);
        let _ = fs::remove_file(&new_path);
        return Err(KosError::Update {
            message: format!(
                "cannot install new binary to {}: {e}",
                current_exe.display()
            ),
        });
    }

    let _ = fs::remove_file(&old_path);

    Ok(release.tag_name.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_name_is_valid_format() {
        let name = asset_name().expect("should detect platform");
        assert!(name.starts_with("kos-"), "unexpected asset name: {name}");
        let parts: Vec<&str> = name.split('-').collect();
        assert_eq!(parts.len(), 3, "expected 3 dash-separated parts: {name}");
        let os = parts[1];
        let arch = parts[2];
        assert!(os == "darwin" || os == "linux", "unexpected os: {os}");
        assert!(
            arch == "arm64" || arch == "amd64",
            "unexpected arch: {arch}"
        );
    }

    #[test]
    fn detect_install_method_for_current_binary() {
        let method = detect_install_method().expect("should detect install method");
        assert_eq!(method, InstallMethod::DirectBinary);
    }

    #[test]
    fn github_release_deserializes_with_assets() {
        let json = r#"{
            "tag_name": "alpha-20260404-120000-abc1234",
            "prerelease": true,
            "published_at": "2026-04-04T12:00:00Z",
            "assets": [
                {
                    "name": "kos-darwin-arm64",
                    "browser_download_url": "https://example.com/kos-darwin-arm64"
                }
            ]
        }"#;
        let release: GitHubRelease =
            serde_json::from_str(json).expect("should deserialize release");
        assert_eq!(release.tag_name, "alpha-20260404-120000-abc1234");
        assert_eq!(release.assets.len(), 1);
        assert_eq!(release.assets[0].name, "kos-darwin-arm64");
    }

    #[test]
    fn github_release_deserializes_without_assets() {
        let json = r#"{
            "tag_name": "alpha-20260404-120000-abc1234",
            "prerelease": true,
            "published_at": "2026-04-04T12:00:00Z"
        }"#;
        let release: GitHubRelease =
            serde_json::from_str(json).expect("should deserialize release without assets");
        assert!(release.assets.is_empty());
    }

    #[test]
    fn find_release_picks_latest_for_channel() {
        let releases = vec![
            GitHubRelease {
                tag_name: "alpha-20260404-120000-abc1234".to_string(),
                prerelease: true,
                published_at: "2026-04-04T12:00:00Z".to_string(),
                assets: vec![],
            },
            GitHubRelease {
                tag_name: "alpha-20260405-080000-def5678".to_string(),
                prerelease: true,
                published_at: "2026-04-05T08:00:00Z".to_string(),
                assets: vec![],
            },
            GitHubRelease {
                tag_name: "stable-1.0.0".to_string(),
                prerelease: false,
                published_at: "2026-04-06T12:00:00Z".to_string(),
                assets: vec![],
            },
        ];
        let result = find_release(&releases, None).unwrap();
        assert_eq!(result.tag_name, "alpha-20260405-080000-def5678");
    }

    #[test]
    fn find_release_exact_version_match() {
        let releases = vec![
            GitHubRelease {
                tag_name: "alpha-20260404-120000-abc1234".to_string(),
                prerelease: true,
                published_at: "2026-04-04T12:00:00Z".to_string(),
                assets: vec![],
            },
            GitHubRelease {
                tag_name: "alpha-20260405-080000-def5678".to_string(),
                prerelease: true,
                published_at: "2026-04-05T08:00:00Z".to_string(),
                assets: vec![],
            },
        ];
        let result = find_release(&releases, Some("alpha-20260404-120000-abc1234")).unwrap();
        assert_eq!(result.tag_name, "alpha-20260404-120000-abc1234");
    }

    #[test]
    fn find_release_partial_match() {
        let releases = vec![
            GitHubRelease {
                tag_name: "alpha-20260404-120000-abc1234".to_string(),
                prerelease: true,
                published_at: "2026-04-04T12:00:00Z".to_string(),
                assets: vec![],
            },
            GitHubRelease {
                tag_name: "alpha-20260405-080000-def5678".to_string(),
                prerelease: true,
                published_at: "2026-04-05T08:00:00Z".to_string(),
                assets: vec![],
            },
        ];
        let result = find_release(&releases, Some("alpha-20260404")).unwrap();
        assert_eq!(result.tag_name, "alpha-20260404-120000-abc1234");
    }
}
