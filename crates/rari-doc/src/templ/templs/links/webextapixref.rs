use std::borrow::Cow;

use rari_templ_func::rari_f;
use rari_types::AnyArg;

use crate::error::DocError;
use crate::templ::api::RariApi;

#[rari_f]
pub fn webextapixref(
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
        env.locale,
        Some(display.as_ref()),
        !no_code.map(|nc| nc.as_bool()).unwrap_or_default(),
        None,
        false,
    )
}
