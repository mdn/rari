use std::fmt::Write;
use std::fs;
use std::sync::OnceLock;

use css_syntax_types::WebrefCss;
use rari_types::globals::data_dir;
use rari_types::locale::Locale;
use serde_json::Value;

use super::l10n::l10n_json_data;
use crate::error::DocError;
use crate::templ::api::RariApi;

pub static CSS_REF: OnceLock<WebrefCss> = OnceLock::new();

pub fn css_ref_data() -> &'static WebrefCss {
    CSS_REF.get_or_init(|| {
        let json_str = fs::read_to_string(data_dir().join("@webref/css").join("webref_css.json"))
            .expect("no data dir");
        serde_json::from_str(&json_str).expect("Failed to parse JSON")
    })
}

pub fn css_l10n_for_value(key: &str, locale: Locale) -> &str {
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
