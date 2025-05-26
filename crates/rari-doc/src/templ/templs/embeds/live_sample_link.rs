use rari_templ_func::rari_f;
use rari_utils::concat_strs;

use crate::error::DocError;
use crate::templ::api::RariApi;

#[rari_f(register = "crate::Templ")]
pub fn livesamplelink(id: String, display: String) -> Result<String, DocError> {
    let id = RariApi::anchorize(&id);
    Ok(concat_strs!(
        r##"<a href="#livesample_fullscreen="##,
        &id,
        r#"">"#,
        &display,
        "</a>"
    ))
}
