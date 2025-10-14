use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CssValuesItem {
    pub name: String,
    #[serde(rename = "specLink", default, skip_serializing_if = "Option::is_none")]
    pub spec_link: Option<crate::SpecLink>,
    pub prose: Option<String>,
    pub r#type: CssValueType,
    pub value: Option<String>,
    pub values: Option<CssValues>,
}
impl From<&CssValuesItem> for CssValuesItem {
    fn from(value: &CssValuesItem) -> Self {
        value.clone()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum CssValueType {
    #[serde(rename = "type")]
    Type,
    #[serde(rename = "function")]
    Function,
    #[serde(rename = "value")]
    Value,
    #[serde(rename = "selector")]
    Selector,
}
impl CssValueType {
    pub fn as_str(&self) -> &str {
        match *self {
            Self::Type => "type",
            Self::Function => "function",
            Self::Value => "value",
            Self::Selector => "selector",
        }
    }
}
impl From<&CssValueType> for CssValueType {
    fn from(value: &CssValueType) -> Self {
        *value
    }
}
impl std::str::FromStr for CssValueType {
    type Err = crate::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, crate::error::ConversionError> {
        match value {
            "type" => Ok(Self::Type),
            "function" => Ok(Self::Function),
            "value" => Ok(Self::Value),
            "selector" => Ok(Self::Selector),
            _ => Err("invalid value".into()),
        }
    }
}
impl std::convert::TryFrom<&str> for CssValueType {
    type Error = crate::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, crate::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for CssValueType {
    type Error = crate::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, crate::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for CssValueType {
    type Error = crate::error::ConversionError;
    fn try_from(value: String) -> Result<Self, crate::error::ConversionError> {
        value.parse()
    }
}
pub type CssValues = BTreeMap<String, CssValuesItem>;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ValueName(String);
impl std::ops::Deref for ValueName {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<ValueName> for String {
    fn from(value: ValueName) -> Self {
        value.0
    }
}
impl From<&ValueName> for ValueName {
    fn from(value: &ValueName) -> Self {
        value.clone()
    }
}
impl std::str::FromStr for ValueName {
    type Err = crate::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, crate::error::ConversionError> {
        if regress::Regex::new("^<[^>]+>$|^.*()$")
            .unwrap()
            .find(value)
            .is_none()
        {
            return Err("doesn't match pattern \"^<[^>]+>$|^.*()$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl std::convert::TryFrom<&str> for ValueName {
    type Error = crate::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, crate::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for ValueName {
    type Error = crate::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, crate::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for ValueName {
    type Error = crate::error::ConversionError;
    fn try_from(value: String) -> Result<Self, crate::error::ConversionError> {
        value.parse()
    }
}
impl<'de> serde::Deserialize<'de> for ValueName {
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ValuesItemType {
    #[serde(rename = "type")]
    Type,
    #[serde(rename = "function")]
    Function,
}
impl ValuesItemType {
    pub fn as_str(&self) -> &str {
        match *self {
            Self::Type => "type",
            Self::Function => "function",
        }
    }
}
impl From<&ValuesItemType> for ValuesItemType {
    fn from(value: &ValuesItemType) -> Self {
        *value
    }
}
impl std::str::FromStr for ValuesItemType {
    type Err = crate::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, crate::error::ConversionError> {
        match value {
            "type" => Ok(Self::Type),
            "function" => Ok(Self::Function),
            _ => Err("invalid value".into()),
        }
    }
}
impl std::convert::TryFrom<&str> for ValuesItemType {
    type Error = crate::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, crate::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for ValuesItemType {
    type Error = crate::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, crate::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for ValuesItemType {
    type Error = crate::error::ConversionError;
    fn try_from(value: String) -> Result<Self, crate::error::ConversionError> {
        value.parse()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Warning {
    pub msg: String,
    pub name: String,
}
impl From<&Warning> for Warning {
    fn from(value: &Warning) -> Self {
        value.clone()
    }
}
