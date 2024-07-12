use rari_templ_func::rari_f;

use crate::error::DocError;

#[rari_f]
pub fn see_compat_table() -> Result<String, DocError> {
    let title =
        rari_l10n::l10n_json_data("Template", "experimental_badge_abbreviation", env.locale)?;
    let copy = rari_l10n::l10n_json_data("Template", "see_compat_table_copy", env.locale)?;

    Ok([
        r#"<div class="notecard experimental"><strong>"#,
        title,
        ":</strong> ",
        copy,
        "</div>",
    ]
    .join(""))
}
