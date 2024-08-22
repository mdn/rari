use std::path::PathBuf;

use chrono::NaiveDateTime;
use rari_data::baseline::SupportStatusWithByKey;
use rari_types::locale::{Locale, Native};
use serde::Serialize;

use super::blog::BlogMeta;
use super::curriculum::{CurriculumIndexEntry, CurriculumSidebarEntry, Template, Topic};
use super::dummy::BlogIndex;
use super::types::PrevNextCurriculum;
use crate::specs::Specification;
use crate::utils::modified_dt;

#[derive(Debug, Clone, Serialize, Default)]
pub struct TocEntry {
    pub text: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Source {
    pub folder: PathBuf,
    pub github_url: String,
    pub last_commit_url: String,
    pub filename: String,
}
#[derive(Debug, Clone, Serialize, Default)]
pub struct Parent {
    pub uri: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Translation {
    pub locale: Locale,
    pub title: String,
    pub native: Native,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Prose {
    pub id: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "isH3")]
    pub is_h3: bool,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Compat {
    pub id: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "isH3")]
    pub is_h3: bool,
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct SpecificationSection {
    pub id: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "isH3")]
    pub is_h3: bool,
    pub specifications: Vec<Specification>,
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Section {
    Prose(Prose),
    BrowserCompatibility(Compat),
    Specifications(SpecificationSection),
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct JsonDoc {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub body: Vec<Section>,
    #[serde(rename = "isActive")]
    pub is_active: bool,
    #[serde(rename = "isMarkdown")]
    pub is_markdown: bool,
    #[serde(rename = "isTranslated")]
    pub is_translated: bool,
    pub locale: Locale,
    pub mdn_url: String,
    #[serde(serialize_with = "modified_dt")]
    pub modified: NaiveDateTime,
    pub native: Native,
    #[serde(rename = "noIndexing")]
    pub no_indexing: bool,
    pub other_translations: Vec<Translation>,
    #[serde(rename = "pageTitle")]
    pub page_title: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parents: Vec<Parent>,
    pub popularity: Option<f64>,
    pub short_title: String,
    #[serde(rename = "sidebarHTML", skip_serializing_if = "Option::is_none")]
    pub sidebar_html: Option<String>,
    #[serde(rename = "sidebarMacro", skip_serializing_if = "Option::is_none")]
    pub sidebar_macro: Option<String>,
    pub source: Source,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub title: String,
    pub toc: Vec<TocEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline: Option<&'static SupportStatusWithByKey>,
    #[serde(rename = "browserCompat", skip_serializing_if = "Vec::is_empty")]
    pub browser_compat: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum HyData {
    BlogIndex(BlogIndex),
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct JsonDoADoc {
    #[serde(rename = "blogMeta", skip_serializing_if = "Option::is_none")]
    pub blog_meta: Option<BlogMeta>,
    pub doc: JsonDoc,
    pub url: String,
    #[serde(rename = "hyData", skip_serializing_if = "Option::is_none")]
    pub hy_data: Option<HyData>,
    #[serde(rename = "pageTitle", skip_serializing_if = "Option::is_none")]
    pub page_title: Option<String>,
}

pub struct BuiltDoc {
    pub json: JsonDoADoc,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct JsonCurriculumDoc {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub body: Vec<Section>,
    pub locale: Locale,
    pub mdn_url: String,
    pub native: Native,
    #[serde(rename = "noIndexing")]
    pub no_indexing: bool,
    #[serde(rename = "pageTitle")]
    pub page_title: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parents: Vec<Parent>,
    pub title: String,
    pub summary: Option<String>,
    pub toc: Vec<TocEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sidebar: Option<Vec<CurriculumSidebarEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<Topic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub modules: Vec<CurriculumIndexEntry>,
    #[serde(rename = "prevNext", skip_serializing_if = "Option::is_none")]
    pub prev_next: Option<PrevNextCurriculum>,
    pub template: Template,
}
#[derive(Debug, Clone, Serialize, Default)]
pub struct JsonCurriculum {
    pub doc: JsonCurriculumDoc,
    pub url: String,
    #[serde(rename = "pageTitle")]
    pub page_title: String,
    pub locale: Locale,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct JsonBlogPostDoc {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub body: Vec<Section>,
    pub mdn_url: String,
    pub native: Native,
    pub locale: Locale,
    #[serde(rename = "noIndexing")]
    pub no_indexing: bool,
    #[serde(rename = "pageTitle")]
    pub page_title: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parents: Vec<Parent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub title: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub toc: Vec<TocEntry>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct JsonBlogPost {
    pub doc: JsonBlogPostDoc,
    pub locale: Locale,
    pub url: String,
    pub image: Option<String>,
    #[serde(rename = "pageTitle")]
    pub page_title: String,
    #[serde(rename = "blogMeta", skip_serializing_if = "Option::is_none")]
    pub blog_meta: Option<BlogMeta>,
    #[serde(rename = "hyData", skip_serializing_if = "Option::is_none")]
    pub hy_data: Option<HyData>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum BuiltDocy {
    Doc(Box<JsonDoADoc>),
    Curriculum(Box<JsonCurriculum>),
    BlogPost(Box<JsonBlogPost>),
}
