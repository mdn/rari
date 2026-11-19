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
use scraper::Html;
use serde::Serialize;

use crate::error::DocError;
use crate::pages::json::{JsonDoc, Section};
use crate::pages::page::{Page, PageLike};
use crate::popularity::popularity_for;

const MAX_SECTION_TITLE_PAGES: usize = 3;

#[derive(Debug, Serialize)]
pub struct SearchItem {
    pub title: String,
    pub url: String,
}

#[derive(Debug, PartialEq)]
pub struct SectionTitle {
    pub title: String,
    pub id: String,
}

/// Section titles of built pages, keyed by page URL.
pub type SectionTitles = HashMap<String, Vec<SectionTitle>>;

/// Builds the search index for the provided pages.
///
/// Writes a per-locale `search-index.json` via [`build_search_items`]: page entries sorted by
/// aggregated popularity (own page views plus the page views of every URL that redirects to the
/// page), followed by the section entries of pages present in `sections`.
///
/// # Errors
///
/// Returns an error if creating or writing a search index file fails.
pub fn build_search_index(docs: &[Page], sections: &SectionTitles) -> Result<(), DocError> {
    let mut locales = HashSet::new();
    for doc in docs {
        locales.insert(doc.locale());
    }

    for locale in locales {
        let out = build_search_items(docs, locale, sections)?;
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
    locale: Locale,
    sections: &SectionTitles,
) -> Result<Vec<SearchItem>, DocError> {
    let mut index = docs
        .iter()
        .filter(|doc| doc.locale() == locale)
        .map(|doc| {
            let popularity = popularity_for(doc.url()).unwrap_or_default();
            let sections = sections
                .get(doc.url())
                .map(Vec::as_slice)
                .unwrap_or_default();
            (doc, popularity, sections)
        })
        .collect::<Vec<_>>();

    let mut section_pages = HashMap::<String, HashSet<&str>>::new();
    for (doc, _, sections) in &index {
        for section in *sections {
            section_pages
                .entry(section_key(&section.title))
                .or_default()
                .insert(doc.url());
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
                    title: section.title.clone(),
                    url: format!("{}#{}", doc.url(), section.id),
                });
            }
        }
    }

    Ok(out)
}

/// Returns the H2 and H3 titles of a built page with the IDs they have on that page.
pub fn section_titles(doc: &JsonDoc) -> Vec<SectionTitle> {
    doc.body
        .iter()
        .filter_map(|section| {
            let (id, title) = match section {
                Section::Prose(s) => (&s.id, &s.title),
                Section::BrowserCompatibility(s) => (&s.id, &s.title),
                Section::Specifications(s) => (&s.id, &s.title),
            };
            let id = id.clone()?;
            let title = title_text(title.as_deref()?);
            (title.chars().count() > 1).then_some(SectionTitle { title, id })
        })
        .collect()
}

/// Converts a heading's inner HTML to whitespace-normalized plain text.
fn title_text(html: &str) -> String {
    let text = if html.contains(['<', '&']) {
        Html::parse_fragment(html)
            .root_element()
            .text()
            .collect::<String>()
    } else {
        html.to_string()
    };
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn section_key(title: &str) -> String {
    title.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_text_strips_markup() {
        struct Case {
            name: &'static str,
            html: &'static str,
            expected: &'static str,
        }

        let cases = vec![
            Case {
                name: "plain",
                html: "Cache directives",
                expected: "Cache directives",
            },
            Case {
                name: "escaped element name",
                html: "<code>&lt;input&gt;</code> types",
                expected: "<input> types",
            },
            Case {
                name: "status icon",
                html: r#"Using  postMessage <span class="icon icon-nonstandard" title="Non-standard."></span>"#,
                expected: "Using postMessage",
            },
        ];

        for case in cases {
            assert_eq!(title_text(case.html), case.expected, "{}", case.name);
        }
    }
}
