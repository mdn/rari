use std::borrow::Cow;

use lol_html::{RewriteStrSettings, element, rewrite_str};
use rari_md::anchor::anchorize;
use rari_types::fm_types::FeatureStatus;
use rari_types::locale::Locale;
use rari_utils::concat_strs;

use crate::error::DocError;
use crate::issues::get_issue_counter;
use crate::pages::page::{Page, PageLike};
use crate::redirects::resolve_redirect;
use crate::resolve::locale_from_url;
use crate::templ::api::RariApi;
use crate::templ::templs::badges::{write_deprecated, write_experimental, write_non_standard};

pub struct LinkModifier<'a> {
    pub badges: &'a [FeatureStatus],
    pub badge_locale: Locale,
    pub code: bool,
    pub only_en_us: bool,
}

pub fn render_internal_link(
    out: &mut String,
    url: &str,
    anchor: Option<&str>,
    content: &str,
    title: Option<&str>,
    modifier: &LinkModifier,
    checked: bool,
) -> Result<(), DocError> {
    out.push_str("<a href=\"");
    out.push_str(url);
    if let Some(anchor) = anchor {
        out.push('#');
        out.push_str(&anchorize(anchor));
    }
    out.push('"');
    if let Some(title) = title {
        out.extend([
            " title=\"",
            &html_escape::encode_quoted_attribute(title),
            "\"",
        ]);
    }
    if modifier.only_en_us {
        out.push_str(" class=\"only-in-en-us\"");
    }
    if checked {
        out.push_str(" data-templ-link");
    }
    out.push('>');
    if modifier.code {
        out.push_str("<code>");
    }
    out.push_str(content);
    if modifier.code {
        out.push_str("</code>");
    }
    if !modifier.badges.is_empty() {
        if modifier.badges.contains(&FeatureStatus::Experimental) {
            write_experimental(out, modifier.badge_locale)?;
        }
        if modifier.badges.contains(&FeatureStatus::NonStandard) {
            write_non_standard(out, modifier.badge_locale)?;
        }
        if modifier.badges.contains(&FeatureStatus::Deprecated) {
            write_deprecated(out, modifier.badge_locale)?;
        }
    }
    out.push_str("</a>");
    Ok(())
}

pub fn render_link_from_page(
    out: &mut String,
    page: &Page,
    modifier: &LinkModifier,
) -> Result<(), DocError> {
    let content = page.short_title().unwrap_or(page.title());
    let decoded_content = html_escape::decode_html_entities(content);
    let encoded_content = html_escape::encode_safe(&decoded_content);
    let content = if content != encoded_content {
        Cow::Owned(encoded_content.into_owned())
    } else {
        Cow::Borrowed(content)
    };
    render_internal_link(out, page.url(), None, &content, None, modifier, true)
}

#[derive(Clone, Copy)]
pub struct LinkFlags {
    pub code: bool,
    pub with_badges: bool,
    pub report: bool,
}

pub fn render_link_via_page(
    out: &mut String,
    link: &str,
    locale: Locale,
    content: Option<&str>,
    title: Option<&str>,
    LinkFlags {
        code,
        with_badges,
        report,
    }: LinkFlags,
) -> Result<(), DocError> {
    let mut url = Cow::Borrowed(link);
    if let Some(link) = link.strip_prefix('/') {
        if locale_from_url(&url).is_none() {
            url = Cow::Owned(concat_strs!("/", locale.as_url_str(), "/docs/", link));
        }
        let (url, anchor) = url.split_once('#').unwrap_or((&url, ""));
        if let Ok(page) = if report {
            RariApi::get_page(url)
        } else {
            RariApi::get_page_ignore_case(url)
        } {
            let url = page.url();
            let content = if let Some(content) = content {
                Cow::Borrowed(content)
            } else {
                let content = page.short_title().unwrap_or(page.title());
                let decoded_content = html_escape::decode_html_entities(content);
                let encoded_content = html_escape::encode_safe(&decoded_content);
                if content != encoded_content {
                    Cow::Owned(encoded_content.into_owned())
                } else {
                    Cow::Borrowed(content)
                }
            };
            return render_internal_link(
                out,
                url,
                if anchor.is_empty() {
                    None
                } else {
                    Some(anchor)
                },
                &content,
                title,
                &LinkModifier {
                    badges: if with_badges { page.status() } else { &[] },
                    badge_locale: locale,
                    code,
                    only_en_us: page.locale() == Locale::EnUs && locale != Locale::EnUs,
                },
                true,
            );
        }
    }

    out.push_str("<a data-templ-link href=\"");
    let content = match content {
        Some(content) => {
            let decoded_content = html_escape::decode_html_entities(content);
            let encoded_content = html_escape::encode_safe(&decoded_content);
            if content != encoded_content {
                Cow::Owned(encoded_content.into_owned())
            } else {
                Cow::Borrowed(content)
            }
        }
        None if url.starts_with('/') => {
            // Fall back to last url path segment.
            let clean_url = url.strip_suffix("/").unwrap_or(&url);
            let content = &clean_url[clean_url.rfind('/').map(|i| i + 1).unwrap_or(0)..];
            Cow::Borrowed(content)
        }
        _ => html_escape::encode_safe(&url),
    };
    out.push_str(&url);
    if let Some(title) = title {
        out.push_str("\" title=\"");
        out.push_str(&html_escape::encode_quoted_attribute(title));
    }
    out.push_str("\">");
    if code {
        out.push_str("<code>");
    }
    out.push_str(&content);
    if code {
        out.push_str("</code>");
    }
    out.push_str("</a>");
    Ok(())
}

