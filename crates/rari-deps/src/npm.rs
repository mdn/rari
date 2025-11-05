use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use chrono::{Duration, Utc};
use flate2::read::GzDecoder;
use indexmap::IndexMap;
use rari_utils::io::read_to_string;
use semver::{Version, VersionReq};
use serde::Deserialize;
use tar::Archive;
use url::Url;

use crate::client::get;
use crate::current::Current;
use crate::error::DepsError;

#[derive(Deserialize, Debug, Clone)]
struct Dist {
    tarball: Url,
}

#[derive(Deserialize, Debug, Clone)]
struct VersionEntry {
    version: Version,
    dist: Dist,
}

#[derive(Deserialize, Debug, Clone)]
struct Package {
    versions: IndexMap<Version, VersionEntry>,
}

fn get_version(package_name: &str, version_req: &VersionReq) -> Result<VersionEntry, DepsError> {
    let package: Package = get(format!("https://registry.npmjs.org/{package_name}"))?.json()?;
    if let Some((_, entry)) = package
        .versions
        .iter()
        .rfind(|(k, _)| version_req.matches(k))
    {
        if let Some((latest, _)) = package.versions.last()
            && latest > &entry.version {
                tracing::warn!(
                    "Update for {package_name} available {} -> {}",
                    entry.version,
                    latest
                );
            }
        Ok(entry.clone())
    } else {
        Err(DepsError::VersionNotFound)
    }
}

/// Download and unpack an npm package for a given version (defaults to latest).
pub fn get_package(
    package: &str,
    version_req: &Option<VersionReq>,
    out_path: &Path,
) -> Result<Option<PathBuf>, DepsError> {
    let star = VersionReq::default();
    let version_req = version_req.as_ref().unwrap_or(&star);
    let package_path = out_path.join(package);
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
        let version_entry = get_version(package, version_req)?;
        let tarball_url = version_entry.dist.tarball;
        let download_update = current.current_version.as_ref() != Some(&version_entry.version);

        if download_update {
            tracing::info!("Updating {package} to {}", version_entry.version);
            if package_path.exists() {
                fs::remove_dir_all(&package_path)?;
            }
            fs::create_dir_all(&package_path)?;
            let mut buf = vec![];
            let _ = get(tarball_url)?.read_to_end(&mut buf)?;
            let gz = GzDecoder::new(&buf[..]);
            let mut ar = Archive::new(gz);
            ar.unpack(&package_path)?;
        }

        fs::write(
            package_path.join("last_check.json"),
            serde_json::to_string_pretty(&Current {
                current_version: Some(version_entry.version),
                latest_last_check: Some(now),
            })?,
        )?;
        if download_update {
            return Ok(Some(package_path));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_version() {
        let e = get_version(
            "/@mdn/browser-compat-data",
            &VersionReq::parse("5.6.33").unwrap(),
        )
        .unwrap();
        println!("{} -> {}", e.version, e.dist.tarball)
    }
}
