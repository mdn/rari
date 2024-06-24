use std::fs;
use std::path::Path;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Deserialize, Serialize, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    pub shortname: String,
    pub current_specification: String,
    pub title: String,
    pub short_title: String,
    pub nightly_url: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct Nightly {
    pub url: String,
    pub status: String,
    pub source_path: Option<String>,
    pub alternate_urls: Vec<String>,
    pub repository: Option<String>,
    pub filename: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub name: String,
    pub url: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct Tests {
    pub repository: String,
    pub test_paths: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct WebSpec {
    pub url: String,
    pub series_composition: String,
    pub shortname: String,
    pub series: Series,
    pub nightly: Option<Nightly>,
    pub organization: String,
    pub groups: Vec<Group>,
    pub title: String,
    pub source: String,
    pub short_title: String,
    pub categories: Vec<String>,
    pub standing: String,
    pub tests: Option<Tests>,
}

#[derive(Debug, Clone, Default)]
pub struct WebSpecs {
    pub specs: IndexMap<String, WebSpec>,
}

impl WebSpecs {
    pub fn from_file(path: &Path) -> Result<Self, Error> {
        let json_str = fs::read_to_string(path)?;
        let list: Vec<WebSpec> = serde_json::from_str(&json_str)?;
        Ok(Self {
            specs: list
                .into_iter()
                .map(|spec| (spec.url.clone(), spec))
                .collect(),
        })
    }

    pub fn get_spec(&self, url: &str) -> Option<&WebSpec> {
        if let Some(spec) = self.specs.get(url) {
            return Some(spec);
        }

        self.specs.values().find(|spec| {
            url.starts_with(&spec.url)
                || spec
                    .nightly
                    .as_ref()
                    .map(|nighty| {
                        url.starts_with(&nighty.url)
                            || nighty.alternate_urls.iter().any(|s| url.starts_with(s))
                            || spec.shortname == spec.series.current_specification
                                && spec
                                    .series
                                    .nightly_url
                                    .as_ref()
                                    .map(|s| url.starts_with(s))
                                    .unwrap_or_default()
                    })
                    .unwrap_or_default()
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct BCDSpecUrls {
    pub specs_urls_by_key: IndexMap<String, Vec<String>>,
}

impl BCDSpecUrls {
    pub fn from_file(path: &Path) -> Result<Self, Error> {
        let json_str = fs::read_to_string(path)?;
        Ok(Self {
            specs_urls_by_key: serde_json::from_str(&json_str)?,
        })
    }
}
