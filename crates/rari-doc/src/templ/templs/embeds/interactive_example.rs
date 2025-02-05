use html_escape::encode_double_quoted_attribute;
use rari_templ_func::rari_f;
use rari_utils::concat_strs;

use crate::error::DocError;
use crate::helpers::l10n::l10n_json_data;
use crate::templ::api::RariApi;

/// Adds an <interactive-example> element to the content
///
/// Parameters:
///  $0 - Name of interactive example
///  $1 - Optional custom height class to set on interactive-example element
///
///  Example call {{InteractiveExample("JavaScript Demo: Array.from()", "taller")}}
#[rari_f]
pub fn interactive_example(name: String, height: Option<String>) -> Result<String, DocError> {
    let title = l10n_json_data("Template", "interactive_example_cta", env.locale)?;
    let id = RariApi::anchorize(title);

    let height = height
        .map(|height| {
            concat_strs!(
                r#" height="#,
                &encode_double_quoted_attribute(&height).as_ref(),
                r#"""#
            )
        })
        .unwrap_or_default();
    Ok(concat_strs!(
        r#"<h2 id=""#,
        &id,
        r#"">"#,
        title,
        "</h2>\n",
        r#"<interactive-example name=""#,
        encode_double_quoted_attribute(&name).as_ref(),
        r#"""#,
        &height,
        r#"></interactive-example>"#
    ))
}
