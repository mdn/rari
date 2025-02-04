use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::Write;
use std::path::PathBuf;

use memoize::memoize;
use rari_types::fm_types::PageType;
use rari_types::globals::{cache_content, deny_warnings};
use rari_types::locale::Locale;

use super::l10n::l10n_json_data;
use super::titles::api_page_title;
use crate::error::DocError;
use crate::html::links::{render_internal_link, LinkModifier};
use crate::pages::page::{Page, PageLike, PageReader};
use crate::redirects::resolve_redirect;
use crate::utils::COLLATOR;
use crate::walker::walk_builder;

fn title_sorter(a: &Page, b: &Page) -> Ordering {
    COLLATOR.with(|c| c.compare(a.title(), b.title()))
}

fn title_api_sorter(a: &Page, b: &Page) -> Ordering {
    COLLATOR.with(|c| c.compare(api_page_title(a), api_page_title(b)))
}

fn slug_sorter(a: &Page, b: &Page) -> Ordering {
    COLLATOR.with(|c| c.compare(a.slug(), b.slug()))
}

fn title_natural_sorter(a: &Page, b: &Page) -> Ordering {
    natural_compare_with_floats(a.title(), b.title())
}

fn slug_natural_sorter(a: &Page, b: &Page) -> Ordering {
    natural_compare_with_floats(a.slug(), b.slug())
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SubPagesSorter {
    #[default]
    Title,
    Slug,
    TitleNatural,
    SlugNatural,
    TitleAPI,
}

impl SubPagesSorter {
    pub fn sorter(&self) -> fn(a: &Page, b: &Page) -> Ordering {
        match self {
            SubPagesSorter::Title => title_sorter,
            SubPagesSorter::Slug => slug_sorter,
            SubPagesSorter::TitleNatural => title_natural_sorter,
            SubPagesSorter::SlugNatural => slug_natural_sorter,
            SubPagesSorter::TitleAPI => title_api_sorter,
        }
    }
}

pub fn write_li_with_badges(
    out: &mut String,
    page: &Page,
    locale: Locale,
    code: bool,
    closed: bool,
) -> Result<(), DocError> {
    let locale_page = if locale != Default::default() {
        &Page::from_url_with_locale_and_fallback(page.url(), locale)?
    } else {
        page
    };
    out.push_str("<li>");
    render_internal_link(
        out,
        locale_page.url(),
        None,
        &html_escape::encode_safe(locale_page.short_title().unwrap_or(locale_page.title())),
        None,
        &LinkModifier {
            badges: page.status(),
            badge_locale: locale,
            code,
            only_en_us: locale_page.locale() != locale,
        },
        true,
    )?;
    if closed {
        write!(out, "</li>")?;
    }
    Ok(())
}

pub fn write_parent_li(out: &mut String, page: &Page, locale: Locale) -> Result<(), DocError> {
    let content = l10n_json_data("Template", "overview", locale)?;
    out.push_str("<li>");
    render_internal_link(
        out,
        page.url(),
        None,
        content,
        None,
        &LinkModifier {
            badges: page.status(),
            badge_locale: locale,
            code: false,
            only_en_us: page.locale() != locale,
        },
        true,
    )?;
    out.push_str("</li>");
    Ok(())
}

pub fn list_sub_pages_reverse_internal(
    out: &mut String,
    url: &str,
    locale: Locale,
    sorter: Option<SubPagesSorter>,
    page_types: &[PageType],
    code: bool,
) -> Result<(), DocError> {
    let sub_pages = get_sub_pages(url, Some(1), sorter.unwrap_or_default())?;

    for sub_page in sub_pages.iter().rev() {
        if !page_types.is_empty() && !page_types.contains(&sub_page.page_type()) {
            continue;
        }
        write_li_with_badges(out, sub_page, locale, code, true)?;
    }
    Ok(())
}

pub struct ListSubPagesContext<'a> {
    pub sorter: Option<SubPagesSorter>,
    pub page_types: &'a [PageType],
    pub code: bool,
    pub include_parent: bool,
}

pub fn list_sub_pages_flattened_internal(
    out: &mut String,
    url: &str,
    locale: Locale,
    depth: Option<usize>,
    ListSubPagesContext {
        sorter,
        page_types,
        code,
        include_parent,
    }: ListSubPagesContext<'_>,
) -> Result<(), DocError> {
    let sub_pages = get_sub_pages(url, depth, sorter.unwrap_or_default())?;
    if include_parent {
        let page = Page::from_url_with_locale_and_fallback(url, locale)?;
        write_parent_li(out, &page, locale)?;
    }
    for sub_page in sub_pages {
        if !page_types.is_empty() && !page_types.contains(&sub_page.page_type()) {
            continue;
        }
        write_li_with_badges(out, &sub_page, locale, code, true)?;
    }
    Ok(())
}
pub fn list_sub_pages_nested_internal(
    out: &mut String,
    url: &str,
    locale: Locale,
    depth: Option<usize>,
    ListSubPagesContext {
        sorter,
        page_types,
        code,
        include_parent,
    }: ListSubPagesContext<'_>,
) -> Result<(), DocError> {
    if depth == Some(0) {
        return Ok(());
    }
    let sub_pages = get_sub_pages(url, Some(1), sorter.unwrap_or_default())?;
    let depth = depth.map(|i| i.saturating_sub(1));
    if include_parent {
        let page = Page::from_url_with_locale_and_fallback(url, locale)?;
        write_parent_li(out, &page, locale)?;
    }
    for sub_page in sub_pages {
        let page_type_match = page_types.is_empty() || page_types.contains(&sub_page.page_type());
        let sub_sub_pages = get_sub_pages(sub_page.url(), depth, sorter.unwrap_or_default())?;
        if sub_sub_pages.is_empty() {
            if page_type_match {
                write_li_with_badges(out, &sub_page, locale, code, true)?;
            }
        } else {
            if page_type_match {
                write_li_with_badges(out, &sub_page, locale, code, false)?;
            }
            let mut sub_pages_out = String::new();

            list_sub_pages_nested_internal(
                &mut sub_pages_out,
                sub_page.url(),
                locale,
                depth,
                ListSubPagesContext {
                    sorter,
                    page_types,
                    code,
                    include_parent,
                },
            )?;
            if !sub_pages_out.is_empty() {
                out.push_str("<ol>");
                out.push_str(&sub_pages_out);
                out.push_str("</ol>");
            }
            if page_type_match {
                out.push_str("</li>");
            }
        }
    }
    Ok(())
}

