use std::borrow::Cow;

use rari_types::fm_types::FeatureStatus;
use rari_types::locale::Locale;

use crate::docs::page::PageLike;
use crate::error::DocError;
use crate::templ::api::RariApi;
use crate::templ::macros::badges::{write_deprecated, write_experimental, write_non_standard};

pub fn render_link(
    out: &mut String,
    link: &str,
    locale: Option<&Locale>,
    content: Option<&str>,
    code: bool,
    title: Option<&str>,
    with_badges: bool,
) -> Result<(), DocError> {
    out.push_str("<a href=\"");
    if link.starts_with('/') {
        let url = if let Some(locale) = locale {
            Cow::Owned(format!("/{}/docs{link}", locale.as_url_str()))
        } else {
            Cow::Borrowed(link)
        };
        let page = RariApi::get_page(&url)?;
        let url = page.url();
        let content =
            html_escape::encode_safe(content.unwrap_or(page.short_title().unwrap_or(page.title())));
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
        if with_badges {
            if page.status().contains(&FeatureStatus::Experimental) {
                write_experimental(out, &page.locale())?;
            }
            if page.status().contains(&FeatureStatus::NonStandard) {
                write_non_standard(out, &page.locale())?;
            }
            if page.status().contains(&FeatureStatus::Deprecated) {
                write_deprecated(out, &page.locale())?;
            }
        }
    } else {
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
