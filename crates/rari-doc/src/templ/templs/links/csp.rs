use rari_templ_func::rari_f;

use crate::error::DocError;
use crate::templ::api::RariApi;

/// Creates a link to a Content Security Policy (CSP) directive reference page on MDN.
///
/// This macro generates links to CSP directive documentation under the
/// Content-Security-Policy HTTP header reference. It formats the link with
/// the directive name and applies code formatting to distinguish CSP directives
/// in the text.
///
/// # Arguments
/// * `directive` - The CSP directive name (e.g., "default-src", "script-src", "img-src")
///
/// # Examples
/// * `{{CSP("default-src")}}` -> links to default-src directive with code formatting
/// * `{{CSP("script-src")}}` -> links to script-src directive
/// * `{{CSP("unsafe-inline")}}` -> links to unsafe-inline keyword
/// * `{{CSP("nonce-")}}` -> links to nonce- source expression
///
/// # Special handling
/// - Uses the directive name as both the URL path and display text
/// - Applies code formatting (`<code>` tags) by default
/// - Links to `/Web/HTTP/Reference/Headers/Content-Security-Policy/{directive}` path structure
/// - Works for directives, keywords, and source expressions within CSP
#[rari_f(register = "crate::Templ")]
pub fn csp(directive: String) -> Result<String, DocError> {
    let url = format!(
        "/{}/docs/Web/HTTP/Reference/Headers/Content-Security-Policy/{directive}",
        env.locale.as_url_str()
    );
    RariApi::link(
        &url,
        Some(env.locale),
        Some(directive.as_ref()),
        true,
        None,
        false,
    )
}
