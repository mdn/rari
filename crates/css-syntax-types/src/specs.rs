use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BrowserSpec {
    pub url: String,
    #[serde(rename = "seriesComposition")]
    pub series_composition: String,
    pub shortname: String,
    pub series: Series,
    #[serde(rename = "seriesVersion")]
    pub series_version: Option<String>,
    #[serde(rename = "formerNames", default)]
    pub former_names: Vec<String>,
    pub nightly: Option<NightlySpec>,
    pub title: String,
    #[serde(rename = "shortTitle")]
    pub short_title: String,
    pub organization: String,
    pub groups: Vec<Group>,
    pub release: Option<ReleaseSpec>,
    pub source: String,
    pub categories: Vec<String>,
    pub standing: String,
    pub tests: Option<TestInfo>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>, // For any additional fields
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Series {
    pub shortname: String,
    #[serde(rename = "currentSpecification")]
    pub current_specification: String,
    pub title: String,
    #[serde(rename = "shortTitle")]
    pub short_title: String,
    #[serde(rename = "releaseUrl")]
    pub release_url: Option<String>,
    #[serde(rename = "nightlyUrl")]
    pub nightly_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NightlySpec {
    pub url: String,
    pub status: String,
    #[serde(rename = "sourcePath")]
    pub source_path: Option<String>,
    #[serde(rename = "alternateUrls", default)]
    pub alternate_urls: Vec<String>,
    pub repository: Option<String>,
    pub filename: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReleaseSpec {
    pub url: String,
    pub status: String,
    pub pages: Option<Vec<String>>,
    pub filename: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Group {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestInfo {
    pub repository: String,
    #[serde(rename = "testPaths")]
    pub test_paths: Vec<String>,
}
