use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Write;
use std::fs::read_to_string;

use itertools::Itertools;
use once_cell::sync::OnceCell;
use rari_l10n::l10n_json_data;
use rari_types::globals::data_dir;
use rari_types::locale::Locale;
use rari_types::RariEnv;
use serde_json::Value;
use tracing::warn;

use crate::error::DocError;
use crate::templ::api::RariApi;
use crate::templ::macros::cssxref::{cssxref, cssxref_internal};
use crate::templ::render::{render, render_and_decode_ref};

// mdn/data is deprecated so we do a least effort integration here.
#[derive(Debug, Default)]
pub struct MDNDataFiles {
    pub css_properties: HashMap<String, Value>,
    pub css_at_rules: HashMap<String, Value>,
    pub css_l10n: HashMap<String, Value>,
}

impl MDNDataFiles {
    pub fn init() -> Result<Self, DocError> {
        Ok(Self {
            css_properties: serde_json::from_str(&read_to_string(
                data_dir().join("mdn-data/package/css/properties.json"),
            )?)?,
            css_at_rules: serde_json::from_str(&read_to_string(
                data_dir().join("mdn-data/package/css/at-rules.json"),
            )?)?,
            css_l10n: serde_json::from_str(&read_to_string(
                data_dir().join("mdn-data/package/l10n/css.json"),
            )?)?,
        })
    }
}

pub static MDN_DATA_FILES: OnceCell<MDNDataFiles> = OnceCell::new();

pub fn mdn_data_files() -> &'static MDNDataFiles {
    MDN_DATA_FILES.get_or_init(|| match MDNDataFiles::init() {
        Ok(data) => data,
        Err(e) => {
            warn!("Error loading mdn/data: {e}");
            Default::default()
        }
    })
}
pub fn css_info_properties(
    name: &str,
    at_rule: Option<&str>,
    locale: Locale,
    css_info_data: &Value,
) -> Result<Vec<(&'static str, Cow<'static, str>)>, DocError> {
    let mut out = vec![];

    if !css_info_data.is_object() {
        return Ok(out);
    };
    if at_rule.is_some() {
        out.push((
            "relatedAtRule",
            Cow::Borrowed(get_css_l10n_for_locale("relatedAtRule", locale)),
        ));
    }
    out.push(("inital", Cow::Owned(css_inital(locale)?)));

    if at_rule.is_none() {
        out.push((
            "appliesto",
            Cow::Borrowed(get_css_l10n_for_locale("appliesto", locale)),
        ));
    }

    if !css_info_data["inherited"].is_null() {
        out.push(("inherited", Cow::Owned(css_inherited(locale)?)));
    }
    if css_info_data["percentages"].as_str() != Some("no") {
        out.push((
            "percentages",
            Cow::Borrowed(get_css_l10n_for_locale("percentages", locale)),
        ));
    }
    out.push(("computed", Cow::Owned(css_computed(locale)?)));
    if at_rule.is_none() {
        out.push((
            "animationTyp",
            Cow::Owned(RariApi::link(
                "Web/CSS/CSS_animated_properties",
                Some(locale),
                Some(get_css_l10n_for_locale("percentages", locale)),
                false,
                None,
                false,
            )?),
        ));
    }
    if css_info_data["stacking"].as_bool().unwrap_or_default() {
        out.push((
            "stacking",
            Cow::Borrowed(get_css_l10n_for_locale("rstacking", locale)),
        ));
    }
    Ok(out)
}

