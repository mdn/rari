use rari_templ_func::rari_f;

use crate::error::DocError;

#[rari_f]
pub fn secure_context_inline() -> Result<String, DocError> {
    let title = rari_l10n::l10n_json_data("Template", "secure_context_label", env.locale)?;
    let copy = rari_l10n::l10n_json_data("Template", "secure_context_inline_copy", env.locale)?;

    Ok([
        r#"<span class="badge inline secure" title=""#,
        &html_escape::encode_double_quoted_attribute(copy),
        r#"">"#,
        title,
        "</span>",
    ]
    .join(""))
}

#[rari_f]
pub fn secure_context_header() -> Result<String, DocError> {
    let title = rari_l10n::l10n_json_data("Template", "secure_context_label", env.locale)?;
    let copy = rari_l10n::l10n_json_data("Template", "secure_context_header_copy", env.locale)?;

    Ok([
        r#"<div class="notecard secure"><strong>"#,
        &html_escape::encode_double_quoted_attribute(title),
        ":</strong> ",
        copy,
        "</div>",
    ]
    .join(""))
}
