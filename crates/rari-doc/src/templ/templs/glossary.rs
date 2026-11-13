use rari_templ_func::rari_f;

use crate::error::DocError;
use crate::templ::api::RariApi;
use crate::utils::dedup_whitespace;

#[rari_f(register = "crate::Templ")]
pub fn glossary(term: String, label: Option<String>) -> Result<String, DocError> {
    let url = format!("/Glossary/{}", dedup_whitespace(&term).replace(' ', "_"));
    RariApi::link(
        &url,
        Some(env.locale),
        Some(&label.unwrap_or(term)),
        false,
        None,
        false,
    )
}