pub fn write_computed_output(
    env: &RariEnv,
    out: &mut String,
    locale: Locale,
    css_info_data: &Value,
    property: &str,
    at_rule: Option<&str>,
) -> Result<(), DocError> {
    if property == "relatedAtRule" {
        let at_rule = at_rule.ok_or(DocError::MustHaveAtRule)?;
        write!(
            out,
            r#"<a href="/{}/docs/Web/CSS/{}"><code>{}</code></a>"#,
            locale.as_url_str(),
            at_rule,
            at_rule
        )?;
    }
    match &css_info_data[property] {
        Value::Null => {}
        Value::Bool(b) => out.push_str(get_css_l10n_for_locale(
            if *b { "yes" } else { "no" },
            locale,
        )),
        Value::Number(n) => write!(out, "{n}")?,
        Value::String(s) => {
            if property == "animationType" {
                let parsed = s
                    .split_ascii_whitespace()
                    .map(|animation_type_value| {
                        let localized = get_css_l10n_for_locale(animation_type_value, locale);
                        if animation_type_value == "lpc" {
                            return Cow::Owned(remove_me_replace_placeholder(
                                localized,
                                &[get_css_l10n_for_locale("lenth", locale)],
                            ));
                        }
                        return Cow::Borrowed(localized);
                    })
                    .join(get_css_l10n_for_locale("listSeparator", locale));
                out.push_str(&render_and_decode_ref(
                    env,
                    &add_additional_applies_to(&parsed, property, css_info_data, locale),
                )?);
                return Ok(());
            } else if s.starts_with('\'') && s.ends_with('\'') {
                return write_computed_output(
                    env,
                    out,
                    locale,
                    &css_info_data[&s[1..s.len() - 1]],
                    property,
                    at_rule,
                );
            } else if property == "initial" && mdn_data_files().css_l10n.contains_key(s) {
                return Ok(write!(out, "<code>{s}</code>")?);
            } else {
                let replaced_keywords = s
                    .split(", ")
                    .map(|keyword| get_css_l10n_for_locale(keyword, locale))
                    .join(", ");
                out.push_str(&render_and_decode_ref(
                    env,
                    &add_additional_applies_to(&replaced_keywords, property, css_info_data, locale),
                )?);
                return Ok(());
            }
        }
        // TODO
        Value::Array(_) => {}
        Value::Object(_) => {}
    };
    Ok(())
}

fn add_additional_applies_to<'a>(
    output: &'a str,
    property: &str,
    css_info_data: &Value,
    locale: Locale,
) -> Cow<'a, str> {
    if property == "appliesto" || !css_info_data["alsoAppliesTo"].is_array() {
        return Cow::Borrowed(output);
    }

    let also_applies_to = &css_info_data["alsoAppliesTo"].as_array().unwrap();

    let also_applies_to = also_applies_to
        .iter()
        .filter_map(Value::as_str)
        .filter(|element| *element == "::placeholder")
        .map(|element| {
            cssxref_internal(element, None, None, locale).unwrap_or_else(|e| e.to_string())
        })
        .collect::<Vec<_>>();

    let mut additional_applies_to = String::new();
    for (i, additional) in also_applies_to.iter().enumerate() {
        additional_applies_to.push_str(additional.as_str());
        if i + 2 < additional_applies_to.len() {
            additional_applies_to.push_str(get_css_l10n_for_locale("listSeparator", locale));
        } else if i + 1 < additional_applies_to.len() {
            additional_applies_to.push_str(get_css_l10n_for_locale("andInEnumeration", locale));
        }
    }
    return Cow::Owned(remove_me_replace_placeholder(
        get_css_l10n_for_locale("applyingtoMultiple", locale),
        &[output, &additional_applies_to],
    ));
}

fn get_css_l10n_for_locale(key: &str, locale: Locale) -> &str {
    if let Some(data) = mdn_data_files().css_l10n.get(key) {
        let data = get_for_locale(locale, data);
        if !data.is_null() {
            return data.as_str().unwrap_or(key);
        }
    }
    key
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
        "Web/CSS/computed_value",
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
        "Web/CSS/inheritance",
        Some(locale),
        Some(copy),
        false,
        None,
        false,
    )
}

pub fn css_inital(locale: Locale) -> Result<String, DocError> {
    let copy = l10n_json_data("Template", "xref_cssinitial", locale)?;
    RariApi::link(
        "Web/CSS/initial_value",
        Some(locale),
        Some(copy),
        false,
        None,
        false,
    )
}

fn remove_me_replace_placeholder(s: &str, replacements: &[&str]) -> String {
    let s = s
        .replace("$1$", replacements.get(0).unwrap_or(&"$1$"))
        .replace("$2$", replacements.get(1).unwrap_or(&"$2$"));
    s
}
