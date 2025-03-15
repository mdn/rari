use rari_templ_func::rari_f;

use crate::error::DocError;
use crate::templ::api::RariApi;

#[rari_f]
pub fn svgattr(name: String) -> Result<String, DocError> {
    let url = format!(
        "/{}/docs/Web/SVG/Reference/Attribute/{}",
        env.locale.as_url_str(),
        name,
    );

    RariApi::link(&url, env.locale, Some(&name), true, None, false)
}
