use std::collections::BTreeMap;
use std::fmt;
use std::marker::PhantomData;
use std::path::Path;

use rari_utils::io::read_to_string;
use serde::de::{self, value, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use url::Url;

use crate::error::Error;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct WebFeatures {
    pub features: BTreeMap<String, FeatureData>,
}

impl WebFeatures {
    pub fn from_file(path: &Path) -> Result<Self, Error> {
        let json_str = read_to_string(path)?;
        Ok(serde_json::from_str(&json_str)?)
    }

    pub fn feature_status(&self, features: &[&str]) -> Option<&SupportStatus> {
        if features.is_empty() {
            return None;
        }

        self.features.values().find_map(|feature_data| {
            if let Some(ref status) = feature_data.status {
                if feature_data
                    .compat_features
                    .iter()
                    .any(|key| features.contains(&key.as_str()))
                {
                    return Some(status);
                }
            }
            None
        })
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct FeatureData {
    /** Specification */
    #[serde(
        deserialize_with = "t_or_vec",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub spec: Vec<Url>,
    /** caniuse.com identifier */
    #[serde(
        deserialize_with = "t_or_vec",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub caniuse: Vec<String>,
    /** Whether a feature is considered a "baseline" web platform feature and when it achieved that status */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<SupportStatus>,
    /** Sources of support data for this feature */
    #[serde(
        deserialize_with = "t_or_vec",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub compat_features: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum BrowserIdentifier {
    Chrome,
    ChromeAndroid,
    Edge,
    Firefox,
    FirefoxAndroid,
    Safari,
    SafariIos,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
#[serde(rename_all = "snake_case")]
pub enum BaselineHighLow {
    High,
    Low,
    #[serde(untagged)]
    False(bool),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SupportStatus {
    /// Whether the feature is Baseline (low substatus), Baseline (high substatus), or not (false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline: Option<BaselineHighLow>,
    /// Date the feature achieved Baseline low status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_low_date: Option<String>,
    /// Date the feature achieved Baseline high status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_high_date: Option<String>,
    /// Browser versions that most-recently introduced the feature
    pub support: BTreeMap<BrowserIdentifier, String>,
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
