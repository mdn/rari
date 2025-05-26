use rari_templ_func::rari_f;
use rari_types::AnyArg;

use crate::error::DocError;
use crate::helpers::subpages::{get_sub_pages, SubPagesSorter};
use crate::html::links::{render_internal_link, LinkModifier};
use crate::pages::page::{Page, PageLike};
use crate::utils::{trim_after, trim_before};

/// List sub pages for sidebar
#[rari_f(crate::Templ)]
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
        let parent = Page::from_url_with_fallback(&url)?;
        sub_pages.insert(0, parent);
    }

    out.push_str("<ol>");
    for page in sub_pages {
        let locale_page = if env.locale != Default::default() {
            &Page::from_url_with_locale_and_fallback(page.url(), env.locale)?
        } else {
            &page
        };
        let title = locale_page.short_title().unwrap_or(locale_page.title());
        let title = trim_before(title, title_only_after.as_deref());
        let title = trim_after(title, title_only_before.as_deref());
        render_internal_link(
            &mut out,
            locale_page.url(),
            None,
            &html_escape::encode_safe(title),
            None,
            &LinkModifier {
                badges: page.status(),
                badge_locale: env.locale,
                code,
                only_en_us: locale_page.locale() != env.locale,
            },
            true,
        )?;
        out.push_str("</li>");
    }
    out.push_str("</ol>");

    Ok(out)
}
