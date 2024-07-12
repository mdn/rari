use rari_templ_func::rari_f;
use rari_types::AnyArg;

use crate::error::DocError;
use crate::templ::api::RariApi;

#[rari_f]
pub fn svgxref(element_name: String, _: Option<AnyArg>) -> Result<String, DocError> {
    let display = format!("<{element_name}>");
    let url = format!(
        "/{}/docs/Web/SVG/Element/{}",
        env.locale.as_url_str(),
        element_name,
    );

    RariApi::link(&url, None, Some(display.as_ref()), true, None, false)
}
