use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};

use itertools::Itertools;
use rari_templ_func::rari_f;
use rari_types::fm_types::{FeatureStatus, PageType};
use rari_utils::concat_strs;

use crate::error::DocError;
use crate::helpers::subpages::get_sub_pages;
use crate::pages::page::PageLike;
use crate::templ::api::RariApi;

#[rari_f]
pub fn css_ref_list() -> Result<String, DocError> {
    let mut index = BTreeMap::<char, HashMap<&str, &str>>::new();

    let css_pages = get_sub_pages("/en-US/docs/Web/CSS", None, Default::default())?;
    for page in css_pages.iter() {
        if !matches!(
            page.page_type(),
            PageType::CssType
                | PageType::CssAtRule
                | PageType::CssKeyword
                | PageType::CssFunction
                | PageType::CssSelector
                | PageType::CssProperty
                | PageType::CssPseudoElement
                | PageType::CssPseudoClass
                | PageType::CssShorthandProperty
                | PageType::CssAtRuleDescriptor
        ) || page
            .status()
            .iter()
            .any(|s| matches!(s, FeatureStatus::Deprecated | FeatureStatus::NonStandard))
        {
            continue;
        }
        let initial = initial_letter(page.title());
        let entry = index.entry(initial).or_default();
        let (url, label) = (page.slug(), page.title());
        entry.entry(url).or_insert(label);
    }

    let mut out = String::new();
    out.push_str(r#"<div class="index">"#);
    for (letter, items) in index {
        out.push_str("<h3>");
        out.push(letter);
        out.push_str("</h3><ul>");
        for (url, label) in items
            .into_iter()
            .sorted_by(|(_, a), (_, b)| compare_items(a, b))
        {
            out.extend([
                "<li>",
                &RariApi::link(
                    &concat_strs!("/", url),
                    env.locale,
                    Some(&html_escape::encode_text(&label)),
                    true,
                    None,
                    false,
                )?,
                "</li>",
            ]);
        }
        out.push_str("</ul>");
    }
    out.push_str(r#"</div>"#);

    Ok(out)
}

fn compare_items(a: &str, b: &str) -> Ordering {
    let ord = a
        .trim_matches(|c: char| !c.is_ascii_alphabetic() && c != '(' && c != ')' && c != '-')
        .cmp(
            b.trim_matches(|c: char| !c.is_ascii_alphabetic() && c != '(' && c != ')' && c != '-'),
        );
    if ord == Ordering::Equal {
        a.cmp(b)
    } else {
        ord
    }
}

fn initial_letter(s: &str) -> char {
    s.chars()
        .find(|&c| c.is_ascii_alphabetic() || c == '-')
        .unwrap_or('?')
        .to_ascii_uppercase()
}
