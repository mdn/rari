//! Name resolution for the unified `xref` template.

use std::collections::HashMap;
use std::sync::LazyLock;

use rari_types::fm_types::PageType;

use crate::error::{DocError, XrefError};
use crate::helpers::subpages::{SubPagesSorter, get_sub_pages};
use crate::pages::page::{Page, PageLike};

static XREF_INDEX: LazyLock<XrefIndex> = LazyLock::new(|| {
    build_index().unwrap_or_else(|error| {
        tracing::error!("failed to build xref index: {error}");
        XrefIndex::default()
    })
});

#[derive(Debug, Default)]
struct XrefIndex {
    slugs: Vec<String>,
    exact: HashMap<String, Vec<usize>>,
    folded: HashMap<String, Vec<usize>>,
}

/// Resolve an xref name to the canonical slug of a page.
///
/// The last segment of `name` is matched against slug leaves, and any
/// preceding segments narrow down ambiguous matches if they appear in the
/// slug in the same order.
pub(crate) fn resolve_xref(name: &str) -> Result<&'static str, XrefError> {
    let index = resolve_xref_index(&XREF_INDEX, name)?;
    Ok(XREF_INDEX.slugs[index].as_str())
}

/// List all pages resolvable by `xref` as `(name, slug)` pairs, sorted by name,
/// where `name` is the shortest argument that resolves to the page.
pub fn xref_names() -> Vec<(String, &'static str)> {
    let index = LazyLock::force(&XREF_INDEX);
    let mut names = (0..index.slugs.len())
        .map(|i| (shortest_name(index, i), index.slugs[i].as_str()))
        .collect::<Vec<_>>();
    names.sort();
    names
}

fn shortest_name(index: &XrefIndex, entry: usize) -> String {
    let slug = &index.slugs[entry];
    let leaf = leaf(slug);
    // Parent segments, nearest first, so ties prefer the closest qualifier.
    let parents = slug.rsplit('/').skip(1).collect::<Vec<_>>();
    let mut masks = (0..1u32 << parents.len()).collect::<Vec<_>>();
    masks.sort_by_key(|mask| (mask.count_ones(), *mask));
    masks
        .into_iter()
        .map(|mask| {
            let mut segments = (0..parents.len())
                .rev()
                .filter(|bit| mask & (1 << bit) != 0)
                .map(|bit| parents[bit])
                .collect::<Vec<_>>();
            segments.push(leaf);
            segments.join("/")
        })
        .find(|name| resolve_xref_index(index, name).is_ok_and(|found| found == entry))
        .unwrap_or_else(|| slug.clone())
}

fn resolve_xref_index(index: &XrefIndex, name: &str) -> Result<usize, XrefError> {
    let (qualifier, leaf) = match name.rsplit_once('/') {
        Some((qualifier, leaf)) => (Some(qualifier), leaf),
        None => (None, name),
    };
    let candidates = index
        .exact
        .get(leaf)
        .or_else(|| index.folded.get(&leaf.to_lowercase()))
        .ok_or_else(|| XrefError::NotFound(name.to_string()))?;
    let matches = candidates.iter().copied().filter(|&candidate| {
        qualifier.is_none_or(|qualifier| matches_qualifier(&index.slugs[candidate], qualifier))
    });
    match matches.collect::<Vec<_>>()[..] {
        [] => Err(XrefError::NotFound(name.to_string())),
        [only] => Ok(only),
        ref matches => Err(XrefError::Ambiguous {
            name: name.to_string(),
            candidates: matches.iter().map(|&i| index.slugs[i].clone()).collect(),
        }),
    }
}

fn leaf(slug: &str) -> &str {
    slug.rsplit('/').next().unwrap_or(slug)
}

fn matches_qualifier(slug: &str, qualifier: &str) -> bool {
    let mut parents = slug.rsplit('/').skip(1);
    qualifier.rsplit('/').all(|expected| {
        parents
            .by_ref()
            .any(|segment| segment.eq_ignore_ascii_case(expected))
    })
}

