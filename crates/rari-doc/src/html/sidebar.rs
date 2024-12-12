use std::borrow::Cow;
use std::collections::HashMap;
pub use std::ops::Deref;
use std::sync::{Arc, LazyLock};

use constcat::concat;
use dashmap::DashMap;
use indexmap::IndexMap;
use rari_types::fm_types::PageType;
use rari_types::globals::cache_content;
use rari_types::locale::{default_locale, Locale};
use rari_utils::concat_strs;
use scraper::{Html, Node, Selector};
use serde::{Deserialize, Serialize, Serializer};
use tracing::{span, Level};

use super::links::{render_link_from_page, render_link_via_page, LinkModifier};
use super::modifier::insert_attribute;
use super::rewriter::post_process_html;
use crate::cached_readers::read_sidebar;
use crate::error::DocError;
use crate::helpers;
use crate::helpers::subpages::{
    list_sub_pages_grouped_internal, list_sub_pages_internal, ListSubPagesContext,
};
use crate::pages::page::{Page, PageLike};
use crate::pages::types::doc::Doc;
use crate::utils::{is_default, serialize_t_or_vec, t_or_vec};

fn cache_side_bar(sidebar: &str) -> bool {
    cache_content()
        && matches!(
            sidebar,
            "cssref"
                | "glossarysidebar"
                | "learnsidebar"
                | "svgref"
                | "httpsidebar"
                | "jssidebar"
                | "htmlsidebar"
                | "accessibilitysidebar"
                | "firefoxsidebar"
                | "webassemblysidebar"
                | "xsltsidebar"
                | "mdnsidebar"
                | "gamessidebar"
                | "mathmlref"
                | "pwasidebar"
                | "addonsidebarmain"
        )
}

type SidebarCache = Arc<DashMap<Locale, HashMap<String, String>>>;

static SIDEBAR_CACHE: LazyLock<SidebarCache> = LazyLock::new(|| Arc::new(DashMap::new()));

pub fn expand_details_and_mark_current_for_inline_sidebar(
    html: &mut Html,
    url: &str,
) -> Result<(), DocError> {
    let a_selector = Selector::parse(&format!("#Quick_links a[href=\"{url}\"]")).unwrap();
    expand_details_and_mark_current(html, a_selector)
}
fn expand_details_and_mark_current_for_sidebar(html: &mut Html, url: &str) -> Result<(), DocError> {
    let a_selector = Selector::parse(&format!("a[href=\"{url}\"]")).unwrap();
    expand_details_and_mark_current(html, a_selector)
}

fn expand_details_and_mark_current(html: &mut Html, a_selector: Selector) -> Result<(), DocError> {
    let mut details = vec![];
    let mut parent_id = None;
    if let Some(a) = html.select(&a_selector).next() {
        let mut next = a.parent();
        if let Some(parent) = &next {
            parent_id = Some(parent.id());
        }
        while let Some(parent) = next {
            if let Node::Element(el) = parent.value() {
                if el.name() == "details" {
                    details.push(parent.id())
                }
            }
            next = parent.parent();
        }
    }
    if let Some(parent_id) = parent_id {
        insert_attribute(html, parent_id, "data-rewriter", "em");
    }
    for details in details {
        insert_attribute(html, details, "open", "");
    }

    Ok(())
}

pub fn postprocess_sidebar<T: PageLike>(
    ks_rendered_sidebar: &str,
    page: &T,
) -> Result<String, DocError> {
    let mut fragment = Html::parse_fragment(ks_rendered_sidebar);

    expand_details_and_mark_current_for_sidebar(&mut fragment, page.url())?;
    let post_processed_html = post_process_html(&fragment.html(), page, true)?;
    Ok::<_, DocError>(post_processed_html)
}

pub fn render_sidebar(s: &str, slug: &str, locale: Locale) -> Result<String, DocError> {
    let rendered_sidebar = if cache_side_bar(s) {
        if let Some(sb) = SIDEBAR_CACHE
            .get(&locale)
            .and_then(|map| map.get(s).map(ToString::to_string))
        {
            return Ok::<_, DocError>(sb);
        }
        let sidebar = read_sidebar(s, locale, slug)?;
        let rendered_sidebar = sidebar.render(s, locale)?;
        SIDEBAR_CACHE
            .entry(locale)
            .or_default()
            .entry(s.to_string())
            .or_insert(rendered_sidebar.clone());
        rendered_sidebar
    } else {
        let sidebar = read_sidebar(s, locale, slug)?;
        sidebar.render_with_slug(s, slug, locale)?
    };
    Ok::<_, DocError>(rendered_sidebar)
}

