use rari_templ_func::rari_f;
use rari_types::AnyArg;

use crate::error::DocError;
use crate::helpers::l10n::l10n_json_data;

/// Creates a link to an IETF RFC (Request for Comments) document.
/// 
/// This macro generates links to RFC documents hosted on the IETF datatracker.
/// It supports linking to specific sections within an RFC and can include
/// custom descriptive content in the link text.
/// 
/// # Arguments
/// * `number` - The RFC number (e.g., 7231, 3986, 2616)
/// * `content` - Optional descriptive content to append to the link text
/// * `anchor` - Optional section number to link to a specific section
/// 
/// # Examples
/// * `{{RFC("7231")}}` -> links to "RFC 7231"
/// * `{{RFC("3986", "URI Generic Syntax")}}` -> links to "RFC 3986: URI Generic Syntax"
/// * `{{RFC("7231", "", "6.1")}}` -> links to "RFC 7231, section 6.1" with anchor
/// * `{{RFC("2616", "HTTP/1.1", "14.9")}}` -> links to "RFC 2616, section 14.9: HTTP/1.1"
/// 
/// # Special handling
/// - Links directly to https://datatracker.ietf.org/doc/html/rfc{number}
/// - Section anchors are formatted as `#section-{anchor}`
/// - Localizes the word "section" based on the current locale
/// - Combines content and section information intelligently in link text
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
