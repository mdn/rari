use rari_types::locale::Locale;

use super::l10n::l10n_json_data;
use crate::error::DocError;
use crate::templ::api::RariApi;

pub fn css_computed(locale: Locale) -> Result<String, DocError> {
    let copy = l10n_json_data("Template", "xref_csscomputed", locale)?;
    RariApi::link(
        "/Web/CSS/Guides/Cascade/Property_value_processing#computed_value",
        Some(locale),
        Some(copy),
        false,
        None,
        false,
    )
}

pub fn css_inherited(locale: Locale) -> Result<String, DocError> {
    let copy = l10n_json_data("Template", "xref_cssinherited", locale)?;
    RariApi::link(
        "/Web/CSS/Guides/Cascade/Inheritance",
        Some(locale),
        Some(copy),
        false,
        None,
        false,
    )
}

pub fn css_initial(locale: Locale) -> Result<String, DocError> {
    let copy = l10n_json_data("Template", "xref_cssinitial", locale)?;
    RariApi::link(
        "/Web/CSS/Guides/Cascade/Property_value_processing#initial_value",
        Some(locale),
        Some(copy),
        false,
        None,
        false,
    )
}

/// Translates a formal definition value from `L10n-CSSFormalDefinitions.json`.
///
/// Falls back to the raw webref value, since translations are filled in
/// incrementally and a missing en-US entry only means the value is new.
pub fn css_l10n_for_value(value: &str, locale: Locale) -> &str {
    l10n_json_data("CSSFormalDefinitions", value, locale)
        .inspect_err(|e| {
            let locale = locale.as_url_str();
            if locale == Locale::default().as_url_str() {
                tracing::warn!(
                    "Missing en-US entry in content/files/jsondata/L10n-CSSFormalDefinitions.json: {value} ({e})"
                );
            } else {
                tracing::info!(
                    "Missing {locale} translation in content/files/jsondata/L10n-CSSFormalDefinitions.json: {value} ({e})"
                );
            }
        })
        .unwrap_or(value)
}

pub fn css_applies_to(locale: Locale) -> Result<String, DocError> {
    Ok(l10n_json_data("Template", "xref_cssappliesto", locale)?.to_string())
}

pub fn css_percentages(locale: Locale) -> Result<String, DocError> {
    Ok(l10n_json_data("Template", "xref_csspercentages", locale)?.to_string())
}

pub fn css_related_at_rule(locale: Locale) -> Result<String, DocError> {
    let copy = l10n_json_data("Template", "xref_cssrelated_at_rule", locale)?;
    RariApi::link(
        "/Web/CSS/Guides/Syntax/At-rules",
        Some(locale),
        Some(copy),
        false,
        None,
        false,
    )
}

pub fn css_animation_type(locale: Locale) -> Result<String, DocError> {
    let copy = l10n_json_data("Template", "xref_cssanimationtype", locale)?;
    RariApi::link(
        "/Web/CSS/Guides/Animations/Animatable_properties",
        Some(locale),
        Some(copy),
        false,
        None,
        false,
    )
}