pub fn build_sidebar(s: &str, doc: &Doc) -> Result<String, DocError> {
    let rendered_sidebar = render_sidebar(s, doc.slug(), doc.locale())?;
    postprocess_sidebar(&rendered_sidebar, doc)
}

pub fn build_sidebars(doc: &Doc) -> Result<Option<String>, DocError> {
    let out = doc
        .meta
        .sidebar
        .iter()
        .map(|s| build_sidebar(s.as_str(), doc))
        .collect::<Result<String, DocError>>()?;
    Ok(if out.is_empty() { None } else { Some(out) })
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
#[serde(transparent)]
pub struct SidebarL10n {
    // Keep the translations in order of insertion,
    // so Sidebar manipulations are deterministic.
    l10n: IndexMap<Locale, IndexMap<String, String>>,
}

impl SidebarL10n {
    pub fn lookup<'a, 'b: 'a>(&'b self, key: &'a str, locale: Locale) -> &'a str {
        let s = self.l10n.get(&locale).and_then(|l| l.get(key));
        if locale == Default::default() || s.is_some() {
            return s.map(|s| s.as_str()).unwrap_or(key);
        }
        self.l10n
            .get(&Locale::default())
            .and_then(|l| l.get(key))
            .map(|s| s.as_str())
            .unwrap_or(key)
    }

    pub fn is_empty(&self) -> bool {
        self.l10n.is_empty()
    }
}

// Serialize the sidebar entries, filtering out the None variant. This is
// used on the top-level sidebar field and the basic entry children field.
fn serialize_sidebar_entries<S>(sidebar: &[SidebarEntry], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Filter out the None variant
    let filtered: Vec<&SidebarEntry> = sidebar
        .iter()
        .filter(|entry| !matches!(entry, SidebarEntry::None))
        .collect();
    filtered.serialize(serializer)
}

fn sidebar_entries_are_empty(entries: &[SidebarEntry]) -> bool {
    entries
        .iter()
        .all(|entry| matches!(entry, SidebarEntry::None))
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
pub struct Sidebar {
    #[serde(serialize_with = "serialize_sidebar_entries")]
    pub sidebar: Vec<SidebarEntry>,
    #[serde(default, skip_serializing_if = "SidebarL10n::is_empty")]
    pub l10n: SidebarL10n,
}

#[derive(Debug, Default)]
pub struct MetaSidebar {
    pub entries: Vec<SidebarMetaEntry>,
    pub l10n: SidebarL10n,
}
impl TryFrom<Sidebar> for MetaSidebar {
    type Error = DocError;

    fn try_from(value: Sidebar) -> Result<Self, Self::Error> {
        Ok(MetaSidebar {
            entries: value
                .sidebar
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, DocError>>()?,
            l10n: value.l10n,
        })
    }
}

impl MetaSidebar {
    fn render_internal(
        &self,
        name: &str,
        slug: Option<&str>,
        locale: Locale,
    ) -> Result<String, DocError> {
        let span = span!(Level::ERROR, "sidebar", sidebar = name,);
        let _enter = span.enter();
        let mut out = String::new();
        out.push_str("<ol>");
        for entry in &self.entries {
            entry.render(&mut out, locale, slug, &self.l10n)?;
        }
        out.push_str("</ol>");
        Ok(out)
    }

    pub fn render(&self, name: &str, locale: Locale) -> Result<String, DocError> {
        self.render_internal(name, None, locale)
    }

    pub fn render_with_slug(
        &self,
        name: &str,
        slug: &str,
        locale: Locale,
    ) -> Result<String, DocError> {
        self.render_internal(name, Some(slug), locale)
    }
}

// used for skipping serialization if the field has the default value
fn details_is_none(details: &Details) -> bool {
    matches!(details, Details::None)
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BasicEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "details_is_none")]
    pub details: Details,
    #[serde(default, skip_serializing_if = "is_default")]
    pub code: bool,
    #[serde(
        default,
        skip_serializing_if = "sidebar_entries_are_empty",
        serialize_with = "serialize_sidebar_entries"
    )]
    pub children: Vec<SidebarEntry>,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubPageEntry {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(
        default,
        deserialize_with = "t_or_vec",
        serialize_with = "serialize_t_or_vec",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub tags: Vec<PageType>,
    #[serde(default, skip_serializing_if = "details_is_none")]
    pub details: Details,
    #[serde(default, skip_serializing_if = "is_default")]
    pub code: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    pub include_parent: bool,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WebExtApiEntry {
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum SidebarEntry {
    Section(BasicEntry),
    ListSubPages(SubPageEntry),
    ListSubPagesGrouped(SubPageEntry),
    WebExtApi(WebExtApiEntry),
    #[serde(untagged)]
    Default(BasicEntry),
    #[serde(untagged)]
    Link(String),
    #[serde(untagged)]
    None,
}

#[derive(Debug, Default)]
pub enum MetaChildren {
    Children(Vec<SidebarMetaEntry>),
    ListSubPages(String, Vec<PageType>, bool, bool),
    ListSubPagesGrouped {
        path: String,
        tags: Vec<PageType>,
        code: bool,
        include_parent: bool,
    },
    WebExtApi,
    #[default]
    None,
}

#[derive(Debug)]
pub enum SidebarMetaEntryContent {
    Link {
        link: Option<String>,
        title: Option<String>,
    },
    LinkWithHash {
        link: String,
        title: Option<String>,
        hash: String,
    },
    Page(Page),
}

impl SidebarMetaEntryContent {
    pub fn from_link_title_hash(
        link: Option<String>,
        title: Option<String>,
        hash: Option<String>,
    ) -> Self {
        match (link, title, hash) {
            (Some(link), title, Some(hash)) => Self::LinkWithHash { link, title, hash },
            (link, title, _) => Self::Link { link, title },
        }
    }
}

impl Default for SidebarMetaEntryContent {
    fn default() -> Self {
        Self::Link {
            link: None,
            title: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Details {
    #[default]
    None,
    Closed,
    Open,
}

impl Details {
    pub fn is_set(&self) -> bool {
        matches!(self, Self::Closed | Self::Open)
    }

    pub fn is_open(&self) -> bool {
        matches!(self, Self::Open)
    }
}

#[derive(Debug, Default)]
pub struct SidebarMetaEntry {
    pub details: Details,
    pub section: bool,
    pub code: bool,
    pub content: SidebarMetaEntryContent,
    pub children: MetaChildren,
}

impl TryFrom<SidebarEntry> for SidebarMetaEntry {
    type Error = DocError;
    fn try_from(value: SidebarEntry) -> Result<Self, Self::Error> {
        let res = match value {
            SidebarEntry::Section(BasicEntry {
                link,
                hash,
                title,
                code,
                children,
                details,
            }) => SidebarMetaEntry {
                section: true,
                details,
                code,
                content: SidebarMetaEntryContent::from_link_title_hash(link, title, hash),
                children: if children.is_empty() {
                    MetaChildren::None
                } else {
                    MetaChildren::Children(
                        children
                            .into_iter()
                            .map(TryInto::try_into)
                            .collect::<Result<_, DocError>>()?,
                    )
                },
            },
            SidebarEntry::ListSubPages(SubPageEntry {
                details,
                tags,
                link,
                hash,
                title,
                path,
                code,
                include_parent,
            }) => SidebarMetaEntry {
                section: false,
                details,
                code: false,
                content: SidebarMetaEntryContent::from_link_title_hash(link, title, hash),
                children: MetaChildren::ListSubPages(path, tags, code, include_parent),
            },
            SidebarEntry::ListSubPagesGrouped(SubPageEntry {
                details,
                tags,
                link,
                hash,
                title,
                path,
                code,
                include_parent,
            }) => SidebarMetaEntry {
                section: false,
                details,
                code: false,
                content: SidebarMetaEntryContent::from_link_title_hash(link, title, hash),
                children: MetaChildren::ListSubPagesGrouped {
                    path,
                    tags,
                    code,
                    include_parent,
                },
            },
            SidebarEntry::Default(BasicEntry {
                link,
                hash,
                title,
                code,
                children,
                details,
            }) => SidebarMetaEntry {
                section: false,
                details,
                code,
                content: SidebarMetaEntryContent::from_link_title_hash(link, title, hash),
                children: if children.is_empty() {
                    MetaChildren::None
                } else {
                    MetaChildren::Children(
                        children
                            .into_iter()
                            .map(TryInto::try_into)
                            .collect::<Result<_, DocError>>()?,
                    )
                },
            },
            SidebarEntry::Link(link) => SidebarMetaEntry {
                section: false,
                details: Details::None,
                code: false,
                content: SidebarMetaEntryContent::from_link_title_hash(Some(link), None, None),
                children: MetaChildren::None,
            },
            SidebarEntry::WebExtApi(WebExtApiEntry { title }) => SidebarMetaEntry {
                section: false,
                code: false,
                details: Details::Closed,
                content: SidebarMetaEntryContent::from_link_title_hash(None, Some(title), None),
                children: MetaChildren::WebExtApi,
            },
            SidebarEntry::None => return Err(DocError::InvalidSidebarEntry),
        };
        Ok(res)
    }
}

impl SidebarMetaEntry {
    pub fn render(
        &self,
        out: &mut String,
        locale: Locale,
        slug: Option<&str>,
        l10n: &SidebarL10n,
    ) -> Result<(), DocError> {
        #[allow(clippy::single_match)]
        out.push_str("<li");
        if self.section {
            out.push_str(" class=\"section\"");
        }
        if self.details.is_set() || !matches!(self.children, MetaChildren::None) {
            out.push_str(" class=\"toggle\"");
        }
        if self.details.is_set() {
            out.push_str("><details");
            if self.details.is_open() {
                out.push_str(" open ")
            }
            out.push_str("><summary")
        }
        out.push('>');
        match &self.content {
            SidebarMetaEntryContent::LinkWithHash { link, title, hash } => {
                let title = title.as_ref().map(|t| l10n.lookup(t.as_str(), locale));
                let hash = l10n.lookup(hash.as_str(), locale);
                let link = concat_strs!(link.as_str(), "#", hash);
                render_link_via_page(out, &link, locale, title, self.code, None, true)?;
            }
            SidebarMetaEntryContent::Link {
                link: Some(link),
                title,
            } => {
                let title = title.as_ref().map(|t| l10n.lookup(t.as_str(), locale));
                render_link_via_page(out, link, locale, title, self.code, None, true)?;
            }
            SidebarMetaEntryContent::Link { link: None, title } => {
                let title = title.as_ref().map(|t| l10n.lookup(t.as_str(), locale));
                if self.code {
                    out.push_str("<code>");
                }
                out.push_str(title.unwrap_or_default());
                if self.code {
                    out.push_str("</code>");
                }
            }
            SidebarMetaEntryContent::Page(page) => {
                render_link_from_page(
                    out,
                    page,
                    &LinkModifier {
                        badges: page.status(),
                        badge_locale: page.locale(),
                        code: self.code,
                        only_en_us: page.locale() != locale,
                    },
                )?;
            }
        }

        if self.details.is_set() {
            out.push_str("</summary>");
        }

        if !matches!(self.children, MetaChildren::None) {
            out.push_str("<ol>");
        }
        match &self.children {
            MetaChildren::Children(children) => {
                for child in children {
                    child.render(out, locale, slug, l10n)?;
                }
            }
            MetaChildren::ListSubPages(url, page_types, code, include_parent) => {
                let url = if url.starts_with(concat!("/", default_locale().as_url_str(), "/")) {
                    Cow::Borrowed(url)
                } else {
                    Cow::Owned(concat_strs!(
                        "/",
                        Locale::default().as_url_str(),
                        "/docs",
                        url
                    ))
                };
                list_sub_pages_internal(
                    out,
                    &url,
                    locale,
                    Some(1),
                    ListSubPagesContext {
                        sorter: None,
                        page_types,
                        code: *code,
                        include_parent: *include_parent,
                    },
                )?
            }
            MetaChildren::ListSubPagesGrouped {
                path,
                tags,
                code,
                include_parent,
            } => {
                let url = if path.starts_with(concat!("/", default_locale().as_url_str(), "/")) {
                    Cow::Borrowed(path)
                } else {
                    Cow::Owned(concat_strs!(
                        "/",
                        Locale::default().as_url_str(),
                        "/docs",
                        path
                    ))
                };
                list_sub_pages_grouped_internal(
                    out,
                    &url,
                    locale,
                    ListSubPagesContext {
                        sorter: None,
                        page_types: tags,
                        code: *code,
                        include_parent: *include_parent,
                    },
                )?
            }
            MetaChildren::WebExtApi => {
                let children = &helpers::webextapi::children(
                    slug.ok_or(DocError::SlugRequiredForSidebarEntry)?,
                    locale,
                )?;
                for child in children {
                    child.render(out, locale, slug, l10n)?;
                }
            }
            MetaChildren::None => {}
        }
        if !matches!(self.children, MetaChildren::None) {
            out.push_str("</ol>");
        }
        if self.details.is_set() {
            out.push_str("</details>");
        }
        out.push_str("</li>");
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_details_ser() {
        let yaml_str = r#"details: closed"#;
        let entry: BasicEntry = serde_yaml_ng::from_str(yaml_str).unwrap();
        assert_eq!(entry.details, Details::Closed);
    }
}
