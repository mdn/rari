use rari_templ_func::rari_f;
use rari_types::AnyArg;

use crate::error::DocError;
use crate::templ::api::RariApi;

#[rari_f]
pub fn htmlxref(
    element_name: String,
    display: Option<String>,
    anchor: Option<String>,
    _: Option<AnyArg>,
) -> Result<String, DocError> {
    let display = display.filter(|s| !s.is_empty());
    let mut code = false;
    let display = display.unwrap_or_else(|| {
        code = true;
        format!("&lt;{element_name}&gt;")
    });
    let mut url = format!(
        "/{}/docs/Web/HTML/Element/{}",
        env.locale.as_url_str(),
        element_name,
    );
    if let Some(anchor) = anchor {
        if !anchor.starts_with('#') {
            url.push('#');
        }
        url.push_str(&anchor);
    }

    RariApi::link(&url, env.locale, Some(display.as_ref()), code, None, false)
}
