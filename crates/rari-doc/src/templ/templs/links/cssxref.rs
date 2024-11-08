use rari_templ_func::rari_f;
use rari_types::fm_types::PageType;
use rari_types::locale::Locale;

use crate::error::DocError;
use crate::pages::page::PageLike;
use crate::templ::api::RariApi;

#[rari_f]
pub fn cssxref(
    name: String,
    display: Option<String>,
    anchor: Option<String>,
) -> Result<String, DocError> {
    let display = display.as_deref().filter(|s| !s.is_empty());
    cssxref_internal(&name, display, anchor.as_deref(), env.locale)
}

pub fn cssxref_internal(
    name: &str,
    display_name: Option<&str>,
    anchor: Option<&str>,
    locale: Locale,
) -> Result<String, DocError> {
    let maybe_display_name = display_name
        .or_else(|| name.rsplit_once('/').map(|(_, s)| s))
        .unwrap_or(name);
    let mut slug = name
        .strip_prefix("&lt;")
        .unwrap_or(name.strip_prefix('<').unwrap_or(name));
    slug = slug
        .strip_suffix("&gt;")
        .unwrap_or(slug.strip_suffix('>').unwrap_or(slug));
    slug = slug.strip_suffix("()").unwrap_or(slug);

    let slug = match name {
        "&lt;color&gt;" => "color_value",
        "&lt;flex&gt;" => "flex_value",
        "&lt;overflow&gt;" => "overflow_value",
        "&lt;position&gt;" => "position_value",
        ":host()" => ":host_function",
        "fit-content()" => "fit_content_function",
        _ => slug,
    };

    let url = format!(
        "/{}/docs/Web/CSS/{slug}{}",
        locale.as_url_str(),
        anchor.unwrap_or_default()
    );

    let display_name = if display_name.is_some() {
        maybe_display_name.to_string()
    } else if let Ok(doc) = RariApi::get_page(&url) {
        match doc.page_type() {
            PageType::CssFunction if !maybe_display_name.ends_with("()") => {
                format!("{maybe_display_name}()")
            }
            PageType::CssType
                if !(maybe_display_name.starts_with("&lt;")
                    && maybe_display_name.ends_with("&gt;")) =>
            {
                format!("&lt;{maybe_display_name}&gt;")
            }
            _ => maybe_display_name.to_string(),
        }
    } else {
        maybe_display_name.to_string()
    };
    RariApi::link(&url, locale, Some(&display_name), true, None, false)
}
