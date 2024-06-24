use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use url::Url;

pub mod error {
    pub struct ConversionError(std::borrow::Cow<'static, str>);
    impl std::error::Error for ConversionError {}
    impl std::fmt::Display for ConversionError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
            std::fmt::Display::fmt(&self.0, f)
        }
    }
    impl std::fmt::Debug for ConversionError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
            std::fmt::Debug::fmt(&self.0, f)
        }
    }
    impl From<&'static str> for ConversionError {
        fn from(value: &'static str) -> Self {
            Self(value.into())
        }
    }
    impl From<String> for ConversionError {
        fn from(value: String) -> Self {
            Self(value.into())
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Css {
    pub atrules: BTreeMap<String, AtRule>,
    pub properties: BTreeMap<String, Property>,
    pub selectors: BTreeMap<String, Selector>,
    pub spec: SpecInExtract,
    pub values: CssValues,
    pub warnings: Option<BTreeMap<String, Warning>>,
}
impl From<&Css> for Css {
    fn from(value: &Css) -> Self {
        value.clone()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AtRule {
    pub descriptors: BTreeMap<String, AtruleDescriptor>,
    pub href: Option<Url>,
    pub name: String,
    pub prose: Option<String>,
    pub value: Option<String>,
    pub values: Option<CssValues>,
}
impl From<&AtRule> for AtRule {
    fn from(value: &AtRule) -> Self {
        value.clone()
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Property {
    pub href: Option<Url>,
    pub name: String,
    #[serde(rename = "newValues", default)]
    pub new_values: Option<String>,
    #[serde(rename = "styleDeclaration", default)]
    pub style_declaration: Vec<String>,
    pub value: Option<String>,
    pub values: Option<CssValues>,
}
impl From<&Property> for Property {
    fn from(value: &Property) -> Self {
        value.clone()
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Selector {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub href: Option<Url>,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prose: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub values: Option<CssValues>,
}
impl From<&Selector> for Selector {
    fn from(value: &Selector) -> Self {
        value.clone()
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AtruleDescriptor {
    #[serde(rename = "for")]
    pub for_: String,
    pub href: Option<Url>,
    pub name: String,
    pub value: Option<String>,
    pub values: Option<CssValues>,
}
impl From<&AtruleDescriptor> for AtruleDescriptor {
    fn from(value: &AtruleDescriptor) -> Self {
        value.clone()
    }
}
pub type CssValues = BTreeMap<String, CssValuesItem>;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CssValuesItem {
    pub href: Option<Url>,
    pub name: String,
    pub prose: Option<String>,
    #[serde(rename = "type")]
    pub type_: CssValueType,
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
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
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
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for CssValueType {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for CssValueType {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
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
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
        if let Ok(v) = value.parse() {
            Ok(Self::Variant0(v))
        } else if let Ok(v) = value.parse() {
            Ok(Self::Variant1(v))
        } else {
            Err("string conversion failed for all variants".into())
        }
    }
}
impl std::convert::TryFrom<&str> for Global {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for Global {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for Global {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
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
    pub spec: SpecInExtract,
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
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
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
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for Interface {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for Interface {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
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
            .map_err(|e: self::error::ConversionError| {
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
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
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
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for Interfacetype {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for Interfacetype {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
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
#[serde(deny_unknown_fields)]
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
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
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
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for Shortname {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for Shortname {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
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
            .map_err(|e: self::error::ConversionError| {
                <D::Error as serde::de::Error>::custom(e.to_string())
            })
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SpecInExtract {
    pub title: String,
    pub url: Url,
}
impl From<&SpecInExtract> for SpecInExtract {
    fn from(value: &SpecInExtract) -> Self {
        value.clone()
    }
}

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
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
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
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for ValueName {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for ValueName {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
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
            .map_err(|e: self::error::ConversionError| {
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
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
        match value {
            "type" => Ok(Self::Type),
            "function" => Ok(Self::Function),
            _ => Err("invalid value".into()),
        }
    }
}
impl std::convert::TryFrom<&str> for ValuesItemType {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<&String> for ValuesItemType {
    type Error = self::error::ConversionError;
    fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl std::convert::TryFrom<String> for ValuesItemType {
    type Error = self::error::ConversionError;
    fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
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
