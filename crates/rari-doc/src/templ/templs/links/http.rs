use rari_templ_func::rari_f;
use rari_types::locale::Locale;
use rari_types::AnyArg;

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
/// * `display` - Optional custom display text for the link
/// * `anchor` - Optional anchor/fragment to append to the URL
/// * `no_code` - Optional flag to disable code formatting (default: false)
///
/// # Examples
/// * `{{HTTPStatus("404")}}` -> links to 404 Not Found status
/// * `{{HTTPStatus("200", "OK")}}` -> custom display text
/// * `{{HTTPStatus("500", "", "#syntax")}}` -> with anchor
/// * `{{HTTPStatus("403", "", "", true)}}` -> disables code formatting
#[rari_f(register = "crate::Templ")]
pub fn httpstatus(
    status: AnyArg,
    display: Option<String>,
    anchor: Option<String>,
    no_code: Option<AnyArg>,
) -> Result<String, DocError> {
    let url = format!(
        "/{}/docs/Web/HTTP/Reference/Status/{}",
        env.locale.as_url_str(),
        status
    );
    http(url, status, display, anchor, no_code, env.locale)
}

/// Creates a link to an HTTP header reference page on MDN.
///
/// This macro generates links to HTTP header documentation. It formats
/// the link with the header name and applies code formatting by default
/// unless disabled.
///
/// # Arguments
/// * `status` - The HTTP header name (e.g., "Content-Type", "Authorization")
/// * `display` - Optional custom display text for the link
/// * `anchor` - Optional anchor/fragment to append to the URL
/// * `no_code` - Optional flag to disable code formatting (default: false)
///
/// # Examples
/// * `{{HTTPHeader("Content-Type")}}` -> links to Content-Type header
/// * `{{HTTPHeader("Authorization", "Auth header")}}` -> custom display text
/// * `{{HTTPHeader("Cache-Control", "", "#syntax")}}` -> with anchor
/// * `{{HTTPHeader("Accept", "", "", true)}}` -> disables code formatting
#[rari_f(register = "crate::Templ")]
pub fn httpheader(
    status: AnyArg,
    display: Option<String>,
    anchor: Option<String>,
    no_code: Option<AnyArg>,
) -> Result<String, DocError> {
    let url = format!(
        "/{}/docs/Web/HTTP/Reference/Headers/{}",
        env.locale.as_url_str(),
        status
    );
    http(url, status, display, anchor, no_code, env.locale)
}

/// Creates a link to an HTTP request method reference page on MDN.
///
/// This macro generates links to HTTP request method documentation.
/// It formats the link with the method name and applies code formatting
/// by default unless disabled.
///
/// # Arguments
/// * `status` - The HTTP method name (e.g., "GET", "POST", "PUT", "DELETE")
/// * `display` - Optional custom display text for the link
/// * `anchor` - Optional anchor/fragment to append to the URL
/// * `no_code` - Optional flag to disable code formatting (default: false)
///
/// # Examples
/// * `{{HTTPMethod("POST")}}` -> links to POST method
/// * `{{HTTPMethod("GET", "GET request")}}` -> custom display text
/// * `{{HTTPMethod("PUT", "", "#syntax")}}` -> with anchor
/// * `{{HTTPMethod("DELETE", "", "", true)}}` -> disables code formatting
#[rari_f(register = "crate::Templ")]
pub fn httpmethod(
    status: AnyArg,
    display: Option<String>,
    anchor: Option<String>,
    no_code: Option<AnyArg>,
) -> Result<String, DocError> {
    let url = format!(
        "/{}/docs/Web/HTTP/Reference/Methods/{}",
        env.locale.as_url_str(),
        status
    );
    http(url, status, display, anchor, no_code, env.locale)
}

/// Internal helper function for HTTP-related link generation.
///
/// This function handles the common logic for creating HTTP reference links
/// including status codes, headers, and methods. It processes anchors,
/// display text formatting, and code styling.
///
/// # Arguments
/// * `url` - The base URL for the HTTP reference page
/// * `status` - The HTTP feature identifier (status code, header name, method)
/// * `display` - Optional custom display text
/// * `anchor` - Optional anchor/fragment to append
/// * `no_code` - Optional flag to disable code formatting
/// * `locale` - The locale for link generation
fn http(
    mut url: String,
    status: AnyArg,
    display: Option<String>,
    anchor: Option<String>,
    no_code: Option<AnyArg>,
    locale: Locale,
) -> Result<String, DocError> {
    let mut display = display.unwrap_or(status.to_string());
    if let Some(anchor) = anchor {
        url.push_str(&anchor);
        display.push('.');
        display.push_str(&anchor);
    }
    let code = !no_code.map(|nc| nc.as_bool()).unwrap_or_default();
    RariApi::link(
        &url,
        Some(locale),
        Some(display.as_ref()),
        code,
        None,
        false,
    )
}
