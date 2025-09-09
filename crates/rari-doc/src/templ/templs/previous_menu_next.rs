use rari_templ_func::rari_f;
use rari_types::locale::Locale;
use rari_utils::concat_strs;

use crate::error::DocError;
use crate::helpers::l10n::l10n_json_data;
use crate::pages::page::PageLike;
use crate::templ::api::RariApi;

#[rari_f(register = "crate::Templ")]
pub fn previousmenunext(
    prev: Option<String>,
    next: Option<String>,
    menu: Option<String>,
) -> Result<String, DocError> {
    previous_next_menu_internal(prev, next, menu, env.locale)
}

#[rari_f(register = "crate::Templ")]
pub fn previousnext(prev: Option<String>, next: Option<String>) -> Result<String, DocError> {
    previous_next_menu_internal(prev, next, None, env.locale)
}

#[rari_f(register = "crate::Templ")]
pub fn previousmenu(prev: Option<String>, menu: Option<String>) -> Result<String, DocError> {
    previous_next_menu_internal(prev, None, menu, env.locale)
}

#[rari_f(register = "crate::Templ")]
pub fn previous(prev: Option<String>) -> Result<String, DocError> {
    previous_next_menu_internal(prev, None, None, env.locale)
}

#[rari_f(register = "crate::Templ")]
pub fn nextmenu(next: Option<String>, menu: Option<String>) -> Result<String, DocError> {
    previous_next_menu_internal(None, next, menu, env.locale)
}

#[rari_f(register = "crate::Templ")]
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
        if !prev.is_empty() {
            let page = RariApi::get_page(&concat_strs!(
                "/",
                locale.as_url_str(),
                "/docs/",
                prev.as_str()
            ))?;
            let title = l10n_json_data("Template", "previous", locale)?;
            generate_link(&mut out, page.slug(), locale, title, "prev")?;
        }
    }
    if let Some(menu) = menu {
        if !menu.is_empty() {
            let page = RariApi::get_page(&concat_strs!(
                "/",
                locale.as_url_str(),
                "/docs/",
                menu.as_str()
            ))?;
            let title = concat_strs!(
                l10n_json_data("Template", "prev_next_menu", locale)?,
                page.title()
            );
            generate_link(&mut out, page.slug(), locale, &title, "menu")?;
        }
    }
    if let Some(next) = next {
        if !next.is_empty() {
            let page = RariApi::get_page(&concat_strs!(
                "/",
                locale.as_url_str(),
                "/docs/",
                next.as_str()
            ))?;
            let title = l10n_json_data("Template", "next", locale)?;
            generate_link(&mut out, page.slug(), locale, title, "next")?;
        }
    }
    out.push_str("</ul>");
    Ok(out)
}

fn generate_link(
    out: &mut String,
    slug: &str,
    locale: Locale,
    title: &str,
    class: &str,
) -> Result<(), DocError> {
    out.extend([
        r#"<li class="#,
        class,
        r#"><a data-templ-link class="button secondary" href="/"#,
        locale.as_url_str(),
        r#"/docs/"#,
        slug,
        r#""><span class="button-wrap">"#,
        &html_escape::encode_safe(title),
        r#"</span></a></li>"#,
    ]);
    Ok(())
}
