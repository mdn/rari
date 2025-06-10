use std::fmt::Write;

use rari_templ_func::rari_f;
use rari_types::AnyArg;

use crate::error::DocError;

/// Embeds a live code sample from the MDN GitHub repository.
/// 
/// This macro creates an iframe that displays live code examples hosted on
/// mdn.github.io. These are typically complete, runnable examples that demonstrate
/// web technologies and APIs in action, hosted directly from the MDN organization's
/// GitHub repositories.
/// 
/// # Arguments
/// * `path` - Path to the example on mdn.github.io (e.g., "dom-examples/web-audio-api/basic/")
/// * `width` - Optional width for the iframe (in pixels or CSS units)
/// * `height` - Optional height for the iframe (in pixels or CSS units)
/// 
/// # Examples
/// * `{{EmbedGHLiveSample("dom-examples/web-audio-api/basic/")}}` -> embeds Web Audio API example
/// * `{{EmbedGHLiveSample("css-examples/flexbox/", "100%", "400")}}` -> with custom dimensions
/// * `{{EmbedGHLiveSample("webextensions-examples/menu-demo/", "800", "600")}}` -> WebExtension example
/// 
/// # Special handling
/// - Links directly to https://mdn.github.io/{path} for live examples
/// - Uses standard iframe embedding without additional security restrictions
/// - Allows custom sizing for different types of examples
/// - Provides direct access to fully functional web applications and demos
#[rari_f(register = "crate::Templ")]
pub fn embedghlivesample(
    path: String,
    width: Option<AnyArg>,
    height: Option<AnyArg>,
) -> Result<String, DocError> {
    let mut out = String::new();
    out.push_str("<iframe ");
    if let Some(width) = width {
        write!(&mut out, r#"width="{}" "#, width)?;
    }
    if let Some(height) = height {
        write!(&mut out, r#"height="{}" "#, height)?;
    }

    out.extend([
        r#"src="https://mdn.github.io/"#,
        path.as_str(),
        r#""></iframe>"#,
    ]);
    Ok(out)
}
