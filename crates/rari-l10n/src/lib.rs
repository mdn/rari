use std::collections::HashMap;

use rari_types::globals::json_l10n_files;
use rari_types::locale::Locale;
use thiserror::Error;

#[derive(Debug, PartialEq, Clone, Copy, Error)]
pub enum L10nError {
    #[error("Invalid key for L10n json data")]
    InvalidKey,
    #[error("EnUS missing in L10n json data")]
    NoEnUs,
}

pub fn l10n_json_data(typ: &str, key: &str, locale: Locale) -> Result<&'static str, L10nError> {
    if let Some(data) = json_l10n_files().get(typ).and_then(|file| file.get(key)) {
        get_for_locale(locale, data)
            .map(|s| s.as_str())
            .ok_or(L10nError::NoEnUs)
    } else {
        Err(L10nError::InvalidKey)
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
