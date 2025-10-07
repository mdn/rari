use serde::{Deserialize, Serialize};
use url::Url;

use crate::SpecLink;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Extensiontype {
    Variant0(Interfacetype),
    Variant1(String),
}
impl From<&Extensiontype> for Extensiontype {
    fn from(value: &Extensiontype) -> Self {
        value.clone()
    }
}
impl From<Interfacetype> for Extensiontype {
    fn from(value: Interfacetype) -> Self {
        Self::Variant0(value)
    }
}
#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(untagged)]
pub enum Global {
    Variant0(Interface),
    Variant1(String),
}
impl From<&Global> for Global {
    fn from(value: &Global) -> Self {
        value.clone()
    }
}
impl std::str::FromStr for Global {
    type Err = crate::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, crate::error::ConversionError> {
        if let Ok(v) = value.parse::<Interface>() {
            Ok(Self::Variant0(v))
        } else {
            let v = value.to_string();
            Ok(Self::Variant1(v))
        }
    }
}
impl std::convert::TryFrom<&str> for Global {
    type Error = crate::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, crate::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for Global {
    type Error = crate::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, crate::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for Global {
    type Error = crate::error::ConversionError;
    fn try_from(value: String) -> Result<Self, crate::error::ConversionError> {
        value.parse()
    }
}

impl From<Interface> for Global {
    fn from(value: Interface) -> Self {
        Self::Variant0(value)
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IdlFragmentInSpec {
    pub fragment: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub href: Option<Url>,
    pub spec: SpecLink,
}
impl From<&IdlFragmentInSpec> for IdlFragmentInSpec {
    fn from(value: &IdlFragmentInSpec) -> Self {
        value.clone()
    }
}
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Interface(String);
impl std::ops::Deref for Interface {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
impl From<Interface> for String {
    fn from(value: Interface) -> Self {
        value.0
    }
}
impl From<&Interface> for Interface {
    fn from(value: &Interface) -> Self {
        value.clone()
    }
}
impl std::str::FromStr for Interface {
    type Err = crate::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, crate::error::ConversionError> {
        if regress::Regex::new("^[A-Z]([A-Za-z0-9_])*$|^console$")
            .unwrap()
            .find(value)
            .is_none()
        {
            return Err("doesn't match pattern \"^[A-Z]([A-Za-z0-9_])*$|^console$\"".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl std::convert::TryFrom<&str> for Interface {
    type Error = crate::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, crate::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for Interface {
    type Error = crate::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, crate::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for Interface {
    type Error = crate::error::ConversionError;
    fn try_from(value: String) -> Result<Self, crate::error::ConversionError> {
        value.parse()
    }
}
impl<'de> serde::Deserialize<'de> for Interface {
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
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Interfaces(pub Vec<Interface>);
impl std::ops::Deref for Interfaces {
    type Target = Vec<Interface>;
    fn deref(&self) -> &Vec<Interface> {
        &self.0
    }
}
impl From<Interfaces> for Vec<Interface> {
    fn from(value: Interfaces) -> Self {
        value.0
    }
}
impl From<&Interfaces> for Interfaces {
    fn from(value: &Interfaces) -> Self {
        value.clone()
    }
}
impl From<Vec<Interface>> for Interfaces {
    fn from(value: Vec<Interface>) -> Self {
        Self(value)
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InterfacesByGlobal(pub std::collections::BTreeMap<Global, Interfaces>);
impl std::ops::Deref for InterfacesByGlobal {
    type Target = std::collections::BTreeMap<Global, Interfaces>;
    fn deref(&self) -> &std::collections::BTreeMap<Global, Interfaces> {
        &self.0
    }
}
impl From<InterfacesByGlobal> for std::collections::BTreeMap<Global, Interfaces> {
    fn from(value: InterfacesByGlobal) -> Self {
        value.0
    }
}
impl From<&InterfacesByGlobal> for InterfacesByGlobal {
    fn from(value: &InterfacesByGlobal) -> Self {
        value.clone()
    }
}
impl From<std::collections::BTreeMap<Global, Interfaces>> for InterfacesByGlobal {
    fn from(value: std::collections::BTreeMap<Global, Interfaces>) -> Self {
        Self(value)
    }
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Interfacetype {
    #[serde(rename = "dictionary")]
    Dictionary,
    #[serde(rename = "interface")]
    Interface,
    #[serde(rename = "interface mixin")]
    InterfaceMixin,
    #[serde(rename = "enum")]
    Enum,
    #[serde(rename = "typedef")]
    Typedef,
    #[serde(rename = "callback")]
    Callback,
    #[serde(rename = "callback interface")]
    CallbackInterface,
    #[serde(rename = "namespace")]
    Namespace,
}
impl From<&Interfacetype> for Interfacetype {
    fn from(value: &Interfacetype) -> Self {
        *value
    }
}
impl Interfacetype {
    pub fn as_str(&self) -> &str {
        match *self {
            Self::Dictionary => "dictionary",
            Self::Interface => "interface",
            Self::InterfaceMixin => "interface mixin",
            Self::Enum => "enum",
            Self::Typedef => "typedef",
            Self::Callback => "callback",
            Self::CallbackInterface => "callback interface",
            Self::Namespace => "namespace",
        }
    }
}
impl std::str::FromStr for Interfacetype {
    type Err = crate::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, crate::error::ConversionError> {
        match value {
            "dictionary" => Ok(Self::Dictionary),
            "interface" => Ok(Self::Interface),
            "interface mixin" => Ok(Self::InterfaceMixin),
            "enum" => Ok(Self::Enum),
            "typedef" => Ok(Self::Typedef),
            "callback" => Ok(Self::Callback),
            "callback interface" => Ok(Self::CallbackInterface),
            "namespace" => Ok(Self::Namespace),
            _ => Err("invalid value".into()),
        }
    }
}
impl std::convert::TryFrom<&str> for Interfacetype {
    type Error = crate::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, crate::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for Interfacetype {
    type Error = crate::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, crate::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for Interfacetype {
    type Error = crate::error::ConversionError;
    fn try_from(value: String) -> Result<Self, crate::error::ConversionError> {
        value.parse()
    }
}
