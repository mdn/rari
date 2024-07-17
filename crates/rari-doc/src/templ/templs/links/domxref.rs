use std::borrow::Cow;

use rari_templ_func::rari_f;
use rari_types::{AnyArg, ArgError};

use crate::error::DocError;
use crate::templ::api::RariApi;

#[rari_f]
pub fn domxref(
    api_name: String,
    display: Option<String>,
    anchor: Option<String>,
    no_code: Option<AnyArg>,
) -> Result<String, DocError> {
    let display = display.as_deref().filter(|s| !s.is_empty());
    let mut display_with_fallback = Cow::Borrowed(display.unwrap_or(api_name.as_str()));
    let api = api_name
        .replace(' ', "_")
        .replace("()", "")
        .replace(".prototype.", ".")
        .replace('.', "/");
    if api.is_empty() {
        return Err(DocError::ArgError(ArgError::MustBeProvided));
    }
    let (first_char_index, _) = api.char_indices().next().unwrap_or_default();
    let mut url = format!(
        "/{}/docs/Web/API/{}{}",
        env.locale.as_url_str(),
        &api[0..first_char_index].to_uppercase(),
        &api[first_char_index..],
    );
    if let Some(anchor) = anchor {
        if !anchor.is_empty() {
            if !anchor.starts_with('#') {
                url.push('#');
                display_with_fallback = Cow::Owned(format!("{}.{}", display_with_fallback, anchor));
            }
            url.push_str(&anchor);
            if let Some(anchor) = anchor.strip_prefix('#') {
                display_with_fallback = Cow::Owned(format!("{}.{}", display_with_fallback, anchor));
            }
        }
    }

    let code = !no_code.map(|nc| nc.as_bool()).unwrap_or_default();
    RariApi::link(
        &url,
        None,
        Some(&display_with_fallback),
        code,
        display,
        false,
    )
}
