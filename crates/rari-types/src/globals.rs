use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, OnceLock};
use std::{env, fs};

use serde::Deserialize;

use crate::error::EnvError;
use crate::locale::Locale;
use crate::settings::{Deps, Settings};
use crate::{globals, HistoryEntry, Popularities};

#[inline(always)]
pub fn content_root() -> &'static Path {
    settings().content_root.as_path()
}

#[inline(always)]
pub fn blog_root() -> Option<&'static Path> {
    settings().blog_root.as_deref()
}

#[inline(always)]
pub fn generic_content_root() -> Option<&'static Path> {
    settings().generic_content_root.as_deref()
}
#[inline(always)]
pub fn curriculum_root() -> Option<&'static Path> {
    settings().curriculum_root.as_deref()
}

#[inline(always)]
pub fn contributor_spotlight_root() -> Option<&'static Path> {
    settings().contributor_spotlight_root.as_deref()
}

#[inline(always)]
pub fn content_translated_root() -> Option<&'static Path> {
    settings().content_translated_root.as_deref()
}

#[inline(always)]
pub fn build_out_root() -> Result<&'static Path, EnvError> {
    settings()
        .build_out_root
        .as_ref()
        .ok_or(EnvError::NoBuildOut)
        .map(|p| p.as_path())
}

#[inline(always)]
pub fn deny_warnings() -> bool {
    settings().deny_warnings
}

#[inline(always)]
pub fn cache_content() -> bool {
    settings().cache_content
}

pub static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn data_dir() -> &'static Path {
    DATA_DIR.get_or_init(|| {
        std::env::var_os("DEPS_DATA_DIR")
            .or_else(|| std::env::var_os("deps_data_dir"))
            .map(PathBuf::from)
            .or_else(dirs::data_local_dir)
            .map(|p| p.join("rari"))
            .unwrap_or_default()
    })
}

pub static SETTINGS: OnceLock<Settings> = OnceLock::new();

pub fn settings() -> &'static Settings {
    SETTINGS.get_or_init(|| Settings::new().expect("error generating settings"))
}

pub static DEPS: OnceLock<Deps> = OnceLock::new();
pub fn deps() -> &'static Deps {
    DEPS.get_or_init(|| Deps::new().expect("error generating deps"))
}

#[derive(Debug, Deserialize)]
pub struct JsonSpecData {
    pub url: String,
}

pub type JsonSpecDataLookup = HashMap<String, String>;

pub static JSON_SPEC_DATA_FILE: OnceLock<JsonSpecDataLookup> = OnceLock::new();

pub fn json_spec_data_lookup() -> &'static JsonSpecDataLookup {
    JSON_SPEC_DATA_FILE.get_or_init(|| {
        let json_str = fs::read_to_string(content_root().join("jsondata/SpecData.json"))
            .expect("unable to read SpecData.json");
        let data: HashMap<String, JsonSpecData> =
            serde_json::from_str(&json_str).expect("unable to parse SpecData.json");
        data.into_iter().map(|(k, v)| (v.url, k)).collect()
    })
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum SVGDataDescription {
    Copy(String),
    L10n(HashMap<Locale, String>),
}

#[derive(Debug, Deserialize)]
pub struct SVGDataContent {
    pub description: SVGDataDescription,
    #[serde(default)]
    pub elements: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SVGData {
    pub categories: Vec<String>,
    pub content: SVGDataContent,
    pub attributes: Vec<String>,
    pub interfaces: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SVGDataContainer {
    elements: HashMap<String, SVGData>,
}

pub type JsonSVGDataLookup = HashMap<String, SVGData>;

pub static JSON_SVG_DATA_FILE: OnceLock<JsonSVGDataLookup> = OnceLock::new();

pub fn json_svg_data_lookup() -> &'static JsonSVGDataLookup {
    JSON_SVG_DATA_FILE.get_or_init(|| {
        let json_str = fs::read_to_string(content_root().join("jsondata/SVGData.json"))
            .expect("unable to read SVGData.json");
        let data: SVGDataContainer =
            serde_json::from_str(&json_str).expect("unable to parse SVGData.json");
        data.elements
    })
}

pub static GIT_HISTORY: LazyLock<HashMap<PathBuf, HistoryEntry>> = LazyLock::new(|| {
    let f = content_root().join("_git_history.json");
    let mut map = if let Ok(json_str) = fs::read_to_string(f) {
        serde_json::from_str(&json_str).expect("unable to parse l10n json")
    } else {
        HashMap::new()
    };
    if let Some(translated_root) = content_translated_root() {
        let f = translated_root.join("_git_history.json");
        if let Ok(json_str) = fs::read_to_string(f) {
            let translated: HashMap<PathBuf, HistoryEntry> =
                serde_json::from_str(&json_str).expect("unable to parse l10n json");
            map.extend(translated);
        };
    }
    map
});
pub fn git_history() -> &'static HashMap<PathBuf, HistoryEntry> {
    &GIT_HISTORY
}

pub static POPULARITIES: LazyLock<Popularities> = LazyLock::new(|| {
    let f = globals::data_dir()
        .join("popularities")
        .join("popularities.json");
    if let Ok(json_str) = fs::read_to_string(f) {
        serde_json::from_str(&json_str).expect("unable to parse l10n json")
    } else {
        Popularities::default()
    }
});
pub fn popularities() -> &'static Popularities {
    &POPULARITIES
}

pub static CONTENT_BRANCH: OnceLock<String> = OnceLock::new();
pub fn content_branch() -> &'static str {
    CONTENT_BRANCH.get_or_init(|| env::var("CONTENT_BRANCH").unwrap_or("main".to_string()))
}

pub fn base_url() -> &'static str {
    &settings().base_url
}
