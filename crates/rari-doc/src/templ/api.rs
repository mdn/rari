use percent_encoding::utf8_percent_encode;
use rari_md::anchor::anchorize;
use rari_types::globals::{deny_warnings, settings};
use rari_types::locale::Locale;

use crate::docs::page::Page;
use crate::error::DocError;
use crate::html::links::render_link_via_page;
use crate::percent::PATH_SEGMENT;
use crate::redirects::resolve_redirect;

pub struct RariApi {}
impl RariApi {
    pub fn anchorize(content: &str) -> String {
        anchorize(content)
    }

    pub fn live_sample_base_url() -> &'static str {
        &settings().live_samples_base_url
    }
    pub fn get_page(url: &str) -> Result<Page, DocError> {
        let redirect = resolve_redirect(url);
        let url = match redirect.as_ref() {
            Some(redirect) if deny_warnings() => {
                return Err(DocError::RedirectedLink {
                    from: url.to_string(),
                    to: redirect.to_string(),
                })
            }
            Some(redirect) => redirect,
            None => url,
        };
        Page::page_from_url_path(url).map_err(Into::into)
    }

    pub fn decode_uri_component(component: &str) -> String {
        utf8_percent_encode(component, PATH_SEGMENT).to_string()
    }

    pub fn interactive_examples_base_url() -> &'static str {
        &settings().interactive_examples_base_url
    }

    pub fn link(
        link: &str,
        locale: Option<Locale>,
        content: Option<&str>,
        code: bool,
        title: Option<&str>,
        with_badge: bool,
    ) -> Result<String, DocError> {
        let mut out = String::new();
        render_link_via_page(&mut out, link, locale, content, code, title, with_badge)?;
        Ok(out)
    }
}
