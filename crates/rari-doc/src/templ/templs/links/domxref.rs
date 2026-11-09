use std::borrow::Cow;

use rari_templ_func::rari_f;
use rari_types::{AnyArg, ArgError};
use tracing::{Level, span};

use crate::error::DocError;
use crate::templ::api::RariApi;
use crate::templ::api_name_index::resolve_api_name;

/// Creates a link to a DOM/Web API reference page on MDN.
///
/// This macro generates links to Web API interfaces, methods, properties, and other
/// DOM-related documentation. It handles various API naming conventions and can
/// automatically format display text and anchors for methods and properties.
///
/// # Arguments
/// * `api` - The API name (interface, method, property, etc.)
/// * `label` - Optional custom display text for the link
/// * `anchor` - Optional anchor/fragment to append to the URL
/// * `plain` - Optional flag to disable code formatting (default: false)
///
/// # Examples
/// * `{{DOMxRef("Document")}}` -> links to Document interface
/// * `{{DOMxRef("document.getElementById()")}}` -> links to getElementById method
/// * `{{DOMxRef("Element.innerHTML", "innerHTML property")}}` -> custom display text
/// * `{{DOMxRef("Node", "", "", true)}}` -> disables code formatting
///
/// # Special handling
/// - Converts spaces to underscores and removes `()` from method names
/// - Handles prototype chain notation (`.prototype.` becomes `/`)
/// - Appends method/property names to display text when using anchors
/// - Formats links with `<code>` tags unless `plain` is true
///
/// # Name resolution
/// The (normalized) `api` is resolved against an index of all
/// `Web/API/*` pages (see [`crate::templ::api_name_index`]). Authors can use:
/// - A full sub-path: `{{domxref("BackgroundFetchManager/fetch")}}`
/// - A bare interface or grouped name: `{{domxref("SyncEvent")}}`
/// - **For `Window` members only**, a leaf segment:
///   `{{domxref("structuredClone")}}` resolves to `Window/structuredClone`.
///   Members of other interfaces must be referenced by full sub-path.
#[rari_f(register = "crate::Templ")]
pub fn domxref(
    api: String,
    label: Option<String>,
    anchor: Option<String>,
    plain: Option<AnyArg>,
) -> Result<String, DocError> {
    let span = span!(Level::ERROR, "domxref", basepath = "/docs/Web/API/");
    let _enter = span.enter();
    let display = label.as_deref().filter(|s| !s.is_empty());
    let mut display_with_fallback = Cow::Borrowed(display.unwrap_or(api.as_str()));
    let api = api
        .replace(' ', "_")
        .replace("()", "")
        .replace(".prototype.", ".")
        .replace('.', "/");
    if api.is_empty() {
        return Err(DocError::ArgError(ArgError::at(
            "domxref",
            1,
            "api",
            ArgError::MustNotBeEmpty,
        )));
    }
    let resolved = resolve_api_name(&api);
    let mut url = format!(
        "/{}/docs/Web/API/{}",
        env.locale.as_url_str(),
        resolved.unwrap_or(&api),
    );
    if let Some(anchor) = anchor {
        if !anchor.starts_with('#') {
            url.push('#');
            display_with_fallback = Cow::Owned(format!("{display_with_fallback}.{anchor}"));
        }
        url.push_str(&anchor);
        if let Some(anchor) = anchor.strip_prefix('#') {
            display_with_fallback = Cow::Owned(format!("{display_with_fallback}.{anchor}"));
        }
    }

    let code = !plain.map(|plain| plain.as_bool()).unwrap_or_default();
    RariApi::link(
        &url,
        Some(env.locale),
        Some(&display_with_fallback),
        code,
        display,
        false,
    )
}
