use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fs};

use once_cell::sync::{Lazy, OnceCell};
use serde::Deserialize;

use crate::error::EnvError;
use crate::settings::Settings;
use crate::{HistoryEntry, Popularities};

#[inline(always)]
pub fn content_root() -> &'static Path {
    settings().content_root.as_path()
}

#[inline(always)]
pub fn blog_root() -> Option<&'static Path> {
    settings().blog_root.as_deref()
}

#[inline(always)]
pub fn curriculum_root() -> Option<&'static Path> {
    settings().curriculum_root.as_deref()
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

pub static DATA_DIR: OnceCell<PathBuf> = OnceCell::new();

pub fn data_dir() -> &'static Path {
    DATA_DIR.get_or_init(|| {
        dirs::data_local_dir()
            .map(|p| p.join("rari"))
            .unwrap_or_default()
    })
}

pub static SETTINGS: OnceCell<Settings> = OnceCell::new();

pub fn settings() -> &'static Settings {
    SETTINGS.get_or_init(|| Settings::new().expect("error generating settings"))
}

#[derive(Debug, Deserialize)]
pub struct JsonSpecData {
    pub url: String,
}

pub type JsonSpecDataLookup = HashMap<String, String>;

pub static JSON_SPEC_DATA_FILE: OnceCell<JsonSpecDataLookup> = OnceCell::new();

pub fn json_spec_data_lookup() -> &'static JsonSpecDataLookup {
    JSON_SPEC_DATA_FILE.get_or_init(|| {
        let json_str = fs::read_to_string(content_root().join("jsondata/SpecData.json"))
            .expect("unable to read SpecData.json");
        let data: HashMap<String, JsonSpecData> =
            serde_json::from_str(&json_str).expect("unabeld to parse SpecData.json");
        data.into_iter().map(|(k, v)| (v.url, k)).collect()
    })
}

pub type JsonL10nFile = HashMap<String, HashMap<String, String>>;

pub static JSON_L10N_FILES: OnceCell<HashMap<String, JsonL10nFile>> = OnceCell::new();

pub fn json_l10n_files() -> &'static HashMap<String, JsonL10nFile> {
    JSON_L10N_FILES.get_or_init(|| {
        content_root()
            .join("jsondata")
            .read_dir()
            .expect("unable to read jsondata dir")
            .filter_map(|f| {
                if let Ok(f) = f {
                    if f.path().is_file()
                        && f.path()
                            .extension()
                            .map_or(false, |ext| ext.eq_ignore_ascii_case("json"))
                        && f.path()
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .map_or(false, |s| s.starts_with("L10n-"))
                    {
                        return Some(f.path());
                    }
                }
                None
            })
            .map(|f| {
                let typ = f
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .strip_prefix("L10n-")
                    .unwrap_or_default();
                let json_str = fs::read_to_string(&f).expect("unable to read l10n json");
                let l10n_json: JsonL10nFile =
                    serde_json::from_str(&json_str).expect("unable to parse l10n json");
                (typ.into(), l10n_json)
            })
            .collect()
    })
}
pub static GIT_HISTORY: Lazy<HashMap<PathBuf, HistoryEntry>> = Lazy::new(|| {
    let f = content_root().join("en-US").join("_history.json");
    if let Ok(json_str) = fs::read_to_string(f) {
        serde_json::from_str(&json_str).expect("unable to parse l10n json")
    } else {
        HashMap::new()
    }
});
pub fn git_history() -> &'static HashMap<PathBuf, HistoryEntry> {
    &GIT_HISTORY
}

pub static POPULARITIES: Lazy<Popularities> = Lazy::new(|| {
    let f = content_root().join("en-US").join("popularities.json");
    if let Ok(json_str) = fs::read_to_string(f) {
        serde_json::from_str(&json_str).expect("unable to parse l10n json")
    } else {
        Popularities::default()
    }
});
pub fn popularities() -> &'static Popularities {
    &POPULARITIES
}

pub static CONTENT_BRANCH: OnceCell<String> = OnceCell::new();
pub fn content_branch() -> &'static str {
    CONTENT_BRANCH.get_or_init(|| env::var("CONTENT_BRANCH").unwrap_or("main".to_string()))
}

pub fn base_url() -> &'static str {
    &settings().base_url
}
