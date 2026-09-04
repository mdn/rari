//! # Search Index Module
//!
//! The `search_index` module provides functionality for building and managing the search index
//! for documentation pages. It takes popularity datainto account when generating the search index
//! files for different locales.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;

use rari_types::globals::build_out_root;
use rari_types::locale::Locale;
use rari_utils::error::RariIoError;
use serde::Serialize;

use crate::error::DocError;
use crate::pages::page::{Page, PageLike};
use crate::redirects::popularity_for;

#[derive(Debug, Serialize)]
struct SearchItem<'a> {
    title: &'a str,
    url: &'a str,
}

/// Builds the search index for the provided pages.
///
/// Sorts pages by aggregated popularity (own page views plus the page views of
/// every URL that redirects to the page) and writes a per-locale
/// `search-index.json`.
///
/// # Errors
///
/// Returns an error if creating or writing a search index file fails.
pub fn build_search_index(docs: &[Page]) -> Result<(), DocError> {
    let mut all_indices: HashMap<Locale, Vec<(&Page, f64)>> = HashMap::new();

    for doc in docs {
        let entry = all_indices.entry(doc.locale()).or_default();
        entry.push((doc, popularity_for(doc.url()).unwrap_or_default()));
    }

    for (locale, mut index) in all_indices.into_iter() {
        if !index.is_empty() {
            index.sort_by(|(da, a), (db, b)| match b.partial_cmp(a) {
                None | Some(Ordering::Equal) => da.title().cmp(db.title()),
                Some(ord) => ord,
            });
            let out = index
                .into_iter()
                .map(|(doc, _)| SearchItem {
                    title: doc.title(),
                    url: doc.url(),
                })
                .collect::<Vec<_>>();
            let out_file = build_out_root()?
                .join(locale.as_folder_str())
                .join("search-index.json");
            let file = File::create(&out_file).map_err(|e| RariIoError {
                source: e,
                path: out_file,
            })?;
            let buffed = BufWriter::new(file);

            serde_json::to_writer(buffed, &out)?;
        }
    }
    Ok(())
}
