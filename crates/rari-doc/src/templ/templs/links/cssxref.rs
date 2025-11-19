use rari_templ_func::rari_f;
use rari_types::fm_types::PageType;
use rari_types::locale::Locale;

use crate::error::DocError;
use crate::pages::page::PageLike;
use crate::templ::api::RariApi;

/// Creates a link to a CSS reference page on MDN.
///
/// This macro generates links to CSS properties, functions, data types, selectors,
/// and other CSS reference documentation. It automatically formats the display text
/// based on the CSS feature type and handles various CSS naming conventions.
///
/// # Arguments
/// * `name` - The CSS feature name (property, function, data type, etc.)
/// * `display` - Optional custom display text for the link
/// * `anchor` - Optional anchor/fragment to append to the URL
///
/// # Examples
/// * `{{CSSxRef("color")}}` -> links to CSS color property
/// * `{{CSSxRef("background-color", "background color")}}` -> custom display text
/// * `{{CSSxRef("calc()", "", "#syntax")}}` -> links to calc() function with anchor
/// * `{{CSSxRef("<color>")}}` -> links to color data type
///
/// # Special handling
/// - Functions automatically get `()` appended if not present
/// - Data types get wrapped in `<>` brackets if not present
/// - Handles HTML entity encoding (`&lt;` and `&gt;`)
/// - Maps special cases like `<color>` to `color_value`
#[rari_f(register = "crate::Templ")]
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
    let maybe_display_name = &display_name
        .or_else(|| name.rsplit_once('/').map(|(_, s)| s))
        .unwrap_or(name);
    let decoded_maybe_display_name = html_escape::decode_html_entities(maybe_display_name);
    let encoded_maybe_display_name = html_escape::encode_text(decoded_maybe_display_name.as_ref());

    // Determine the original name for classification
    let mut slug = name
        .strip_prefix("&lt;")
        .unwrap_or(name.strip_prefix('<').unwrap_or(name));
    slug = slug
        .strip_suffix("&gt;")
        .unwrap_or(slug.strip_suffix('>').unwrap_or(slug));
    slug = slug.strip_suffix("()").unwrap_or(slug);

    // Apply special case mappings
    let slug = match name {
        "&lt;color&gt;" | "<color>" => "color_value",
        "&lt;flex&gt;" | "<flex>" => "flex_value",
        "&lt;overflow&gt;" | "<overflow>" => "overflow_value",
        "&lt;position&gt;" | "<position>" => "position_value",
        ":host()" => ":host_function",
        "fit-content()" => "fit_content_function",
        _ => slug,
    };

    let base_url = format!("/{}/docs/Web/CSS/Reference/", locale.as_url_str());
    // Determine the URL path based on the new structure
    let url_path = if name.starts_with("&lt;") || name.starts_with('<') {
        // Types go under Web/CSS/Reference/Values
        format!("Values/{slug}")
    } else if name.ends_with("()") {
        // Functions go under Web/CSS/Reference/Values
        format!("Values/{slug}")
    } else if name.starts_with('@') {
        // At-rules go under Web/CSS/Reference/At-rules
        format!("At-rules/{slug}")
    } else if name.starts_with(':') {
        // Pseudo-classes and pseudo-elements go under Web/CSS/Reference/Selectors
        format!("Selectors/{slug}")
    } else {
        // Everything else: check Properties first, then Values
        let url_path = format!("Properties/{slug}");
        let url = format!("{}{}", &base_url, &url_path);
        if RariApi::get_page_nowarn(&url).is_ok() {
            url_path
        } else {
            // Fall back to Values
            format!("Values/{slug}")
        }
    };

    let url = format!("{}{}{}", &base_url, &url_path, anchor.unwrap_or_default());

    let display_name = if display_name.is_some() {
        encoded_maybe_display_name.to_string()
    } else if let Ok(doc) = RariApi::get_page_nowarn(&url) {
        match doc.page_type() {
            PageType::CssFunction if !encoded_maybe_display_name.ends_with("()") => {
                format!("{encoded_maybe_display_name}()")
            }
            PageType::CssType
                if !(encoded_maybe_display_name.starts_with("&lt;")
                    && encoded_maybe_display_name.ends_with("&gt;")) =>
            {
                format!("&lt;{encoded_maybe_display_name}&gt;")
            }
            _ => encoded_maybe_display_name.to_string(),
        }
    } else {
        encoded_maybe_display_name.to_string()
    };
    RariApi::link(&url, Some(locale), Some(&display_name), true, None, false)
}
