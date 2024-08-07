use std::collections::HashMap;
pub use std::ops::Deref;
use std::sync::{Arc, LazyLock, RwLock};

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
use crate::helpers;
use crate::helpers::subpages::{list_sub_pages_grouped_internal, list_sub_pages_internal};
use crate::utils::t_or_vec;

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
        )
}

type SidebarCache = Arc<RwLock<HashMap<Locale, HashMap<String, String>>>>;

static SIDEBAR_CACHE: LazyLock<SidebarCache> =
    LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

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
        add_attribute(html, parent_id, "data-rewriter", "em");
    }
    for details in details {
        add_attribute(html, details, "open", "");
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
            .read()
            .map_err(|_| DocError::SidebarCachePoisoned)?
            .get(&locale)
            .and_then(|map| map.get(s))
        {
            return Ok::<_, DocError>(sb.to_string());
        }
        let sidebar = read_sidebar(s, locale, slug)?;
        let rendered_sidebar = sidebar.render(locale)?;
        SIDEBAR_CACHE
            .write()
            .map_err(|_| DocError::SidebarCachePoisoned)?
            .entry(locale)
            .or_default()
            .entry(s.to_string())
            .or_insert(rendered_sidebar.clone());
        rendered_sidebar
    } else {
        let sidebar = read_sidebar(s, locale, slug)?;
        sidebar.render_with_slug(slug, locale)?
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

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(transparent)]
pub struct SidebarL10n {
    l10n: HashMap<Locale, HashMap<String, String>>,
}

impl SidebarL10n {
    pub fn lookup<'a, 'b: 'a>(&'b self, key: &'a str, locale: Locale) -> &'a str {
        let s = self.l10n.get(&locale).and_then(|l| l.get(key));
        if locale == Default::default() || s.is_some() {
            return s.map(|s| s.as_str()).unwrap_or(key);
        }
        self.l10n
            .get(&Default::default())
            .and_then(|l| l.get(key))
            .map(|s| s.as_str())
            .unwrap_or(key)
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Sidebar {
    pub sidebar: Vec<SidebarEntry>,
    #[serde(default)]
    pub l10n: SidebarL10n,
}

#[derive(Debug, Default)]
pub struct MetaSidebar {
    pub entries: Vec<SidebarMetaEntry>,
    pub l10n: SidebarL10n,
}
impl From<Sidebar> for MetaSidebar {
    fn from(value: Sidebar) -> Self {
        MetaSidebar {
            entries: value.sidebar.into_iter().map(Into::into).collect(),
            l10n: value.l10n,
        }
    }
}

impl MetaSidebar {
    pub fn render(&self, locale: Locale) -> Result<String, DocError> {
        let mut out = String::new();
        out.push_str("<ol>");
        for entry in &self.entries {
            entry.render(&mut out, locale, None, &self.l10n)?;
        }
        out.push_str("</ol>");
        Ok(out)
    }
    pub fn render_with_slug(&self, slug: &str, locale: Locale) -> Result<String, DocError> {
        let mut out = String::new();
        out.push_str("<ol>");
        for entry in &self.entries {
            entry.render(&mut out, locale, Some(slug), &self.l10n)?;
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
    #[serde(default)]
    pub details: Details,
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
    pub details: Details,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase", tag = "type")]
pub struct WebExtApiEntry {
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug)]
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
}

#[derive(Debug, Default)]
pub enum MetaChildren {
    Children(Vec<SidebarMetaEntry>),
    ListSubPages(String, Vec<PageType>),
    ListSubPagesGrouped(String, Vec<PageType>),
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

impl From<SidebarEntry> for SidebarMetaEntry {
    fn from(value: SidebarEntry) -> Self {
        match value {
            SidebarEntry::Section(BasicEntry {
                link,
                title,
                code,
                children,
                details,
            }) => SidebarMetaEntry {
                section: true,
                details,
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
                details,
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
                details,
                code: false,
                content: SidebarMetaEntryContent::Link { link, title },
                children: MetaChildren::ListSubPagesGrouped(path, tags),
            },
            SidebarEntry::Default(BasicEntry {
                link,
                title,
                code,
                children,
                details,
            }) => SidebarMetaEntry {
                section: false,
                details,
                code,
                content: SidebarMetaEntryContent::Link { link, title },
                children: if children.is_empty() {
                    MetaChildren::None
                } else {
                    MetaChildren::Children(children.into_iter().map(Into::into).collect())
                },
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
            SidebarEntry::WebExtApi(WebExtApiEntry { title }) => SidebarMetaEntry {
                section: false,
                code: false,
                details: Details::Closed,
                content: SidebarMetaEntryContent::Link {
                    link: None,
                    title: Some(title),
                },
                children: MetaChildren::WebExtApi,
            },
        }
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
            SidebarMetaEntryContent::Link {
                link: Some(link),
                title,
            } => {
                let title = title.as_ref().map(|t| l10n.lookup(t.as_str(), locale));
                render_link_via_page(out, link, Some(locale), title, self.code, None, true)?;
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
            MetaChildren::ListSubPages(url, page_types) => {
                list_sub_pages_internal(out, url, locale, Some(1), None, page_types)?
            }
            MetaChildren::ListSubPagesGrouped(url, page_types) => {
                list_sub_pages_grouped_internal(out, url, locale, None, page_types)?
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
        let entry: BasicEntry = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(entry.details, Details::Closed);
    }
}
