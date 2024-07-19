use std::fmt::Write;

use rari_templ_func::rari_f;
use rari_types::AnyArg;

use crate::error::DocError;

#[rari_f]
pub fn embded_jsfiddle(
    url: String,
    options: Option<String>,
    height: Option<AnyArg>,
) -> Result<String, DocError> {
    let mut out = String::new();
    out.push_str(r#"<iframe allowfullscreen="allowfullscreen" width="756" "#);
    if let Some(height) = height {
        if !height.is_empty() {
            write!(&mut out, r#"height="{}" "#, height)?;
        }
    }
    out.extend([
        r#"src=""#,
        url.as_str(),
        "embedded/",
        options.as_deref().unwrap_or_default(),
        if options.as_ref().map(|s| s.is_empty()).unwrap_or_default() {
            ""
        } else {
            "/"
        },
        r#""></iframe>"#,
    ]);
    Ok(out)
}
