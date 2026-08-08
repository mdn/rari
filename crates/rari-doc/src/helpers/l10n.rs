use std::collections::HashMap;
use std::fs;
use std::sync::LazyLock;

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

// Look up a translation from mdn/content's `jsondata folder.
// `typ` refers to the `L10n-<typ>.json` file.
pub fn l10n_json_data(typ: &str, key: &str, locale: Locale) -> Result<&'static str, L10nError> {
    if let Some(data) = JSON_L10N_FILES.get(typ).and_then(|file| file.get(key)) {
        // get_for_locale(locale, data)
        if let Some(value) = data.get(locale.as_url_str()) {
            Some(value)
        } else if locale != Locale::default() {
            tracing::warn!(
                "Localized value is missing in content/files/jsondata/L10n-{}.json: {} (locale: {})",
                typ,
                key,
                locale.as_url_str()
            );
            data.get(Locale::default().as_url_str())
        } else {
            None
        }
        .map(|s| s.as_str())
        .ok_or(L10nError::NoEnUs)
    } else {
        Err(L10nError::InvalidKey(key.to_string()))
    }
}

pub type JsonL10nFile = HashMap<String, HashMap<String, String>>;

static JSON_L10N_FILES: LazyLock<HashMap<String, JsonL10nFile>> = LazyLock::new(|| {
    content_root()
        .join("jsondata")
        .read_dir()
        .expect("unable to read jsondata dir")
        .filter_map(|f| {
            if let Ok(f) = f
                && f.path().is_file()
                && f.path()
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
                && f.path()
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .is_some_and(|s| s.starts_with("L10n-"))
            {
                return Some(f.path());
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
});
