use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Links(pub std::collections::BTreeMap<Url, LinksValue>);
impl std::ops::Deref for Links {
    type Target = std::collections::BTreeMap<Url, LinksValue>;
    fn deref(&self) -> &std::collections::BTreeMap<Url, LinksValue> {
        &self.0
    }
}
impl From<Links> for std::collections::BTreeMap<Url, LinksValue> {
    fn from(value: Links) -> Self {
        value.0
    }
}
impl From<&Links> for Links {
    fn from(value: &Links) -> Self {
        value.clone()
    }
}
impl From<std::collections::BTreeMap<Url, LinksValue>> for Links {
    fn from(value: std::collections::BTreeMap<Url, LinksValue>) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LinksValue {
    #[serde(default)]
    pub anchors: Vec<String>,
    #[serde(rename = "specShortname", default)]
    pub spec_shortname: Option<Shortname>,
}
impl From<&LinksValue> for LinksValue {
    fn from(value: &LinksValue) -> Self {
        value.clone()
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Shortname(String);
impl std::ops::Deref for Shortname {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<Shortname> for String {
    fn from(value: Shortname) -> Self {
        value.0
    }
}
impl From<&Shortname> for Shortname {
    fn from(value: &Shortname) -> Self {
        value.clone()
    }
}
impl std::str::FromStr for Shortname {
    type Err = crate::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, crate::error::ConversionError> {
        if regress::Regex::new("^[\\w\\-]+((?<=-v?\\d+)\\.\\d+)?$")
            .unwrap()
            .find(value)
            .is_none()
        {
            return Err("doesn't match pattern \"^[\\w\\-]+((?<=-v?\\d+)\\.\\d+)?$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl std::convert::TryFrom<&str> for Shortname {
    type Error = crate::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, crate::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for Shortname {
    type Error = crate::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, crate::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for Shortname {
    type Error = crate::error::ConversionError;
    fn try_from(value: String) -> Result<Self, crate::error::ConversionError> {
        value.parse()
    }
}
impl<'de> serde::Deserialize<'de> for Shortname {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(|e: crate::error::ConversionError| {
                <D::Error as serde::de::Error>::custom(e.to_string())
            })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct SpecLink {
    pub title: String,
    pub url: Url,
}
impl From<&SpecLink> for SpecLink {
    fn from(value: &SpecLink) -> Self {
        value.clone()
    }
}
