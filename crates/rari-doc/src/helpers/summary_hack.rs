use itertools::Itertools;
use rari_md::{M2HOptions, m2h_internal};
use scraper::Html;

use crate::error::DocError;
use crate::pages::page::{Page, PageLike};
use crate::templ::render::render_for_summary;

/// There's a few places were we still transplant content.
/// Yari had a hidden hacky way to do this and we have to mimic this for now.
pub fn get_hacky_summary_md(page: &Page) -> Result<String, DocError> {
    let summary_md = page
        .content()
        .lines()
        .skip_while(|line| {
            line.trim().is_empty()
                || line.starts_with("{{") && line.ends_with("}}")
                || line.starts_with("##")
        })
        .take_while(|line| {
            !(line.trim().is_empty()
                || (line.starts_with("{{") && line.ends_with("}}") || line.starts_with("##")))
        })
        .join("\n");
    if summary_md.is_empty() {
        Ok(String::from("No summary found."))
    } else {
        render_for_summary(&summary_md).and_then(|md| {
            Ok(m2h_internal(
                md.trim(),
                page.locale(),
                M2HOptions { sourcepos: false },
            )?)
        })
    }
}

/// Trims a `<p>` tag in good faith.
/// This does not check if theres a `<p>` as root and will
/// result in invalid html for input like:
/// ```html
/// <p>foo</p>bar
/// ```
pub fn strip_paragraph_unchecked(input: &str) -> &str {
    input.trim().strip_prefix("<p>").unwrap_or(input)
}

pub fn text_content(html_str: &str) -> String {
    let fragment = Html::parse_fragment(html_str);
    fragment.root_element().text().collect()
}
