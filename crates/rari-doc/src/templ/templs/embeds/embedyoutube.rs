use std::borrow::Cow;

use rari_templ_func::rari_f;

use crate::error::DocError;

#[rari_f]
pub fn embed_youtube(video_id: String, title: Option<String>) -> Result<String, DocError> {
    let title = title
        .as_deref()
        .map(|s| html_escape::encode_double_quoted_attribute(s));
    let mut out = String::new();
    out.extend([
        r#"<iframe width="560" height="315" "#,
        r#"src="https://www.youtube-nocookie.com/embed/"#,
        video_id.as_str(),
        r#"" title=""#,
        &title.unwrap_or(Cow::Borrowed("Youtube video")),
        r#"" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>"#,
        r#""></iframe>"#,
    ]);
    Ok(out)
}