// All indexed page types live under this root. Supporting page types elsewhere
// requires reading their roots too, or all pages filtered by page type.
const XREF_ROOT: &str = "/en-US/docs/Web/Accessibility/ARIA/Reference";

fn build_index() -> Result<XrefIndex, DocError> {
    let pages = get_sub_pages(XREF_ROOT, None, SubPagesSorter::Slug)?;
    Ok(build_index_from_pages(&pages))
}

fn build_index_from_pages<'a>(pages: impl IntoIterator<Item = &'a Page>) -> XrefIndex {
    build_index_from_slugs(
        pages
            .into_iter()
            .filter(|page| is_xref_page_type(page.page_type()))
            .map(|page| page.slug()),
    )
}

fn build_index_from_slugs<'a>(slugs: impl IntoIterator<Item = &'a str>) -> XrefIndex {
    let slugs = slugs.into_iter().map(String::from).collect::<Vec<_>>();

    let mut exact: HashMap<String, Vec<usize>> = HashMap::new();
    let mut folded: HashMap<String, Vec<usize>> = HashMap::new();
    for (index, slug) in slugs.iter().enumerate() {
        let leaf = leaf(slug);
        exact.entry(leaf.to_string()).or_default().push(index);
        folded.entry(leaf.to_lowercase()).or_default().push(index);
    }

    XrefIndex {
        slugs,
        exact,
        folded,
    }
}

fn is_xref_page_type(page_type: PageType) -> bool {
    matches!(page_type, PageType::AriaAttribute | PageType::AriaRole)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> XrefIndex {
        build_index_from_slugs([
            "Web/Accessibility/ARIA/Reference/Roles/alert_role",
            "Web/Accessibility/ARIA/Reference/Roles/structural_roles",
            "Web/Accessibility/ARIA/Reference/Attributes/aria-sort",
            "Web/Other/alert_role",
        ])
    }

    #[test]
    fn shortest_names_resolve_uniquely() {
        let index = fixture();
        let cases = [
            (
                "Web/Accessibility/ARIA/Reference/Roles/alert_role",
                "Roles/alert_role",
            ),
            ("Web/Other/alert_role", "Other/alert_role"),
            (
                "Web/Accessibility/ARIA/Reference/Attributes/aria-sort",
                "aria-sort",
            ),
        ];
        for (slug, expected) in cases {
            let entry = index.slugs.iter().position(|s| s == slug).unwrap();
            assert_eq!(shortest_name(&index, entry), expected, "[{slug}]");
        }
    }

    #[test]
    fn names_resolve_to_slugs() {
        let index = fixture();
        let cases = [
            (
                "ARIA/alert_role",
                Ok("Web/Accessibility/ARIA/Reference/Roles/alert_role"),
            ),
            ("Other/alert_role", Ok("Web/Other/alert_role")),
            (
                "Aria-Sort",
                Ok("Web/Accessibility/ARIA/Reference/Attributes/aria-sort"),
            ),
            (
                "Roles/alert_role",
                Ok("Web/Accessibility/ARIA/Reference/Roles/alert_role"),
            ),
            (
                "ARIA/Roles/alert_role",
                Ok("Web/Accessibility/ARIA/Reference/Roles/alert_role"),
            ),
            (
                "Roles/ARIA/alert_role",
                Err(XrefError::NotFound("Roles/ARIA/alert_role".into())),
            ),
            (
                "alert_role",
                Err(XrefError::Ambiguous {
                    name: "alert_role".into(),
                    candidates: vec![
                        "Web/Accessibility/ARIA/Reference/Roles/alert_role".into(),
                        "Web/Other/alert_role".into(),
                    ],
                }),
            ),
            (
                "ARIA/missing",
                Err(XrefError::NotFound("ARIA/missing".into())),
            ),
            ("missing", Err(XrefError::NotFound("missing".into()))),
        ];
        for (name, expected) in cases {
            let resolved =
                resolve_xref_index(&index, name).map(|entry| index.slugs[entry].as_str());
            assert_eq!(resolved, expected, "[{name}]");
        }
    }
}
