use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::Path;

use indexmap::IndexMap;
use rari_utils::concat_strs;
use rari_utils::io::read_to_string;
use schemars::{JsonSchema, Schema, SchemaGenerator};
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
    pub features: IndexMap<String, FeatureEnum>,
    pub bcd_keys: Vec<KeyStatus>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct KeyStatus {
    bcd_key_spaced: String,
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
        let features: IndexMap<String, FeatureEnum> = dirty_map
            .features
            .into_iter()
            .filter_map(|(k, v)| {
                serde_json::from_value::<FeatureEnum>(v)
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
            .filter_map(|(feature, fe)| match fe {
                FeatureEnum::Feature(fd) => {
                    Some(fd.compat_features.iter().map(|bcd_key| KeyStatus {
                        bcd_key: bcd_key.clone(),
                        bcd_key_spaced: spaced(bcd_key),
                        feature: feature.clone(),
                    }))
                }
                _ => None,
            })
            .flat_map(|x| x.into_iter())
            .collect();
        bcd_keys.sort_by(|a, b| a.bcd_key_spaced.cmp(&b.bcd_key_spaced));
        bcd_keys.dedup_by(|a, b| a.bcd_key_spaced == b.bcd_key_spaced);

        let map = WebFeatures { features, bcd_keys };
        Ok(map)
    }

    pub fn sub_keys(&self, bcd_key_spaced: &str) -> &[KeyStatus] {
        let suffix = concat_strs!(bcd_key_spaced, " ");
        if let Ok(start) = self
            .bcd_keys
            .binary_search_by_key(&bcd_key_spaced, |ks| &ks.bcd_key_spaced)
            && start < self.bcd_keys.len()
            && let Some(end) = self.bcd_keys[start + 1..]
                .iter()
                .position(|ks| !ks.bcd_key_spaced.starts_with(&suffix))
        {
            return &self.bcd_keys[start + 1..start + 1 + end];
        }
        &[]
    }

    // Compute status according to:
    // https://github.com/mdn/yari/issues/11546#issuecomment-2531611136
    pub fn baseline_by_bcd_key(&self, bcd_key: &str) -> Option<Baseline<'_>> {
        let bcd_key_spaced = &spaced(bcd_key);
        if let Some(feature) = self.feature_data_by_key(bcd_key_spaced)
            && let Some(status_for_key) = feature
                .status
                .by_compat_key
                .as_ref()
                .and_then(|by_key| by_key.get(bcd_key))
        {
            let sub_keys = self.sub_keys(bcd_key_spaced);
            let sub_status = sub_keys
                .iter()
                .map(|sub_key| {
                    self.feature_data_by_name(&sub_key.feature)
                        .and_then(|feature| feature.status.by_compat_key.as_ref())
                        .and_then(|by_key| by_key.get(&sub_key.bcd_key))
                        .map(|status_for_key| status_for_key.baseline)
                })
                .collect::<Vec<_>>();

            let asterisk = if sub_status
                .iter()
                .all(|baseline| baseline == &Some(status_for_key.baseline))
            {
                false
            } else {
                match status_for_key.baseline {
                    BaselineHighLow::False => {
                        let Support {
                            chrome,
                            chrome_android,
                            firefox,
                            firefox_android,
                            safari,
                            safari_ios,
                            ..
                        } = &status_for_key.support;
                        !(chrome == chrome_android
                            && firefox == firefox_android
                            && safari == safari_ios)
                    }
                    BaselineHighLow::Low => !sub_status
                        .iter()
                        .all(|ss| matches!(ss, Some(BaselineHighLow::Low | BaselineHighLow::High))),
                    _ => true,
                }
            };
            return Some(Baseline {
                support: status_for_key,
                asterisk,
                feature,
            });
        }
        None
    }

    fn feature_data_by_key(&self, bcd_key_spaced: &str) -> Option<&FeatureData> {
        if let Ok(i) = self
            .bcd_keys
            .binary_search_by(|ks| ks.bcd_key_spaced.as_str().cmp(bcd_key_spaced))
        {
            let feature_name = &self.bcd_keys[i].feature;
            return self.feature_data_by_name(feature_name);
        }
        None
    }

    fn feature_data_by_name(&self, feature_name: &str) -> Option<&FeatureData> {
        if let Some(feature_enum) = self.features.get(feature_name) {
            match feature_enum {
                FeatureEnum::Feature(feature_data) => {
                    if feature_data.discouraged.is_some() {
                        return None;
                    }
                    return Some(feature_data);
                }
                _ => {
                    return None;
                }
            }
        }
        None
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum FeatureEnum {
    Feature(Box<FeatureData>),
    Moved(FeatureMovedData),
    Split(FeatureSplitData),
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct FeatureMovedData {
    pub redirect_target: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct FeatureSplitData {
    pub redirect_targets: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct FeatureData {
    /** Specification */
    #[serde(default, skip_serializing)]
    pub spec: Vec<Url>,
    /** caniuse.com identifier */
    #[serde(default, skip_serializing)]
    pub caniuse: Vec<String>,
    /** Whether a feature is considered a "baseline" web platform feature and when it achieved that status */
    pub status: SupportStatusWithByKey,
    /** Sources of support data for this feature */
    #[serde(default, skip_serializing)]
    pub compat_features: Vec<String>,
    #[serde(skip_serializing)]
    pub description: String,
    pub description_html: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[serde(skip_serializing)]
    pub group: Vec<String>,
    pub name: String,
    #[serde(default, skip_serializing)]
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
    fn schema_name() -> Cow<'static, str> {
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
    pub baseline: BaselineHighLow,
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
    pub baseline: BaselineHighLow,
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
