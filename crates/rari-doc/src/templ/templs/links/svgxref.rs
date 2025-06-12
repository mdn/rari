use rari_templ_func::rari_f;
use rari_types::locale::Locale;
use rari_types::AnyArg;

use crate::error::DocError;
use crate::templ::api::RariApi;

/// Creates a link to an SVG element reference page on MDN.
///
/// This macro generates links to SVG element documentation. It automatically
/// formats the display text with angle brackets (e.g., `<circle>`) and applies
/// code formatting to distinguish SVG elements in the text.
///
/// # Arguments
/// * `element_name` - The SVG element name (without angle brackets)
/// * `_` - Unused parameter (kept for compatibility)
///
/// # Examples
/// * `{{SVGElement("circle")}}` -> links to `<circle>` element with code formatting
/// * `{{SVGElement("path")}}` -> links to `<path>` element
/// * `{{SVGElement("svg")}}` -> links to root `<svg>` element
///
/// # Special handling
/// - Automatically wraps element name in `&lt;` and `&gt;` for display
/// - Uses code formatting (`<code>` tags) by default
/// - Links to `/Web/SVG/Reference/Element/{element_name}` path structure
#[rari_f(register = "crate::Templ")]
pub fn svgelement(element_name: String, _: Option<AnyArg>) -> Result<String, DocError> {
    svgxref_internal(&element_name, env.locale)
}

pub fn svgxref_internal(element_name: &str, locale: Locale) -> Result<String, DocError> {
    let display = format!("&lt;{element_name}&gt;");
    let url = format!(
        "/{}/docs/Web/SVG/Reference/Element/{}",
        locale.as_url_str(),
        element_name,
    );

    RariApi::link(&url, locale, Some(display.as_ref()), true, None, false)
}
