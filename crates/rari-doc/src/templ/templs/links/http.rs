use rari_templ_func::rari_f;
use rari_types::AnyArg;
use rari_types::locale::Locale;

use crate::error::DocError;
use crate::templ::api::RariApi;

/// Creates a link to an HTTP status code reference page on MDN.
///
/// This macro generates links to HTTP response status code documentation.
/// It formats the link with the status code number and applies code formatting
/// by default unless disabled.
///
/// # Arguments
/// * `status` - The HTTP status code (e.g., 200, 404, 500)
/// * `label` - Optional custom display text for the link
///
/// # Examples
/// * `{{HTTPStatus("404")}}` -> links to 404 Not Found status
/// * `{{HTTPStatus("200", "OK")}}` -> custom display text
#[rari_f(register = "crate::Templ")]
pub fn httpstatus(status: AnyArg, label: Option<String>) -> Result<String, DocError> {
    let url = format!(
        "/{}/docs/Web/HTTP/Reference/Status/{}",
        env.locale.as_url_str(),
        status
    );
    http(url, status, label, false, env.locale)
}

/// Creates a link to an HTTP header reference page on MDN.
///
/// This macro generates links to HTTP header documentation. It formats
/// the link with the header name and applies code formatting by default
/// unless disabled.
///
/// # Arguments
/// * `header` - The HTTP header name (e.g., "Content-Type", "Authorization")
/// * `label` - Optional custom display text for the link
/// * `_deprecated_3` - Deprecated parameter (no longer used)
/// * `plain` - Optional flag to disable code formatting (default: false)
///
/// # Examples
/// * `{{HTTPHeader("Content-Type")}}` -> links to Content-Type header
/// * `{{HTTPHeader("Authorization", "Auth header")}}` -> custom display text
/// * `{{HTTPHeader("Accept", "", "", true)}}` -> disables code formatting
#[rari_f(register = "crate::Templ")]
pub fn httpheader(
    header: AnyArg,
    label: Option<String>,
    _deprecated_3: Option<String>,
    plain: Option<AnyArg>,
) -> Result<String, DocError> {
    let url = format!(
        "/{}/docs/Web/HTTP/Reference/Headers/{}",
        env.locale.as_url_str(),
        header
    );
    let plain = plain.is_some_and(|plain| plain.as_bool());
    http(url, header, label, plain, env.locale)
}

/// Creates a link to an HTTP request method reference page on MDN.
///
/// This macro generates links to HTTP request method documentation.
/// It formats the link with the method name and applies code formatting
/// by default unless disabled.
///
/// # Arguments
/// * `method` - The HTTP method name (e.g., "GET", "POST", "PUT", "DELETE")
///
/// # Examples
/// * `{{HTTPMethod("POST")}}` -> links to POST method
#[rari_f(register = "crate::Templ")]
pub fn httpmethod(method: AnyArg) -> Result<String, DocError> {
    let url = format!(
        "/{}/docs/Web/HTTP/Reference/Methods/{}",
        env.locale.as_url_str(),
        method
    );
    http(url, method, None, false, env.locale)
}

/// Internal helper function for HTTP-related link generation.
///
/// # Arguments
/// * `url` - The base URL for the HTTP reference page
/// * `name` - The HTTP feature identifier (status code, header name, method)
/// * `label` - Optional custom display text
/// * `plain` - Whether to disable code formatting
/// * `locale` - The locale for link generation
fn http(
    url: String,
    name: AnyArg,
    label: Option<String>,
    plain: bool,
    locale: Locale,
) -> Result<String, DocError> {
    let display = label.unwrap_or_else(|| name.to_string());
    RariApi::link(
        &url,
        Some(locale),
        Some(display.as_ref()),
        !plain,
        None,
        false,
    )
}
