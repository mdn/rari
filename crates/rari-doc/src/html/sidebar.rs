use std::collections::HashMap;
pub use std::ops::Deref;
use std::sync::{Arc, RwLock};

use html5ever::{namespace_url, ns, QualName};
use once_cell::sync::Lazy;
use rari_types::fm_types::PageType;
use rari_types::globals::cache_content;
use rari_types::locale::Locale;
use scraper::{Html, Node, Selector};
use serde::{Deserialize, Serialize};

use super::links::render_link;
use super::rewriter::post_process_html;
use crate::cached_readers::read_sidebar;
use crate::docs::doc::Doc;
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

fn highlight_current(mut html: Html, url: &str) -> Result<String, DocError> {
    let a_selector = Selector::parse(&format!("a[href=\"{url}\"]")).unwrap();
    let mut details = vec![];
    if let Some(a) = html.select(&a_selector).next() {
        let mut next = a.parent();
        while let Some(parent) = next {
            if let Node::Element(el) = parent.value() {
                if el.name() == "details" {
                    details.push(parent.id())
                }
            }
            next = parent.parent();
        }
    }
    for details in details {
        if let Some(mut details) = html.tree.get_mut(details) {
            if let Node::Element(ref mut el) = details.value() {
                el.attrs.insert(
                    QualName {
                        prefix: None,
                        ns: ns!(),
                        local: "open".into(),
                    },
                    "".into(),
                );
            }
        }
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
            let sidebar = read_sidebar(s, locale)?;
            let rendered_sidebar = sidebar.render(&locale)?;
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
            let pre_processed_html = highlight_current(fragment, &doc.meta.url)?;
            let post_processed_html = post_process_html(&pre_processed_html, doc, true)?;
            Ok::<_, DocError>(post_processed_html)
        })
        .collect::<Result<String, DocError>>()?;
    Ok(if out.is_empty() { None } else { Some(out) })
}

#[derive(Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct Sidebar {
    pub entries: Vec<SidebarEntry>,
}

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
    pub fn render(&self, locale: &Locale) -> Result<String, DocError> {
        let mut out = String::new();
        out.push_str("<ol>");
        for entry in &self.entries {
            entry.render(&mut out, locale)?;
        }
        out.push_str("</ol>");
        Ok(out)
    }
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", tag = "type")]
pub struct BasicEntry {
    pub link: Option<String>,
    pub title: Option<String>,
    #[serde(default)]
    pub children: Vec<SidebarEntry>,
}

#[derive(Serialize, Deserialize, Default)]
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

#[derive(Serialize, Deserialize)]
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

pub enum MetaChildren {
    Children(Vec<SidebarMetaEntry>),
    ListSubPages(String, Vec<PageType>),
    ListSubPagesGrouped(String, Vec<PageType>),
    None,
}

pub struct SidebarMetaEntry {
    pub details: bool,
    pub section: bool,
    pub link: Option<String>,
    pub title: Option<String>,
    pub children: MetaChildren,
}

impl From<SidebarEntry> for SidebarMetaEntry {
    fn from(value: SidebarEntry) -> Self {
        match value {
            SidebarEntry::Section(BasicEntry {
                link,
                title,
                children,
            }) => SidebarMetaEntry {
                section: true,
                details: false,
                link,
                title,
                children: if children.is_empty() {
                    MetaChildren::None
                } else {
                    MetaChildren::Children(children.into_iter().map(Into::into).collect())
                },
            },
            SidebarEntry::Details(BasicEntry {
                link,
                title,
                children,
            }) => SidebarMetaEntry {
                section: false,
                details: true,
                link,
                title,
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
                details,
                link,
                title,
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
                details,
                link,
                title,
                children: MetaChildren::ListSubPagesGrouped(path, tags),
            },
            SidebarEntry::Default(BasicEntry {
                link,
                title,
                children,
            }) => SidebarMetaEntry {
                section: false,
                details: false,
                link,
                title,
                children: MetaChildren::Children(children.into_iter().map(Into::into).collect()),
            },
            SidebarEntry::Link(link) => SidebarMetaEntry {
                section: false,
                details: false,
                link: Some(link),
                title: None,
                children: MetaChildren::None,
            },
        }
    }
}

impl SidebarMetaEntry {
    pub fn render(&self, out: &mut String, locale: &Locale) -> Result<(), DocError> {
        out.push_str("<li");
        if self.section {
            out.push_str(" class=\"section\"");
        }
        if self.details || !matches!(self.children, MetaChildren::None) {
            out.push_str(" class=\"toggle\"");
        }
        if self.details {
            out.push_str("><details><summary");
        }
        out.push('>');
        if let Some(link) = &self.link {
            render_link(
                out,
                link,
                Some(locale),
                self.title.as_deref(),
                false,
                None,
                true,
            )?;
        } else {
            out.push_str(self.title.as_deref().unwrap_or_default());
        }

        if self.details {
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
        if self.details {
            out.push_str("</details>");
        }
        out.push_str("</li>");
        Ok(())
    }
}
