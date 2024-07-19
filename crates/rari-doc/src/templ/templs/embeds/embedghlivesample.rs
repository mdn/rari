use std::fmt::Write;

use rari_templ_func::rari_f;
use rari_types::AnyArg;

use crate::error::DocError;

#[rari_f]
pub fn embed_gh_live_sample(
    path: String,
    width: Option<AnyArg>,
    height: Option<AnyArg>,
) -> Result<String, DocError> {
    let mut out = String::new();
    out.push_str("<iframe ");
    if let Some(width) = width {
        if !width.is_empty() {
            write!(&mut out, r#"width="{}" "#, width)?;
        }
    }
    if let Some(height) = height {
        if !height.is_empty() {
            write!(&mut out, r#"height="{}" "#, height)?;
        }
    }

    out.extend([
        r#"src="https://mdn.github.io/"#,
        path.as_str(),
        r#""></iframe>"#,
    ]);
    Ok(out)
}
