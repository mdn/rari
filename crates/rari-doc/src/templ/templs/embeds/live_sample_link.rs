use rari_templ_func::rari_f;
use rari_utils::concat_strs;

use crate::error::DocError;
use crate::templ::api::RariApi;

/// Creates a link to open a live sample in fullscreen mode.
/// 
/// This macro generates a link that opens an embedded live code sample in a fullscreen
/// view, providing users with a larger workspace to interact with and examine the example.
/// The link targets the live sample by its heading ID and displays custom link text.
/// 
/// # Arguments
/// * `id` - ID of the heading that contains the live sample to link to
/// * `display` - Display text for the link (e.g., "Open in fullscreen", "View full example")
/// 
/// # Examples
/// * `{{LiveSampleLink("Basic_example", "View full example")}}` -> creates fullscreen link for "Basic example" sample
/// * `{{LiveSampleLink("Interactive_demo", "Open in fullscreen")}}` -> link with "Open in fullscreen" text
/// * `{{LiveSampleLink("Complex_layout", "See full layout")}}` -> custom link text for complex examples
/// 
/// # Special handling
/// - Converts heading ID to anchor format for proper targeting
/// - Uses fragment identifier with "livesample_fullscreen=" parameter
/// - Creates accessible links with descriptive text
/// - Works in conjunction with EmbedLiveSample macro for complete functionality
#[rari_f(register = "crate::Templ")]
pub fn livesamplelink(id: String, display: String) -> Result<String, DocError> {
    let id = RariApi::anchorize(&id);
    Ok(concat_strs!(
        r##"<a href="#livesample_fullscreen="##,
        &id,
        r#"">"#,
        &display,
        "</a>"
    ))
}
