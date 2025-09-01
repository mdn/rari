//! # The JSON Module
//!
//! This module provides structs used for serializing all content data for MDN.
//! Ultimately, after processing the markdown sources, data is written to individual `index.json`
//! files for each page in the system, using these structs.

use std::path::PathBuf;

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use rari_data::baseline::Baseline;
use rari_types::fm_types::PageType;
use rari_types::locale::{Locale, Native};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::types::contributors::Usernames;
use super::types::curriculum::{CurriculumIndexEntry, CurriculumSidebarEntry, Template, Topic};
use crate::cached_readers::PaginationData;
use crate::html::code::Code;
use crate::issues::DisplayIssues;
use crate::pages::templates::{
    BlogRenderer, ContributorSpotlightRenderer, CurriculumRenderer, DocPageRenderer,
    GenericRenderer, HomeRenderer, SpaRenderer,
};
use crate::pages::types::blog::BlogMeta;
use crate::pages::types::utils::FmTempl;
use crate::specs::Specification;
use crate::utils::modified_dt;

/// Represents an entry in a Table of Contents (ToC), used to navigate a single page. This is
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
#[derive(Debug, Clone, Serialize, Default, JsonSchema)]
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
/// * `github_url` - A `String` that holds the GitHUb URL to the source file.
/// * `last_commit_url` - A `String` that holds the URL to the last commit in the GitHub repository.
/// * `filename` - A `String` that specifies the name of the source file.
#[derive(Debug, Clone, Serialize, Default, JsonSchema)]
pub struct Source {
    pub folder: PathBuf,
    pub github_url: String,
    pub last_commit_url: String,
    pub filename: String,
}

/// Represents a parent entity in a page structure.
///
/// The `Parent` struct contains metadata about a parent entity, containing its URL and title.
/// This is typically used to represent hierarchical relationships in the page tree,
/// such as a parent page or section. A documentation page has a list of `Parent` items, for example.
///
/// # Fields
///
/// * `uri` - A `String` that holds the URL of the parent entity.
/// * `title` - A `String` that holds the title of the parent entity.
#[derive(Debug, Clone, Serialize, Default, JsonSchema)]
pub struct Parent {
    pub uri: String,
    pub title: String,
}

/// Represents a translation entry in the list of other available translations for a page.
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
#[derive(Debug, Clone, Serialize, Default, JsonSchema)]
pub struct Translation {
    pub locale: Locale,
    pub title: String,
    pub native: Native,
}

/// Represents a prose section on a page, one of the possible `Section` items in the list of body sections.
///
/// The `Prose` struct is used to define a section of prose content within a page.
/// It includes optional metadata such as an identifier and title, as well as the content itself.
/// Additionally, it can specify whether the prose's title is rendered as a H3 HTML heading.
///
/// # Fields
///
/// * `id` - An `Option<String>` that holds an optional `id` element attribute for the prose section.
/// * `title` - An `Option<String>` that holds an optional title for the prose section.
/// * `is_h3` - A `bool` that indicates whether the prose section's `title` will be rendered as a &lt;H3&gt;
///   heading. This field is serialized as `isH3`.
/// * `content` - A `String` that holds the actual prose HTML content.
#[derive(Debug, Clone, Serialize, Default, JsonSchema)]
pub struct Prose {
    pub id: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "isH3")]
    pub is_h3: bool,
    pub content: String,
}

