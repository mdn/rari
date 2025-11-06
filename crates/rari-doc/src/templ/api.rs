use std::borrow::Cow;

use percent_encoding::utf8_percent_encode;
use rari_md::anchor::anchorize;
use rari_types::globals::{deny_warnings, settings};
use rari_types::locale::Locale;

use crate::error::DocError;
use crate::html::links::{LinkFlags, render_link_via_page};
use crate::issues::get_issue_counter;
use crate::pages::page::Page;
use crate::percent::PATH_SEGMENT;
use crate::redirects::resolve_redirect;

enum LinkWarn {
    No,
    All,
    IgnoreCase,
}

pub struct RariApi {}
impl RariApi {
    pub fn anchorize(content: &str) -> Cow<'_, str> {
        anchorize(content)
    }

    pub fn live_sample_base_url() -> &'static str {
        &settings().live_samples_base_url
    }
    pub fn get_page_nowarn(url: &str) -> Result<Page, DocError> {
        RariApi::get_page_internal(url, LinkWarn::No)
    }

    pub fn get_page_ignore_case(url: &str) -> Result<Page, DocError> {
        RariApi::get_page_internal(url, LinkWarn::IgnoreCase)
    }

    pub fn get_page(url: &str) -> Result<Page, DocError> {
        RariApi::get_page_internal(url, LinkWarn::All)
    }

    fn get_page_internal(url: &str, warn: LinkWarn) -> Result<Page, DocError> {
        let redirect = resolve_redirect(url);
        let url = match redirect.as_ref() {
            Some(redirect) => {
                if !matches!(warn, LinkWarn::No) {
                    let ill_cased = url.to_lowercase() == redirect.to_lowercase();
                    match warn {
                        LinkWarn::All | LinkWarn::IgnoreCase if !ill_cased => {
                            let ic = get_issue_counter();
                            tracing::warn!(
                                source = "templ-redirected-link",
                                ic = ic,
                                url = url,
                                href = redirect.as_ref()
                            );
                        }
                        LinkWarn::All if ill_cased => {
                            let ic = get_issue_counter();
                            tracing::warn!(
                                source = "ill-cased-link",
                                ic = ic,
                                url = url,
                                redirect = redirect.as_ref()
                            );
                        }
                        _ => {}
                    }
                }
                if deny_warnings() {
                    return Err(DocError::RedirectedLink {
                        from: url.to_string(),
                        to: redirect.to_string(),
                    });
                } else {
                    redirect
                }
            }
            None => url,
        };
        Page::from_url_with_fallback(url).map_err(|e| {
            if let DocError::PageNotFound(_, _) = e
                && !matches!(warn, LinkWarn::No)
            {
                let ic = get_issue_counter();
                tracing::warn!(source = "templ-broken-link", ic = ic, url = url);
            }
            e
        })
    }

    pub fn decode_uri_component(component: &str) -> String {
        utf8_percent_encode(component, PATH_SEGMENT).to_string()
    }

    pub fn interactive_examples_base_url() -> &'static str {
        &settings().interactive_examples_base_url
    }

    /**
     * Renders a link to a page. If a locale is provided, it will be used to generate the link.
     * If no locale is provided, either the locale implicit in the link or the default locale will be used.
     *
     * @param link The URL of the page.
     * @param locale The optional locale of the page.
     * @param content The content of the link.
     * @param code Whether the link is for code.
     * @param title The title of the link.
     * @param with_badges Whether to include badges.
     * @return The rendered link.
     */
    pub fn link(
        link: &str,
        locale: Option<Locale>,
        content: Option<&str>,
        code: bool,
        title: Option<&str>,
        with_badges: bool,
    ) -> Result<String, DocError> {
        let mut out = String::new();
        let (url, locale) = if let Some(locale) = locale {
            (link.strip_prefix("/en-US/docs").unwrap_or(link), locale)
        } else {
            (link, Locale::default())
        };
        render_link_via_page(
            &mut out,
            url,
            locale,
            content,
            title,
            LinkFlags {
                code,
                with_badges,
                report: false,
            },
        )?;
        Ok(out)
    }
}
