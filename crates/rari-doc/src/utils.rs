//! # Utilities Module
//!
//! The `utils` module provides a collection of utility functions and constants that are used
//! throughout the system. These utilities include functions for handling frontmatter,
//! serialization, date-time formatting, and more.

use std::cmp::max;
use std::collections::HashSet;
use std::fmt;
use std::marker::PhantomData;
use std::path::Path;
use std::str::FromStr;
use std::sync::OnceLock;
use std::sync::mpsc::Sender;

use chrono::{NaiveDate, NaiveDateTime};
use icu_collator::options::{CollatorOptions, Strength};
use icu_collator::preferences::CollationNumericOrdering;
use icu_collator::{Collator, CollatorBorrowed, CollatorPreferences};
use icu_locale_core::locale;
use rari_types::error::EnvError;
use rari_types::globals::{blog_root, content_root, content_translated_root, settings};
use rari_types::locale::{Locale, LocaleError};
use serde::de::{self, SeqAccess, Visitor, value};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::DocError;
use crate::pages::page::{Page, PageCategory};

const FM_START_DELIM: &str = "---\n";
const FM_START_DELIM_LEN: usize = FM_START_DELIM.len();
const FM_END_DELIM: &str = "\n---\n";
const FM_END_DELIM_LEN: usize = FM_END_DELIM.len();

/// Splits the content into frontmatter and the rest of the content.
///
/// This function searches for the start and end delimiters of the frontmatter within the given content.
/// If both delimiters are found, it returns a tuple containing the frontmatter as an `Option<&str>`
/// and the character offset to the rest of the content. If the delimiters are not found, it returns `None` for the
/// frontmatter and `0` for the offset.
///
/// # Arguments
///
/// * `content` - A string slice that holds the content to be split.
///
/// # Returns
///
/// * `(Option<&str>, usize)` - A tuple where the first element is an `Option<&str>` containing the frontmatter
///   if found, and the second element is the offset to the rest of the content.
/// * `(None, 0)` - If the frontmatter delimiters are not found.
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

/// Serializes a value as `None`.
///
/// This function is a utility for custom serialization logic. It always serializes the given value as `None`,
/// regardless of the actual value.
pub(crate) fn as_null<S>(serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_none()
}

/// Serializes a `NaiveDateTime` in a custom format (ISO 8601).
///
/// This function serializes a `NaiveDateTime` object as a string in the format `"%Y-%m-%dT%H:%M:%S%.3fZ"`.
/// This format includes the date, time, and milliseconds, followed by a 'Z' to indicate UTC time.
/// Example serialized value: "2021-03-01T00:00:00.000Z".
pub(crate) fn modified_dt<S>(ndt: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&ndt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
}

/// Deserializes a value that can be either a single item or a list of items into a `Vec<T>`.
///
/// This function handles the case where a JSON value can be either a single item or a list of items.
/// If the value is a single item, it wraps it in a vector. If the value is already a list, it deserializes
/// it directly into a vector.
pub(crate) fn t_or_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
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

/// Serializes a `Vec<T>` as either a single element or a sequence of elements.
///
/// This function serializes a vector of items. If the vector contains exactly one item,
/// it serializes the item directly as a single element. If the vector contains more than one item,
/// it serializes the vector as a sequence of elements.
pub(crate) fn serialize_t_or_vec<T, S>(value: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error>
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

/// Returns the root path for the given locale.
///
/// This function determines the root path for the specified locale. If the locale is `Locale::EnUs`,
/// it returns the content root path. For other locales, it attempts to return the translated content root path.
/// If the translated content root path is not set, it returns an `EnvError::NoTranslatedContent` error.
///
/// # Arguments
///
/// * `locale` - A `Locale` that specifies the locale for which the root path is to be determined.
pub fn root_for_locale(locale: Locale) -> Result<&'static Path, EnvError> {
    match locale {
        Locale::EnUs => Ok(content_root()),
        _ => content_translated_root().ok_or(EnvError::NoTranslatedContent),
    }
}

