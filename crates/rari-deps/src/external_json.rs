use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::error::DepsError;

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct LastChecked {
    pub latest_last_check: Option<DateTime<Utc>>,
}

pub fn get_json(name: &str, url: &str, out_path: &Path) -> Result<Option<PathBuf>, DepsError> {
    let package_path = out_path.join(name);
    let last_check_path = package_path.join("last_check.json");
    let now = Utc::now();
    let current = fs::read_to_string(last_check_path)
        .ok()
        .and_then(|current| serde_json::from_str::<LastChecked>(&current).ok())
        .unwrap_or_default();
    if current.latest_last_check.unwrap_or_default() < now - Duration::days(1) {
        if package_path.exists() {
            fs::remove_dir_all(&package_path)?;
        }
        fs::create_dir_all(&package_path)?;
        let buf = reqwest::blocking::get(url)?.bytes()?;

        let out_file = package_path.join("data.json");
        let file = File::create(out_file).unwrap();
        let mut buffed = BufWriter::new(file);
        buffed.write_all(buf.as_ref())?;

        fs::write(
            package_path.join("last_check.json"),
            serde_json::to_string_pretty(&LastChecked {
                latest_last_check: Some(now),
            })?,
        )?;
    }
    Ok(None)
}
