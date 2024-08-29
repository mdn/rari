use std::cmp::max;
use std::collections::HashSet;
use std::fmt;
use std::marker::PhantomData;
use std::path::Path;
use std::str::FromStr;
use std::sync::mpsc::Sender;
use std::sync::OnceLock;

use chrono::NaiveDateTime;
use icu_collator::{Collator, CollatorOptions, Strength};
use icu_locid::locale;
use rari_types::error::EnvError;
use rari_types::globals::{blog_root, content_root, content_translated_root};
use rari_types::locale::{Locale, LocaleError};
use serde::de::{self, value, SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::docs::page::PageCategory;
use crate::error::DocError;

const FM_START_DELIM: &str = "---\n";
const FM_START_DELIM_LEN: usize = FM_START_DELIM.len();
const FM_END_DELIM: &str = "\n---\n";
const FM_END_DELIM_LEN: usize = FM_END_DELIM.len();

pub fn split_fm(content: &str) -> (Option<&str>, usize) {
    let start = content.find(FM_START_DELIM);
    let end = content.find(FM_END_DELIM);
    match (start, end) {
        (Some(s), Some(e)) => (
            Some(&content[s + FM_START_DELIM_LEN..e]),
            e + FM_END_DELIM_LEN,
        ),
        _ => (None, 0),
    }
}

pub fn as_null<S>(serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_none()
}

pub fn modified_dt<S>(ndt: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&ndt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
}

pub fn t_or_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct TOrVec<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for TOrVec<T>
    where
        T: Deserialize<'de>,
    {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or list of strings")
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(vec![Deserialize::deserialize(
                value::StrDeserializer::new(s),
            )?])
        }

        fn visit_seq<S>(self, seq: S) -> Result<Self::Value, S::Error>
        where
            S: SeqAccess<'de>,
        {
            Deserialize::deserialize(value::SeqAccessDeserializer::new(seq))
        }
    }

    deserializer.deserialize_any(TOrVec::<T>(PhantomData))
}

pub fn serialize_t_or_vec<T, S>(value: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Serialize,
    S: Serializer,
{
    if value.len() == 1 {
        // Serialize as a single element
        value[0].serialize(serializer)
    } else {
        // Serialize as a sequence
        let mut seq = serializer.serialize_seq(Some(value.len()))?;
        for item in value {
            seq.serialize_element(item)?;
        }
        seq.end()
    }
}

pub fn root_for_locale(locale: Locale) -> Result<&'static Path, EnvError> {
    match locale {
        Locale::EnUs => Ok(content_root()),
        _ => content_translated_root().ok_or(EnvError::NoTranslatedContent),
    }
}

pub fn locale_and_typ_from_path(path: &Path) -> Result<(Locale, PageCategory), DocError> {
    if path.starts_with(content_root()) {
        return Ok((Locale::EnUs, PageCategory::Doc));
    }

    if let Some(root) = blog_root() {
        if path.starts_with(root) {
            return Ok((Locale::EnUs, PageCategory::BlogPost));
        }
    }
    if let Some(root) = content_translated_root() {
        if let Ok(relative) = path.strip_prefix(root) {
            if let Some(locale_str) = relative.components().next() {
                let locale_str = locale_str
                    .as_os_str()
                    .to_str()
                    .ok_or(LocaleError::NoLocaleInPath)?;
                let locale = Locale::from_str(locale_str)?;
                return Ok((locale, PageCategory::Doc));
            }
        }
    }
    Err(DocError::LocaleError(LocaleError::NoLocaleInPath))
}

pub fn dedup_ws(s: &str) -> String {
    let mut out = s.trim().to_owned();
    let mut prev = ' ';
    out.retain(|c| {
        let result = c != ' ' || prev != ' ';
        prev = c;
        result
    });
    out
}

pub fn readtime(s: &str) -> usize {
    /*
    const READ_TIME_FILTER = /[\w<>.,!?]+/;
    const HIDDEN_CODE_BLOCK_MATCH = /```.*hidden[\s\S]*?```/g;

    function calculateReadTime(copy: string): number {
      return Math.max(
        1,
        Math.round(
          copy
            .replace(HIDDEN_CODE_BLOCK_MATCH, "")
            .split(/\s+/)
            .filter((w) => READ_TIME_FILTER.test(w)).length / 220
        )
      );
    }
    */
    let mut words = 0;
    let mut in_fence = false;
    let mut skipping = false;
    for line in s.lines() {
        if line.starts_with("```") {
            if !in_fence && line.contains("hidden") {
                skipping = true;
            }
            in_fence = !in_fence;
            if !in_fence && skipping {
                skipping = false;
            }
        }
        if skipping {
            continue;
        }
        words += line
            .split_whitespace()
            .filter(|c| {
                c.chars().all(|c| {
                    c.is_alphabetic()
                        || c.is_numeric()
                        || ['<', '>', '_', '.', ',', '!', '?'].contains(&c)
                })
            })
            .count();
    }
    max(1, words).div_ceil(220)
}

pub fn deduplicate<T: Eq + Clone + std::hash::Hash>(vec: Vec<T>) -> Vec<T> {
    let mut seen = vec.iter().cloned().collect::<HashSet<_>>();
    vec.into_iter().filter(|item| seen.remove(item)).collect()
}

#[cfg(test)]
mod text {
    use super::*;

    #[test]
    fn test_trim_ws() {
        assert_eq!(dedup_ws(" foo  bar 20       00    "), "foo bar 20 00");
        assert_eq!(dedup_ws("    "), "");
    }

    #[test]
    fn test_locale_from_path() {
        let en_us = content_root();

        let path = en_us.to_path_buf().join("en-us/web/html/index.md");
        assert_eq!(
            locale_and_typ_from_path(&path).unwrap(),
            (Locale::EnUs, PageCategory::Doc)
        );
    }

    #[test]
    fn test_readtime() {
        let s = format!(
            r#"Foo
```hidden
{}
```
"#,
            "a lot of words.".repeat(100)
        );
        assert_eq!(readtime(&s), 1);
    }
}

thread_local! {
    pub static COLLATOR: Collator =  {
        let locale = locale!("en-US").into();
        let mut options = CollatorOptions::new();
        options.strength = Some(Strength::Primary);
        Collator::try_new(&locale, options).unwrap()
    };
}

pub static TEMPL_RECORDER_SENDER: OnceLock<Sender<String>> = OnceLock::new();
thread_local! {
    pub static TEMPL_RECORDER: Option<Sender<String>> = {
        TEMPL_RECORDER_SENDER.get().cloned()
    };
}

pub fn trim_after<'a>(input: &'a str, pat: Option<&str>) -> &'a str {
    if let Some(pat) = pat {
        if let Some(i) = input.find(pat) {
            return &input[..=i];
        }
    }
    input
}

pub fn trim_fefore<'a>(input: &'a str, pat: Option<&str>) -> &'a str {
    if let Some(pat) = pat {
        if let Some(i) = input.find(pat) {
            return &input[i..];
        }
    }
    input
}
