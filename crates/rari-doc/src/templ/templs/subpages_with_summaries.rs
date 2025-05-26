use rari_templ_func::rari_f;

use crate::error::DocError;
use crate::helpers::subpages::{get_sub_pages, SubPagesSorter};
use crate::helpers::summary_hack::{get_hacky_summary_md, strip_paragraph_unchecked};
use crate::pages::page::PageLike;

#[rari_f(crate::Templ)]
pub fn subpages_with_summaries() -> Result<String, DocError> {
    let mut out = String::new();
    let sub_pages = get_sub_pages(env.url, Some(1), SubPagesSorter::Title)?;

    out.push_str("<dl>");
    for page in sub_pages {
        out.extend([
            r#"<dt class="landingPageList"><a href=""#,
            page.url(),
            r#"">"#,
            &html_escape::encode_safe(page.title()),
            r#"</a></dt><dd class="landingPageList"><p>"#,
            strip_paragraph_unchecked(get_hacky_summary_md(&page)?.as_str()),
            r#"</p></dd>"#,
        ]);
    }
    out.push_str("</dl>");
    Ok(out)
}
