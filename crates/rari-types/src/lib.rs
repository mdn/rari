use std::fmt::Display;

use chrono::{DateTime, NaiveDateTime};
use indexmap::IndexMap;
use locale::Locale;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::fm_types::{FeatureStatus, PageType};

pub mod error;
pub mod fm_types;
pub mod globals;
pub mod locale;
pub mod settings;
pub mod templ;

#[derive(Clone, Debug, Error)]
pub enum ArgError {
    #[error("must be a string")]
    MustBeString,
    #[error("must be an integer")]
    MustBeInt,
    #[error("must be a boolean")]
    MustBeBool,
    #[error("must be provided")]
    MustBeProvided,
    #[error("must not be empty")]
    MustNotBeEmpty,
    #[error("could not be parsed")]
    MustBeParsable,
    #[error("is provided more than once")]
    Duplicate,
    #[error("{templ} has no argument named {name}")]
    UnknownName { templ: &'static str, name: String },
    #[error("{templ} has a positional argument after a named one")]
    PositionalAfterNamed { templ: String },
    #[error("{templ} argument {pos} ({name}) {source}")]
    At {
        templ: &'static str,
        pos: usize,
        name: &'static str,
        #[source]
        source: Box<ArgError>,
    },
}

impl ArgError {
    pub fn at(templ: &'static str, pos: usize, name: &'static str, source: ArgError) -> Self {
        ArgError::At {
            templ,
            pos,
            name,
            source: Box::new(source),
        }
    }
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Quotes {
    Double,
    Single,
    Back,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Arg {
    String(String, Quotes),
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl Arg {
    pub fn is_blank(&self) -> bool {
        matches!(self, Arg::String(s, _) if s.is_empty())
    }
}

/// Macro arguments: positional ones in order, followed by named (`key=value`) ones.
///
/// `None` marks an argument that could not be parsed.
#[derive(PartialEq, Clone, Debug, Default)]
pub struct Args {
    pub positional: Vec<Option<Arg>>,
    pub named: Vec<(String, Option<Arg>)>,
}

impl From<Vec<Option<Arg>>> for Args {
    fn from(positional: Vec<Option<Arg>>) -> Self {
        Args {
            positional,
            named: vec![],
        }
    }
}

impl Args {
    /// Assigns the arguments to the macro's `params`, returning one slot per parameter
    /// (`None` if not provided). Parameters starting with `_` can't be named.
    pub fn bind(
        self,
        templ: &'static str,
        params: &[&'static str],
    ) -> Result<Vec<Option<Option<Arg>>>, ArgError> {
        let mut slots: Vec<Option<Option<Arg>>> = self.positional.into_iter().map(Some).collect();
        slots.resize(slots.len().max(params.len()), None);
        for (name, value) in self.named {
            let Some(i) = params
                .iter()
                .position(|param| !param.starts_with('_') && *param == name)
            else {
                return Err(ArgError::UnknownName { templ, name });
            };
            if slots[i].is_some() {
                return Err(ArgError::at(templ, i + 1, params[i], ArgError::Duplicate));
            }
            slots[i] = Some(value);
        }
        slots.truncate(params.len());
        Ok(slots)
    }
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct AnyArg {
    pub value: Arg,
}

impl AnyArg {
    pub fn as_int(&self) -> i64 {
        match &self.value {
            Arg::String(s, _) => s
                .trim_matches(|c: char| !c.is_numeric())
                .parse()
                .unwrap_or(0),
            Arg::Int(n) => *n,
            Arg::Float(f) => *f as i64,
            Arg::Bool(true) => 1,
            Arg::Bool(false) => 0,
        }
    }

    pub fn as_bool(&self) -> bool {
        match &self.value {
            Arg::String(s, _) => !s.is_empty(),
            Arg::Int(n) => *n != 0,
            Arg::Float(f) => *f != 0f64 && !f.is_nan(),
            Arg::Bool(b) => *b,
        }
    }
}

impl Display for AnyArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.value {
            Arg::String(s, _) => f.write_str(s),
            Arg::Int(n) => f.write_fmt(format_args!("{n}")),
            Arg::Float(n) => f.write_fmt(format_args!("{n}")),
            Arg::Bool(b) => f.write_fmt(format_args!("{b}")),
        }
    }
}

impl TryFrom<Arg> for AnyArg {
    type Error = ArgError;

    fn try_from(value: Arg) -> Result<Self, Self::Error> {
        Ok(AnyArg { value })
    }
}

impl TryInto<String> for Arg {
    type Error = ArgError;

    fn try_into(self) -> Result<String, Self::Error> {
        if let Arg::String(s, _) = self {
            Ok(s)
        } else {
            Err(ArgError::MustBeString)
        }
    }
}

impl TryInto<i64> for Arg {
    type Error = ArgError;

    fn try_into(self) -> Result<i64, Self::Error> {
        if let Arg::Int(i) = self {
            Ok(i)
        } else {
            Err(ArgError::MustBeInt)
        }
    }
}

impl TryInto<bool> for Arg {
    type Error = ArgError;

    fn try_into(self) -> Result<bool, Self::Error> {
        if let Arg::Bool(b) = self {
            Ok(b)
        } else {
            Err(ArgError::MustBeBool)
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct RariEnv<'a> {
    pub url: &'a str,
    pub locale: Locale,
    pub title: &'a str,
    pub tags: &'a [String],
    pub browser_compat: &'a [String],
    pub spec_urls: &'a [String],
    pub page_type: PageType,
    pub slug: &'a str,
    pub status: &'a [FeatureStatus],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HistoryEntry {
    pub modified: NaiveDateTime,
    pub hash: String,
}

impl HistoryEntry {
    pub fn new(date: &str, hash: &str) -> Self {
        Self {
            modified: DateTime::parse_from_rfc3339(date)
                .unwrap_or_default()
                .naive_utc(),
            hash: hash.to_string(),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Popularities {
    pub popularities: IndexMap<String, f64>,
    pub date: NaiveDateTime,
}