/// Determines the locale and page category from the given file path.
///
/// This function attempts to determine the locale and page category (`Doc`, `BlogPost`, etc) based on the provided
/// file path. It checks if the path starts with the content root, blog root, or translated content root directories,
/// and returns the corresponding locale and page category. If the path does not match any of these roots,
/// it returns a `DocError::LocaleError`.
///
/// # Arguments
///
/// * `path` - A reference to a `Path` that holds the file path to be analyzed.
///
/// # Returns
///
/// * `Result<(Locale, PageCategory), DocError>` - Returns a tuple containing the locale and page category if successful,
///   or a `DocError` if the locale cannot be determined from the path.
///
/// # Errors
///
/// This function will return an error if:
/// - The path does not contain a recognizable locale.
pub(crate) fn locale_and_typ_from_path(path: &Path) -> Result<(Locale, PageCategory), DocError> {
    if path.starts_with(content_root()) {
        return Ok((Locale::EnUs, PageCategory::Doc));
    }

    if let Some(root) = blog_root()
        && path.starts_with(root)
    {
        return Ok((Locale::EnUs, PageCategory::BlogPost));
    }
    if let Some(root) = content_translated_root()
        && let Ok(relative) = path.strip_prefix(root)
        && let Some(locale_str) = relative.components().next()
    {
        let locale_str = locale_str
            .as_os_str()
            .to_str()
            .ok_or(LocaleError::NoLocaleInPath)?;
        let locale = Locale::from_str(locale_str)?;
        return Ok((locale, PageCategory::Doc));
    }
    Err(DocError::LocaleError(LocaleError::NoLocaleInPath))
}

/// Removes consecutive whitespace characters from a string, leaving only single spaces.
///
/// This function trims leading and trailing whitespace from the input string and then
/// removes consecutive whitespace characters, leaving only single spaces between words.
/// The resulting string is returned.
///
/// # Arguments
///
/// * `s` - A string slice that holds the input string to be processed.
///
/// # Returns
///
/// * `String` - Returns a new string with consecutive whitespace characters reduced to single spaces.
pub(crate) fn dedup_whitespace(s: &str) -> String {
    let mut out = s.trim().to_owned();
    let mut prev = ' ';
    out.retain(|c| {
        let result = c != ' ' || prev != ' ';
        prev = c;
        result
    });
    out
}

/// Calculates the estimated read time for a given string.
///
/// This function estimates the read time for the provided string by counting the number of words
/// and dividing by the average reading speed (220 words per minute). It also handles hidden code blocks
/// (denoted by triple backticks and the word "hidden") by excluding them from the word count.
///
/// # Arguments
///
/// * `s` - A string slice that holds the input text to be analyzed.
///
/// # Returns
///
/// * `usize` - Returns the estimated read time in minutes. The minimum read time is 1 minute.
pub(crate) fn calculate_read_time_minutes(s: &str) -> usize {
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

/// Removes duplicate elements from a vector while preserving the order of the first occurrence.
///
/// This function takes a vector of items, removes any duplicate elements, and returns a new vector
/// containing only unique elements. The order of the first occurrence of each element is preserved.
///
/// # Type Parameters
///
/// * `T` - The type of the items in the vector. Must implement `Eq`, `Clone`, and `std::hash::Hash`.
///
/// # Arguments
///
/// * `vec` - A vector of items from which duplicates are to be removed.
///
/// # Returns
///
/// * `Vec<T>` - Returns a new vector containing only unique elements from the input vector.
pub(crate) fn deduplicate<T: Eq + Clone + std::hash::Hash>(vec: Vec<T>) -> Vec<T> {
    let mut seen = vec.iter().cloned().collect::<HashSet<_>>();
    vec.into_iter().filter(|item| seen.remove(item)).collect()
}

#[cfg(test)]
mod text {
    use super::*;

    #[test]
    fn test_trim_ws() {
        assert_eq!(
            dedup_whitespace(" foo  bar 20       00    "),
            "foo bar 20 00"
        );
        assert_eq!(dedup_whitespace("    "), "");
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
        assert_eq!(calculate_read_time_minutes(&s), 1);
    }
}

thread_local! {
    pub static COLLATOR: CollatorBorrowed<'static> =  {
        let mut prefs = CollatorPreferences::from(locale!("en-US"));
        prefs.numeric_ordering = Some(CollationNumericOrdering::True);
        let mut options = CollatorOptions::default();
        options.strength = Some(Strength::Primary);
        Collator::try_new(prefs, options).unwrap()
    };
}

pub static TEMPL_RECORDER_SENDER: OnceLock<Sender<String>> = OnceLock::new();
thread_local! {
    pub static TEMPL_RECORDER: Option<Sender<String>> = {
        TEMPL_RECORDER_SENDER.get().cloned()
    };
}

pub fn trim_after<'a>(input: &'a str, pat: Option<&str>) -> &'a str {
    if let Some(pat) = pat
        && let Some(i) = input.find(pat)
    {
        return &input[..=i];
    }
    input
}

pub fn trim_before<'a>(input: &'a str, pat: Option<&str>) -> &'a str {
    if let Some(pat) = pat
        && let Some(i) = input.find(pat)
    {
        return &input[i..];
    }
    input
}

pub fn is_default<T: PartialEq + Default>(value: &T) -> bool {
    value == &T::default()
}

pub fn filter_unpublished_blog_post(post: &&Page, now: &NaiveDate) -> bool {
    settings().blog_unpublished
        || if let Page::BlogPost(post) = post {
            post.meta.published && &post.meta.date <= now
        } else {
            false
        }
}
