use std::fmt::Display;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_variant::to_variant_name;
use thiserror::Error;

use crate::globals::settings;

#[derive(PartialEq, Debug, Clone, Copy, Deserialize, Serialize, Default, PartialOrd, Eq, Ord)]
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
    PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy, Deserialize, Serialize, Default, Hash,
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

impl Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(to_variant_name(self).map_err(|_| std::fmt::Error)?)
    }
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

    pub fn all() -> &'static [Self] {
        settings()
            .active_locales
            .as_deref()
            .unwrap_or([Locale::EnUs].as_slice())
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
