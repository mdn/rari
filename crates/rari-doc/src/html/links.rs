use std::borrow::Cow;

use rari_md::anchor::anchorize;
use rari_types::fm_types::FeatureStatus;
use rari_types::locale::Locale;
use rari_utils::concat_strs;
use tracing::warn;

use crate::error::DocError;
use crate::pages::page::{Page, PageLike};
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
) -> Result<(), DocError> {
    out.push_str("<a href=\"");
    out.push_str(url);
    if let Some(anchor) = anchor {
        out.push('#');
        out.push_str(&anchorize(anchor));
    }
    if let Some(title) = title {
        out.push_str("\" title=\"");
        out.push_str(&html_escape::encode_quoted_attribute(title));
    }
    if modifier.only_en_us {
        out.push_str("\" class=\"only-in-en-us")
    }
    out.push_str("\">");
    if modifier.code {
        out.push_str("<code>");
    }
    out.push_str(content);
    if modifier.code {
        out.push_str("</code>");
    }
    out.push_str("</a>");
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
    render_internal_link(out, page.url(), None, &content, None, modifier)
}

pub fn render_link_via_page(
    out: &mut String,
    link: &str,
    locale: Locale,
    content: Option<&str>,
    code: bool,
    title: Option<&str>,
    with_badges: bool,
) -> Result<(), DocError> {
    let mut url = Cow::Borrowed(link);
    if let Some(link) = link.strip_prefix('/') {
        if locale_from_url(&url).is_none() {
            url = Cow::Owned(concat_strs!("/", locale.as_url_str(), "/docs/", link));
        }
        let (url, anchor) = url.split_once('#').unwrap_or((&url, ""));
        match RariApi::get_page(url) {
            Ok(page) => {
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
                );
            }
            Err(e) => {
                if !Page::ignore_link_check(url) {
                    warn!(source = "broken-link", url = url,)
                }
            }
        }
    }

    out.push_str("<a href=\"");
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
