use rari_templ_func::rari_f;
use rari_types::AnyArg;

use crate::error::DocError;
use crate::templ::api::RariApi;

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
/// - Tries main JavaScript Reference first, then Global Objects
/// - Handles special cases like `try...catch` statements
/// - Falls back to URI component decoding if no page found
/// - Formats links with `<code>` tags unless `no_code` is true
#[rari_f(register = "crate::Templ")]
pub fn jsxref(
    api_name: String,
    display: Option<String>,
    anchor: Option<String>,
    no_code: Option<AnyArg>,
) -> Result<String, DocError> {
    let display = display.as_deref().filter(|s| !s.is_empty());
    let global_objects = "Global_Objects";
    let display = display.unwrap_or(api_name.as_str());
    let mut url = format!("/{}/docs/Web/JavaScript/Reference/", &env.locale);
    let mut base_path = url.clone();

    let mut slug = api_name.replace("()", "").replace(".prototype.", ".");
    if !slug.contains("/") && slug.contains('.') {
        // Handle try...catch case
        slug = slug.replace('.', "/");
    }

    let page_url = format!("{url}{slug}");
    let object_page_url = format!("{url}{global_objects}/{slug}");

    let page = RariApi::get_page_nowarn(&page_url);
    let object_page = RariApi::get_page_nowarn(&object_page_url);
    if let Ok(_page) = page {
        url.push_str(&slug)
    } else if let Ok(_object_page) = object_page {
        base_path.extend([global_objects, "/"]);
        url.extend([global_objects, "/", &slug]);
    } else {
        url.push_str(&RariApi::decode_uri_component(&api_name));
    }

    if let Some(anchor) = anchor {
        if !anchor.starts_with('#') {
            url.push('#');
        }
        url.push_str(&anchor);
    }

    let code = !no_code.map(|nc| nc.as_bool()).unwrap_or_default();
    RariApi::link(&url, Some(env.locale), Some(display), code, None, false)
}