/// Represents a browser compatibility (BCD) section on a page, one of the possible `Section` items in the list of body sections.
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
///   heading. This field is serialized as `isH3`.
/// * `query` - A `String` that holds the query string for BCD data.
/// * `content` - An `Option<String>` that holds the optional content of the compatibility section. This field
///   is skipped during serialization if it is `None`.
#[derive(Debug, Clone, Serialize, Default, JsonSchema)]
pub struct Compat {
    pub id: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "isH3")]
    pub is_h3: bool,
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// Represents a specifications section on a page, one of the possible `Section` items in the list of body sections.
///
/// The `SpecificationSection` struct is used to define a section that contains one or more specifications.
/// It includes optional metadata such as an identifier, title, and content, as well as a query string for BCD data,
/// a flag indicating whether the section is an H3 heading, and the list of `Specification` items.
///
/// # Fields
///
/// * `id` - An `Option<String>` that holds an optional `id` element attribute for the specification section.
/// * `title` - An `Option<String>` that holds an optional title for the specification section.
/// * `is_h3` - A `bool` that indicates whether the specification section's `title` will be rendered as a &lt;H3&gt;
/// * `specifications` - A `Vec<Specification>` that holds the list of `Specification` items within the section.
/// * `query` - A `String` that holds the BCD query string associated with the specification section.
/// * `content` - An `Option<String>` that holds the optional content of the specification section. This field is
///   skipped during serialization if it is `None`.
#[derive(Debug, Clone, Serialize, Default, JsonSchema)]
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

/// Represents a section in the documentation.
///
/// The `Section` enum is used to define different types of sections that can be part of the documentation.
/// Each variant corresponds to a specific type of section, encapsulating the relevant data.
///
/// # Variants
///
/// * `Prose` - Represents a prose section, containing general content.
/// * `BrowserCompatibility` - Represents a browser compatibility section, containing compatibility information.
/// * `Specifications` - Represents a specifications section, containing multiple specifications.
///
/// # Fields
///
/// * `Prose(Prose)` - A variant that holds a `Prose` struct, which includes the prose content.
/// * `BrowserCompatibility(Compat)` - A variant that holds a `Compat` struct, which includes compatibility information.
/// * `Specifications(SpecificationSection)` - A variant that holds a `SpecificationSection` struct, which includes
///   one or more specifications.
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Section {
    Prose(Prose),
    BrowserCompatibility(Compat),
    Specifications(SpecificationSection),
}

/// Represents a documentation page in the system, holding the majority of content in MDN.
///
/// The `JsonDoc` struct contains metadata and content for a documentation page.
/// It includes various fields that describe the page's properties, content sections,
/// translations, and other relevant information.
///
/// # Fields
///
/// * `body` - A `Vec<Section>` that holds the content sections of the document. This field is skipped during serialization if it is empty.
///   This is the main content of the page, containing a list of prose, compatibility, and specification sections.
/// * `is_active` - A `bool` that indicates whether the document is active. Serialized as `isActive`.
/// * `is_markdown` - A `bool` that indicates whether the document is in Markdown format. Serialized as `isMarkdown`.
/// * `is_translated` - A `bool` that indicates whether the document has been translated. Serialized as `isTranslated`.
/// * `locale` - A `Locale` that specifies the locale of the document.
/// * `mdn_url` - A `String` that holds the MDN URL of the document.
/// * `modified` - A `NaiveDateTime` that specifies the last modified date and time of the document. Serialized using the `modified_dt` function.
/// * `native` - A `Native` that holds the native representation of the locale, i.e. "Deutsch", "Español" etc.
/// * `no_indexing` - A `bool` that indicates whether the document should be excluded from indexing. Serialized as `noIndexing`.
/// * `other_translations` - A `Vec<Translation>` that holds translations of the document.
/// * `page_title` - A `String` that holds the title of the page. Serialized as `pageTitle`.
/// * `parents` - A `Vec<Parent>` that holds the parent entities of the document. This field is skipped during serialization if it is empty.
/// * `popularity` - An `Option<f64>` that holds the popularity score of the document.
/// * `short_title` - A `String` that holds the short title of the document.
/// * `sidebar_html` - An `Option<String>` that holds the HTML content for the sidebar. Serialized as `sidebarHTML` and skipped
///   during serialization if it is `None`.
/// * `sidebar_macro` - An `Option<String>` that holds the macro content for the sidebar. Serialized as `sidebarMacro` and
///   skipped during serialization if it is `None`.
/// * `source` - A `Source` that holds the git source control information of the document.
/// * `summary` - An `Option<String>` that holds the summary of the document. This field is skipped during serialization if it is `None`.
/// * `title` - A `String` that holds the title of the document.
/// * `toc` - A `Vec<TocEntry>` that holds the table of contents entries for the document.
/// * `baseline` - An `Option<&'static SupportStatusWithByKey>` that holds the baseline support status. This field is skipped during
///   serialization if it is `None`.
/// * `browser_compat` - A `Vec<String>` that holds the browser compatibility information. Serialized as `browserCompat` and skipped
///   during serialization if it is empty.
/// * `page_type` - A `PageType` that specifies the type of the page, for example `LandingPage`, `LearnModule`, `CssAtRule` or
///   `HtmlAttribute`. Serialized as `pageType`.
#[derive(Debug, Clone, Serialize, Default, JsonSchema)]
#[schemars(rename = "Doc")]
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
    pub baseline: Option<Baseline<'static>>,
    #[serde(rename = "browserCompat", skip_serializing_if = "Vec::is_empty")]
    pub browser_compat: Vec<String>,
    #[serde(rename = "pageType")]
    pub page_type: PageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flaws: Option<DisplayIssues>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub live_samples: Option<Vec<Code>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub banners: Vec<FmTempl>,
}

