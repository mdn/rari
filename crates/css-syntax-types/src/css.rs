use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::SpecLink;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebrefCssOld {
    pub atrules: BTreeMap<String, AtRule>,
    pub functions: BTreeMap<String, Function>,
    pub properties: BTreeMap<String, Property>,
    pub selectors: BTreeMap<String, Selector>,
    pub types: BTreeMap<String, Type>,
}
impl From<&WebrefCssOld> for WebrefCssOld {
    fn from(value: &WebrefCssOld) -> Self {
        value.clone()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebrefCss {
    pub atrules: BTreeMap<String, BTreeMap<String, AtRule>>,
    pub functions: BTreeMap<String, BTreeMap<String, Function>>,
    pub properties: BTreeMap<String, BTreeMap<String, Property>>,
    pub selectors: BTreeMap<String, BTreeMap<String, Selector>>,
    pub types: BTreeMap<String, BTreeMap<String, Type>>,
}
impl From<&WebrefCss> for WebrefCss {
    fn from(value: &WebrefCss) -> Self {
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
    pub descriptors: BTreeMap<String, AtRuleDescriptor>,
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
pub struct AtRuleDescriptor {
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
impl From<&AtRuleDescriptor> for AtRuleDescriptor {
    fn from(value: &AtRuleDescriptor) -> Self {
        value.clone()
    }
}
