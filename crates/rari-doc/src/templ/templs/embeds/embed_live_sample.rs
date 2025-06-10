use std::fmt::Write;

use rari_templ_func::rari_f;
use rari_types::AnyArg;

use crate::error::DocError;
use crate::issues::get_issue_counter;
use crate::templ::api::RariApi;
use crate::utils::dedup_whitespace;

/// Embeds a live code sample from the current page in an iframe.
/// 
/// This macro creates an interactive iframe that displays the result of HTML, CSS, 
/// and JavaScript code blocks found in the current page. The code blocks are identified
/// by their heading ID and combined to create a runnable example.
/// 
/// # Arguments
/// * `id` - Optional ID of the heading that contains the code blocks to embed
/// * `width` - Optional width for the iframe (in pixels or CSS units)
/// * `height` - Optional height for the iframe (minimum 60px enforced)
/// * `_deprecated_3` - Deprecated parameter (no longer used)
/// * `_deprecated_4` - Deprecated parameter (no longer used) 
/// * `_deprecated_5` - Deprecated parameter (no longer used)
/// * `allowed_features` - Optional iframe `allow` attribute for feature policy
/// * `sandbox` - Optional additional sandbox restrictions ("allow-modals", "allow-forms", "allow-popups")
/// 
/// # Examples
/// * `{{EmbedLiveSample("Basic_example")}}` -> embeds code under "Basic example" heading
/// * `{{EmbedLiveSample("Demo", "100%", "300")}}` -> with custom width and height
/// * `{{EmbedLiveSample("Interactive", "", "400", "", "", "", "camera; microphone")}}` -> with feature policy
/// * `{{EmbedLiveSample("Form_demo", "", "", "", "", "", "", "allow-forms")}}` -> with sandbox permissions
/// 
/// # Special handling
/// - Converts heading ID to anchor format for iframe targeting
/// - Enforces minimum height of 60px for usability
/// - Applies secure sandbox by default with "allow-same-origin allow-scripts"
/// - Validates sandbox attributes for security (only allows safe options)
/// - Uses lazy loading for performance optimization
/// - Generates accessible title from the heading ID
#[allow(clippy::too_many_arguments)]
#[rari_f(register = "crate::Templ")]
pub fn embedlivesample(
    id: Option<String>,
    width: Option<AnyArg>,
    height: Option<AnyArg>,
    _deprecated_3: Option<String>,
    _deprecated_4: Option<String>,
    _deprecated_5: Option<String>,
    allowed_features: Option<String>,
    sandbox: Option<String>,
) -> Result<String, DocError> {
    let id = id.unwrap_or_default();
    let title = dedup_whitespace(&id.replace('_', " "));
    let id = RariApi::anchorize(&id);
    let mut out = String::new();
    out.extend([
        r#"<div class="code-example"><div class="example-header"></div><iframe class="sample-code-frame" title=""#,
        &html_escape::encode_quoted_attribute(&title),
        r#" sample" id="frame_"#,
        &id,
        r#"" "#
    ]);
    if let Some(width) = width {
        write!(&mut out, r#"width="{}" "#, width)?;
    }
    if let Some(height) = height {
        // TODO: fix this
        if height.as_int() < 60 {
            write!(&mut out, r#"height="60" "#)?;
        } else {
            write!(&mut out, r#"height="{}" "#, height)?;
        }
    }
    out.extend([
        r#"src="about:blank" data-live-path=""#,
        env.url,
        if env.url.ends_with('/') { "" } else { "/" },
        r#"" data-live-id=""#,
        &id,
        r#"" "#,
    ]);
    if let Some(allowed_features) = allowed_features {
        write!(&mut out, r#"allow="{}" "#, allowed_features)?;
    }
    out.push_str(r#"sandbox=""#);
    if let Some(sandbox) = sandbox {
        let is_sane = sandbox.split_ascii_whitespace().all(|attr| {
            if matches!(attr, "allow-modals" | "allow-forms" | "allow-popups") {
                true
            } else {
                let ic = get_issue_counter();
                tracing::warn!(source = "templ-invalid-arg", ic = ic, arg = attr);
                false
            }
        });
        if is_sane {
            out.extend([&sandbox, " "]);
        }
    }
    out.push_str(r#"allow-same-origin allow-scripts" loading="lazy"></iframe></div>"#);
    Ok(out)
}