impl JsonDocMetadata {
    pub fn from_json_doc(value: JsonDoc, hash: String) -> Self {
        let JsonDoc {
            is_active,
            is_markdown,
            is_translated,
            locale,
            mdn_url,
            modified,
            native,
            no_indexing,
            other_translations,
            page_title,
            parents,
            popularity,
            short_title,
            source,
            summary,
            title,
            baseline,
            browser_compat,
            page_type,
            ..
        } = value;
        Self {
            is_active,
            is_markdown,
            is_translated,
            locale,
            mdn_url,
            modified,
            native,
            no_indexing,
            other_translations,
            page_title,
            parents,
            popularity,
            short_title,
            source,
            summary,
            title,
            baseline,
            browser_compat,
            page_type,
            hash,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonDocMetadata {
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
    pub source: Source,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline: Option<Baseline<'static>>,
    #[serde(rename = "browserCompat", skip_serializing_if = "Vec::is_empty")]
    pub browser_compat: Vec<String>,
    #[serde(rename = "pageType")]
    pub page_type: PageType,
    pub hash: String,
}

/// Represents the outermost structure of the serialized JSON for a document page. This struct
/// is written to the `index.json` file during a build.
///
/// The `JsonDocPage` struct contains the main document and its associated URL.
///
/// # Fields
///
/// * `doc` - A `JsonDoc` that holds the main content and metadata of the documentation page.
/// * `url` - A `String` that holds the URL of the documentation page.
#[derive(Debug, Clone, Serialize, Default, JsonSchema)]
#[schemars(rename = "DocPage")]
pub struct JsonDocPage {
    pub doc: JsonDoc,
    pub url: String,
    pub renderer: DocPageRenderer,
}

/// Represents an index of blog posts in the documentation system.
///
/// The `BlogIndex` struct contains a list of metadata for blog posts. This is used to
/// manage TODO: what is this used for?
///
/// # Fields
///
/// * `posts` - A `Vec<BlogMeta>` that holds the metadata for each blog post.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct BlogIndex {
    pub posts: Vec<BlogMeta>,
    pub pagination: PaginationData,
}

/// Represents a curriculum document in the system.
///
/// The `JsonCurriculumDoc` struct contains metadata and content for a curriculum page.
/// It includes various fields that describe the page's properties, content sections,
/// translations, and other relevant information.
///
/// # Fields
///
/// * `body` - A `Vec<Section>` that holds the list of content sections of the document. This field is skipped during serialization if it is empty.
/// * `locale` - A `Locale` that specifies the locale of the document.
/// * `mdn_url` - A `String` that holds the MDN URL of the document.
/// * `native` - A `Native` that holds the native representation of the locale.
/// * `no_indexing` - A `bool` that indicates whether the document should be excluded from indexing by search engines. Serialized as `noIndexing`.
/// * `page_title` - A `String` that holds the title of the page. Serialized as `pageTitle`.
/// * `parents` - A `Vec<Parent>` that holds the parent entities of the document. This field is skipped during serialization if it is empty.
/// * `title` - A `String` that holds the title of the document.
/// * `summary` - An `Option<String>` that holds the summary of the document. This field is skipped during serialization if it is `None`.
/// * `toc` - A `Vec<TocEntry>` that holds the table of contents entries for the document.
/// * `sidebar` - An `Option<Vec<CurriculumSidebarEntry>>` that holds the sidebar entries for the curriculum. This field is skipped during
///   serialization if it is `None`.
/// * `topic` - An `Option<Topic>` that holds the topic of the curriculum. This field is skipped during serialization if it is `None`.
/// * `group` - An `Option<String>` that holds the group of the curriculum. This field is skipped during serialization if it is `None`.
/// * `modules` - A `Vec<CurriculumIndexEntry>` that holds the modules of the curriculum. This field is skipped during serialization if it is empty.
/// * `prev_next` - An `Option<PrevNextByUrl>` that holds the previous and next URLs for navigation. Serialized as `prevNext` and skipped during
///   serialization if it is `None`.
/// * `template` - A `Template` that specifies the template used for rendering the document.
#[derive(Debug, Clone, Serialize, Default, JsonSchema)]
#[schemars(rename = "CurriculumDoc")]
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

/// Represents the outermost structure of the serialized JSON for a curriculum page. This struct
/// is written to the `index.json` file during a build.
///
/// The `JsonCurriculumPage` struct contains metadata and content for a curriculum page.
/// It includes the main curriculum document, the URL of the page, the page title, and the locale.
///
/// # Fields
///
/// * `doc` - A `JsonCurriculumDoc` that holds the main content and metadata of the curriculum page.
/// * `url` - A `String` that holds the URL of the curriculum page.
/// * `page_title` - A `String` that holds the title of the curriculum page. Serialized as `pageTitle`.
/// * `locale` - A `Locale` that specifies the locale of the curriculum page.
#[derive(Debug, Clone, Serialize, Default, JsonSchema)]
#[schemars(rename = "CurriculumPage")]
pub struct JsonCurriculumPage {
    pub doc: JsonCurriculumDoc,
    pub url: String,
    #[serde(rename = "pageTitle")]
    pub page_title: String,
    pub locale: Locale,
    pub renderer: CurriculumRenderer,
}

/// Represents a blog post in the system.
///
/// The `JsonBlogPostDoc` struct contains metadata and content for a blog post.
/// It includes various fields that describe the blog post's properties, content sections,
/// translations, and other relevant information.
///
/// # Fields
///
/// * `body` - A `Vec<Section>` that holds the content sections of the blog post. This field is skipped
///   during serialization if it is empty.
/// * `mdn_url` - A `String` that holds the MDN URL of the blog post.
/// * `native` - A `Native` that holds the native representation of the locale.
/// * `locale` - A `Locale` that specifies the locale of the blog post.
/// * `no_indexing` - A `bool` that indicates whether the blog post should be excluded from indexing
///   by search engines. Serialized as `noIndexing`.
/// * `page_title` - A `String` that holds the title of the blog post. Serialized as `pageTitle`.
/// * `parents` - A `Vec<Parent>` that holds the parent entities of the blog post. This field is skipped
///   during serialization if it is empty.
/// * `summary` - An `Option<String>` that holds the summary of the blog post. This field is skipped during
///   serialization if it is `None`.
/// * `title` - A `String` that holds the title of the blog post.
/// * `toc` - A `Vec<TocEntry>` that holds the table of contents entries for the blog post. This field is
///   skipped during serialization if it is empty.
#[derive(Debug, Clone, Serialize, Default, JsonSchema)]
#[schemars(rename = "BlogPostDoc")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub live_samples: Option<Vec<Code>>,
}

/// Represents the outermost structure of the serialized JSON for a blog post. This struct
/// is written to the `index.json` file during a build.
///
/// The `JsonBlogPostPage` struct contains metadata and content for a blog post page.
/// It includes the main blog post document, locale, URL, optional image, page title,
/// optional blog metadata, and optional blog index data.
///
/// # Fields
///
/// * `doc` - A `JsonBlogPostDoc` that holds the main content and metadata of the blog post.
/// * `locale` - A `Locale` that specifies the locale of the blog post page.
/// * `url` - A `String` that holds the URL of the blog post page.
/// * `image` - An `Option<String>` that holds the URL of an image associated with the blog post, if available.
/// * `page_title` - A `String` that holds the title of the blog post page. Serialized as `pageTitle`.
/// * `blog_meta` - An `Option<BlogMeta>` that holds additional metadata about the blog post, if available.
///   Serialized as `blogMeta` and skipped during serialization if it is `None`.
/// * `hy_data` - An `Option<BlogIndex>` that holds data related to the blog index, if available.
///   Serialized as `hyData` and skipped during serialization if it is `None`.
/// * `common` - Common data, e.g. description.
#[derive(Debug, Clone, Serialize, Default, JsonSchema)]
#[schemars(rename = "BlogPostPage")]
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
    #[serde(flatten)]
    pub common: CommonJsonData,
    pub renderer: BlogRenderer,
}

