pub mod csp;
pub mod cssxref;
pub mod domxref;
pub mod htmlxref;
pub mod http_header;
pub mod jsxref;
pub mod link;
pub mod rfc;
pub mod svgxref;

/*
use rari_types::AnyArg;
use rari_templ_func::rari_f;

use crate::docs::page::PageLike;
use crate::error::DocError;
use crate::templ::api::RariApi;

/// Creates a link to a page.
///
/// Parameters:
///  $0  Page link
#[rari_f]
pub fn doc_link(
    url: Option<String>,
    content: Option<String>,
    code: Option<bool>,
) -> Result<String, DocError> {
    let url = url.map(|url| format!("/{}/docs{url}", env.locale.as_url_str()));
    let url = url.as_deref().unwrap_or(env.url);
    let page = RariApi::get_page(url)?;
    link_internal(
        page.url(),
        &page,
        content.as_deref(),
        code.unwrap_or_default(),
    )
}
/// Creates a link to a page.
///
/// Parameters:
///  $0  Page link
#[rari_f]
pub fn link(
    url: Option<String>,
    content: Option<String>,
    code: Option<bool>,
) -> Result<String, DocError> {
    let url = url.as_deref().unwrap_or(env.url);
    let page = RariApi::get_page(url)?;
    link_internal(
        page.url(),
        &page,
        content.as_deref(),
        code.unwrap_or_default(),
    )
}

*/
