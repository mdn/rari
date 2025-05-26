use std::fmt::Write;

use rari_templ_func::rari_f;
use rari_types::AnyArg;

use crate::error::DocError;

#[rari_f(crate::Templ)]
pub fn embed_jsfiddle(
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
