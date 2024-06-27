use std::collections::HashMap;
pub use std::ops::Deref;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;
use rari_types::fm_types::PageType;
use rari_types::globals::cache_content;
use rari_types::locale::Locale;
use scraper::{Html, Node, Selector};
use serde::{Deserialize, Serialize};

use super::links::{render_link_from_page, render_link_via_page, LinkModifier};
use super::modifier::add_attribute;
use super::rewriter::post_process_html;
use crate::cached_readers::read_sidebar;
use crate::docs::doc::Doc;
use crate::docs::page::{Page, PageLike};
use crate::error::DocError;
use crate::templ::macros::listsubpages::{
    list_sub_pages_grouped_internal, list_sub_pages_internal,
};
use crate::utils::t_or_vec;

fn cache_side_bar(sidebar: &str) -> bool {
    cache_content()
        && match sidebar {
            "cssref" => true,
            "jsref" => false,
            "glossarysidebar" => true,
            _ => false,
        }
}

type SidebarCache = Arc<RwLock<HashMap<Locale, HashMap<String, String>>>>;

static SIDEBAR_CACHE: Lazy<SidebarCache> = Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

fn expand_details_to_for_current(mut html: Html, url: &str) -> Result<String, DocError> {
    let a_selector = Selector::parse(&format!("a[href=\"{url}\"]")).unwrap();
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
        add_attribute(&mut html, parent_id, "data-rewriter", "em");
    }
    for details in details {
        add_attribute(&mut html, details, "open", "");
    }

    Ok(html.html())
}
pub fn render_sidebar(doc: &Doc) -> Result<Option<String>, DocError> {
    let locale = doc.meta.locale;
    let out = doc
        .meta
        .sidebar
        .iter()
        .map(|s| {
            let cache = cache_side_bar(s);
            if cache {
                if let Some(sb) = SIDEBAR_CACHE
                    .read()
                    .map_err(|_| DocError::SidebarCachePoisoned)?
                    .get(&locale)
                    .and_then(|map| map.get(s))
                {
                    return Ok::<_, DocError>(sb.to_owned());
                }
            }
            let sidebar = read_sidebar(s, locale, doc.slug())?;
            let rendered_sidebar = sidebar.render(locale)?;
            if cache {
                SIDEBAR_CACHE
                    .write()
                    .map_err(|_| DocError::SidebarCachePoisoned)?
                    .entry(locale)
                    .or_default()
                    .entry(s.clone())
                    .or_insert(rendered_sidebar.clone());
            }
            Ok::<_, DocError>(rendered_sidebar)
        })
        .map(|ks_rendered_sidebar| {
            let fragment = Html::parse_fragment(&ks_rendered_sidebar?);
            let pre_processed_html = expand_details_to_for_current(fragment, &doc.meta.url)?;
            let post_processed_html = post_process_html(&pre_processed_html, doc, true)?;
            Ok::<_, DocError>(post_processed_html)
        })
        .collect::<Result<String, DocError>>()?;
    Ok(if out.is_empty() { None } else { Some(out) })
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(transparent)]
pub struct Sidebar {
    pub entries: Vec<SidebarEntry>,
}

#[derive(Debug)]
pub struct MetaSidebar {
    pub entries: Vec<SidebarMetaEntry>,
}
impl From<Sidebar> for MetaSidebar {
    fn from(value: Sidebar) -> Self {
        MetaSidebar {
            entries: value.entries.into_iter().map(Into::into).collect(),
        }
    }
}

