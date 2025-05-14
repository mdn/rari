use rari_templ_func::rari_f;

use crate::error::DocError;
use crate::helpers::subpages::get_sub_pages;
use crate::helpers::summary_hack::{get_hacky_summary_md, strip_paragraph_unchecked};
use crate::pages::page::PageLike;

#[rari_f]
pub fn glossarydisambiguation() -> Result<String, DocError> {
    let mut out = String::new();
    let pages = get_sub_pages(
        env.url,
        Some(1),
        crate::helpers::subpages::SubPagesSorter::Title,
    )?;
    out.push_str("<dl>");

    for page in pages {
        let summary = get_hacky_summary_md(&page)?;
        out.extend([
            r#"<dt><a href=""#,
            page.url(),
            r#"">"#,
            &html_escape::encode_safe(page.title()),
            r#"</a></dt><dd>"#,
            strip_paragraph_unchecked(summary.as_str()),
            r#"</dd>"#,
        ]);
    }
    out.push_str("</dl>");

    Ok(out)
}
