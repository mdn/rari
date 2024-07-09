use std::borrow::Cow;

use rari_types::fm_types::FeatureStatus;
use rari_types::locale::Locale;

use crate::docs::page::{Page, PageLike};
use crate::error::DocError;
use crate::templ::api::RariApi;
use crate::templ::macros::badges::{write_deprecated, write_experimental, write_non_standard};

pub struct LinkModifier<'a> {
    pub badges: &'a [FeatureStatus],
    pub badge_locale: Locale,
    pub code: bool,
    pub only_en_us: bool,
}

pub fn render_internal_link(
    out: &mut String,
    url: &str,
    content: &str,
    title: Option<&str>,
    modifier: &LinkModifier,
) -> Result<(), DocError> {
    out.push_str("<a href=\"");
    out.push_str(url);
    if let Some(title) = title {
        out.push_str("\" title=\"");
        out.push_str(title);
    }
    if modifier.only_en_us {
        out.push_str("\" class=\"only-in-en-us")
    }
    out.push_str("\">");
    if modifier.code {
        out.push_str("<code>");
    }
    let content = html_escape::encode_safe(content);
    out.push_str(&content);
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
    let content = html_escape::encode_safe(page.short_title().unwrap_or(page.title()));
    render_internal_link(out, page.url(), &content, None, modifier)
}

pub fn render_link_via_page(
    out: &mut String,
    link: &str,
    locale: Option<Locale>,
    content: Option<&str>,
    code: bool,
    title: Option<&str>,
    with_badges: bool,
) -> Result<(), DocError> {
    if link.starts_with('/') {
        let url = if let Some(locale) = locale {
            Cow::Owned(format!("/{}/docs{link}", locale.as_url_str()))
        } else {
            Cow::Borrowed(link)
        };
        let page = RariApi::get_page(&url)?;
        let url = page.url();
        let content = content.unwrap_or(page.short_title().unwrap_or(page.title()));
        render_internal_link(
            out,
            url,
            content,
            title,
            &LinkModifier {
                badges: if with_badges { page.status() } else { &[] },
                badge_locale: locale.unwrap_or(page.locale()),
                code,
                only_en_us: page.locale() != locale.unwrap_or_default(),
            },
        )?;
    } else {
        out.push_str("<a href=\"");
        let url = link;
        let content = html_escape::encode_safe(content.unwrap_or(link));
        out.push_str(url);
        if let Some(title) = title {
            out.push_str("\" title=\"");
            out.push_str(title);
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
    }
    Ok(())
}
