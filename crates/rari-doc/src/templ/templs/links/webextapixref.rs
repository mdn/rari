use std::borrow::Cow;

use rari_templ_func::rari_f;
use rari_types::AnyArg;

use crate::error::DocError;
use crate::templ::api::RariApi;

/// Creates a link to a WebExtensions API reference page on MDN.
///
/// This macro generates links to WebExtensions (browser extension) API documentation.
/// It handles various API naming conventions including namespaces, methods, and properties,
/// and can automatically format display text and anchors for nested API references.
///
/// # Arguments
/// * `api` - The WebExtensions API name (namespace, method, property, etc.)
/// * `display` - Optional custom display text for the link
/// * `anchor` - Optional anchor/fragment to append to the URL
/// * `no_code` - Optional flag to disable code formatting (default: false)
///
/// # Examples
/// * `{{WebExtAPIRef("tabs")}}` -> links to tabs API namespace
/// * `{{WebExtAPIRef("tabs.query")}}` -> links to tabs.query method
/// * `{{WebExtAPIRef("runtime.onMessage", "onMessage event")}}` -> custom display text
/// * `{{WebExtAPIRef("storage.local", "", "#get")}}` -> with anchor to specific method
/// * `{{WebExtAPIRef("alarms", "", "", true)}}` -> disables code formatting
///
/// # Special handling
/// - Converts spaces to underscores and removes `()` from method names for URLs
/// - Handles dot notation (`.`) by converting to `/` for URL paths
/// - Appends anchor information to display text when anchors are used
/// - Formats links with `<code>` tags unless `no_code` is true
/// - Links to `/Mozilla/Add-ons/WebExtensions/API/{api}` path structure
#[rari_f(register = "crate::Templ")]
pub fn webextapiref(
    api: String,
    display: Option<String>,
    anchor: Option<String>,
    no_code: Option<AnyArg>,
) -> Result<String, DocError> {
    let display = display.as_deref().filter(|s| !s.is_empty());
    let mut display = display.map(Cow::Borrowed).unwrap_or(Cow::Borrowed(&api));
    let mut url = format!(
        "/{}/docs/Mozilla/Add-ons/WebExtensions/API/{}",
        env.locale.as_url_str(),
        &api.replace(' ', "_").replace("()", "").replace('.', "/"),
    );
    if let Some(anchor) = anchor {
        if !anchor.starts_with('#') {
            url.push('#');
        }
        url.push_str(&anchor);
        display.to_mut().push('#');
        display.to_mut().push_str(&anchor);
    };

    RariApi::link(
        &url,
        Some(env.locale),
        Some(display.as_ref()),
        !no_code.map(|nc| nc.as_bool()).unwrap_or_default(),
        None,
        false,
    )
}
