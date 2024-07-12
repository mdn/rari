use crate::{docs::page::PageLike, error::DocError};

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
