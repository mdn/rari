use std::fs::{self, File};
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Datelike, Utc};
use rari_types::Popularities;
use rari_utils::io::read_to_string;
use serde::Deserialize;

use crate::current::Current;
use crate::error::DepsError;

#[derive(Debug, Clone, Deserialize)]
pub struct PopularityRow {
    #[serde(rename = "Page")]
    pub page: String,
    #[serde(rename = "Pageviews")]
    pub page_views: f64,
}

const CURRENT_URL: &str = "https://popularities.mdn.mozilla.net/current.csv";
const LIMIT: usize = 20_000;

fn should_update(now: &DateTime<Utc>, current: &Option<DateTime<Utc>>) -> bool {
    let now_date = now.date_naive();
    if let Some(current) = current {
        let current_date = current.date_naive();
        // True if it'a at least 2nd day of a new month vs. current.
        (current_date.year() < now_date.year() || current_date.month() < now_date.month())
            && now_date.day() > 1
    } else {
        true
    }
}

pub fn update_popularities(base_path: &Path) -> Result<Option<PathBuf>, DepsError> {
    let package_path = base_path.join("popularities");
    let last_check_path = package_path.join("last_check.json");
    let now = Utc::now();
    let current = read_to_string(last_check_path)
        .ok()
        .and_then(|current| serde_json::from_str::<Current>(&current).ok())
        .unwrap_or_default();

    if should_update(&now, &current.latest_last_check) {
        let mut popularities = Popularities {
            popularities: Default::default(),
            date: Utc::now().naive_utc(),
        };

        let mut max = f64::INFINITY;
        let pop_csv = reqwest::blocking::get(CURRENT_URL).expect("unable to download popularities");
        let mut rdr = csv::Reader::from_reader(pop_csv);
        for row in rdr.deserialize::<PopularityRow>().flatten().take(LIMIT) {
            if row.page.contains("/docs/") && !row.page.contains(['$', '?']) {
                if max.is_infinite() {
                    max = row.page_views;
                }
                popularities
                    .popularities
                    .insert(row.page, row.page_views / max);
            }
        }

        let artifact_path = package_path.join("popularities.json");
        if package_path.exists() {
            fs::remove_dir_all(&package_path)?;
        }
        fs::create_dir_all(&package_path)?;

        let file = File::create(artifact_path).unwrap();
        let buffed = BufWriter::new(file);

        serde_json::to_writer_pretty(buffed, &popularities).unwrap();

        fs::write(
            package_path.join("last_check.json"),
            serde_json::to_string_pretty(&Current {
                current_version: None,
                latest_last_check: Some(now),
            })?,
        )?;
        return Ok(Some(package_path));
    }
    Ok(None)
}
