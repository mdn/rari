use rari_templ_func::rari_f;

use crate::error::DocError;
use crate::templ::api::RariApi;

#[rari_f]
pub fn mathmlxref(element_name: String) -> Result<String, DocError> {
    let element_name = element_name.to_lowercase();
    let display = format!("&lt;{element_name}&gt;");
    let url = format!(
        "/{}/docs/Web/MathML/Element/{}",
        env.locale.as_url_str(),
        element_name,
    );

    RariApi::link(&url, None, Some(&display), true, Some(&display), false)
}