pub fn list_sub_pages_flattened_grouped_internal(
    out: &mut String,
    url: &str,
    locale: Locale,
    depth: Option<usize>,
    ListSubPagesContext {
        sorter,
        page_types,
        code,
        include_parent,
    }: ListSubPagesContext<'_>,
) -> Result<(), DocError> {
    let sub_pages = get_sub_pages(url, depth, sorter.unwrap_or_default())?;

    let mut grouped = BTreeMap::new();
    for sub_page in sub_pages.iter() {
        if !page_types.is_empty() && !page_types.contains(&sub_page.page_type()) {
            continue;
        }
        let title = sub_page.title();
        let prefix_index = if !title.is_empty() {
            title[1..].find('-').map(|i| i + 1)
        } else {
            None
        };
        if let Some(prefix) = prefix_index.map(|i| &title[..i]) {
            grouped
                .entry(prefix)
                .and_modify(|l: &mut Vec<_>| l.push(sub_page))
                .or_insert(vec![sub_page]);
        } else {
            grouped.insert(sub_page.title(), vec![sub_page]);
        }
    }
    if include_parent {
        let page = Page::from_url_with_locale_and_fallback(url, locale)?;
        write_parent_li(out, &page, locale)?;
    }
    for (prefix, group) in grouped {
        let keep_group = group.len() > 2;
        if keep_group {
            out.push_str("<li class=\"toggle\"><details><summary>");
            out.push_str(prefix);
            out.push_str("-*</summary><ol>");
        }
        for sub_page in group {
            write_li_with_badges(out, sub_page, locale, code, true)?;
        }
        if keep_group {
            out.push_str("</ol></details></li>");
        }
    }
    Ok(())
}

pub fn get_sub_pages(
    url: &str,
    depth: Option<usize>,
    sorter: SubPagesSorter,
) -> Result<Vec<Page>, DocError> {
    let redirect = resolve_redirect(url);
    let url = match redirect.as_ref() {
        Some(redirect) if deny_warnings() => {
            return Err(DocError::RedirectedLink {
                from: url.to_string(),
                to: redirect.to_string(),
            })
        }
        Some(redirect) => redirect,
        None => url,
    };
    let doc = Page::from_url_with_fallback(url)?;
    let full_path = doc.full_path();
    if let Some(folder) = full_path.parent() {
        let sub_folders = read_sub_folders(folder.to_path_buf(), depth)?;

        let mut sub_pages = sub_folders
            .iter()
            .filter(|f| f.as_path() != full_path)
            .map(|p| Page::read(p, Some(doc.locale())))
            .collect::<Result<Vec<_>, DocError>>()?;
        sub_pages.sort_by(sorter.sorter());
        return Ok(sub_pages);
    }
    Ok(vec![])
}

fn read_sub_folders(folder: PathBuf, depth: Option<usize>) -> Result<Vec<PathBuf>, ignore::Error> {
    if cache_content() {
        read_sub_folders_internal(folder, depth)
    } else {
        memoized_original_read_sub_folders_internal(folder, depth)
    }
}

#[memoize(SharedCache)]
#[allow(non_snake_case)]
fn read_sub_folders_internal(
    folder: PathBuf,
    depth: Option<usize>,
) -> Result<Vec<PathBuf>, ignore::Error> {
    Ok(walk_builder(&[folder], None)?
        .max_depth(depth.map(|i| i + 1))
        .build()
        .filter_map(|f| f.ok())
        .filter(|f| f.file_type().map(|ft| ft.is_file()).unwrap_or(false))
        .map(|f| f.into_path())
        .collect())
}

fn split_into_parts(s: &str) -> Vec<(bool, &str)> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut end = 0;
    let mut in_number = false;

    for (i, c) in s.char_indices() {
        if c.is_ascii_digit() || c == '.' {
            if !in_number {
                if start != end {
                    parts.push((false, &s[start..end]));
                    start = end
                }
                in_number = true;
            }
        } else if in_number {
            if start != end {
                parts.push((true, &s[start..end]));
                start = end
            }
            in_number = false;
        }
        end = i + c.len_utf8()
    }

    if start != end {
        parts.push((in_number, &s[start..end]));
    }

    parts
}

fn natural_compare_with_floats(a: &str, b: &str) -> Ordering {
    let parts_a = split_into_parts(a);
    let parts_b = split_into_parts(b);

    for (part_a, part_b) in parts_a.iter().zip(parts_b.iter()) {
        let order = if part_a.0 && part_b.0 {
            let num_a: f64 = part_a.1.parse().unwrap_or(f64::NEG_INFINITY);
            let num_b: f64 = part_b.1.parse().unwrap_or(f64::INFINITY);
            num_a.partial_cmp(&num_b).unwrap()
        } else {
            part_a.1.cmp(part_b.1)
        };
        if order != Ordering::Equal {
            return order;
        }
    }
    parts_a.len().cmp(&parts_b.len())
}
