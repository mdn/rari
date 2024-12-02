//! # Search Index Module
//!
//! The `search_index` module provides functionality for building and managing the search index
//! for documentation pages. It takes popularity datainto account when generating the search index
//! files for different locales.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;

use rari_types::globals::{self, build_out_root};
use rari_types::locale::Locale;
use rari_types::Popularities;
use rari_utils::error::RariIoError;
use rari_utils::io::read_to_string;
use serde::Serialize;

use crate::error::DocError;
use crate::pages::page::{Page, PageLike};

#[derive(Debug, Serialize)]
struct SearchItem<'a> {
    title: &'a str,
    url: &'a str,
}

/// Builds the search index for the provided pages.
///
/// This function reads popularity data from a JSON file, sorts the documentation pages based on their popularity,
/// and generates search index files for different locales. The search index files are written to the output directory
/// and contain the title and URL of each documentation page.
///
/// # Arguments
///
/// * `docs` - A slice of `Page` objects representing the documentation pages to be indexed.
///
/// # Returns
///
/// * `Result<(), DocError>` - Returns `Ok(())` if the search index is built successfully,
///   or a `DocError` if an error occurs during the process.
///
/// # Errors
///
/// This function will return an error if:
/// - The popularity data file cannot be read.
/// - The popularity data cannot be parsed.
/// - An error occurs while creating or writing to the search index files.
pub fn build_search_index(docs: &[Page]) -> Result<(), DocError> {
    let in_file = globals::data_dir()
        .join("popularities")
        .join("popularities.json");
    let json_str = read_to_string(in_file)?;
    let popularities: Popularities = serde_json::from_str(&json_str)?;

    let mut all_indices: HashMap<Locale, Vec<(&Page, f64)>> = HashMap::new();

    for doc in docs {
        let entry = all_indices.entry(doc.locale()).or_default();
        entry.push((
            doc,
            popularities
                .popularities
                .get(doc.url())
                .cloned()
                .unwrap_or_default(),
        ));
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
