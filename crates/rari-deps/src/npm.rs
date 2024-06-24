use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Duration, Utc};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tar::Archive;

use crate::error::DepsError;

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Current {
    pub latest_last_check: Option<DateTime<Utc>>,
    pub version: String,
}

/// Download and unpack an npm package for a given version (defaults to latest).
pub fn get_package(
    package: &str,
    version: Option<&str>,
    out_path: &Path,
) -> Result<Option<PathBuf>, DepsError> {
    let version = version.unwrap_or("latest");
    let package_path = out_path.join(package);
    let last_check_path = package_path.join("last_check.json");
    let now = Utc::now();
    let current = fs::read_to_string(last_check_path)
        .ok()
        .and_then(|current| serde_json::from_str::<Current>(&current).ok())
        .unwrap_or_default();
    if version != current.version
        || version == "latest"
            && current.latest_last_check.unwrap_or_default() < now - Duration::days(1)
    {
        let body: Value =
            reqwest::blocking::get(format!("https://registry.npmjs.org/{package}/{version}"))?
                .json()?;

        let latest_version = body["version"]
            .as_str()
            .ok_or(DepsError::WebRefMissingVersionError)?;
        let tarball_url = body["dist"]["tarball"]
            .as_str()
            .ok_or(DepsError::WebRefMissingTarballError)?;
        let package_json_path = package_path.join("package").join("package.json");
        let download_update = if package_json_path.exists() {
            let json_str = fs::read_to_string(package_json_path)?;
            let package_json: Value = serde_json::from_str(&json_str)?;
            let current_version = package_json["version"]
                .as_str()
                .ok_or(DepsError::WebRefMissingVersionError)?;
            current_version == latest_version
        } else {
            true
        };

        if download_update {
            if package_path.exists() {
                fs::remove_dir_all(&package_path)?;
            }
            fs::create_dir_all(&package_path)?;
            let mut buf = vec![];
            let _ = reqwest::blocking::get(tarball_url)?.read_to_end(&mut buf)?;
            let gz = GzDecoder::new(&buf[..]);
            let mut ar = Archive::new(gz);
            ar.unpack(&package_path)?;
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
