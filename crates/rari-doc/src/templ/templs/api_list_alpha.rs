use std::collections::BTreeMap;

use rari_templ_func::rari_f;
use rari_types::fm_types::PageType;

use crate::error::DocError;
use crate::helpers::subpages::{SubPagesSorter, get_sub_pages};
use crate::pages::page::PageLike;
use crate::templ::api::RariApi;
use crate::templ::index::{index_letter, index_letter_label_and_id, render_index_navigation};

/// Fragment ID prefix, shared by the navigation and the letter headings.
const INDEX_ID_PREFIX: &str = "index-interfaces";

#[rari_f(register = "crate::Templ")]
pub fn apilistalpha() -> Result<String, DocError> {
    let mut out = String::new();
    let pages = get_sub_pages("/en-US/docs/Web/API", Some(1), SubPagesSorter::Title)?;
    let mut pages_by_letter: BTreeMap<char, Vec<_>> = BTreeMap::new();
    for page in pages
        .iter()
        .filter(|page| page.page_type() == PageType::WebApiInterface)
    {
        // Group by the rendered label, so the letter always matches what is shown.
        let page_label = page.short_title().unwrap_or(page.title());
        if let Some(letter) = page_label.chars().next().map(index_letter) {
            pages_by_letter
                .entry(letter)
                .or_default()
                .push((page, page_label));
        }
    }

    out.push_str(r#"<div class="index">"#);
    render_index_navigation(&mut out, pages_by_letter.keys().copied(), INDEX_ID_PREFIX);
    for (letter, pages) in pages_by_letter {
        let (label, id) = index_letter_label_and_id(letter, INDEX_ID_PREFIX);
        out.extend([r#"<h3 id=""#, &id, r#"">"#, &label, "</h3><ul>"]);
        for (page, page_label) in pages {
            out.extend([
                "<li>",
                &RariApi::link(
                    page.url(),
                    Some(env.locale),
                    None,
                    true,
                    Some(page_label),
                    true,
                )?,
                "</li>",
            ]);
        }
        out.push_str("</ul>");
    }
    out.push_str("</div>");

    Ok(out)
}
