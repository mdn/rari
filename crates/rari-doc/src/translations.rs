use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

use rari_types::locale::Locale;

use crate::cached_readers::{STATIC_DOC_PAGE_FILES, STATIC_DOC_PAGE_TRANSLATED_FILES};
use crate::pages::page::PageLike;

pub type TranslationsOf<'a> = BTreeMap<Locale, &'a str>;

pub type AllTranslationsOf<'a> = HashMap<&'a str, TranslationsOf<'a>>;

pub static TRANSLATIONS_BY_SLUG: OnceLock<AllTranslationsOf> = OnceLock::new();

pub fn init_translations_from_static_docs() {
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

pub fn get_translations_for(slug: &str, locale: Locale) -> Vec<(Locale, String)> {
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
