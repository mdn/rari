use rari_templ_func::rari_f;

use crate::error::DocError;
use crate::templ::api::RariApi;

/// Creates a link to an SVG attribute reference page on MDN.
/// 
/// This macro generates links to SVG attribute documentation. It formats
/// the link with the attribute name and applies code formatting to distinguish
/// SVG attributes in the text.
/// 
/// # Arguments
/// * `name` - The SVG attribute name (e.g., "fill", "stroke", "viewBox")
/// 
/// # Examples  
/// * `{{SVGAttr("fill")}}` -> links to fill attribute with code formatting
/// * `{{SVGAttr("stroke-width")}}` -> links to stroke-width attribute
/// * `{{SVGAttr("viewBox")}}` -> links to viewBox attribute
/// 
/// # Special handling
/// - Uses the attribute name as both the URL path and display text
/// - Applies code formatting (`<code>` tags) by default
/// - Links to `/Web/SVG/Reference/Attribute/{name}` path structure
#[rari_f(register = "crate::Templ")]
pub fn svgattr(name: String) -> Result<String, DocError> {
    let url = format!(
        "/{}/docs/Web/SVG/Reference/Attribute/{}",
        env.locale.as_url_str(),
        name,
    );

    RariApi::link(&url, env.locale, Some(&name), true, None, false)
}
