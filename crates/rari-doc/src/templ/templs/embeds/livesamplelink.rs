use rari_templ_func::rari_f;
use rari_utils::concat_strs;

use crate::error::DocError;
use crate::templ::api::RariApi;

#[allow(clippy::too_many_arguments)]
#[rari_f]
pub fn live_sample_link(id: String, display: String) -> Result<String, DocError> {
    let id = RariApi::anchorize(&id);
    Ok(concat_strs!(
        r##"<a href="#livesample_fullscreen="##,
        &id,
        r#"">"#,
        &display,
        "</a>"
    ))
}