/// Represents a contributor spotlight page in the documentation system.
///
/// The `ContributorSpotlightHyData` struct contains metadata and content for a contributor spotlight,
/// including sections, contributor name, folder name, featured status, profile image, profile image alt text,
/// usernames, and a quote. This is used to display detailed information about a featured contributor.
///
/// # Fields
///
/// * `sections` - A `Vec<Section>` that holds the content sections related to the contributor.
/// * `contributor_name` - A `String` that holds the name of the contributor. Serialized as `contributorName`.
/// * `folder_name` - A `String` that holds the name of the folder containing the contributor's data. Serialized as `folderName`.
/// * `is_featured` - A `bool` that indicates whether the contributor is featured. Serialized as `isFeatured`.
/// * `profile_img` - A `String` that holds the URL of the contributor's profile image. Serialized as `profileImg`.
/// * `profile_img_alt` - A `String` that holds the alt text for the contributor's profile image. Serialized as `profileImgAlt`.
/// * `usernames` - A `Usernames` struct that holds the usernames associated with the contributor.
/// * `quote` - A `String` that holds a quote from the contributor.
#[derive(Debug, Clone, Serialize, JsonSchema)]
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

/// Represents the outermost structure of the serialized JSON for a contributor spotlight page. This struct
/// is written to the `index.json` file during a build.
///
/// The `JsonContributorSpotlightPage` struct contains metadata and content for a contributor spotlight page.
/// It includes the URL of the page, the page title, and the data related to the contributor.
///
/// # Fields
///
/// * `url` - A `String` that holds the URL of the contributor spotlight page.
/// * `page_title` - A `String` that holds the title of the contributor spotlight page. Serialized as `pageTitle`.
/// * `hy_data` - A `ContributorSpotlightHyData` that holds the data related to the contributor. Serialized as `hyData`.
/// * `common` - Common data, e.g. description.
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[schemars(rename = "ContributorSpotlightPage")]
pub struct JsonContributorSpotlightPage {
    pub url: String,
    pub short_title: String,
    #[serde(rename = "pageTitle")]
    pub page_title: String,
    #[serde(rename = "hyData")]
    pub hy_data: ContributorSpotlightHyData,
    #[serde(flatten)]
    pub common: CommonJsonData,
    pub renderer: ContributorSpotlightRenderer,
}

