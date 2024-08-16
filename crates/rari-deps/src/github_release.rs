use std::fs::{self, File};
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Duration, Utc};
use rari_utils::io::read_to_string;
use reqwest::redirect::Policy;
use serde::{Deserialize, Serialize};

use crate::error::DepsError;

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Current {
    pub latest_last_check: Option<DateTime<Utc>>,
    pub version: String,
}

/// Download and unpack an npm package for a given version (defaults to latest).
pub fn get_artifact(
    base_url: &str,
    artifact: &str,
    name: &str,
    version: Option<&str>,
    out_path: &Path,
) -> Result<Option<PathBuf>, DepsError> {
    let version = version.unwrap_or("latest");
    let package_path = out_path.join(name);
    let last_check_path = package_path.join("last_check.json");
    let now = Utc::now();
    let current = read_to_string(last_check_path)
        .ok()
        .and_then(|current| serde_json::from_str::<Current>(&current).ok())
        .unwrap_or_default();
    if version != current.version
        || version == "latest"
            && current.latest_last_check.unwrap_or_default() < now - Duration::days(1)
    {
        let prev_url = format!(
            "{base_url}/releases/download/{}/{artifact}",
            current.version
        );
        let url = if version == "latest" {
            let client = reqwest::blocking::ClientBuilder::default()
                .redirect(Policy::none())
                .build()?;
            let res = client
                .get(format!("{base_url}/releases/latest/download/{artifact}"))
                .send()?;
            res.headers()
                .get("location")
                .ok_or(DepsError::InvalidGitHubVersion)?
                .to_str()?
                .to_string()
        } else {
            format!("{base_url}/releases/download/{version}/{artifact}")
        };

        let artifact_path = package_path.join(artifact);
        let download_update = if artifact_path.exists() {
            prev_url != url
        } else {
            true
        };

        if download_update {
            if package_path.exists() {
                fs::remove_dir_all(&package_path)?;
            }
            fs::create_dir_all(&package_path)?;
            let mut buf = vec![];
            let _ = reqwest::blocking::get(url)?.read_to_end(&mut buf)?;

            let file = File::create(artifact_path).unwrap();
            let mut buffed = BufWriter::new(file);

            buffed.write_all(&buf)?;
        }

        if version == "latest" {
            fs::write(
                package_path.join("last_check.json"),
                serde_json::to_string_pretty(&Current {
                    version: version.to_string(),
                    latest_last_check: Some(now),
                })?,
            )?;
        }
        if download_update {
            return Ok(Some(package_path));
        }
    }
    Ok(None)
}
