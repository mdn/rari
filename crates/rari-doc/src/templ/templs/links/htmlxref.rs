use rari_templ_func::rari_f;
use rari_types::AnyArg;

use crate::error::DocError;
use crate::templ::api::RariApi;

/// Creates a link to an HTML element reference page on MDN.
///
/// This macro generates links to HTML element documentation. It automatically
/// formats the display text with angle brackets (e.g., `<div>`) unless custom
/// display text is provided, and applies code formatting by default.
///
/// # Arguments
/// * `element_name` - The HTML element name (without angle brackets)
/// * `display` - Optional custom display text for the link
/// * `anchor` - Optional anchor/fragment to append to the URL
/// * `_` - Unused parameter (kept for compatibility)
///
/// # Examples
/// * `{{HTMLElement("div")}}` -> links to `<div>` element with code formatting
/// * `{{HTMLElement("input", "input element")}}` -> custom display text
/// * `{{HTMLElement("form", "", "#attributes")}}` -> links to form element with anchor
///
/// # Special handling
/// - Automatically wraps element name in `&lt;` and `&gt;` for display
/// - Uses code formatting (`<code>` tags) by default unless custom display text provided
/// - Links to `/Web/HTML/Reference/Elements/{element_name}` path structure
#[rari_f(register = "crate::Templ")]
pub fn htmlelement(
    element_name: String,
    display: Option<String>,
    anchor: Option<String>,
    _: Option<AnyArg>,
) -> Result<String, DocError> {
    let display = display.filter(|s| !s.is_empty());
    let mut code = false;
    let display = display.unwrap_or_else(|| {
        code = true;
        format!("&lt;{element_name}&gt;")
    });
    let mut url = format!(
        "/{}/docs/Web/HTML/Reference/Elements/{}",
        env.locale.as_url_str(),
        element_name,
    );
    if let Some(anchor) = anchor {
        if !anchor.starts_with('#') {
            url.push('#');
        }
        url.push_str(&anchor);
    }

    RariApi::link(
        &url,
        Some(env.locale),
        Some(display.as_ref()),
        code,
        None,
        false,
    )
}
