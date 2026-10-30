//! # Search Index Module
//!
//! Builds a locale-specific search index from page and section titles.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufWriter;

use rari_types::globals::build_out_root;
use rari_types::locale::Locale;
use rari_utils::error::RariIoError;
use scraper::{Html, Selector};
use serde::Serialize;
use tracing::warn;

use crate::error::DocError;
use crate::html::modifier::add_missing_ids;
use crate::pages::page::{Page, PageLike};
use crate::popularity::popularity_for;

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
///
/// Writes a per-locale `search-index.json` via [`build_search_items`]: page entries sorted by
/// aggregated popularity (own page views plus the page views of every URL that redirects to the
/// page), each followed by its section entries.
///
/// # Errors
///
/// Returns an error if creating or writing a search index file fails.
pub fn build_search_index(docs: &[Page]) -> Result<(), DocError> {
    let mut locales = HashSet::new();
    for doc in docs {
        locales.insert(doc.locale());
    }

    for locale in locales {
        let out = build_search_items(docs, locale)?;
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
pub fn build_search_items(docs: &[Page], locale: Locale) -> Result<Vec<SearchItem>, DocError> {
    let mut index = docs
        .iter()
        .filter(|doc| doc.locale() == locale)
        .map(|doc| {
            let popularity = popularity_for(doc.url()).unwrap_or_default();
            (doc, popularity, section_titles(doc))
        })
        .collect::<Vec<_>>();

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

/// Returns the H2 and H3 titles of a page with their anchor URLs, skipping repeated IDs.
fn section_titles(page: &Page) -> Vec<SectionTitle> {
    let Page::Doc(doc) = page else {
        return Vec::new();
    };

    let sections = match rari_md::extract_headings(doc.content()) {
        Some(headings) => headings
            .into_iter()
            .filter(|(title, _)| is_indexable_title(title))
            .map(|(title, id)| SectionTitle {
                title: title.trim().to_string(),
                url: format!("{}#{id}", page.url()),
            })
            .collect(),
        None => section_titles_from_rendered_html(page).unwrap_or_else(|e| {
            warn!("no section titles for {}: {e}", page.url());
            Vec::new()
        }),
    };

    let mut seen = HashSet::new();
    sections
        .into_iter()
        .filter(|section| seen.insert(section.url.clone()))
        .collect()
}

fn is_indexable_title(title: &str) -> bool {
    let title = title.trim();
    title.chars().count() > 1 && !title.contains("{{")
}

fn section_titles_from_rendered_html(page: &Page) -> Result<Vec<SectionTitle>, DocError> {
    let rendered = page.render()?;
    let mut html = Html::parse_fragment(&rendered);
    add_missing_ids(&mut html)?;
    let selector = Selector::parse("h2[id], h3[id]").unwrap();
    let mut sections = Vec::new();
    for heading in html.select(&selector) {
        let title = heading.text().collect::<String>();
        if !is_indexable_title(&title) {
            continue;
        }
        let Some(id) = heading.attr("id") else {
            continue;
        };
        sections.push(SectionTitle {
            title: title.trim().to_string(),
            url: format!("{}#{}", page.url(), id.to_lowercase()),
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
