//! # Translations Module
//!
//! The `translations` module provides functionality for managing and accessing the translated titles
//! by slug and locale cache.

use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

use rari_types::globals::cache_content;
use rari_types::locale::Locale;

use crate::cached_readers::{STATIC_DOC_PAGE_FILES, STATIC_DOC_PAGE_TRANSLATED_FILES};
use crate::pages::json::Translation;
use crate::pages::page::{Page, PageLike};
use crate::resolve::strip_locale_from_url;

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

/// Determines all available translations for a specific page.
///
/// # Arguments
///
/// * `doc` - The page for which the translations should be determined.
///
/// # Returns
///
/// * `Vec<Translation>` - The vector of translations (including the current locale).
pub(crate) fn other_translations<T: PageLike>(doc: &T) -> Vec<Translation> {
    get_other_translations_for(doc)
        .into_iter()
        .map(|(locale, title)| Translation {
            native: locale.into(),
            locale,
            title,
        })
        .collect()
}

fn get_other_translations_for<T: PageLike>(doc: &T) -> Vec<(Locale, String)> {
    let slug = doc.slug();
    let locale = doc.locale();

    if cache_content() && slug.contains("/docs/") {
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
    } else {
        let (_, url) = strip_locale_from_url(doc.url());
        Locale::for_generic_and_spas()
            .iter()
            .filter_map(|l| {
                if *l == locale {
                    Some((*l, doc.title().to_string()))
                } else {
                    let other_url = &format!("/{}{}", *l, url);
                    Page::from_url(other_url)
                        .ok()
                        .map(|d| (*l, d.title().to_string()))
                }
            })
            .collect()
    }
}
