use rari_templ_func::rari_f;

use crate::docs::page::PageLike;
use crate::error::DocError;
use crate::templ::api::RariApi;
use crate::templ::templs::links::link::link_internal;

#[rari_f]
pub fn csp(directive: String) -> Result<String, DocError> {
    let url = format!(
        "/{}/docs/Web/HTTP/Headers/Content-Security-Policy/{directive}",
        env.locale.as_url_str()
    );
    let page = RariApi::get_page(&url)?;
    link_internal(page.url(), &page, Some(&directive), true)
}
