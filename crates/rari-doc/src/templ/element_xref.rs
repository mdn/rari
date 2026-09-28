//! Shared page lookup and linking for HTML, SVG, and MathML elements.

use std::collections::HashMap;
use std::sync::LazyLock;

use rari_types::fm_types::PageType;
use rari_types::locale::Locale;

use crate::error::DocError;
use crate::helpers::subpages::{SubPagesSorter, get_sub_pages};
use crate::pages::page::PageLike;
use crate::templ::api::RariApi;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ElementFamily {
    Html,
    Svg,
    Mathml,
}

impl ElementFamily {
    fn root(self) -> &'static str {
        match self {
            Self::Html => "/en-US/docs/Web/HTML/Reference/Elements",
            Self::Svg => "/en-US/docs/Web/SVG/Reference/Element",
            Self::Mathml => "/en-US/docs/Web/MathML/Reference/Element",
        }
    }

    fn page_type(self) -> PageType {
        match self {
            Self::Html => PageType::HtmlElement,
            Self::Svg => PageType::SvgElement,
            Self::Mathml => PageType::MathmlElement,
        }
    }

    fn slug_prefix(self) -> &'static str {
        match self {
            Self::Html => "Web/HTML/Reference/Elements/",
            Self::Svg => "Web/SVG/Reference/Element/",
            Self::Mathml => "Web/MathML/Reference/Element/",
        }
    }
}

#[derive(Default)]
struct ElementIndex {
    paths: HashMap<(ElementFamily, String), String>,
    leaves: HashMap<(ElementFamily, String), Option<String>>,
}

static ELEMENT_INDEX: LazyLock<ElementIndex> = LazyLock::new(build_index);

fn build_index() -> ElementIndex {
    let mut index = ElementIndex::default();
    for family in [
        ElementFamily::Html,
        ElementFamily::Svg,
        ElementFamily::Mathml,
    ] {
        let pages = get_sub_pages(family.root(), None, SubPagesSorter::Slug)
            .unwrap_or_else(|error| panic!("failed to build {family:?} element index: {error}"));
        for page in pages {
            if page.page_type() != family.page_type() {
                continue;
            }
            let Some(sub_slug) = page.slug().strip_prefix(family.slug_prefix()) else {
                continue;
            };
            if sub_slug.is_empty() {
                continue;
            }

            let canonical_slug = page.slug().to_string();
            index
                .paths
                .insert((family, sub_slug.to_string()), canonical_slug.clone());
            let leaf = sub_slug.rsplit('/').next().unwrap_or(sub_slug);
            insert_leaf(&mut index, family, leaf, &canonical_slug);
        }
    }
    index
}

fn insert_leaf(index: &mut ElementIndex, family: ElementFamily, name: &str, slug: &str) {
    let entry = index
        .leaves
        .entry((family, name.to_string()))
        .or_insert_with(|| Some(slug.to_string()));
    if entry.as_deref().is_some_and(|existing| existing != slug) {
        *entry = None;
    }
}

fn resolve_element_slug(family: ElementFamily, name: &str) -> Option<&'static str> {
    ELEMENT_INDEX
        .paths
        .get(&(family, name.to_string()))
        .or_else(|| {
            ELEMENT_INDEX
                .leaves
                .get(&(family, name.to_string()))
                .and_then(Option::as_ref)
        })
        .map(String::as_str)
}

pub(crate) fn link_element(
    family: ElementFamily,
    element_name: &str,
    display: &str,
    code: bool,
    title: Option<&str>,
    anchor: Option<&str>,
    locale: Locale,
) -> Result<String, DocError> {
    let fallback_slug = format!("{}{element_name}", family.slug_prefix());
    let slug = resolve_element_slug(family, element_name).unwrap_or(&fallback_slug);
    let mut url = format!("/{}/docs/{slug}", locale.as_url_str());
    if let Some(anchor) = anchor {
        if !anchor.starts_with('#') {
            url.push('#');
        }
        url.push_str(anchor);
    }

    RariApi::link(&url, Some(locale), Some(display), code, title, false)
}
