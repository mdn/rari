use std::fs::{self, File};
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use chrono::{Duration, Utc};
use rari_utils::io::read_to_string;
use semver::{Version, VersionReq};
use serde::Deserialize;

use crate::client::get;
use crate::current::Current;
use crate::error::DepsError;
#[derive(Deserialize, Debug, Clone)]
struct VersionEntry {
    tag_name: Option<String>,
}

type Releases = Vec<VersionEntry>;

#[derive(Deserialize, Debug)]
struct GithubError {
    message: String,
}

fn get_version(repo: &str, version_req: &VersionReq) -> Result<(Version, String), DepsError> {
    let url = format!("https://api.github.com/repos/{repo}/releases?per_page=10");
    let resp = get(&url)?;

    if !resp.status().is_success() {
        let status = resp.status();
        let err: Result<GithubError, _> = resp.json();
        return Err(match err {
            Ok(e) => DepsError::UpstreamError(format!("GitHub error: {} ({url})", e.message)),
            Err(_) => DepsError::UpstreamError(format!("GitHub HTTP {}", status)),
        });
    }

    let releases: Releases = serde_json::from_value(resp.json()?)?;
    if let Some(version) = releases.iter().find_map(|k| {
        let version = k
            .tag_name
            .as_ref()
            .and_then(|v| Version::parse(v.trim_start_matches('v')).ok());
        if version
            .as_ref()
            .map(|v| version_req.matches(v))
            .unwrap_or_default()
        {
            version.map(|version| (version, k.tag_name.clone().unwrap()))
        } else {
            None
        }
    }) {
        Ok(version)
    } else {
        Err(DepsError::VersionNotFound)
    }
}
/// Download a github release artifact for a given version (defaults to latest).
pub fn get_artifact(
    repo: &str,
    artifact: &str,
    name: &str,
    version_req: &Option<VersionReq>,
    out_path: &Path,
) -> Result<Option<PathBuf>, DepsError> {
    let star = VersionReq::default();

    let version_req = version_req.as_ref().unwrap_or(&star);
    let package_path = out_path.join(name);
    let last_check_path = package_path.join("last_check.json");
    let now = Utc::now();
    let current = read_to_string(last_check_path)
        .ok()
        .and_then(|current| serde_json::from_str::<Current>(&current).ok())
        .unwrap_or_default();
    if !current
        .current_version
        .as_ref()
        .map(|v| version_req.matches(v))
        .unwrap_or_default()
        || current.latest_last_check.unwrap_or_default() < now - Duration::days(1)
    {
        let (version, tag_name) = get_version(repo, version_req)?;
        let url = format!("https://github.com/{repo}/releases/download/{tag_name}/{artifact}");

        let artifact_path = package_path.join(artifact);
        let download_update = current.current_version.as_ref() != Some(&version);

        if download_update {
            tracing::info!("Updating {repo} ({artifact}) to {version}");
            if package_path.exists() {
                fs::remove_dir_all(&package_path)?;
            }
            fs::create_dir_all(&package_path)?;
            let mut buf = vec![];
            let _ = get(url)?.read_to_end(&mut buf)?;

            let file = File::create(artifact_path).unwrap();
            let mut buffed = BufWriter::new(file);

            buffed.write_all(&buf)?;
        }

        fs::write(
            package_path.join("last_check.json"),
            serde_json::to_string_pretty(&Current {
                current_version: Some(version),
                latest_last_check: Some(now),
            })?,
        )?;
        if download_update {
            return Ok(Some(package_path));
        }
    }
    Ok(None)
}
