use std::borrow::Cow;
use std::fmt::Write;
use std::sync::OnceLock;

use indexmap::IndexMap;
use itertools::Itertools;
use rari_types::RariEnv;
use rari_types::globals::data_dir;
use rari_types::locale::Locale;
use rari_utils::io::read_to_string;
use serde_json::Value;
use tracing::warn;

use super::l10n::l10n_json_data;
use crate::error::DocError;
use crate::templ::api::RariApi;
use crate::templ::render::render_and_decode_ref;
use crate::templ::templs::links::cssxref::cssxref_internal;

// mdn/data is deprecated so we do a least effort integration here.
#[derive(Debug, Default)]
pub struct MDNDataFiles {
    pub css_properties: IndexMap<String, Value>,
    pub css_at_rules: IndexMap<String, Value>,
    pub css_types: IndexMap<String, Value>,
    pub css_l10n: IndexMap<String, Value>,
    pub css_syntaxes: IndexMap<String, Value>,
    pub css_seclectors: IndexMap<String, Value>,
    pub css_units: IndexMap<String, Value>,
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
            css_types: serde_json::from_str(&read_to_string(
                data_dir().join("mdn-data/package/css/types.json"),
            )?)?,
            css_syntaxes: serde_json::from_str(&read_to_string(
                data_dir().join("mdn-data/package/css/syntaxes.json"),
            )?)?,
            css_seclectors: serde_json::from_str(&read_to_string(
                data_dir().join("mdn-data/package/css/selectors.json"),
            )?)?,
            css_units: serde_json::from_str(&read_to_string(
                data_dir().join("mdn-data/package/css/units.json"),
            )?)?,
        })
    }
}

pub static MDN_DATA_FILES: OnceLock<MDNDataFiles> = OnceLock::new();

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

    out.push(("initial", Cow::Owned(css_initial(locale)?)));

    if at_rule.is_none() {
        out.push((
            "appliesto",
            Cow::Borrowed(get_css_l10n_for_locale("appliesTo", locale)),
        ));
    }

    if !css_info_data["inherited"].is_null() {
        out.push(("inherited", Cow::Owned(css_inherited(locale)?)));
    }

    if css_info_data["percentages"].as_str() != Some("no")
        && !css_info_data["percentages"].is_null()
    {
        out.push((
            "percentages",
            Cow::Borrowed(get_css_l10n_for_locale("percentages", locale)),
        ));
    }

    out.push(("computed", Cow::Owned(css_computed(locale)?)));

    if at_rule.is_none() {
        out.push((
            "animationType",
            Cow::Owned(RariApi::link(
                "/Web/CSS/Guides/Animations/Animatable_properties",
                Some(locale),
                Some(get_css_l10n_for_locale("animationType", locale)),
                false,
                None,
                false,
            )?),
        ));
    }
    if css_info_data["stacking"].as_bool().unwrap_or_default() {
        out.push((
            "stacking",
            Cow::Borrowed(get_css_l10n_for_locale("createsStackingContext", locale)),
        ));
    }
    Ok(out)
}

const INITIAL_L10N_VALUES: [&str; 9] = [
    "\"\"",
    "\". \"",
    "autoForSmartphoneBrowsersSupportingInflation",
    "dependsOnUserAgent",
    "noPracticalInitialValue",
    "noneButOverriddenInUserAgentCSS",
    "seeProse",
    "startOrNamelessValueIfLTRRightIfRTL",
    "zoomForTheTopLevelNoneForTheRest",
];

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
        return Ok(());
    }
    let data = &css_info_data[property];
    match data {
        Value::Null => {
            //write_missing(out, locale)?;
        }
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
                                &[get_css_l10n_for_locale("length", locale)],
                            ));
                        }
                        Cow::Borrowed(localized)
                    })
                    .join(get_css_l10n_for_locale("listSeparator", locale));
                out.push_str(&render_and_decode_ref(
                    env,
                    &add_additional_applies_to(&parsed, property, css_info_data, locale),
                )?);
                return Ok(());
            } else if s.starts_with('\'') && s.ends_with('\'') {
                let s_data = mdn_data_files()
                    .css_properties
                    .get(&s[1..s.len() - 1])
                    .unwrap_or(&Value::Null);
                return write_computed_output(env, out, locale, s_data, property, at_rule);
            } else if property == "initial" && !INITIAL_L10N_VALUES.contains(&s.as_str()) {
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
        Value::Array(a) => {
            let mut tmp = String::new();
            tmp.push_str(get_css_l10n_for_locale("asLonghands", locale));
            tmp.push_str("<br /><ul>");
            for longhand in a.iter().filter_map(Value::as_str) {
                tmp.push_str("<li>{{cssxref(\"");
                tmp.push_str(longhand);
                tmp.push_str("\")}}: ");
                let longhand_data = mdn_data_files()
                    .css_properties
                    .get(longhand)
                    .unwrap_or(&Value::Null);
                if !longhand_data.is_null() {
                    write_computed_output(env, &mut tmp, locale, longhand_data, property, at_rule)?;
                } else {
                    write_missing(&mut tmp, locale)?;
                }
                tmp.push_str("</li>")
            }
            tmp.push_str("</ul>");
            out.push_str(&render_and_decode_ref(
                env,
                &add_additional_applies_to(&tmp, property, css_info_data, locale),
            )?);
            return Ok(());
        }
        Value::Object(_) => {
            if let Some(localized) = get_for_locale(locale, data).as_str() {
                out.push_str(&render_and_decode_ref(
                    env,
                    &add_additional_applies_to(localized, property, css_info_data, locale),
                )?);
            } else {
                write_missing(out, locale)?;
            }
            return Ok(());
        }
    };
    Ok(())
}

