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

/// Represents an entry in a Table of Contents (ToC), used to navigate a singel page. This is
/// used on the right side of a typical page and allows users to quickly jump to a specific
/// heading in the page.
///
/// The `TocEntry` struct is used to define individual entries in a Table of Contents.
/// Each entry consists of the text to be displayed and a corresponding identifier.
///
/// # Fields
///
/// * `text` - A `String` that holds the display text of the ToC entry. This can
///   contain HTML.
/// * `id` - The `id` attribute of the target element in the page.
/// ```
#[derive(Debug, Clone, Serialize, Default)]
pub struct TocEntry {
    pub text: String,
    pub id: String,
}

/// Represents the git source control information for a documentation page.
///
/// The `Source` struct contains metadata about the source of a documentation page,
/// including the folder path, GitHub URL, last commit URL, and the filename.
///
/// # Fields
///
/// * `folder` - A `PathBuf` that specifies the directory where the source file is located.
/// * `github_url` - A `String` that holds the GitHUb URL to the spource file.
/// * `last_commit_url` - A `String` that holds the URL to the last commit in the GitHub repository.
/// * `filename` - A `String` that specifies the name of the source file.
#[derive(Debug, Clone, Serialize, Default)]
pub struct Source {
    pub folder: PathBuf,
    pub github_url: String,
    pub last_commit_url: String,
    pub filename: String,
}

/// Represents a parent entity in the page structure.
///
/// The `Parent` struct contains metadata about a parent entity, containing its URI and title.
/// This is typically used to represent hierarchical relationships in the page tree,
/// such as a parent page or section. A documentation page has a list of `Parent` items, for example.
///
/// # Fields
///
/// * `uri` - A `String` that holds the URI of the parent entity.
/// * `title` - A `String` that holds the title of the parent entity.
#[derive(Debug, Clone, Serialize, Default)]
pub struct Parent {
    pub uri: String,
    pub title: String,
}

/// Represents a translation entry in the list of other available translations for a documentation page.
///
/// The `Translation` struct contains metadata about a translation, including the locale,
/// title, and native representation. This is used to display translations for other languages
/// in the documentation.
///
/// # Fields
///
/// * `locale` - A `Locale` that specifies the locale of the translation.
/// * `title` - A `String` that holds the translated title.
/// * `native` - A `Native` representing the locale in a locale-native spelling, ie. "Deutsch".
#[derive(Debug, Clone, Serialize, Default)]
pub struct Translation {
    pub locale: Locale,
    pub title: String,
    pub native: Native,
}

/// Represents a prose section on a documentation page, one of the possible items in the list of "body" sections.
///
/// The `Prose` struct is used to define a section of prose content within the documentation.
/// It includes optional metadata such as an identifier and title, as well as the content itself.
/// Additionally, it can specify whether the prose's title is rendered as a H3 HTML heading.
///
/// # Fields
///
/// * `id` - An `Option<String>` that holds an optional `id` element attribute for the prose section.
/// * `title` - An `Option<String>` that holds an optional title for the prose section.
/// * `is_h3` - A `bool` that indicates whether the prose section's `title` will be rendered as a &lt;H3&gt;
///    heading. This field is serialized as `isH3`.
/// * `content` - A `String` that holds the actual prose HTML content.
#[derive(Debug, Clone, Serialize, Default)]
pub struct Prose {
    pub id: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "isH3")]
    pub is_h3: bool,
    pub content: String,
}

/// Represents a browser compatibility (BCD) section on a documentation page.
///
/// The `Compat` struct is used to define a compatibility section (BCD) within the documentation page.
/// It includes optional metadata such as an identifier, title, and content, as well as the important
/// query string to get to the underlying BCD data. Additionally, it can specify whether the title
/// is rendered is a H3 HTML heading.
///
/// # Fields
///
/// * `id` - An `Option<String>` that holds an optional `id` element attribute for the compatibility section.
/// * `title` - An `Option<String>` that holds an optional title for the compatibility section.
/// * `is_h3` - A `bool` that indicates whether the compatibility section's `title` will be rendered as a &lt;H3&gt;
///    heading. This field is serialized as `isH3`.
/// * `query` - A `String` that holds the query string for BCD data.
/// * `content` - An `Option<String>` that holds the optional content of the compatibility section. This field
///    is skipped during serialization if it is `None`.
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
pub struct JsonDocPage {
    pub doc: JsonDoc,
    pub url: String,
}

pub struct BuiltDoc {
    pub json: JsonDocPage,
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
    pub prev_next: Option<PrevNextByUrl>,
    pub template: Template,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct JsonCurriculumPage {
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
pub struct JsonBlogPostPage {
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
pub struct JsonContributorSpotlightPage {
    pub url: String,
    #[serde(rename = "pageTitle")]
    pub page_title: String,
    #[serde(rename = "hyData")]
    pub hy_data: ContributorSpotlightHyData,
}

/// Represents the different JSON artifacts of built pages.
///
/// The `BuiltPage` enum is used to classify various types of built pages that can be
/// generated by the system. Each variant corresponds to a specific type of page,
/// encapsulated in a `Box` to allow for efficient memory management and dynamic dispatch.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum BuiltPage {
    /// Represents a standard documentation page, backed by a Markdown source.
    Doc(Box<JsonDocPage>),
    /// Represents a curriculum page, backed by a Markdown source
    Curriculum(Box<JsonCurriculumPage>),
    /// Represents a blog post, backed by a Markdown source
    BlogPost(Box<JsonBlogPostPage>),
    /// Represents a contributor spotlight page, backed by a Markdown source.
    ContributorSpotlight(Box<JsonContributorSpotlightPage>),
    /// Represents a generic page, i.e Observatory FAQ, About pages, etc.
    GenericPage(Box<JsonGenericPage>),
    /// Represents a basic single-page application. i.e. AI Help, Observatory, etc.
    SPA(Box<JsonSPAPage>),
    /// Represents the home page.
    Home(Box<JsonHomePage>),
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(default)]
pub struct PrevNextBySlug {
    pub previous: Option<SlugNTitle>,
    pub next: Option<SlugNTitle>,
}

impl PrevNextBySlug {
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
pub struct PrevNextByUrl {
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
pub struct JsonSPAPage {
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
pub struct JsonHomePage {
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
