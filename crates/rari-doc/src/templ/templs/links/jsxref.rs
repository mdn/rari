use rari_templ_func::rari_f;
use rari_types::AnyArg;

use crate::error::DocError;
use crate::templ::api::RariApi;
use crate::templ::js_ref_index::resolve_js_ref;

/// Creates a link to a JavaScript reference page on MDN.
///
/// This macro generates links to JavaScript language features including objects,
/// methods, properties, statements, operators, and other JavaScript reference
/// documentation. It intelligently routes to either the main JavaScript Reference
/// or the Global Objects section based on the API name.
///
/// # Arguments
/// * `api_name` - The JavaScript feature name (object, method, property, etc.)
/// * `display` - Optional custom display text for the link
/// * `anchor` - Optional anchor/fragment to append to the URL
/// * `no_code` - Optional flag to disable code formatting (default: false)
///
/// # Examples
/// * `{{JSxRef("Array")}}` -> links to Array global object
/// * `{{JSxRef("Array.prototype.map")}}` -> links to Array map method
/// * `{{JSxRef("Promise", "Promises")}}` -> custom display text
/// * `{{JSxRef("if...else")}}` -> links to if...else statement
/// * `{{JSxRef("typeof", "", "", true)}}` -> disables code formatting
///
/// # Special handling
/// - Removes `()` from method names for URL generation
/// - Converts `.prototype.` notation to `/` for URL paths
/// - Falls back to URI component decoding if no page found
/// - Formats links with `<code>` tags unless `no_code` is true
///
/// # Name resolution
/// The normalized `api_name` is resolved against an index of all
/// `Web/JavaScript/Reference/*` pages (see [`crate::templ::js_ref_index`]).
/// Authors can use:
/// - A full sub-path: `{{JSxRef("Statements/for...of")}}`
/// - A bare global-object name or dotted member:
///   `{{JSxRef("Array")}}`, `{{JSxRef("Array.prototype.map")}}`
/// - **For namespace-class members only** (`Intl`, `Temporal`), a path with
///   the namespace omitted: `{{JSxRef("Collator")}}` resolves to
///   `Intl/Collator`, `{{JSxRef("Collator/compare")}}` to
///   `Intl/Collator/compare`.
#[rari_f(register = "crate::Templ")]
pub fn jsxref(
    api_name: String,
    display: Option<String>,
    anchor: Option<String>,
    no_code: Option<AnyArg>,
) -> Result<String, DocError> {
    let display = display.as_deref().filter(|s| !s.is_empty());
    let display = display.unwrap_or(api_name.as_str());

    let normalized = api_name.replace("()", "").replace(".prototype.", ".");
    let normalized = if !normalized.contains('/') && normalized.contains('.') {
        normalized.replace('.', "/")
    } else {
        normalized
    };

    let base = format!("/{}/docs/Web/JavaScript/Reference/", env.locale);
    let mut url = if let Some(resolved) = resolve_js_ref(&normalized) {
        format!("{base}{resolved}")
    } else {
        format!("{base}{}", RariApi::decode_uri_component(&api_name))
    };

    if let Some(anchor) = anchor {
        if !anchor.starts_with('#') {
            url.push('#');
        }
        url.push_str(&anchor);
    }

    let code = !no_code.map(|nc| nc.as_bool()).unwrap_or_default();
    RariApi::link(&url, Some(env.locale), Some(display), code, None, false)
}
