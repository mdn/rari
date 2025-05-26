use rari_templ_func::rari_f;
use rari_types::AnyArg;

use crate::error::DocError;
use crate::helpers::l10n::l10n_json_data;

#[rari_f(register = "crate::Templ")]
pub fn rfc(
    number: AnyArg,
    content: Option<String>,
    anchor: Option<AnyArg>,
) -> Result<String, DocError> {
    let (content, anchor): (String, String) = match (content, anchor) {
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
                ", {} {anchor}: {content}",
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
