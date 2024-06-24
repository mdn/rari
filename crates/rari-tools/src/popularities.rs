use std::fs::File;
use std::io::BufWriter;

use chrono::Utc;
use rari_types::globals::content_root;
use rari_types::Popularities;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PopularityRow {
    #[serde(rename = "Page")]
    pub page: String,
    #[serde(rename = "Pageviews")]
    pub page_views: f64,
}

const CURRENT_URL: &str = "https://popularities.mdn.mozilla.net/current.csv";

pub fn update_popularities(limit: usize) -> Popularities {
    let mut popularities = Popularities {
        popularities: Default::default(),
        date: Utc::now().naive_utc(),
    };
    let mut max = f64::INFINITY;
    let pop_csv = reqwest::blocking::get(CURRENT_URL).expect("unable to download popularities");
    let mut rdr = csv::Reader::from_reader(pop_csv);
    for row in rdr.deserialize::<PopularityRow>().flatten().take(limit) {
        if row.page.contains("/docs/") && !row.page.contains(['$', '?']) {
            if max.is_infinite() {
                max = row.page_views;
            }
            popularities
                .popularities
                .insert(row.page, row.page_views / max);
        }
    }
    let out_file = content_root().join("en-US").join("popularities.json");
    let file = File::create(out_file).unwrap();
    let buffed = BufWriter::new(file);

    serde_json::to_writer_pretty(buffed, &popularities).unwrap();
    popularities
}
