use std::borrow::Cow;

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
    let display = display.as_deref().filter(|s| !s.is_empty());
    let element_name = element_name.to_lowercase();
    let mut code = false;
    let display = display.map(Cow::Borrowed).unwrap_or_else(|| {
        if element_name.contains(' ') {
            Cow::Borrowed(element_name.as_str())
        } else {
            code = true;
            Cow::Owned(format!("<{element_name}>"))
        }
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

    RariApi::link(&url, None, Some(display.as_ref()), code, None, false)
}
