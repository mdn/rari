use rari_l10n::l10n_json_data;
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

/// Crates a link to a CSP header page.
#[rari_f]
pub fn csp(directive: String) -> Result<String, DocError> {
    let url = format!(
        "/{}/docs/Web/HTTP/Headers/Content-Security-Policy/{directive}",
        env.locale.as_url_str()
    );
    let page = RariApi::get_page(&url)?;
    link_internal(page.url(), &page, Some(&directive), true)
}

/// Crates a link to a HTTP header page.
#[rari_f]
pub fn http_header(slug: String, content: Option<String>) -> Result<String, DocError> {
    let url = format!("/{}/docs/Web/HTTP/Headers/{slug}", env.locale.as_url_str());
    let page = RariApi::get_page(&url)?;
    link_internal(page.url(), &page, content.as_deref(), true)
}

#[rari_f]
pub fn rfc(
    number: i64,
    content: Option<String>,
    anchor: Option<String>,
) -> Result<String, DocError> {
    let content = content.and_then(|c| if c.is_empty() { None } else { Some(c) });
    let anchor = anchor.and_then(|a| if a.is_empty() { None } else { Some(a) });
    let (content, anchor): (String, String) = match (content, anchor) {
        (None, None) => Default::default(),
        (None, Some(anchor)) => (
            format!(
                ", {} {anchor}",
                l10n_json_data("Common", "section", &env.locale).unwrap_or("Section")
            ),
            format!("#section-{anchor}"),
        ),
        (Some(content), None) => (format!(": {content}"), Default::default()),
        (Some(content), Some(anchor)) => (
            format!(
                ": {content}, {} {anchor}",
                l10n_json_data("Common", "section", &env.locale).unwrap_or("Section")
            ),
            format!("#section-{anchor}"),
        ),
    };
    Ok(format!(
        r#"<a href="https://datatracker.ietf.org/doc/html/rfc{number}{anchor}">RFC {number}{content}</a>"#
    ))
}

pub fn link_internal(
    url: &str,
    page: &impl PageLike,
    content: Option<&str>,
    code: bool,
) -> Result<String, DocError> {
    let content = content.unwrap_or(page.short_title().unwrap_or(page.title()));
    Ok(if code {
        format!(r#"<a href="{url}"><code>{content}</code></a>"#)
    } else {
        format!(r#"<a href="{url}">{content}</a>"#)
    })
}
#[cfg(test)]
mod test {

    use crate::error::DocError;
    use crate::templ::render::render;

    #[test]
    fn test_link() -> Result<(), DocError> {
        let env = rari_types::RariEnv {
            ..Default::default()
        };
        let out = render(&env, r#"{{ link("/en-US/docs/basic") }}"#)?;
        assert_eq!(out, r#"<a href="/en-US/docs/Basic">The Basic Page</a>"#);
        Ok(())
    }
}
