use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt;
use std::marker::PhantomData;
use std::path::Path;

use indexmap::IndexMap;
use rari_utils::concat_strs;
use rari_utils::io::read_to_string;
use schemars::schema::Schema;
use schemars::{JsonSchema, SchemaGenerator};
use serde::de::{self, value, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use url::Url;

use crate::error::Error;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Baseline<'a> {
    #[serde(flatten)]
    pub support: &'a SupportStatus,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub asterisk: bool,
    pub feature: &'a FeatureData,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct WebFeatures {
    pub features: IndexMap<String, FeatureData>,
    pub bcd_keys: Vec<KeyStatus>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct KeyStatus {
    bcd_key: String,
    feature: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DirtyWebFeatures {
    pub features: IndexMap<String, Value>,
}

#[inline]
fn spaced(bcd_key: &str) -> String {
    bcd_key.replace('.', " ")
}

impl WebFeatures {
    pub fn from_file(path: &Path) -> Result<Self, Error> {
        let json_str = read_to_string(path)?;
        let dirty_map: DirtyWebFeatures = serde_json::from_str(&json_str)?;
        let features: IndexMap<String, FeatureData> = dirty_map
            .features
            .into_iter()
            .filter_map(|(k, v)| {
                serde_json::from_value::<FeatureData>(v)
                    .inspect_err(|e| {
                        tracing::error!("Error serializing baseline for {}: {}", k, &e)
                    })
                    .ok()
                    .map(|v| (k, v))
            })
            .collect();
        // bcd_keys is a sorted by KeyStatus.bcd_key
        // We replace "." with " " so the sorting is stable as in:
        // http headers Content-Security-Policy
        // http headers Content-Security-Policy base-uri
        // http headers Content-Security-Policy child-src
        // http headers Content-Security-Policy-Report-Only
        //
        // instead of:
        // http.headers.Content-Security-Policy
        // http.headers.Content-Security-Policy-Report-Only
        // http.headers.Content-Security-Policy.base-uri
        // http.headers.Content-Security-Policy.child-src
        //
        // This allows to simple return ranges when looking for keys prefixed with
        // `http headers Content-Security-Policy`
        let mut bcd_keys: Vec<KeyStatus> = features
            .iter()
            .flat_map(|(feature, fd)| {
                fd.compat_features.iter().map(|bcd_key| KeyStatus {
                    bcd_key: spaced(bcd_key),
                    feature: feature.clone(),
                })
            })
            .collect();
        bcd_keys.sort_by(|a, b| a.bcd_key.cmp(&b.bcd_key));
        bcd_keys.dedup_by(|a, b| a.bcd_key == b.bcd_key);

        let map = WebFeatures { features, bcd_keys };
        Ok(map)
    }

    pub fn sub_keys(&self, bcd_key: &str) -> &[KeyStatus] {
        let suffix = concat_strs!(bcd_key, " ");
        if let Ok(start) = self
            .bcd_keys
            .binary_search_by_key(&bcd_key, |ks| &ks.bcd_key)
        {
            if start < self.bcd_keys.len() {
                if let Some(end) = self.bcd_keys[start + 1..]
                    .iter()
                    .position(|ks| !ks.bcd_key.starts_with(&suffix))
                {
                    return &self.bcd_keys[start + 1..start + 1 + end];
                }
            }
        }
        &[]
    }

    // Compute status according to:
    // https://github.com/mdn/yari/issues/11546#issuecomment-2531611136
    pub fn feature_status(&self, bcd_key: &str) -> Option<Baseline> {
        let bcd_key_spaced = &spaced(bcd_key);
        if let Some(feature) = self.feature_status_internal(bcd_key_spaced) {
            if let Some(status) = feature.status.as_ref() {
                if let Some(support) = status
                    .by_compat_key
                    .as_ref()
                    .and_then(|by_key| by_key.get(bcd_key))
                {
                    let sub_keys = self.sub_keys(bcd_key_spaced);
                    let sub_status = sub_keys
                        .iter()
                        .map(|sub_key| {
                            self.feature_internal_with_feature_name(&sub_key.feature)
                                .and_then(|feature| feature.status.as_ref())
                                .and_then(|status| status.baseline)
                        })
                        .collect::<Vec<_>>();

                    if sub_status
                        .iter()
                        .all(|baseline| baseline == &status.baseline)
                    {
                        return Some(Baseline {
                            support,
                            asterisk: false,
                            feature,
                        });
                    }
                    match status.baseline {
                        Some(BaselineHighLow::False) => {
                            let Support {
                                chrome,
                                chrome_android,
                                firefox,
                                firefox_android,
                                safari,
                                safari_ios,
                                ..
                            } = &status.support;
                            if chrome == chrome_android
                                && firefox == firefox_android
                                && safari == safari_ios
                            {
                                return Some(Baseline {
                                    support,
                                    asterisk: false,
                                    feature,
                                });
                            }
                        }
                        Some(BaselineHighLow::Low) => {
                            if sub_status.iter().all(|ss| {
                                matches!(ss, Some(BaselineHighLow::Low | BaselineHighLow::High))
                            }) {
                                return Some(Baseline {
                                    support,
                                    asterisk: false,
                                    feature,
                                });
                            }
                        }
                        _ => {}
                    }
                    return Some(Baseline {
                        support,
                        asterisk: true,
                        feature,
                    });
                }
            }
        }
        None
    }

    fn feature_status_internal(&self, bcd_key_spaced: &str) -> Option<&FeatureData> {
        if let Ok(i) = self
            .bcd_keys
            .binary_search_by(|ks| ks.bcd_key.as_str().cmp(bcd_key_spaced))
        {
            let feature_name = &self.bcd_keys[i].feature;
            return self.feature_internal_with_feature_name(feature_name);
        }
        None
    }

    fn feature_internal_with_feature_name(&self, feature_name: &str) -> Option<&FeatureData> {
        if let Some(feature_data) = self.features.get(feature_name) {
            if feature_data.discouraged.is_some() {
                return None;
            }
            return Some(feature_data);
        }
        None
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct FeatureData {
    /** Specification */
    #[serde(deserialize_with = "t_or_vec", default, skip_serializing)]
    pub spec: Vec<Url>,
    /** caniuse.com identifier */
    #[serde(deserialize_with = "t_or_vec", default, skip_serializing)]
    pub caniuse: Vec<String>,
    /** Whether a feature is considered a "baseline" web platform feature and when it achieved that status */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<SupportStatusWithByKey>,
    /** Sources of support data for this feature */
    #[serde(deserialize_with = "t_or_vec", default, skip_serializing)]
    pub compat_features: Vec<String>,
    #[serde(skip_serializing)]
    pub description: String,
    pub description_html: String,
    #[serde(
        deserialize_with = "t_or_vec",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    #[serde(skip_serializing)]
    pub group: Vec<String>,
    pub name: String,
    #[serde(deserialize_with = "t_or_vec", default, skip_serializing)]
    pub snapshot: Vec<String>,
    /** Whether developers are formally discouraged from using this feature */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discouraged: Option<Discouraged>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct Discouraged {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    according_to: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    alternatives: Vec<String>,
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
#[derive(
    Deserialize, Serialize, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, JsonSchema,
)]
pub struct Support {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    chrome: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    chrome_android: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    edge: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    firefox: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    firefox_android: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    safari: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    safari_ios: Option<String>,
}
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BaselineHighLow {
    High,
    Low,
    #[serde(
        untagged,
        serialize_with = "serialize_false",
        deserialize_with = "deserialize_false"
    )]
    False,
}

// Deriving JsonSchema fails to type the false case. So we do it manually.
impl JsonSchema for BaselineHighLow {
    fn schema_name() -> String {
        "BaselineHighLow".into()
    }

    fn schema_id() -> Cow<'static, str> {
        concat!(module_path!(), "::BaselineHighLow").into()
    }

    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        serde_json::from_str(
            r#"{"oneOf": [
          {
            "type": "string",
            "enum": ["high", "low"]
          },
          {
            "type": "boolean",
            "enum": [false]
          }
        ]}"#,
        )
        .unwrap()
    }
}

fn serialize_false<S>(serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_bool(false)
}

fn deserialize_false<'de, D>(deserializer: D) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
{
    let value = bool::deserialize(deserializer)?;
    if !value {
        Ok(())
    } else {
        Err(serde::de::Error::custom("expected false"))
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
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
    pub support: Support,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct SupportStatusWithByKey {
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
    pub support: Support,
    #[serde(default, skip_serializing)]
    pub by_compat_key: Option<BTreeMap<String, SupportStatus>>,
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_baseline_high_low() {
        let json = r#"false"#;
        let bl = serde_json::from_str::<BaselineHighLow>(json);
        assert!(matches!(bl, Ok(BaselineHighLow::False)));
        let json = r#""high""#;
        let bl = serde_json::from_str::<BaselineHighLow>(json);
        assert!(matches!(bl, Ok(BaselineHighLow::High)));
    }
}
