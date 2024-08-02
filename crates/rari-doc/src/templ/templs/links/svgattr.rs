use rari_templ_func::rari_f;

use crate::error::DocError;
use crate::templ::api::RariApi;

#[rari_f]
pub fn svgattr(name: String) -> Result<String, DocError> {
    let url = format!(
        "/{}/docs/Web/SVG/Attribute/{}",
        env.locale.as_url_str(),
        name,
    );

    RariApi::link(&url, None, Some(&name), true, None, false)
}