/// Represents the different JSON artifacts of built pages.
///
/// The `BuiltPage` enum is used to classify various types of built pages that can be
/// generated by the system. Each variant corresponds to a specific type of page,
/// encapsulated in a `Box` to allow for efficient memory management and dynamic dispatch.
#[derive(Debug, Clone, Serialize, JsonSchema)]
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
    SPA(Box<JsonSpaPage>),
    /// Represents the home page.
    Home(Box<JsonHomePage>),
}

/// Represents the previous and next navigation links by slug.
///
/// The `PrevNextBySlug` struct contains the slug and title for "previous" and "next" links, respectively.
/// This is used to facilitate horizontal navigation between related pages, primarily in the Blog section
/// of the site.
///
/// # Fields
///
/// * `previous` - An `Option<SlugNTitle>` that holds the the slug and title for the previous page, if available.
/// * `next` - An `Option<SlugNTitle>` that holds the the slug and title for the next page, if available.
#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
#[serde(default)]
pub struct PrevNextBySlug {
    pub previous: Option<SlugNTitle>,
    pub next: Option<SlugNTitle>,
}

impl PrevNextBySlug {
    /// Helper function used to suppress serializing if both `previous` and `next` are `None`.
    pub fn is_none(&self) -> bool {
        self.previous.is_none() && self.next.is_none()
    }
}

