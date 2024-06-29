use std::borrow::Cow;
use std::path::PathBuf;

use percent_encoding::utf8_percent_encode;
use rari_l10n::l10n_json_data;
use rari_md::anchor::anchorize;
use rari_types::globals::{deny_warnings, settings};
use rari_types::locale::Locale;

use crate::docs::page::{Page, PageLike, PageReader};
use crate::error::DocError;
use crate::html::links::render_link_via_page;
use crate::percent::PATH_SEGMENT;
use crate::redirects::resolve_redirect;
use crate::utils::COLLATOR;
use crate::walker::walk_builder;

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

    pub fn get_sub_pages(url: &str, depth: Option<usize>) -> Result<Vec<Page>, DocError> {
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
        let doc = Page::page_from_url_path(url)?;
        if let Some(folder) = doc.full_path().parent() {
            let sub_folders = walk_builder(&[folder], None)?
                .max_depth(depth.map(|i| i + 1))
                .build()
                .filter_map(|f| f.ok())
                .filter(|f| f.file_type().map(|ft| ft.is_file()).unwrap_or(false))
                .map(|f| f.into_path())
                .collect::<Vec<PathBuf>>();

            let mut sub_pages = sub_folders
                .iter()
                .map(Page::read)
                .collect::<Result<Vec<_>, DocError>>()?;
            sub_pages.sort_by(|a, b| COLLATOR.with(|c| c.compare(a.title(), b.title())));
            return Ok(sub_pages);
        }
        Ok(vec![])
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
        if let Err(DocError::IOError(_)) =
            render_link_via_page(&mut out, link, locale, content, code, title, with_badge)
        {
            let title_for_missing_page =
                l10n_json_data("Common", "summary", locale.unwrap_or_default()).unwrap_or_default();
            let content = content.unwrap_or(link);
            let content = if code {
                Cow::Owned(format!("<code>{}</code>", content))
            } else {
                Cow::Borrowed(content)
            };
            return Ok(format!(
                r#"<a class="page-not-created" title="{title_for_missing_page}">{content}</a>"#
            ));
        }

        Ok(out)
    }
}
