//! Lazy index of en-US HTML, SVG, and MathML element page slugs.
//!
//! Used by the `htmlelement`, `svgelement`, and `mathmlelement` templates to
//! resolve an element name to its canonical slug instead of hard-coding the
//! `Web/<Family>/Reference/Element(s)/<name>` path, so links follow content
//! reorganizations. Slugs are locale-invariant, so walking the en-US tree is
//! enough and the result is reused for every locale.
//!
//! Each element page is indexed under its path relative to the family root
//! and, for pages nested below the root, also under its leaf segment, so a
//! future grouping such as `Elements/forms/select` stays reachable as
//! `{{HTMLElement("select")}}`. A leaf shared by several pages is marked
//! ambiguous and only resolves via its full path.

use std::collections::HashMap;
use std::sync::LazyLock;

use rari_types::fm_types::PageType;
use rari_types::locale::Locale;

use crate::error::DocError;
use crate::helpers::subpages::{SubPagesSorter, get_sub_pages};
use crate::pages::page::PageLike;
use crate::templ::api::RariApi;

#[derive(Clone, Copy, Debug)]
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
struct FamilyIndex {
    /// Sub path below the family root to canonical slug.
    paths: HashMap<String, String>,
    /// Leaf segment to canonical slug, or `None` when the leaf is ambiguous.
    leaves: HashMap<String, Option<String>>,
}

struct ElementIndex {
    html: FamilyIndex,
    svg: FamilyIndex,
    mathml: FamilyIndex,
}

impl ElementIndex {
    fn family(&self, family: ElementFamily) -> &FamilyIndex {
        match family {
            ElementFamily::Html => &self.html,
            ElementFamily::Svg => &self.svg,
            ElementFamily::Mathml => &self.mathml,
        }
    }
}

static ELEMENT_INDEX: LazyLock<ElementIndex> = LazyLock::new(build_index);

fn build_index() -> ElementIndex {
    ElementIndex {
        html: build_family_index(ElementFamily::Html),
        svg: build_family_index(ElementFamily::Svg),
        mathml: build_family_index(ElementFamily::Mathml),
    }
}

fn build_family_index(family: ElementFamily) -> FamilyIndex {
    let pages = get_sub_pages(family.root(), None, SubPagesSorter::Slug).expect(match family {
        ElementFamily::Html => {
            "failed to build HTML element index from /en-US/docs/Web/HTML/Reference/Elements"
        }
        ElementFamily::Svg => {
            "failed to build SVG element index from /en-US/docs/Web/SVG/Reference/Element"
        }
        ElementFamily::Mathml => {
            "failed to build MathML element index from /en-US/docs/Web/MathML/Reference/Element"
        }
    });
    let mut index = FamilyIndex::default();
    for page in pages {
        if page.page_type() != family.page_type() {
            continue;
        }
        let Some(sub_slug) = page.slug().strip_prefix(family.slug_prefix()) else {
            continue;
        };
        index_one(&mut index, sub_slug, page.slug());
    }
    index
}

fn index_one(index: &mut FamilyIndex, sub_slug: &str, canonical_slug: &str) {
    if sub_slug.is_empty() {
        return;
    }
    index
        .paths
        .insert(sub_slug.to_string(), canonical_slug.to_string());
    let leaf = sub_slug.rsplit('/').next().unwrap_or(sub_slug);
    insert_leaf(index, leaf, canonical_slug);
}

fn insert_leaf(index: &mut FamilyIndex, name: &str, slug: &str) {
    let entry = index
        .leaves
        .entry(name.to_string())
        .or_insert_with(|| Some(slug.to_string()));
    if entry.as_deref().is_some_and(|existing| existing != slug) {
        *entry = None;
    }
}

fn resolve_element_slug(family: ElementFamily, name: &str) -> Option<&'static str> {
    resolve_from_index(ELEMENT_INDEX.family(family), name)
}

fn resolve_from_index<'a>(index: &'a FamilyIndex, name: &str) -> Option<&'a str> {
    index
        .paths
        .get(name)
        .or_else(|| index.leaves.get(name).and_then(Option::as_ref))
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

#[cfg(test)]
mod tests {
    use super::*;

    const PREFIX: &str = "Web/HTML/Reference/Elements/";

    fn fixture() -> FamilyIndex {
        let mut index = FamilyIndex::default();
        for sub_slug in [
            "a",
            "Heading_Elements",
            "input",
            "forms/input",
            "forms/select",
            "media/track",
            "table/track",
        ] {
            index_one(&mut index, sub_slug, &format!("{PREFIX}{sub_slug}"));
        }
        index
    }

    #[test]
    fn resolve_cases() {
        struct Case {
            name: &'static str,
            input: &'static str,
            expected: Option<&'static str>,
        }
        let cases = vec![
            Case {
                name: "flat page resolves by sub slug",
                input: "a",
                expected: Some("Web/HTML/Reference/Elements/a"),
            },
            Case {
                name: "nested page resolves by full sub path",
                input: "forms/select",
                expected: Some("Web/HTML/Reference/Elements/forms/select"),
            },
            Case {
                name: "nested page resolves by unique leaf",
                input: "select",
                expected: Some("Web/HTML/Reference/Elements/forms/select"),
            },
            Case {
                name: "ambiguous leaf does not resolve",
                input: "track",
                expected: None,
            },
            Case {
                name: "exact path beats ambiguous leaf",
                input: "input",
                expected: Some("Web/HTML/Reference/Elements/input"),
            },
            Case {
                name: "lookup is case sensitive",
                input: "heading_elements",
                expected: None,
            },
            Case {
                name: "unknown name does not resolve",
                input: "blink",
                expected: None,
            },
        ];

        let index = fixture();
        for case in cases {
            assert_eq!(
                resolve_from_index(&index, case.input),
                case.expected,
                "{}",
                case.name
            );
        }
    }

    #[test]
    fn empty_sub_slug_is_skipped() {
        let mut index = FamilyIndex::default();
        index_one(&mut index, "", "Web/HTML/Reference/Elements");
        assert!(index.paths.is_empty());
        assert!(index.leaves.is_empty());
    }
}