/// Represents a navigation link with a title and slug.
///
/// The `SlugNTitle` struct is used to define a single "previous" or "next" navigation link, used by `PrevNextBySlug`
///
/// # Fields
///
/// * `title` - A `String` that holds the title of the navigation link.
/// * `slug` - A `String` that holds the slug of the navigation link.
#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
pub struct SlugNTitle {
    pub title: String,
    pub slug: String,
}

/// Represents the previous and next navigation links by URL.
///
/// The `PrevNextBySlug` struct contains the URL and title for "previous" and "next" links, respectively.
/// This is used to facilitate horizontal navigation between related pages, primarily in the Curriculum
/// section of the site.
///
/// # Fields
///
/// * `previous` - An `Option<UrlNTitle>` that holds the the URL and title for the previous page, if available.
/// * `next` - An `Option<UrlNTitle>` that holds the the URL and title for the next page, if available.
#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
#[serde(default)]
pub struct PrevNextByUrl {
    pub prev: Option<UrlNTitle>,
    pub next: Option<UrlNTitle>,
}

/// Represents a navigation link with a title and URL.
///
/// The `SlugNTitle` struct is used to define a single "previous" or "next" navigation link, used by `PrevNextBySlug`
///
/// # Fields
///
/// * `title` - A `String` that holds the title of the navigation link.
/// * `url` - A `String` that holds the URL of the navigation link.
#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
pub struct UrlNTitle {
    pub title: String,
    pub url: String,
}

/// Represents a Single Page Application (SPA) page in the documentation system, i.e. AI Help, Observatory, etc.
///
/// The `JsonSPAPage` struct contains metadata and content for an SPA page.
/// It includes various fields that describe the page's properties, such as the slug,
/// title, description, and indexing options.
///
/// # Fields
///
/// * `slug` - A `&'static str` that holds the unique identifier for the SPA page.
/// * `page_title` - A `&'static str` that holds the title of the SPA page.
/// * `page_description` - An `Option<&'static str>` that holds the description of the SPA page, if available.
/// * `only_follow` - A `bool` that indicates whether the page should only be followed by search engines.
/// * `no_indexing` - A `bool` that indicates whether the page should be excluded from indexing by search engines.
/// * `page_not_found` - A `bool` that indicates whether the page represents a "Page Not Found" (404) error.
/// * `url` - A `String` that holds the URL of the page.
/// * `common` - Common data, e.g. description.
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "SPAPage")]
pub struct JsonSpaPage {
    pub slug: &'static str,
    pub page_title: &'static str,
    pub page_description: Option<&'static str>,
    pub only_follow: bool,
    pub no_indexing: bool,
    pub page_not_found: bool,
    pub url: String,
    #[serde(flatten)]
    pub common: CommonJsonData,
    pub renderer: SpaRenderer,
}

