use rari_templ_func::rari_f;

use crate::error::DocError;
use crate::templ::api::RariApi;
use crate::utils::dedup_ws;

#[rari_f]
pub fn glossary(term_name: String, display_name: Option<String>) -> Result<String, DocError> {
    let url = format!("/Glossary/{}", dedup_ws(&term_name).replace(' ', "_"));
    RariApi::link(
        &url,
        Some(env.locale),
        Some(&display_name.unwrap_or(term_name)),
        false,
        None,
        false,
    )
}
