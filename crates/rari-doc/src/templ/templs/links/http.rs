use rari_templ_func::rari_f;
use rari_types::AnyArg;

use crate::error::DocError;
use crate::templ::api::RariApi;

#[rari_f]
pub fn http_status(
    status: AnyArg,
    display: Option<String>,
    anchor: Option<String>,
    no_code: Option<AnyArg>,
) -> Result<String, DocError> {
    let url = format!(
        "/{}/docs/Web/HTTP/Status/{}",
        env.locale.as_url_str(),
        status
    );
    http(url, status, display, anchor, no_code)
}

#[rari_f]
pub fn http_header(
    status: AnyArg,
    display: Option<String>,
    anchor: Option<String>,
    no_code: Option<AnyArg>,
) -> Result<String, DocError> {
    let url = format!(
        "/{}/docs/Web/HTTP/Header/{}",
        env.locale.as_url_str(),
        status
    );
    http(url, status, display, anchor, no_code)
}

#[rari_f]
pub fn http_method(
    status: AnyArg,
    display: Option<String>,
    anchor: Option<String>,
    no_code: Option<AnyArg>,
) -> Result<String, DocError> {
    let url = format!(
        "/{}/docs/Web/HTTP/Method/{}",
        env.locale.as_url_str(),
        status
    );
    http(url, status, display, anchor, no_code)
}

fn http(
    mut url: String,
    status: AnyArg,
    display: Option<String>,
    anchor: Option<String>,
    no_code: Option<AnyArg>,
) -> Result<String, DocError> {
    let mut display = display.unwrap_or(status.to_string());
    if let Some(anchor) = anchor {
        url.push_str(&anchor);
        display.push('.');
        display.push_str(&anchor);
    }
    let code = !no_code.map(|nc| nc.as_bool()).unwrap_or_default();
    RariApi::link(&url, None, Some(display.as_ref()), code, None, false)
}
