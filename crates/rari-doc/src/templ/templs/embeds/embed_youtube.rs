use std::borrow::Cow;

use rari_templ_func::rari_f;
use rari_utils::concat_strs;

use crate::error::DocError;

/// Embeds a YouTube video in an iframe with privacy-enhanced mode.
/// 
/// This macro creates a responsive YouTube iframe embed using the privacy-enhanced
/// youtube-nocookie.com domain, which provides better privacy protection by not
/// setting tracking cookies until the user interacts with the video.
/// 
/// # Arguments
/// * `video_id` - YouTube video ID (the part after "v=" in YouTube URLs)
/// * `title` - Optional descriptive title for accessibility (defaults to "YouTube video")
/// 
/// # Examples
/// * `{{EmbedYouTube("dQw4w9WgXcQ")}}` -> embeds video with default title
/// * `{{EmbedYouTube("dQw4w9WgXcQ", "Rick Astley - Never Gonna Give You Up")}}` -> with custom title
/// 
/// # Special handling
/// - Uses youtube-nocookie.com for enhanced privacy protection
/// - Sets standard video dimensions (560x315) for optimal display
/// - Enables modern YouTube features via allow attribute:
///   - accelerometer, autoplay, clipboard-write, encrypted-media, gyroscope, picture-in-picture
/// - Includes allowfullscreen attribute for fullscreen video playback
/// - HTML-escapes the title attribute for security and accessibility
#[rari_f(register = "crate::Templ")]
pub fn embedyoutube(video_id: String, title: Option<String>) -> Result<String, DocError> {
    let title = title
        .as_deref()
        .map(|s| html_escape::encode_double_quoted_attribute(s))
        .unwrap_or(Cow::Borrowed("YouTube video"));
    Ok(concat_strs!(
        r#"<iframe width="560" height="315" "#,
        r#"src="https://www.youtube-nocookie.com/embed/"#,
        video_id.as_str(),
        r#"" title=""#,
        &title,
        r#"" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>"#
    ))
}
