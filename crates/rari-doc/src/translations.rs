//! # Translations Module
//!
//! The `translations` module provides functionality for managing and accessing the translated titles
//! by slug and locale cache.

use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

use rari_types::locale::Locale;

use crate::cached_readers::{STATIC_DOC_PAGE_FILES, STATIC_DOC_PAGE_TRANSLATED_FILES};
use crate::pages::page::PageLike;

pub type TranslationsOf<'a> = BTreeMap<Locale, &'a str>;

pub type AllTranslationsOf<'a> = HashMap<&'a str, TranslationsOf<'a>>;

pub static TRANSLATIONS_BY_SLUG: OnceLock<AllTranslationsOf> = OnceLock::new();

/// Initializes translated page titles from documentation pages and caches them.
///
/// This function reads documentation pages from the static caches (`STATIC_DOC_PAGE_FILES` and
/// `STATIC_DOC_PAGE_TRANSLATED_FILES`), extracts the translations (locale and title) for each page, and stores
/// them in a global cache (`TRANSLATIONS_BY_SLUG`). The translations are indexed by the page slug and locale,
///  allowing for efficient retrieval of translation titles for specific slugs.
///
/// # Panics
///
/// This function will panic if the `TRANSLATIONS_BY_SLUG` global cache has already been set.
pub(crate) fn init_translations_from_static_docs() {
    let mut all = HashMap::new();

    for cache in [&STATIC_DOC_PAGE_FILES, &STATIC_DOC_PAGE_TRANSLATED_FILES] {
        if let Some(static_pages) = cache.get() {
            for page in static_pages.values() {
                let entry: &mut TranslationsOf<'static> = all.entry(page.slug()).or_default();
                entry.insert(page.locale(), page.title());
            }
        };
    }

    TRANSLATIONS_BY_SLUG.set(all).unwrap();
}

/// Retrieves translations for a specific slug, _excluding_ the specified locale.
///
/// This function looks up translations for the given slug in the global `TRANSLATIONS_BY_SLUG` cache.
/// It filters out the translation for the specified locale and returns a vector of tuples containing
/// the locale and the corresponding title for each translation.
///
/// # Arguments
///
/// * `slug` - A string slice that holds the slug of the documentation page.
/// * `locale` - A `Locale` that specifies the locale to be excluded from the results.
///
/// # Returns
///
/// * `Vec<(Locale, String)>` - Returns a vector of tuples, where each tuple contains a `Locale` and a `String`
///   representing the title of the translation. If no translations are found, an empty vector is returned.
pub(crate) fn get_other_translations_for(slug: &str, locale: Locale) -> Vec<(Locale, String)> {
    TRANSLATIONS_BY_SLUG
        .get()
        .and_then(|by_slug| {
            by_slug.get(slug).map(|translations| {
                translations
                    .iter()
                    .filter_map(|(t_locale, title)| {
                        if *t_locale != locale {
                            Some((*t_locale, title.to_string()))
                        } else {
                            None
                        }
                    })
                    .collect()
            })
        })
        .unwrap_or_default()
}
