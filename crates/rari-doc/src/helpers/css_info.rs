use rari_types::locale::Locale;
use serde_json::Value;
use std::fmt::Write;

use super::l10n::l10n_json_data;
use crate::error::DocError;
use crate::templ::api::RariApi;

pub fn css_l10n_for_value(key: &str, locale: Locale) -> &str {
    // If a (localized) value is not found, we emit a warning and use the key as a value here.
    // This is different from the `l10n_json_data` call that relies on at least the default locale valueto be present.
    l10n_json_data("CSSFormalDefinitions", key, locale)
        .inspect_err(|e| tracing::warn!("Localized value for formal definition is missing in content/files/jsondata/L10n-CSSFormalDefinitions.json: {} ({})", key, e))
        .unwrap_or(key)
}

pub fn get_for_locale(locale: Locale, lookup: &Value) -> &Value {
    let value = &lookup[locale.as_url_str()];
    if !value.is_null() {
        value
    } else if locale != Locale::default() {
        &lookup[Locale::default().as_url_str()]
    } else {
        &Value::Null
    }
}

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

pub fn css_applies_to(locale: Locale) -> Result<String, DocError> {
    Ok(l10n_json_data("Template", "xref_cssappliesto", locale)?.to_string())
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

pub fn css_percentages(locale: Locale) -> Result<String, DocError> {
    Ok(l10n_json_data("Template", "xref_csspercentages", locale)?.to_string())
}

pub fn write_missing(out: &mut String, locale: Locale) -> Result<(), DocError> {
    let missing = l10n_json_data("CSS", "missing", locale)?;
    Ok(write!(out, "<span style=\"color:red;\">{missing}</span>")?)
}
