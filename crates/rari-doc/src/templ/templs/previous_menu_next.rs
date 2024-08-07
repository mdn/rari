use rari_l10n::l10n_json_data;
use rari_templ_func::rari_f;
use rari_types::locale::Locale;

use crate::docs::page::PageLike;
use crate::error::DocError;
use crate::templ::api::RariApi;

#[rari_f]
pub fn previous_next_menu(
    prev: Option<String>,
    next: Option<String>,
    menu: Option<String>,
) -> Result<String, DocError> {
    previous_next_menu_internal(prev, next, menu, env.locale)
}

#[rari_f]
pub fn previous_next(prev: Option<String>, next: Option<String>) -> Result<String, DocError> {
    previous_next_menu_internal(prev, next, None, env.locale)
}

#[rari_f]
pub fn previous_menu(prev: Option<String>, menu: Option<String>) -> Result<String, DocError> {
    previous_next_menu_internal(prev, None, menu, env.locale)
}

#[rari_f]
pub fn previous(prev: Option<String>) -> Result<String, DocError> {
    previous_next_menu_internal(prev, None, None, env.locale)
}

#[rari_f]
pub fn next_menu(next: Option<String>, menu: Option<String>) -> Result<String, DocError> {
    previous_next_menu_internal(None, next, menu, env.locale)
}

#[rari_f]
pub fn next(next: Option<String>) -> Result<String, DocError> {
    previous_next_menu_internal(None, next, None, env.locale)
}

fn previous_next_menu_internal(
    prev: Option<String>,
    next: Option<String>,
    menu: Option<String>,
    locale: Locale,
) -> Result<String, DocError> {
    let mut out = String::new();
    out.push_str(r#"<ul class="prev-next">"#);
    if let Some(prev) = prev {
        let title = l10n_json_data("Template", "previous", locale)?;
        generate_link(&mut out, &prev, locale, title)?;
    }
    if let Some(menu) = menu {
        let page = RariApi::get_page(&["", locale.as_url_str(), "docs", menu.as_str()].join("/"))?;
        let title = [
            l10n_json_data("Template", "prev_next_menu", locale)?,
            page.title(),
        ]
        .join("");
        generate_link(&mut out, page.slug(), locale, &title)?;
    }
    if let Some(next) = next {
        let title = l10n_json_data("Template", "next", locale)?;
        generate_link(&mut out, &next, locale, title)?;
    }
    out.push_str("</ul>");
    Ok(out)
}
fn generate_link(
    out: &mut String,
    slug: &str,
    locale: Locale,
    title: &str,
) -> Result<(), DocError> {
    out.extend([
        r#"<li><a class="button secondary" href="/"#,
        locale.as_url_str(),
        r#"/docs/"#,
        slug,
        r#""><span class="button-wrap">"#,
        title,
        r#"</span></a></li>"#,
    ]);
    Ok(())
}
