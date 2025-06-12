use std::fmt::Write;

use rari_templ_func::rari_f;
use rari_types::AnyArg;

use crate::error::DocError;

/// Embeds a JSFiddle code example in an iframe.
///
/// This macro creates an iframe that displays interactive code examples from JSFiddle,
/// allowing users to view and experiment with HTML, CSS, and JavaScript code directly
/// in the browser. The embed supports various display options and custom sizing.
///
/// # Arguments
/// * `url` - Base JSFiddle URL (e.g., "https://jsfiddle.net/username/fiddle_id")
/// * `options` - Optional display options for the embed (e.g., "js,html,css,result" or "result")
/// * `height` - Optional height for the iframe (in pixels)
///
/// # Examples
/// * `{{JSFiddleEmbed("https://jsfiddle.net/user/abc123")}}` -> basic JSFiddle embed
/// * `{{JSFiddleEmbed("https://jsfiddle.net/user/abc123", "result", "400")}}` -> shows only result with custom height
/// * `{{JSFiddleEmbed("https://jsfiddle.net/user/abc123", "js,result")}}` -> shows JavaScript and result tabs
///
/// # Special handling
/// - Automatically appends "/embedded/" to the JSFiddle URL for proper embedding
/// - Supports JSFiddle's tab options (js, html, css, result) in the options parameter
/// - Uses standard width (756px) with customizable height
/// - Includes allowfullscreen attribute for better user experience
/// - Wraps iframe in paragraph tags for proper content flow
#[rari_f(register = "crate::Templ")]
pub fn jsfiddleembed(
    url: String,
    options: Option<String>,
    height: Option<AnyArg>,
) -> Result<String, DocError> {
    let mut out = String::new();
    out.push_str(r#"<p><iframe allowfullscreen="allowfullscreen" width="756" "#);
    if let Some(height) = height {
        write!(&mut out, r#"height="{}" "#, height)?;
    }
    out.extend([
        r#"src=""#,
        url.as_str(),
        "embedded/",
        options.as_deref().unwrap_or_default(),
        if options.as_ref().map(|s| !s.is_empty()).unwrap_or_default() {
            "/"
        } else {
            ""
        },
        r#""></iframe></p>"#,
    ]);
    Ok(out)
}
