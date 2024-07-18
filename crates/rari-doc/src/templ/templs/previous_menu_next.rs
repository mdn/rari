use rari_l10n::l10n_json_data;
use rari_templ_func::rari_f;
use rari_types::locale::Locale;

use crate::docs::page::PageLike;
use crate::error::DocError;
use crate::templ::api::RariApi;

#[rari_f]
pub fn previous_menu_next(
    prev: Option<String>,
    next: Option<String>,
    menu: Option<String>,
) -> Result<String, DocError> {
    let mut out = String::new();
    out.push_str(r#"<ul class="prev-next">"#);
    if let Some(prev) = prev {
        let title = l10n_json_data("Template", "previous", env.locale)?;
        generate_link(&mut out, &prev, env.locale, title)?;
    }
    if let Some(menu) = menu {
        let page =
            RariApi::get_page(&["", env.locale.as_url_str(), "docs", menu.as_str()].join("/"))?;
        let title = [
            l10n_json_data("Template", "prev_next_menu", env.locale)?,
            page.title(),
        ]
        .join("");
        generate_link(&mut out, page.slug(), env.locale, &title)?;
    }
    if let Some(next) = next {
        let title = l10n_json_data("Template", "next", env.locale)?;
        generate_link(&mut out, &next, env.locale, title)?;
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
