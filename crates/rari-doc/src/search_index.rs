use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;

use rari_types::globals::{build_out_root, content_root};
use rari_types::locale::Locale;
use rari_types::Popularities;
use rari_utils::io::read_to_string;
use serde::Serialize;

use crate::error::DocError;
use crate::pages::page::{Page, PageLike};

#[derive(Debug, Serialize)]
pub struct SearchItem<'a> {
    title: &'a str,
    url: &'a str,
}

pub fn build_search_index(docs: &[Page]) -> Result<(), DocError> {
    let in_file = content_root().join("en-US").join("popularities.json");
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
            let file = File::create(out_file)?;
            let buffed = BufWriter::new(file);

            serde_json::to_writer(buffed, &out)?;
        }
    }
    Ok(())
}
