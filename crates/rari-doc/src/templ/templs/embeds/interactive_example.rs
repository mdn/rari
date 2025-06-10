use html_escape::encode_double_quoted_attribute;
use rari_templ_func::rari_f;
use rari_utils::concat_strs;

use crate::error::DocError;
use crate::helpers::l10n::l10n_json_data;
use crate::templ::api::RariApi;

/// Creates an interactive example element for hands-on code demonstrations.
/// 
/// This macro generates an `<interactive-example>` custom element that provides
/// interactive code examples for learning web technologies. It creates a localized
/// section heading and embeds the interactive component with optional height customization.
/// 
/// # Arguments
/// * `name` - Descriptive name of the interactive example (displayed in heading)
/// * `height` - Optional height class for the interactive element ("shorter", "taller", etc.)
/// 
/// # Examples
/// * `{{InteractiveExample("JavaScript Demo: Array.from()")}}` -> basic interactive example
/// * `{{InteractiveExample("CSS Flexbox Layout", "taller")}}` -> with custom height
/// * `{{InteractiveExample("Web API: Fetch", "shorter")}}` -> with shorter height
/// 
/// # Special handling
/// - Generates localized "Try it" section heading with proper anchor ID
/// - HTML-escapes the example name for security and proper display
/// - Creates accessible heading structure for screen readers
/// - Supports custom height classes for different types of content
/// - Uses semantic HTML with proper heading hierarchy
#[rari_f(register = "crate::Templ")]
pub fn interactiveexample(name: String, height: Option<String>) -> Result<String, DocError> {
    let title = l10n_json_data("Template", "interactive_example_cta", env.locale)?;
    let id = RariApi::anchorize(title);

    let height = height
        .map(|height| {
            concat_strs!(
                r#" height=""#,
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
