use rari_templ_func::rari_f;

use crate::error::DocError;
use crate::templ::api::RariApi;

#[rari_f(crate::Templ)]
pub fn csp(directive: String) -> Result<String, DocError> {
    let url = format!(
        "/{}/docs/Web/HTTP/Reference/Headers/Content-Security-Policy/{directive}",
        env.locale.as_url_str()
    );
    RariApi::link(
        &url,
        env.locale,
        Some(directive.as_ref()),
        true,
        None,
        false,
    )
}
