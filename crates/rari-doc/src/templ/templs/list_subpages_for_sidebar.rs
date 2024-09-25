use rari_templ_func::rari_f;
use rari_types::AnyArg;

use crate::error::DocError;
use crate::helpers::subpages::{add_inline_badges, get_sub_pages, SubPagesSorter};
use crate::pages::page::{Page, PageLike};
use crate::utils::{trim_after, trim_fefore};

/// List sub pages for sidebar
#[rari_f]
pub fn list_subpages_for_sidebar(
    url: String,
    no_code: Option<AnyArg>,
    include_parent: Option<AnyArg>,
    title_only_after: Option<String>,
    title_only_before: Option<String>,
) -> Result<String, DocError> {
    let mut out = String::new();
    let include_parent = include_parent.map(|i| i.as_bool()).unwrap_or_default();
    let mut sub_pages = get_sub_pages(&url, Some(1), SubPagesSorter::Title)?;
    if sub_pages.is_empty() && !include_parent {
        return Ok(out);
    }
    let code = !no_code.map(|b| b.as_bool()).unwrap_or_default();
    if include_parent {
        let parent = Page::from_url(&url)?;
        sub_pages.insert(0, parent);
    }

    out.push_str("<ol>");
    for page in sub_pages {
        let locale_page = if env.locale != Default::default() {
            &Page::from_url_with_other_locale_and_fallback(page.url(), Some(env.locale))?
        } else {
            &page
        };
        let title = locale_page.short_title().unwrap_or(locale_page.title());
        let title = trim_fefore(title, title_only_after.as_deref());
        let title = trim_after(title, title_only_before.as_deref());
        out.extend([
            r#"<li><a href=""#,
            locale_page.url(),
            r#"">"#,
            if code { "<code>" } else { "" },
            &html_escape::encode_safe(title),
            if code { "</code>" } else { "" },
            r#"</a>"#,
        ]);
        add_inline_badges(&mut out, &page, env.locale)?;
        out.push_str("</li>");
    }
    out.push_str("</ol>");

    Ok(out)
}
