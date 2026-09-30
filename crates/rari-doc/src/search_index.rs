//! # Search Index Module
//!
//! Builds a locale-specific search index from page and section titles.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufWriter;

use rari_types::Popularities;
use rari_types::globals::{self, build_out_root};
use rari_types::locale::Locale;
use rari_utils::error::RariIoError;
use rari_utils::io::read_to_string;
use scraper::{Html, Selector};
use serde::Serialize;

use crate::error::DocError;
use crate::html::modifier::add_missing_ids;
use crate::pages::page::{Page, PageLike};

const MAX_SECTION_TITLE_PAGES: usize = 3;

#[derive(Debug, Serialize)]
pub struct SearchItem {
    pub title: String,
    pub url: String,
}

#[derive(Debug)]
struct SectionTitle {
    title: String,
    url: String,
}

/// Builds the search index for the provided pages.
pub fn build_search_index(docs: &[Page]) -> Result<(), DocError> {
    let in_file = globals::data_dir()
        .join("popularities")
        .join("popularities.json");
    let json_str = read_to_string(in_file)?;
    let popularities: Popularities = serde_json::from_str(&json_str)?;

    let mut locales = HashSet::new();
    for doc in docs {
        locales.insert(doc.locale());
    }

    for locale in locales {
        let out = build_search_items(docs, &popularities, locale)?;
        if out.is_empty() {
            continue;
        }

        let out_file = build_out_root()?
            .join(locale.as_folder_str())
            .join("search-index.json");
        let file = File::create(&out_file).map_err(|e| RariIoError {
            source: e,
            path: out_file,
        })?;
        serde_json::to_writer(BufWriter::new(file), &out)?;
    }
    Ok(())
}

/// Builds search items for one locale. Section titles occurring on more than three distinct pages
/// are omitted to keep repeated boilerplate out of the index.
pub fn build_search_items(
    docs: &[Page],
    popularities: &Popularities,
    locale: Locale,
) -> Result<Vec<SearchItem>, DocError> {
    let mut index = docs
        .iter()
        .filter(|doc| doc.locale() == locale)
        .map(|doc| {
            let sections = section_titles(doc)?;
            let popularity = popularities
                .popularities
                .get(doc.url())
                .cloned()
                .unwrap_or_default();
            Ok((doc, popularity, sections))
        })
        .collect::<Result<Vec<_>, DocError>>()?;

    let mut section_pages = HashMap::<String, HashSet<String>>::new();
    for (doc, _, sections) in &index {
        for section in sections {
            section_pages
                .entry(section_key(&section.title))
                .or_default()
                .insert(doc.url().to_string());
        }
    }

    index.sort_by(|(a, a_popularity, _), (b, b_popularity, _)| {
        match b_popularity.partial_cmp(a_popularity) {
            None | Some(Ordering::Equal) => a.title().cmp(b.title()),
            Some(ordering) => ordering,
        }
    });

    let mut out = index
        .iter()
        .map(|(doc, _, _)| SearchItem {
            title: doc.title().to_string(),
            url: doc.url().to_string(),
        })
        .collect::<Vec<_>>();

    for (doc, _, sections) in index {
        for section in sections {
            if section_pages[&section_key(&section.title)].len() <= MAX_SECTION_TITLE_PAGES
                && !section.title.eq_ignore_ascii_case(doc.title())
            {
                out.push(SearchItem {
                    title: section.title,
                    url: section.url,
                });
            }
        }
    }

    Ok(out)
}

fn section_titles(page: &Page) -> Result<Vec<SectionTitle>, DocError> {
    if !matches!(page, Page::Doc(_)) {
        return Ok(Vec::new());
    }

    let rendered = page.render()?;
    let mut html = Html::parse_fragment(&rendered);
    add_missing_ids(&mut html)?;
    let selector = Selector::parse("h2[id], h3[id]").unwrap();
    let mut sections = Vec::new();
    for heading in html.select(&selector) {
        let title = heading.text().collect::<String>().trim().to_string();
        if title.is_empty() || title.chars().count() == 1 || title.contains("{{") {
            continue;
        }
        let Some(id) = heading.attr("id") else {
            continue;
        };
        sections.push(SectionTitle {
            title,
            url: format!("{}#{id}", page.url()),
        });
    }
    Ok(sections)
}

fn section_key(title: &str) -> String {
    title
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}
