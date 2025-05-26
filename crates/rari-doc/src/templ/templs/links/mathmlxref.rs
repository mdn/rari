use rari_templ_func::rari_f;
use rari_utils::concat_strs;

use crate::error::DocError;
use crate::templ::api::RariApi;

#[rari_f(crate::Templ)]
pub fn mathmlxref(element_name: String) -> Result<String, DocError> {
    let element_name = element_name.to_lowercase();
    let display = concat_strs!("&lt;", element_name.as_str(), "&gt;");
    let title = concat_strs!("<", element_name.as_str(), ">");
    let url = concat_strs!(
        "/",
        env.locale.as_url_str(),
        "/docs/Web/MathML/Reference/Element/",
        element_name.as_str()
    );

    RariApi::link(&url, env.locale, Some(&display), true, Some(&title), false)
}
