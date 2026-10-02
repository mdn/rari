use std::fmt::Display;
use std::iter::once;
use std::str::FromStr;
use std::sync::LazyLock;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_variant::to_variant_name;
use thiserror::Error;

use crate::globals::{settings, translated_content_roots};

#[derive(
    PartialEq, Debug, Clone, Copy, Deserialize, Serialize, Default, PartialOrd, Eq, Ord, JsonSchema,
)]
pub enum Native {
    #[default]
    #[serde(rename = "English (US)")]
    EnUS,
    #[serde(rename = "Deutsch")]
    De,
    #[serde(rename = r#"Español"#)]
    Es,
    #[serde(rename = r#"Français"#)]
    Fr,
    #[serde(rename = r#"日本語"#)]
    Ja,
    #[serde(rename = r#"한국어"#)]
    Ko,
    #[serde(rename = r#"Português (do Brasil)"#)]
    PtBr,
    #[serde(rename = r#"Русский"#)]
    Ru,
    #[serde(rename = r#"中文 (简体)"#)]
    ZhCn,
    #[serde(rename = r#"正體中文 (繁體)"#)]
    ZhTw,
}

impl From<Locale> for Native {
    fn from(value: Locale) -> Self {
        match value {
            Locale::EnUs => Self::EnUS,
            Locale::De => Self::De,
            Locale::Es => Self::Es,
            Locale::Fr => Self::Fr,
            Locale::Ja => Self::Ja,
            Locale::Ko => Self::Ko,
            Locale::PtBr => Self::PtBr,
            Locale::Ru => Self::Ru,
            Locale::ZhCn => Self::ZhCn,
            Locale::ZhTw => Self::ZhTw,
        }
    }
}

#[derive(Debug, Error)]
pub enum LocaleError {
    #[error("invalid locale: {0}")]
    InvalidLocale(String),
    #[error("no locale in path")]
    NoLocaleInPath,
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

#[derive(
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    Default,
    Hash,
    JsonSchema,
)]
pub enum Locale {
    #[default]
    #[serde(rename = "en-US")]
    EnUs,
    #[serde(rename = "de")]
    De,
    #[serde(rename = "es")]
    Es,
    #[serde(rename = "fr")]
    Fr,
    #[serde(rename = "ja")]
    Ja,
    #[serde(rename = "ko")]
    Ko,
    #[serde(rename = "pt-BR")]
    PtBr,
    #[serde(rename = "ru")]
    Ru,
    #[serde(rename = "zh-CN")]
    ZhCn,
    #[serde(rename = "zh-TW")]
    ZhTw,
}

pub const fn default_locale() -> Locale {
    Locale::EnUs
}

/// Selects which locales an operation applies to.
#[derive(Debug, Clone, Copy)]
pub enum LocaleFilter<'a> {
    /// Apply to every available locale.
    All,
    /// Apply only to the listed locales.
    Only(&'a [Locale]),
}

impl<'a> From<Option<&'a [Locale]>> for LocaleFilter<'a> {
    fn from(locales: Option<&'a [Locale]>) -> Self {
        match locales {
            Some(l) => LocaleFilter::Only(l),
            None => LocaleFilter::All,
        }
    }
}

impl Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(to_variant_name(self).map_err(|_| std::fmt::Error)?)
    }
}

static ACTIVE_TRANSLATED_LOCALES: &[Locale] = &[
    Locale::Es,
    Locale::Fr,
    Locale::Ja,
    Locale::Ko,
    Locale::PtBr,
    Locale::Ru,
    Locale::ZhCn,
    Locale::ZhTw,
];

static LOCALES_FOR_GENERICS_AND_SPAS: LazyLock<Vec<Locale>> = LazyLock::new(|| {
    once(&Locale::EnUs)
        .chain(ACTIVE_TRANSLATED_LOCALES.iter())
        .chain(settings().additional_locales_for_generics_and_spas.iter())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>()
});

static TRANSLATED_LOCALES: LazyLock<Vec<Locale>> = LazyLock::new(|| {
    translated_locales_with(
        &settings().additional_locales_for_generics_and_spas,
        &settings().optional_translated_locales,
    )
});