impl MetaSidebar {
    pub fn render(&self, locale: Locale) -> Result<String, DocError> {
        let mut out = String::new();
        out.push_str("<ol>");
        for entry in &self.entries {
            entry.render(&mut out, locale)?;
        }
        out.push_str("</ol>");
        Ok(out)
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase", tag = "type")]
pub struct BasicEntry {
    pub link: Option<String>,
    pub title: Option<String>,
    #[serde(default)]
    pub code: bool,
    #[serde(default)]
    pub children: Vec<SidebarEntry>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase", tag = "type")]
pub struct SubPageEntry {
    pub path: String,
    pub title: Option<String>,
    pub link: Option<String>,
    #[serde(deserialize_with = "t_or_vec", default)]
    pub tags: Vec<PageType>,
    #[serde(default)]
    pub details: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum SidebarEntry {
    Section(BasicEntry),
    Details(BasicEntry),
    ListSubPages(SubPageEntry),
    ListSubPagesGrouped(SubPageEntry),
    #[serde(untagged)]
    Default(BasicEntry),
    #[serde(untagged)]
    Link(String),
}

#[derive(Debug, Default)]
pub enum MetaChildren {
    Children(Vec<SidebarMetaEntry>),
    ListSubPages(String, Vec<PageType>),
    ListSubPagesGrouped(String, Vec<PageType>),
    #[default]
    None,
}

#[derive(Debug)]
pub enum SidebarMetaEntryContent {
    Link {
        link: Option<String>,
        title: Option<String>,
    },
    Page(Page),
}

impl Default for SidebarMetaEntryContent {
    fn default() -> Self {
        Self::Link {
            link: None,
            title: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
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

impl From<bool> for Details {
    fn from(value: bool) -> Self {
        if value {
            Self::Closed
        } else {
            Self::None
        }
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

impl From<SidebarEntry> for SidebarMetaEntry {
    fn from(value: SidebarEntry) -> Self {
        match value {
            SidebarEntry::Section(BasicEntry {
                link,
                title,
                code,
                children,
            }) => SidebarMetaEntry {
                section: true,
                details: Details::None,
                code,
                content: SidebarMetaEntryContent::Link { link, title },
                children: if children.is_empty() {
                    MetaChildren::None
                } else {
                    MetaChildren::Children(children.into_iter().map(Into::into).collect())
                },
            },
            SidebarEntry::Details(BasicEntry {
                link,
                title,
                code,
                children,
            }) => SidebarMetaEntry {
                section: false,
                details: Details::Closed,
                code,
                content: SidebarMetaEntryContent::Link { link, title },
                children: if children.is_empty() {
                    MetaChildren::None
                } else {
                    MetaChildren::Children(children.into_iter().map(Into::into).collect())
                },
            },
            SidebarEntry::ListSubPages(SubPageEntry {
                details,
                tags,
                link,
                title,
                path,
            }) => SidebarMetaEntry {
                section: false,
                details: details.into(),
                code: false,
                content: SidebarMetaEntryContent::Link { link, title },
                children: MetaChildren::ListSubPages(path, tags),
            },
            SidebarEntry::ListSubPagesGrouped(SubPageEntry {
                details,
                tags,
                link,
                title,
                path,
            }) => SidebarMetaEntry {
                section: false,
                details: details.into(),
                code: false,
                content: SidebarMetaEntryContent::Link { link, title },
                children: MetaChildren::ListSubPagesGrouped(path, tags),
            },
            SidebarEntry::Default(BasicEntry {
                link,
                title,
                code,
                children,
            }) => SidebarMetaEntry {
                section: false,
                details: Details::None,
                code,
                content: SidebarMetaEntryContent::Link { link, title },
                children: MetaChildren::Children(children.into_iter().map(Into::into).collect()),
            },
            SidebarEntry::Link(link) => SidebarMetaEntry {
                section: false,
                details: Details::None,
                code: false,
                content: SidebarMetaEntryContent::Link {
                    link: Some(link),
                    title: None,
                },
                children: MetaChildren::None,
            },
        }
    }
}

impl SidebarMetaEntry {
    pub fn render(&self, out: &mut String, locale: Locale) -> Result<(), DocError> {
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
            SidebarMetaEntryContent::Link {
                link: Some(link),
                title,
            } => {
                render_link_via_page(
                    out,
                    link,
                    Some(locale),
                    title.as_deref(),
                    self.code,
                    None,
                    true,
                )?;
            }
            SidebarMetaEntryContent::Link { link: None, title } => {
                if self.code {
                    out.push_str("<code>");
                }
                out.push_str(title.as_deref().unwrap_or_default());
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
                    child.render(out, locale)?;
                }
            }
            MetaChildren::ListSubPages(url, page_types) => {
                list_sub_pages_internal(out, url, locale, page_types)?
            }
            MetaChildren::ListSubPagesGrouped(url, page_types) => {
                list_sub_pages_grouped_internal(out, url, locale, page_types)?
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
