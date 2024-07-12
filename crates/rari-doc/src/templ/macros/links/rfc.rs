use rari_l10n::l10n_json_data;
use rari_templ_func::rari_f;
use rari_types::AnyArg;

use crate::error::DocError;

#[rari_f]
pub fn rfc(
    number: AnyArg,
    content: Option<String>,
    anchor: Option<AnyArg>,
) -> Result<String, DocError> {
    let content = content.and_then(|c| if c.is_empty() { None } else { Some(c) });
    let anchor_str = anchor.and_then(|a| if a.is_empty() { None } else { Some(a) });
    let (content, anchor): (String, String) = match (content, anchor_str) {
        (None, None) => Default::default(),
        (None, Some(anchor)) => (
            format!(
                ", {} {anchor}",
                l10n_json_data("Common", "section", env.locale)?
            ),
            format!("#section-{anchor}"),
        ),
        (Some(content), None) => (format!(": {content}"), Default::default()),
        (Some(content), Some(anchor)) => (
            format!(
                ": {content}, {} {anchor}",
                l10n_json_data("Common", "section", env.locale)?
            ),
            format!("#section-{anchor}"),
        ),
    };
    let number = number.as_int();
    Ok(format!(
        r#"<a href="https://datatracker.ietf.org/doc/html/rfc{number}{anchor}">RFC {number}{content}</a>"#
    ))
}
