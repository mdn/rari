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
pub struct Css {
    pub atrules: BTreeMap<String, AtRule>,
    pub functions: BTreeMap<String, Function>,
    pub properties: BTreeMap<String, Property>,
    pub selectors: BTreeMap<String, Selector>,
    pub types: BTreeMap<String, Type>,
}
impl From<&Css> for Css {
    fn from(value: &Css) -> Self {
        value.clone()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AtRule {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prose: Option<String>,
    #[serde(rename = "specLink", default, skip_serializing_if = "Option::is_none")]
    pub spec_link: Option<SpecLink>,
    #[serde(default)]
    pub descriptors: BTreeMap<String, AtruleDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syntax: Option<String>,
}
impl From<&AtRule> for AtRule {
    fn from(value: &AtRule) -> Self {
        value.clone()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Function {
    pub name: String,
    #[serde(rename = "specLink", default, skip_serializing_if = "Option::is_none")]
    pub spec_link: Option<SpecLink>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#for: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prose: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syntax: Option<String>,
}
impl From<&Function> for Function {
    fn from(value: &Function) -> Self {
        value.clone()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Type {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#for: Option<Vec<String>>,
    #[serde(rename = "specLink", default, skip_serializing_if = "Option::is_none")]
    pub spec_link: Option<SpecLink>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prose: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syntax: Option<String>,
}
impl From<&Type> for Type {
    fn from(value: &Type) -> Self {
        value.clone()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Property {
    pub name: String,
    #[serde(rename = "specLink", default, skip_serializing_if = "Option::is_none")]
    pub spec_link: Option<SpecLink>,
    #[serde(
        rename = "legacyAliasOf",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub legacy_alias_of: Option<String>,
    #[serde(rename = "newValues", default, skip_serializing_if = "Option::is_none")]
    pub new_values: Option<String>,
    #[serde(rename = "styleDeclaration", default)]
    pub style_declaration: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syntax: Option<String>,
}
impl From<&Property> for Property {
    fn from(value: &Property) -> Self {
        value.clone()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Selector {
    pub name: String,
    #[serde(rename = "specLink", default, skip_serializing_if = "Option::is_none")]
    pub spec_link: Option<SpecLink>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prose: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub syntax: Option<String>,
}
impl From<&Selector> for Selector {
    fn from(value: &Selector) -> Self {
        value.clone()
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AtruleDescriptor {
    pub name: String,
    #[serde(rename = "specLink", default, skip_serializing_if = "Option::is_none")]
    pub spec_link: Option<SpecLink>,
    pub r#for: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub syntax: Option<String>,
}
impl From<&AtruleDescriptor> for AtruleDescriptor {
    fn from(value: &AtruleDescriptor) -> Self {
        value.clone()
    }
}
pub type CssValues = BTreeMap<String, CssValuesItem>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CssValuesItem {
    pub name: String,
    #[serde(rename = "specLink", default, skip_serializing_if = "Option::is_none")]
    pub spec_link: Option<SpecLink>,
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
        if let Ok(v) = value.parse::<Interface>() {
            Ok(Self::Variant0(v))
        } else {
            let v = value.to_string();
            Ok(Self::Variant1(v))
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
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub struct SpecLink {
    pub title: String,
    pub url: Url,
}
impl From<&SpecLink> for SpecLink {
    fn from(value: &SpecLink) -> Self {
        value.clone()
    }
}

// In order to use it with in a BtreeSet<SpecLink>, implement `PartialOrd` and `Ord`
impl PartialOrd for SpecLink {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SpecLink {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // First compare by title, then by URL if titles are equal
        self.title
            .cmp(&other.title)
            .then_with(|| self.url.cmp(&other.url))
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

// browser specs

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BrowserSpec {
    pub url: String,
    #[serde(rename = "seriesComposition")]
    pub series_composition: String,
    pub shortname: String,
    pub series: Series,
    #[serde(rename = "seriesVersion")]
    pub series_version: Option<String>,
    #[serde(rename = "formerNames", default)]
    pub former_names: Vec<String>,
    pub nightly: Option<NightlySpec>,
    pub title: String,
    #[serde(rename = "shortTitle")]
    pub short_title: String,
    pub organization: String,
    pub groups: Vec<Group>,
    pub release: Option<ReleaseSpec>,
    pub source: String,
    pub categories: Vec<String>,
    pub standing: String,
    pub tests: Option<TestInfo>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>, // For any additional fields
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Series {
    pub shortname: String,
    #[serde(rename = "currentSpecification")]
    pub current_specification: String,
    pub title: String,
    #[serde(rename = "shortTitle")]
    pub short_title: String,
    #[serde(rename = "releaseUrl")]
    pub release_url: Option<String>,
    #[serde(rename = "nightlyUrl")]
    pub nightly_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NightlySpec {
    pub url: String,
    pub status: String,
    #[serde(rename = "sourcePath")]
    pub source_path: Option<String>,
    #[serde(rename = "alternateUrls", default)]
    pub alternate_urls: Vec<String>,
    pub repository: Option<String>,
    pub filename: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReleaseSpec {
    pub url: String,
    pub status: String,
    pub pages: Option<Vec<String>>,
    pub filename: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Group {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestInfo {
    pub repository: String,
    #[serde(rename = "testPaths")]
    pub test_paths: Vec<String>,
}
