use std::path::PathBuf;

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use rari_data::baseline::SupportStatusWithByKey;
use rari_types::fm_types::PageType;
use rari_types::locale::{Locale, Native};
use serde::{Deserialize, Serialize};

use super::types::contributors::Usernames;
use super::types::curriculum::{CurriculumIndexEntry, CurriculumSidebarEntry, Template, Topic};
use crate::pages::types::blog::BlogMeta;
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
    #[serde(rename = "pageType")]
    pub page_type: PageType,
}

#[derive(Debug, Clone, Serialize)]
pub struct BlogIndex {
    pub posts: Vec<BlogMeta>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct JsonDoADoc {
    pub doc: JsonDoc,
    pub url: String,
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
    pub hy_data: Option<BlogIndex>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContributorSpotlightHyData {
    pub sections: Vec<Section>,
    #[serde(rename = "contributorName")]
    pub contributor_name: String,
    #[serde(rename = "folderName")]
    pub folder_name: String,
    #[serde(rename = "isFeatured")]
    pub is_featured: bool,
    #[serde(rename = "profileImg")]
    pub profile_img: String,
    #[serde(rename = "profileImgAlt")]
    pub profile_img_alt: String,
    pub usernames: Usernames,
    pub quote: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonContributorSpotlight {
    pub url: String,
    #[serde(rename = "pageTitle")]
    pub page_title: String,
    #[serde(rename = "hyData")]
    pub hy_data: ContributorSpotlightHyData,
}

/// Represents the different types of built documentation.
///
/// The `BuiltDocy` enum is used to classify various types of pages that can be
/// generated by the system. Each variant corresponds to a specific type of documentation,
/// encapsulated in a `Box` to allow for efficient memory management and dynamic dispatch.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum BuiltDocy {
    /// Represents a standard documentation page, backed by a Markdown source.
    Doc(Box<JsonDoADoc>),
    /// Represents a curriculum page.
    Curriculum(Box<JsonCurriculum>),
    /// Represents a blog post.
    BlogPost(Box<JsonBlogPost>),
    /// Represents a contributor spotlight page.
    ContributorSpotlight(Box<JsonContributorSpotlight>),
    /// Represents a generic page, i.e Observatory docs, About pages, etc.
    GenericPage(Box<JsonGenericPage>),
    /// Represents a basic single-page application.
    BasicSPA(Box<JsonBasicSPA>),
    /// Represents a home page single-page application.
    HomePageSPA(Box<JsonHomePageSPA>),
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(default)]
pub struct PrevNextBlog {
    pub previous: Option<SlugNTitle>,
    pub next: Option<SlugNTitle>,
}

impl PrevNextBlog {
    pub fn is_none(&self) -> bool {
        self.previous.is_none() && self.next.is_none()
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct SlugNTitle {
    pub title: String,
    pub slug: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(default)]
pub struct PrevNextCurriculum {
    pub prev: Option<UrlNTitle>,
    pub next: Option<UrlNTitle>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct UrlNTitle {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonBasicSPA {
    pub slug: &'static str,
    pub page_title: &'static str,
    pub page_description: Option<&'static str>,
    pub only_follow: bool,
    pub no_indexing: bool,
    pub page_not_found: bool,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HomePageFeaturedArticle {
    pub mdn_url: String,
    pub summary: String,
    pub title: String,
    pub tag: Option<Parent>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HomePageFeaturedContributor {
    pub contributor_name: String,
    pub url: String,
    pub quote: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NameUrl {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HomePageLatestNewsItem {
    pub url: String,
    pub title: String,
    pub author: Option<String>,
    pub source: NameUrl,
    pub published_at: NaiveDate,
}

#[derive(Debug, Clone, Serialize)]
pub struct HomePageRecentContribution {
    pub number: i64,
    pub title: String,
    pub updated_at: DateTime<Utc>,
    pub url: String,
    pub repo: NameUrl,
}

#[derive(Debug, Clone, Serialize)]
pub struct ItemContainer<T>
where
    T: Clone + Serialize,
{
    pub items: Vec<T>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonHomePageSPAHyData {
    pub page_description: Option<&'static str>,
    pub featured_articles: Vec<HomePageFeaturedArticle>,
    pub featured_contributor: Option<HomePageFeaturedContributor>,
    pub latest_news: ItemContainer<HomePageLatestNewsItem>,
    pub recent_contributions: ItemContainer<HomePageRecentContribution>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonHomePageSPA {
    pub hy_data: JsonHomePageSPAHyData,
    pub page_title: &'static str,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonGenericHyData {
    pub sections: Vec<Section>,
    pub title: String,
    pub toc: Vec<TocEntry>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonGenericPage {
    pub hy_data: JsonGenericHyData,
    pub page_title: String,
    pub url: String,
    pub id: String,
}
