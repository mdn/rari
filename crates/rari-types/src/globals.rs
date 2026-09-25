use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{LazyLock, OnceLock};
use std::{env, fs};

use serde::Deserialize;

use crate::error::EnvError;
use crate::locale::Locale;
use crate::settings::{Deps, Settings};
use crate::{HistoryEntry, Popularities, globals};

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

pub fn translated_content_root_for_locale(locale: Locale) -> Option<&'static Path> {
    translated_content_root_for_locale_in(settings(), locale)
}

fn translated_content_root_for_locale_in(settings: &Settings, locale: Locale) -> Option<&Path> {
    settings
        .translated_content_sources
        .get(&locale)
        .map(|source| source.root.as_path())
        .or(settings.content_translated_root.as_deref())
}

pub fn translated_content_repository(locale: Locale) -> &'static str {
    translated_content_repository_in(settings(), locale)
}

fn translated_content_repository_in(settings: &Settings, locale: Locale) -> &str {
    settings
        .translated_content_sources
        .get(&locale)
        .map(|source| source.repository.as_str())
        .unwrap_or(if locale == Locale::De {
            "translated-content-de"
        } else {
            "translated-content"
        })
}

#[cfg(test)]
mod translated_content_tests {
    use super::*;
    use crate::settings::TranslatedContentSource;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn separate_locale_root_overrides_primary_for_discovery_and_selection() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let fixture = std::env::temp_dir().join(format!("rari-translated-sources-{suffix}"));
        let primary = fixture.join("primary");
        let dedicated = fixture.join("dedicated");
        for path in [primary.join("fr"), primary.join("de"), dedicated.join("de")] {
            fs::create_dir_all(path).unwrap();
        }
        let mut settings = Settings {
            content_translated_root: Some(primary.clone()),
            ..Settings::default()
        };
        settings.translated_content_sources.insert(
            Locale::De,
            TranslatedContentSource {
                root: dedicated.clone(),
                repository: "translated-content-de".into(),
            },
        );
        let all = translated_content_locale_paths_in(&settings, None);
        assert_eq!(all, vec![dedicated.join("de"), primary.join("fr")]);
        assert_eq!(
            translated_content_locale_paths_in(&settings, Some(&[Locale::De])),
            vec![dedicated.join("de")]
        );
        assert_eq!(
            translated_content_root_for_locale_in(&settings, Locale::Fr),
            Some(primary.as_path())
        );
        assert_eq!(
            translated_content_repository_in(&settings, Locale::De),
            "translated-content-de"
        );
        fs::remove_dir_all(fixture).unwrap();
    }
}

pub fn translated_content_repository_for_root(root: &Path) -> &'static str {
    if content_translated_root() == Some(root) {
        return "translated-content";
    }
    settings()
        .translated_content_sources
        .values()
        .find(|source| source.root == root)
        .map(|source| source.repository.as_str())
        .unwrap_or("translated-content")
}

pub fn translated_content_roots() -> Vec<&'static Path> {
    let mut roots = Vec::new();
    if let Some(root) = content_translated_root() {
        roots.push(root);
    }
    for source in settings().translated_content_sources.values() {
        if !roots.contains(&source.root.as_path()) {
            roots.push(&source.root);
        }
    }
    roots
}

pub fn translated_content_locale_paths(locales: Option<&[Locale]>) -> Vec<PathBuf> {
    translated_content_locale_paths_in(settings(), locales)
}

fn translated_content_locale_paths_in(
    settings: &Settings,
    locales: Option<&[Locale]>,
) -> Vec<PathBuf> {
    let mut selected = Vec::new();
    let mut other_paths = Vec::new();
    if let Some(locales) = locales {
        selected.extend(
            locales
                .iter()
                .copied()
                .filter(|locale| *locale != Locale::EnUs),
        );
    } else {
        if let Some(root) = settings.content_translated_root.as_deref()
            && let Ok(entries) = root.read_dir()
        {
            for entry in entries
                .filter_map(Result::ok)
                .filter(|entry| entry.path().is_dir())
            {
                match entry
                    .file_name()
                    .to_str()
                    .and_then(|name| Locale::from_str(name).ok())
                {
                    Some(locale) if locale != Locale::EnUs => selected.push(locale),
                    Some(_) => {}
                    None => other_paths.push(entry.path()),
                }
            }
        }
        selected.extend(settings.translated_content_sources.keys().copied());
    }
    selected.sort_unstable();
    selected.dedup();
    other_paths.extend(selected.into_iter().filter_map(|locale| {
        translated_content_root_for_locale_in(settings, locale)
            .map(|root| root.join(locale.as_folder_str()))
    }));
    other_paths
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
    DEPS.get_or_init(Deps::new)
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
    for translated_root in translated_content_roots() {
        let f = translated_root.join("_git_history.json");
        if let Ok(json_str) = fs::read_to_string(f) {
            let mut translated: HashMap<PathBuf, HistoryEntry> =
                serde_json::from_str(&json_str).expect("unable to parse l10n json");
            translated.retain(|path, _| {
                path.components()
                    .next()
                    .and_then(|part| part.as_os_str().to_str())
                    .and_then(|name| Locale::from_str(name).ok())
                    .is_some_and(|locale| {
                        locale != Locale::EnUs
                            && translated_content_root_for_locale(locale) == Some(translated_root)
                    })
            });
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