fn add_additional_applies_to<'a>(
    output: &'a str,
    property: &str,
    css_info_data: &Value,
    locale: Locale,
) -> Cow<'a, str> {
    if property != "appliesto" || !css_info_data["alsoAppliesTo"].is_array() {
        return Cow::Borrowed(output);
    }

    let also_applies_to = &css_info_data["alsoAppliesTo"].as_array().unwrap();

    let also_applies_to = also_applies_to
        .iter()
        .filter_map(Value::as_str)
        .filter(|element| *element != "::placeholder" && !element.is_empty())
        .map(|element| {
            cssxref_internal(element, None, None, locale).unwrap_or_else(|e| e.to_string())
        })
        .collect::<Vec<_>>();

    if also_applies_to.is_empty() {
        return Cow::Borrowed(output);
    }

    let mut additional_applies_to = String::new();
    for (i, additional) in also_applies_to.iter().enumerate() {
        additional_applies_to.push_str(additional.as_str());
        if also_applies_to.len() - i > 2 {
            additional_applies_to.push_str(get_css_l10n_for_locale("listSeparator", locale));
        } else if also_applies_to.len() - i > 1 {
            additional_applies_to.push_str(get_css_l10n_for_locale("andInEnumeration", locale));
        }
    }
    Cow::Owned(remove_me_replace_placeholder(
        get_css_l10n_for_locale("applyingToMultiple", locale),
        &[output, &additional_applies_to],
    ))
}

pub fn get_css_l10n_for_locale(key: &str, locale: Locale) -> &str {
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

pub fn write_missing(out: &mut String, locale: Locale) -> Result<(), DocError> {
    let missing = l10n_json_data("CSS", "missing", locale)?;
    Ok(write!(out, "<span style=\"color:red;\">{missing}</span>")?)
}

fn remove_me_replace_placeholder(s: &str, replacements: &[&str]) -> String {
    s.replace("$1$", replacements.first().unwrap_or(&"$1$"))
        .replace("$2$", replacements.get(1).unwrap_or(&"$2$"))
}

const CSS_GRAMMAR_KEYWORDS: [&str; 6] = [
    "anchor-visible",
    "decimal",
    "relative-colorimetric",
    "symbolic",
    "true",
    "upright",
];

fn is_css_numeric_token(value: &str) -> bool {
    let value = value.strip_suffix('%').unwrap_or(value);
    if value.parse::<f64>().is_ok() {
        return true;
    }
    let split_at = value
        .char_indices()
        .find_map(|(index, character)| character.is_ascii_alphabetic().then_some(index))
        .unwrap_or(value.len());
    let (number, unit) = value.split_at(split_at);

    !number.is_empty()
        && number.parse::<f64>().is_ok()
        && (unit.is_empty() || unit.bytes().all(|byte| byte.is_ascii_alphabetic()))
}

fn is_unicode_range(value: &str) -> bool {
    value.strip_prefix("U+").is_some_and(|range| {
        !range.is_empty()
            && range
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() || byte == b'-' || byte == b'?')
    })
}

fn is_css_type_token(value: &str) -> bool {
    value
        .strip_prefix('<')
        .and_then(|value| value.strip_suffix('>'))
        .is_some_and(|name| {
            !name.is_empty()
                && name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'(' | b')'))
        })
}

fn is_css_grammar_token(value: &str) -> bool {
    value.chars().all(|character| !character.is_alphabetic())
        || is_css_numeric_token(value)
        || value == "auto"
        || is_unicode_range(value)
        || is_css_type_token(value)
}

