use rari_templ_func::rari_f;
use rari_types::AnyArg;
use tracing::warn;

use crate::error::DocError;

#[rari_f]
pub fn deprecated_header(version: Option<AnyArg>) -> Result<String, DocError> {
    if version.is_some() {
        warn!("Do not use deprectaed header with parameter!")
    }
    let title = rari_l10n::l10n_json_data("Template", "deprecated_badge_abbreviation", env.locale)?;
    let copy = rari_l10n::l10n_json_data("Template", "deprecated_header_copy", env.locale)?;

    Ok([
        r#"<div class="notecard deprecated"><p><strong>"#,
        title,
        ":</strong> ",
        copy,
        "</p></div>",
    ]
    .join(""))
}

#[rari_f]
pub fn available_in_workers(typ: Option<String>) -> Result<String, DocError> {
    let default_typ = "available_in_worker__default";
    let typ = typ
        .map(|s| s.to_lowercase())
        .map(|typ| format!("available_in_worker__{typ}"));
    let copy = rari_l10n::l10n_json_data(
        "Template",
        typ.as_deref().unwrap_or(default_typ),
        env.locale,
    )
    .unwrap_or(rari_l10n::l10n_json_data(
        "Template",
        default_typ,
        env.locale,
    )?);

    Ok([r#"<div class="notecard note"><p> "#, copy, "</p></div>"].join(""))
}

#[rari_f]
pub fn see_compat_table() -> Result<String, DocError> {
    let title =
        rari_l10n::l10n_json_data("Template", "experimental_badge_abbreviation", env.locale)?;
    let copy = rari_l10n::l10n_json_data("Template", "see_compat_table_copy", env.locale)?;

    Ok([
        r#"<div class="notecard experimental"><p><strong>"#,
        title,
        ":</strong> ",
        copy,
        "</p></div>",
    ]
    .join(""))
}

#[rari_f]
pub fn secure_context_header() -> Result<String, DocError> {
    let title = rari_l10n::l10n_json_data("Template", "secure_context_label", env.locale)?;
    let copy = rari_l10n::l10n_json_data("Template", "secure_context_header_copy", env.locale)?;

    Ok([
        r#"<div class="notecard secure"><p><strong>"#,
        &html_escape::encode_double_quoted_attribute(title),
        ":</strong> ",
        copy,
        "</p></div>",
    ]
    .join(""))
}

#[rari_f]
pub fn non_standard_header() -> Result<String, DocError> {
    let title =
        rari_l10n::l10n_json_data("Template", "non_standard_badge_abbreviation", env.locale)?;
    let copy = rari_l10n::l10n_json_data("Template", "non_standard_header_copy", env.locale)?;

    Ok([
        r#"<div class="notecard nonstandard"><p><strong>"#,
        title,
        ":</strong> ",
        copy,
        "</p></div>",
    ]
    .join(""))
}
