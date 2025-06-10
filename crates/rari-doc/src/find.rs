use rari_types::locale::Locale;

use crate::cached_readers::{STATIC_DOC_PAGE_FILES, STATIC_DOC_PAGE_TRANSLATED_FILES};
use crate::error::DocError;
use crate::pages::page::Page;

pub fn doc_pages_from_slugish(slugish: &str, locale: Locale) -> Result<Vec<Page>, DocError> {
    let cache = if locale == Locale::EnUs {
        &STATIC_DOC_PAGE_FILES
    } else {
        &STATIC_DOC_PAGE_TRANSLATED_FILES
    };
    cache.get().map_or_else(
        || Err(DocError::FileCacheBroken),
        |static_files| {
            Ok(static_files
                .iter()
                .filter_map(|((l, s), v)| {
                    if locale == *l && s.contains(slugish) {
                        Some(v)
                    } else {
                        None
                    }
                })
                .take(100)
                .cloned()
                .collect())
        },
    )
}