fn is_css_grammar_value(value: &str) -> bool {
    CSS_GRAMMAR_KEYWORDS.contains(&value)
        || (!value.is_empty() && value.split_ascii_whitespace().all(is_css_grammar_token))
}

/// Returns an intentionally unlocalized CSS grammar value or localized prose.
///
/// New prose must be added to content's `L10n-CSSFormalDefinitions.json`; an
/// error is returned rather than silently rendering the English source value.
fn css_l10n_for_value_from<'a, F>(
    value: &'a str,
    locale: Locale,
    lookup: F,
) -> Result<Cow<'a, str>, DocError>
where
    F: FnOnce(&str, Locale) -> Result<&'a str, super::l10n::L10nError>,
{
    if is_css_grammar_value(value) {
        return Ok(Cow::Borrowed(value));
    }
    lookup(value, locale).map(Cow::Borrowed).map_err(|e| {
        DocError::InvalidTempl(format!(
            "Missing CSS formal-definition localization for {value}: {e}"
        ))
    })
}

pub fn css_l10n_for_value<'a>(value: &'a str, locale: Locale) -> Result<Cow<'a, str>, DocError> {
    css_l10n_for_value_from(value, locale, |value, locale| {
        l10n_json_data("CSSFormalDefinitions", value, locale)
    })
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

#[cfg(test)]
mod tests {
    use rari_types::RariEnv;
    use rari_types::locale::Locale;
    use serde_json::json;

    use crate::helpers::l10n::L10nError;

    use super::{
        CSS_GRAMMAR_KEYWORDS, css_l10n_for_value_from, get_css_l10n_for_locale,
        is_css_grammar_value, write_computed_output,
    };

    fn render_initial_value(initial_value: &str, locale: Locale) -> String {
        let mut out = String::new();
        let css_info_data = json!({ "initial": initial_value });
        let env = RariEnv {
            locale,
            ..Default::default()
        };

        write_computed_output(&env, &mut out, locale, &css_info_data, "initial", None)
            .expect("rendering initial value should succeed");

        out
    }

    #[test]
    fn renders_keyword_initial_value_as_code_for_locales() {
        assert_eq!(render_initial_value("all", Locale::Ja), "<code>all</code>");
    }

    #[test]
    fn renders_known_initial_l10n_value_through_translation_branch() {
        let localized = get_css_l10n_for_locale("dependsOnUserAgent", Locale::Ja);

        assert_eq!(
            render_initial_value("dependsOnUserAgent", Locale::Ja),
            localized
        );
    }

    #[test]
    fn css_grammar_values_are_intentionally_unlocalized() {
        let grammar_values = [
            "\"\"",
            "\"*\"",
            "\"-\"",
            "\". \"",
            "0",
            "0 \"\"",
            "0 1 auto",
            "0%",
            "0% 0%",
            "0px",
            "0px 0px",
            "0s",
            "1",
            "100%",
            "1dppx",
            "1px",
            "2",
            "4",
            "50% 50%",
            "8",
            "?",
            "U+0-10FFFF",
            "anchor-visible",
            "decimal",
            "relative-colorimetric",
            "symbolic",
            "true",
            "upright",
        ];
        let prose_values = ["1 auto word", "see text", "specified <length>"];

        assert_eq!(CSS_GRAMMAR_KEYWORDS.len(), 6);
        assert!(grammar_values.into_iter().all(is_css_grammar_value));
        assert!(is_css_grammar_value("<length>"));
        assert!(
            prose_values
                .into_iter()
                .all(|value| !is_css_grammar_value(value))
        );
    }

    #[test]
    fn css_grammar_values_bypass_the_localization_lookup() {
        let value = css_l10n_for_value_from("0", Locale::EnUs, |_, _| {
            Err(L10nError::InvalidKey("unexpected lookup".into()))
        })
        .expect("CSS grammar should not require a localization");

        assert_eq!(value, "0");
    }

    #[test]
    fn missing_prose_localization_is_an_error() {
        let error = css_l10n_for_value_from("see text", Locale::EnUs, |_, _| {
            Err(L10nError::InvalidKey("see text".into()))
        })
        .expect_err("prose must have an en-US localization");

        assert!(
            error
                .to_string()
                .contains("Missing CSS formal-definition localization")
        );
    }

    #[test]
    fn duplicate_values_share_one_localization_key() {
        let value = css_l10n_for_value_from("same value", Locale::EnUs, |key, _| {
            assert_eq!(key, "same value");
            Ok("localized value")
        })
        .expect("one key should resolve repeated source values");

        assert_eq!(value, "localized value");
    }
}
