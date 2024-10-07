use std::fmt::Write;

use rari_templ_func::rari_f;
use rari_types::AnyArg;

use crate::error::DocError;
use crate::templ::api::RariApi;
use crate::utils::dedup_ws;

#[allow(clippy::too_many_arguments)]
#[rari_f]
pub fn live_sample(
    id: String,
    width: Option<AnyArg>,
    height: Option<AnyArg>,
    _deprecated_3: Option<String>,
    _deprecated_4: Option<String>,
    _deprecated_5: Option<String>,
    allowed_features: Option<String>,
) -> Result<String, DocError> {
    let title = dedup_ws(&id.replace('_', " "));
    let id = RariApi::anchorize(&id);
    let mut out = String::new();
    out.push_str(r#"<div class="code-example"><div class="example-header"></div><iframe class="sample-code-frame" title=""#);
    out.push_str(&html_escape::encode_quoted_attribute(&title));
    out.push_str(r#" sample" id="frame_"#);
    out.push_str(&id);
    out.push_str(r#"" "#);
    if let Some(width) = width {
        if !width.is_empty() {
            write!(&mut out, r#"width="{}" "#, width)?;
        }
    }
    if let Some(height) = height {
        if !height.is_empty() {
            // TODO: fix this
            if height.as_int() < 60 {
                write!(&mut out, r#"height="{}" "#, height.as_int())?;
            } else {
                write!(&mut out, r#"height="{}" "#, height)?;
            }
        }
    }
    if let Some(allowed_features) = allowed_features {
        write!(&mut out, r#"allow="{}" "#, allowed_features)?;
    }
    write!(
        &mut out,
        r#"src="{}{}{}runner.html?id={}" "#,
        RariApi::live_sample_base_url(),
        env.url,
        if env.url.ends_with('/') { "" } else { "/" },
        id
    )?;
    out.push_str(r#"sandbox="allow-same-origin allow-scripts" "#);
    out.push_str(r#"loading="lazy"></iframe></div>"#);
    Ok(out)
}
