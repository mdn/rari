use std::collections::HashMap;
use std::fs;
use std::sync::OnceLock;

use rari_types::globals::content_root;
use rari_types::locale::Locale;
use thiserror::Error;

#[derive(Debug, PartialEq, Clone, Error)]
pub enum L10nError {
    #[error("Invalid key for L10n json data: {0}")]
    InvalidKey(String),
    #[error("EnUS missing in L10n json data")]
    NoEnUs,
}

pub fn l10n_json_data(typ: &str, key: &str, locale: Locale) -> Result<&'static str, L10nError> {
    if let Some(data) = json_l10n_files().get(typ).and_then(|file| file.get(key)) {
        get_for_locale(locale, data)
            .map(|s| s.as_str())
            .ok_or(L10nError::NoEnUs)
    } else {
        Err(L10nError::InvalidKey(key.to_string()))
    }
}

pub fn get_for_locale<T>(locale: Locale, lookup: &HashMap<String, T>) -> Option<&T> {
    if let Some(value) = lookup.get(locale.as_url_str()) {
        Some(value)
    } else if locale != Locale::default() {
        lookup.get(Locale::default().as_url_str())
    } else {
        None
    }
}
pub type JsonL10nFile = HashMap<String, HashMap<String, String>>;

pub static JSON_L10N_FILES: OnceLock<HashMap<String, JsonL10nFile>> = OnceLock::new();

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