/// Represents a featured article (usually a blog post or documentation page) on the home page.
///
/// The `HomePageFeaturedArticle` struct contains metadata about the featured article,
/// including its URL, summary, title, and an optional parent tag. This is used to display
/// featured articles prominently on the home page of MDN.
///
/// # Fields
///
/// * `mdn_url` - A `String` that holds the MDN URL of the featured article.
/// * `summary` - A `String` that holds a brief summary of the featured article.
/// * `title` - A `String` that holds the title of the featured article.
/// * `tag` - An `Option<Parent>` that holds an optional parent for the featured article, which is
///   used for categorization.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct HomePageFeaturedArticle {
    pub mdn_url: String,
    pub summary: String,
    pub title: String,
    pub tag: Option<Parent>,
}

/// The `HomePageFeaturedContributor` struct contains metadata about a featured contributor item
/// on the home page, including their name, a URL to their profile or related content, and the
/// displayed quote.
///
/// # Fields
///
/// * `contributor_name` - A `String` that holds the name of the featured contributor.
/// * `url` - A `String` that holds the URL to the contributor's profile or related content.
/// * `quote` - A `String` that holds a quote from the featured contributor.
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HomePageFeaturedContributor {
    pub contributor_name: String,
    pub url: String,
    pub quote: String,
}

/// The `NameUrl` struct is used to store a pair of a name and a corresponding URL.
/// This is used in the "Latest news" and "Recent contributions" sections of the home page.
///
/// # Fields
///
/// * `name` - A `String` that holds the name or title of the entity.
/// * `url` - A `String` that holds the URL associated with the entity.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct NameUrl {
    pub name: String,
    pub url: String,
}

/// Represents an item in the latest news section on the home page.
///
/// The `HomePageLatestNewsItem` struct contains metadata about a news item,
/// including its URL, title, author, source, and publication date. This is used
/// to display the latest news items prominently on the home page of the documentation system.
///
/// # Fields
///
/// * `url` - A `String` that holds the URL to the news item.
/// * `title` - A `String` that holds the title of the news item.
/// * `author` - An `Option<String>` that holds the name of the author of the news item, if available.
/// * `source` - A `NameUrl` that holds the source of the news item, including the name and URL of the source.
/// * `published_at` - A `NaiveDate` that specifies the publication date of the news item.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct HomePageLatestNewsItem {
    pub url: String,
    pub title: String,
    pub author: Option<String>,
    pub source: NameUrl,
    pub published_at: NaiveDate,
    pub summary: String,
}

/// Represents a recent contribution item on the home page.
///
/// The `HomePageRecentContribution` struct contains metadata about a recent contribution,
/// including its number, title, update time, URL, and repository information. This is used
/// to display recent contributions prominently on the home page of the documentation system.
///
/// # Fields
///
/// * `number` - An `i64` that holds the number of the contribution (e.g., issue or pull request number).
/// * `title` - A `String` that holds the title of the contribution.
/// * `updated_at` - A `DateTime<Utc>` that specifies the time of the contribution.
/// * `url` - A `String` that holds the URL to the contribution.
/// * `repo` - A `NameUrl` that holds the repository information, including the name and URL of the repository.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct HomePageRecentContribution {
    pub number: i64,
    pub title: String,
    pub updated_at: DateTime<Utc>,
    pub url: String,
    pub repo: NameUrl,
}

/// A container for holding a collection of items, used to hold latest news items and recent contributions
/// on the home page.
///
/// The `ItemContainer` struct is a generic container that holds a vector of items of type `T`.
/// The type `T` must implement the `Clone` and `Serialize` traits.
///
/// # Type Parameters
///
/// * `T` - The type of items contained in the `ItemContainer`. Must implement `Clone` and `Serialize`.
///
/// # Fields
///
/// * `items` - A `Vec<T>` that holds the collection of items.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ItemContainer<T>
where
    T: Clone + Serialize,
{
    pub items: Vec<T>,
}

