use std::collections::BTreeMap;
use std::fmt::Write;
use std::str::FromStr;

use rari_templ_func::rari_f;
use rari_types::fm_types::{FeatureStatus, PageType};
use rari_types::locale::Locale;

use crate::docs::page::PageLike;
use crate::error::DocError;
use crate::templ::api::RariApi;
use crate::templ::macros::badges::{write_deprecated, write_experimental, write_non_standard};

/// List sub pages
///
/// Parameters:
///  $0  Base url
///  $1  Title
///  $3  Page types
#[rari_f]
pub fn list_sub_pages(
    url: Option<String>,
    title: Option<String>,
    page_types: Option<String>,
) -> Result<String, DocError> {
    let url = url.as_deref().unwrap_or(env.url);
    let title = title.as_deref().unwrap_or(env.title);
    let mut out = String::new();
    write!(out, "<details><summary>{}</summary><ol>", title)?;
    list_sub_pages_internal(
        &mut out,
        url,
        &env.locale,
        page_types
            .map(|pt| {
                pt.split(',')
                    .filter_map(|pt| PageType::from_str(pt.trim()).ok())
                    .collect::<Vec<_>>()
            })
            .as_deref()
            .unwrap_or_default(),
    )?;
    out.push_str("</ol></details>");

    Ok(out)
}

#[rari_f]
pub fn list_sub_pages_grouped(
    url: Option<String>,
    title: Option<String>,
    page_types: Option<String>,
) -> Result<String, DocError> {
    let url = url.as_deref().unwrap_or(env.url);
    let title = title.as_deref().unwrap_or(env.title);
    let mut out = String::new();
    out.push_str("<details><summary>");
    out.push_str(&html_escape::encode_safe(title));
    out.push_str("</summary><ol>");
    list_sub_pages_grouped_internal(
        &mut out,
        url,
        &env.locale,
        page_types
            .map(|pt| {
                pt.split(',')
                    .filter_map(|pt| PageType::from_str(pt.trim()).ok())
                    .collect::<Vec<_>>()
            })
            .as_deref()
            .unwrap_or_default(),
    )?;
    out.push_str("</ol></details>");
    Ok(out)
}

fn write_li_with_badges(
    out: &mut impl Write,
    page: &impl PageLike,
    locale: &Locale,
) -> std::fmt::Result {
    write!(
        out,
        "<li><a href=\"{}\">{}</a>",
        page.url(),
        html_escape::encode_safe(page.short_title().unwrap_or(page.title()))
    )?;
    if page.status().contains(&FeatureStatus::Experimental) {
        write_experimental(out, locale)?;
    }
    if page.status().contains(&FeatureStatus::NonStandard) {
        write_non_standard(out, locale)?;
    }
    if page.status().contains(&FeatureStatus::Deprecated) {
        write_deprecated(out, locale)?;
    }
    write!(out, "</li>")
}

pub fn list_sub_pages_internal(
    out: &mut impl Write,
    url: &str,
    locale: &Locale,
    page_types: &[PageType],
) -> Result<(), DocError> {
    let sub_pages = RariApi::get_sub_pages(url, None)?;

    for sub_page in sub_pages {
        if !page_types.is_empty() && !page_types.contains(&sub_page.page_type()) {
            continue;
        }
        write_li_with_badges(out, &sub_page, locale)?;
    }
    Ok(())
}

pub fn list_sub_pages_grouped_internal(
    out: &mut String,
    url: &str,
    locale: &Locale,
    page_types: &[PageType],
) -> Result<(), DocError> {
    let sub_pages = RariApi::get_sub_pages(url, None)?;

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
    for (prefix, group) in grouped {
        let keep_group = group.len() > 3;
        if keep_group {
            out.push_str("<li class=\"toggle\"><details><summary>");
            out.push_str(prefix);
            out.push_str("}-*</summary><ol>");
        }
        for sub_page in group {
            write_li_with_badges(out, sub_page, locale)?;
        }
        if keep_group {
            out.push_str("</ol></details></li>");
        }
    }
    Ok(())
}
#[cfg(test)]
mod test {}