fn translated_locales_with(additional: &[Locale], optional: &[Locale]) -> Vec<Locale> {
    let mut locales = ACTIVE_TRANSLATED_LOCALES.to_vec();
    for locale in additional.iter().chain(optional) {
        if !locales.contains(locale) {
            locales.push(*locale);
        }
    }
    locales
}

impl Locale {
    pub const fn as_url_str(&self) -> &str {
        match *self {
            Self::EnUs => "en-US",
            Self::De => "de",
            Self::Es => "es",
            Self::Fr => "fr",
            Self::Ja => "ja",
            Self::Ko => "ko",
            Self::PtBr => "pt-BR",
            Self::Ru => "ru",
            Self::ZhCn => "zh-CN",
            Self::ZhTw => "zh-TW",
        }
    }
    pub const fn as_folder_str(&self) -> &str {
        match *self {
            Self::EnUs => "en-us",
            Self::PtBr => "pt-br",
            Self::ZhCn => "zh-cn",
            Self::ZhTw => "zh-tw",
            _ => self.as_url_str(),
        }
    }

    pub fn for_generic_and_spas() -> &'static [Self] {
        if translated_content_roots().is_empty() {
            [Locale::EnUs].as_slice()
        } else {
            &LOCALES_FOR_GENERICS_AND_SPAS
        }
    }

    pub fn translated() -> &'static [Self] {
        &TRANSLATED_LOCALES
    }
}

impl FromStr for Locale {
    type Err = LocaleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "en-US" | "en-us" => Ok(Self::EnUs),
            "de" => Ok(Self::De),
            "es" => Ok(Self::Es),
            "fr" => Ok(Self::Fr),
            "ja" => Ok(Self::Ja),
            "ko" => Ok(Self::Ko),
            "pt-br" | "pt-BR" => Ok(Self::PtBr),
            "ru" => Ok(Self::Ru),
            "zh-cn" | "zh-CN" => Ok(Self::ZhCn),
            "zh-tw" | "zh-TW" => Ok(Self::ZhTw),
            _ => Err(LocaleError::InvalidLocale(s.into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_parser_rejects_whitespace_and_unknown_names() {
        struct Case {
            name: &'static str,
            input: &'static str,
            expected: Result<Locale, &'static str>,
        }
        let cases = [
            Case {
                name: "german",
                input: "de",
                expected: Ok(Locale::De),
            },
            Case {
                name: "whitespace",
                input: " de ",
                expected: Err("invalid locale:  de "),
            },
            Case {
                name: "unknown",
                input: " xx ",
                expected: Err("invalid locale:  xx "),
            },
            Case {
                name: "empty",
                input: " ",
                expected: Err("invalid locale:  "),
            },
        ];
        for case in cases {
            let actual = case.input.parse::<Locale>();
            match case.expected {
                Ok(expected) => assert_eq!(actual.unwrap(), expected, "{}", case.name),
                Err(expected) => {
                    assert_eq!(actual.unwrap_err().to_string(), expected, "{}", case.name)
                }
            }
        }
    }

    #[test]
    fn translated_locale_selection_retains_defaults_and_legacy_setting() {
        let defaults = [
            Locale::Es,
            Locale::Fr,
            Locale::Ja,
            Locale::Ko,
            Locale::PtBr,
            Locale::Ru,
            Locale::ZhCn,
            Locale::ZhTw,
        ];
        struct Case {
            name: &'static str,
            additional: &'static [Locale],
            optional: &'static [Locale],
            extras: &'static [Locale],
        }
        let cases = [
            Case {
                name: "default",
                additional: &[],
                optional: &[],
                extras: &[],
            },
            Case {
                name: "legacy German",
                additional: &[Locale::De],
                optional: &[],
                extras: &[Locale::De],
            },
            Case {
                name: "optional German",
                additional: &[],
                optional: &[Locale::De],
                extras: &[Locale::De],
            },
            Case {
                name: "no duplicate",
                additional: &[Locale::De],
                optional: &[Locale::De],
                extras: &[Locale::De],
            },
        ];
        for case in cases {
            let mut expected = defaults.to_vec();
            expected.extend_from_slice(case.extras);
            assert_eq!(
                translated_locales_with(case.additional, case.optional),
                expected,
                "{}",
                case.name
            );
        }
    }
}
