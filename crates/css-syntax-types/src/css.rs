use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::SpecLink;

/// Represents a CSS specification. Each field contains a map of scopes, with each scope containing a map of
/// CSS entities to their respective definitions.
/// The top-level map of each field contains the scope (i.e. `for` references in webref parlance) of the specification.
/// The second level map is a simple name->spec relation.
/// Every field also have a `__global_scope__` field where all entries are held. In case of duplicates in the `__global_scope__`,
/// the last entry wins. Not a problem since those should be properly handled by their respective scopes.
/// The scope used is ultimately derived from the browser_compat key in the page's frontmatter.
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
    #[serde(
        rename = "extendedSpecLinks",
        skip_serializing_if = "Vec::is_empty",
        default
    )]
    pub extended_spec_links: Vec<SpecLink>,
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
    #[serde(
        rename = "extendedSpecLinks",
        skip_serializing_if = "Vec::is_empty",
        default
    )]
    pub extended_spec_links: Vec<SpecLink>,
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
    #[serde(
        rename = "extendedSpecLinks",
        skip_serializing_if = "Vec::is_empty",
        default
    )]
    pub extended_spec_links: Vec<SpecLink>,
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
    #[serde(
        rename = "extendedSpecLinks",
        skip_serializing_if = "Vec::is_empty",
        default
    )]
    pub extended_spec_links: Vec<SpecLink>,
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
    #[serde(
        rename = "extendedSpecLinks",
        skip_serializing_if = "Vec::is_empty",
        default
    )]
    pub extended_spec_links: Vec<SpecLink>,
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
