use rari_templ_func::rari_f;
use rari_utils::concat_strs;

use crate::error::DocError;
use crate::helpers::l10n::l10n_json_data;

#[rari_f(register = "crate::Templ")]
pub fn securecontext_inline() -> Result<String, DocError> {
    let label = l10n_json_data("Template", "secure_context_label", env.locale)?;
    let copy = l10n_json_data("Template", "secure_context_inline_copy", env.locale)?;

    Ok(write_inline_label(label, copy, "secure"))
}

#[rari_f(register = "crate::Templ")]
pub fn readonlyinline() -> Result<String, DocError> {
    let copy = l10n_json_data("Template", "readonly_badge_title", env.locale)?;
    let label = l10n_json_data("Template", "readonly_badge_abbreviation", env.locale)?;

    Ok(write_inline_label(label, copy, "readonly"))
}

pub fn write_inline_label(label: &str, copy: &str, typ: &str) -> String {
    concat_strs!(
        r#"<span class="badge inline "#,
        typ,
        r#"" title=""#,
        &html_escape::encode_double_quoted_attribute(copy),
        r#"">"#,
        label,
        "</span>"
    )
}