/// Validate internal `<a>` links in template-generated HTML.
///
/// Some templates (e.g. `csssyntax`, `cssinfo`) emit `<a>` tags by hand
/// without going through `RariApi::get_page`, so they bypass the
/// `templ-broken-link` / `templ-redirected-link` reporting that other
/// link-producing templates get for free. Untagged links also reach
/// `fix_link` later, which would surface them as plain `broken-link` /
/// `redirected-link` issues without a markdown sourcepos.
///
/// This walks the rendered HTML, and for each internal `<a href="/...">`
/// that doesn't already carry `data-templ-link`:
/// - emits `templ-redirected-link` / `templ-ill-cased-link` if the URL
///   resolves through a redirect,
/// - emits `templ-broken-link` if the (resolved) URL doesn't exist,
/// - tags the element with `data-templ-link` so `fix_link` skips it.
///
/// Must be called from inside the surrounding `templ` tracing span so
/// the warnings pick up the macro name and source location.
pub fn post_process_templ_links(html: &str) -> Result<String, DocError> {
    let element_content_handlers = vec![element!("a[href]:not([data-templ-link])", |el| {
        let Some(href) = el.get_attribute("href") else {
            return Ok(());
        };
        if href.starts_with('/') {
            let href_no_hash = &href[..href.find('#').unwrap_or(href.len())];
            let resolved = resolve_redirect(href_no_hash);
            match resolved.as_deref() {
                Some(redirect) if href_no_hash.eq_ignore_ascii_case(redirect) => {
                    let ic = get_issue_counter();
                    tracing::warn!(
                        source = "templ-ill-cased-link",
                        ic = ic,
                        url = href.as_str(),
                        redirect = redirect,
                    );
                }
                Some(redirect) => {
                    let ic = get_issue_counter();
                    tracing::warn!(
                        source = "templ-redirected-link",
                        ic = ic,
                        url = href.as_str(),
                        redirect = redirect,
                    );
                }
                None => {}
            }
            let target = resolved.as_deref().unwrap_or(href_no_hash);
            if !Page::ignore_link_check(target) && !Page::exists_with_fallback(target) {
                let ic = get_issue_counter();
                tracing::warn!(source = "templ-broken-link", ic = ic, url = href.as_str());
            }
            el.set_attribute("data-templ-link", "")?;
        }
        Ok(())
    })];
    Ok(rewrite_str(
        html,
        RewriteStrSettings {
            element_content_handlers,
            ..Default::default()
        },
    )?)
}

#[cfg(test)]
mod tests {
    use super::post_process_templ_links;

    #[test]
    fn tags_internal_links_so_fix_link_skips_them() {
        // Before this helper existed, formal-syntax `<a>` tags reached
        // `fix_link` without `data-templ-link` and surfaced as plain
        // `broken-link`/`redirected-link` issues with no sourcepos. The
        // post-processor must now tag every untagged internal link.
        let input = r#"<a href="/en-US/docs/Web/CSS/Reference/Values/foo">foo</a>"#;
        let output = post_process_templ_links(input).unwrap();
        assert!(
            output.contains("data-templ-link"),
            "expected internal link to be tagged with `data-templ-link`, got: {output}"
        );
    }

    #[test]
    fn leaves_external_links_alone() {
        let input = r#"<a href="https://drafts.csswg.org/css-conditional-5/">spec</a>"#;
        let output = post_process_templ_links(input).unwrap();
        assert!(
            !output.contains("data-templ-link"),
            "external link should not be tagged: {output}"
        );
    }

    #[test]
    fn skips_already_tagged_links() {
        // Links emitted via `RariApi::link` / `render_internal_link` already
        // carry `data-templ-link` and have already been validated by the
        // template, so we mustn't re-process and double-warn.
        let input =
            r#"<a data-templ-link href="/en-US/docs/Web/CSS/Reference/Properties/color">x</a>"#;
        let output = post_process_templ_links(input).unwrap();
        // Still exactly one occurrence (we didn't add another).
        assert_eq!(output.matches("data-templ-link").count(), 1);
    }
}
