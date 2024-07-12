use rari_templ_func::rari_f;

use crate::{
    docs::page::PageLike,
    error::DocError,
    templ::{api::RariApi, templs::links::link::link_internal},
};

#[rari_f]
pub fn http_header(slug: String, content: Option<String>) -> Result<String, DocError> {
    let url = format!("/{}/docs/Web/HTTP/Headers/{slug}", env.locale.as_url_str());
    let page = RariApi::get_page(&url)?;
    link_internal(page.url(), &page, content.as_deref(), true)
}