/// Represents all data that is displayed on the home page.
///
/// The `JsonHomePageSPAHyData` struct contains metadata and content for the home page,
/// including the page description, featured articles, featured contributor, latest news, and recent contributions.
///
/// # Fields
///
/// * `page_description` - An `Option<&'static str>` that holds the description of the home page, if available.
/// * `featured_articles` - A `Vec<HomePageFeaturedArticle>` that holds a list of featured articles to be displayed on the home page.
/// * `featured_contributor` - An `Option<HomePageFeaturedContributor>` that holds information about a featured contributor, if available.
/// * `latest_news` - An `ItemContainer<HomePageLatestNewsItem>` that holds the latest news items to be displayed on the home page.
/// * `recent_contributions` - An `ItemContainer<HomePageRecentContribution>` that holds the recent contributions to be displayed on the home page.
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "HomePageSPAHyData")]
pub struct JsonHomePageSPAHyData {
    pub page_description: Option<&'static str>,
    pub featured_articles: Vec<HomePageFeaturedArticle>,
    pub featured_contributor: Option<HomePageFeaturedContributor>,
    pub latest_news: ItemContainer<HomePageLatestNewsItem>,
    pub recent_contributions: ItemContainer<HomePageRecentContribution>,
}

/// Represents the outermost home page structure in the documentation system. This is written to the `index.json` file during a build.
///
/// The `JsonHomePage` struct contains metadata and content for the home page,
/// including the content data, page title, and URL. This is used to manage and display
/// the home page within the documentation system.
///
/// # Fields
///
/// * `hy_data` - A `JsonHomePageSPAHyData` that holds the content data related to the home page,
///   including featured articles, contributors, latest news, and recent contributions.
/// * `page_title` - A `&'static str` that holds the title of the home page.
/// * `url` - A `String` that holds the URL of the page.
/// * `common` - Common data, e.g. description.
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "HomePage")]
pub struct JsonHomePage {
    pub hy_data: JsonHomePageSPAHyData,
    pub page_title: &'static str,
    pub url: String,
    #[serde(flatten)]
    pub common: CommonJsonData,
    pub renderer: HomeRenderer,
}

/// Represents the data for a generic page in the system. Generic pages are used for various purposes,
/// such as the Observatory FAQ, About pages, etc.
///
/// The `JsonGenericHyData` struct contains metadata and content for a generic page,
/// including sections, title, and table of contents (ToC) entries. This is used to manage
/// and display generic pages within the documentation system.
///
/// # Fields
///
/// * `sections` - A `Vec<Section>` that holds the content sections of the generic page.
/// * `title` - A `String` that holds the title of the generic page.
/// * `toc` - A `Vec<TocEntry>` that holds the table of contents entries for the generic page.
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "GenericHyData")]
pub struct JsonGenericHyData {
    pub sections: Vec<Section>,
    pub title: String,
    pub toc: Vec<TocEntry>,
}

/// Represents the outermost generic page structure in the documentation system. This is written to
/// the `index.json` file during a build.
///
/// The `JsonGenericPage` struct contains metadata and content for a generic page,
/// including the content data, page title, URL, and an identifier. This is used to manage
/// and display generic pages within the documentation system.
///
/// # Fields
///
/// * `hy_data` - A `JsonGenericHyData` that holds the content data related to the generic page,
///   including sections, title, and table of contents (ToC) entries.
/// * `page_title` - A `String` that holds the title of the generic page.
/// * `url` - A `String` that holds the URL of the generic page.
/// * `id` - A `String` that holds the unique identifier for the generic page.
/// * `common` - Common data, e.g. description.
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "GenericPage")]
pub struct JsonGenericPage {
    pub hy_data: JsonGenericHyData,
    pub short_title: Option<String>,
    pub page_title: String,
    pub url: String,
    pub id: String,
    #[serde(flatten)]
    pub common: CommonJsonData,
    pub renderer: GenericRenderer,
}

#[derive(Debug, Clone, Default, Serialize, JsonSchema)]
#[schemars(rename = "GenericPage")]
pub struct CommonJsonData {
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parents: Vec<Parent>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub other_translations: Vec<Translation>,
}
