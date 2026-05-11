//! Lazy index of all en-US `Web/CSS/*` page slugs.
//!
//! Used by the `cssxref` template to resolve CSS feature names (properties,
//! types, functions, selectors, at-rules, descriptors) to their canonical
//! sub-path under `Web/CSS/`, replacing per-call `RariApi::get_page_nowarn`
//! lookups during URL construction.

use std::collections::HashMap;
use std::sync::LazyLock;

use crate::helpers::subpages::{SubPagesSorter, get_sub_pages};
use crate::pages::page::PageLike;

const WEB_CSS_PREFIX: &str = "Web/CSS/";
const REFERENCE_PREFIX: &str = "Reference/";

/// One of the four `Web/CSS/Reference/<Category>/` buckets a `cssxref` lookup
/// can target. The `as_str` form is the lowercase path segment used as the
/// first half of an index key.
#[derive(Clone, Copy, Debug)]
pub(crate) enum CssReferenceCategory {
    Properties,
    Values,
    Selectors,
    AtRules,
}

impl CssReferenceCategory {
    fn as_str(self) -> &'static str {
        match self {
            Self::Properties => "properties",
            Self::Values => "values",
            Self::Selectors => "selectors",
            Self::AtRules => "at-rules",
        }
    }
}

static CSS_FEATURE_INDEX: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(build_index);

fn build_index() -> HashMap<String, Vec<String>> {
    // Slugs are locale-invariant in MDN content, so it's enough to walk the
    // en-US tree — the resulting paths are reused unchanged across locales
    // when constructing URLs.
    let pages = get_sub_pages("/en-US/docs/Web/CSS", None, SubPagesSorter::Slug)
        .expect("failed to build cssxref CSS feature index from /en-US/docs/Web/CSS");

    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for page in pages {
        let Some(sub_slug) = page.slug().strip_prefix(WEB_CSS_PREFIX) else {
            continue;
        };
        index_one(&mut map, sub_slug);
    }
    map
}

/// Add the index entries for a single `Web/CSS/<sub_slug>` page.
///
/// Each page is indexed under:
/// - if under `Reference/`, the category-relative key (e.g. `Properties/color`,
///   `Values/color_value`, `Selectors/:hover`, `At-rules/@media`,
///   `At-rules/@media/color`)
/// - for pages whose name ends in `_value` or `_function`, an alias without
///   the suffix (e.g. `Values/color_value` is also indexed under
///   `Values/color`, so `{{cssxref("<color>")}}` resolves correctly after
///   the macro strips the brackets; `Selectors/:host_function` is also
///   indexed under `Selectors/:host`, so `{{cssxref(":host()")}}` resolves).
///   The convention is Values-centric, but the alias applies regardless of
///   category for cases like the selector-function pages.
///
/// The full sub-path (e.g. `Reference/Properties/color`) is intentionally
/// not indexed: no content uses `{{cssxref("Reference/…")}}` in practice.
fn index_one(map: &mut HashMap<String, Vec<String>>, sub_slug: &str) {
    if sub_slug.is_empty() {
        return;
    }

    let Some(after_ref) = sub_slug.strip_prefix(REFERENCE_PREFIX) else {
        return;
    };
    map.entry(after_ref.to_lowercase())
        .or_default()
        .push(sub_slug.to_string());

    let Some((category, name)) = after_ref.split_once('/') else {
        return;
    };

    // `_value` and `_function` suffixes disambiguate types/functions from
    // properties of the same name (Values pages) and also appear on
    // selector-function pages (e.g. `Selectors/:host_function`). Index a
    // suffix-less alias so callers don't need to know the convention.
    for suffix in ["_value", "_function"] {
        if let Some(without) = name.strip_suffix(suffix) {
            let alias = format!("{category}/{without}").to_lowercase();
            map.entry(alias).or_default().push(sub_slug.to_string());
        }
    }
}

/// Look up a CSS feature by `category` and `slug` (e.g. `color`, `color_value`,
/// `:hover`, `@media`). The `slug` half of the key is compared case-insensitively.
///
/// When the bucket contains multiple candidates, they are ranked by:
/// 1. Exact match wins over alias matches — `Values/color` (exact) beats a
///    suffix-alias pointing into a different canonical slug.
/// 2. `_value` beats `_function` when both alias to the same bare key — `<…>`
///    and `…()` syntax already disambiguate type vs function, so the ambiguous
///    cases are bare names like `{{cssxref("url")}}` which in practice refer
///    to the data type, not the function.
///
/// The returned `&'static str` borrows from `CSS_FEATURE_INDEX`, which is a
/// `LazyLock` that lives for the rest of the process.
pub(crate) fn resolve_css_feature(
    category: CssReferenceCategory,
    slug: &str,
) -> Option<&'static str> {
    resolve_from_map(&CSS_FEATURE_INDEX, category, slug)
}

fn resolve_from_map<'a>(
    map: &'a HashMap<String, Vec<String>>,
    category: CssReferenceCategory,
    slug: &str,
) -> Option<&'a str> {
    let cat = category.as_str();
    let mut key = String::with_capacity(cat.len() + 1 + slug.len());
    key.push_str(cat);
    key.push('/');
    key.push_str(slug);
    key.make_ascii_lowercase();
    let candidates = map.get(&key)?;
    candidates
        .iter()
        .min_by_key(|v| {
            // Every indexed canonical starts with `Reference/` (see `index_one`).
            let after_ref = &v[REFERENCE_PREFIX.len()..];
            // ASCII-only comparison: CSS slugs never contain non-ASCII
            // characters, and the bucket key is already lowercase.
            (
                !after_ref.eq_ignore_ascii_case(&key),
                !v.ends_with("_value"),
            )
        })
        .map(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build an index from a representative set of Web/CSS sub-slugs using
    /// the real `index_one` per-slug logic.
    fn fixture() -> HashMap<String, Vec<String>> {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        for sub_slug in [
            "Reference/At-rules/@media",
            "Reference/At-rules/@media/color",
            "Reference/Properties/background-color",
            "Reference/Properties/color",
            "Reference/Selectors/:hover",
            "Reference/Values/calc",
            "Reference/Values/color_value",
            "Reference/Values/fit-content_function",
            "Reference/Values/url_function",
            "Reference/Values/url_value",
        ] {
            index_one(&mut map, sub_slug);
        }
        map
    }

    #[test]
    fn property_resolves_by_category_key() {
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, CssReferenceCategory::Properties, "color"),
            Some("Reference/Properties/color")
        );
        assert_eq!(
            resolve_from_map(&map, CssReferenceCategory::Properties, "background-color"),
            Some("Reference/Properties/background-color")
        );
    }

    #[test]
    fn type_resolves_via_value_suffix_alias() {
        let map = fixture();
        // `<color>` is normalized to slug `color` and looked up as `Values/color`.
        assert_eq!(
            resolve_from_map(&map, CssReferenceCategory::Values, "color"),
            Some("Reference/Values/color_value")
        );
    }

    #[test]
    fn function_resolves_by_exact_match() {
        let map = fixture();
        // `calc()` is normalized to `calc` and looked up as `Values/calc`.
        assert_eq!(
            resolve_from_map(&map, CssReferenceCategory::Values, "calc"),
            Some("Reference/Values/calc")
        );
    }

    #[test]
    fn function_resolves_via_function_suffix_alias() {
        let map = fixture();
        // `fit-content()` is special-cased to slug `fit-content_function`
        // and looked up as `Values/fit-content_function` — exact match.
        assert_eq!(
            resolve_from_map(&map, CssReferenceCategory::Values, "fit-content_function"),
            Some("Reference/Values/fit-content_function")
        );
        // But the suffix-less alias also resolves (to the same page).
        assert_eq!(
            resolve_from_map(&map, CssReferenceCategory::Values, "fit-content"),
            Some("Reference/Values/fit-content_function")
        );
    }

    #[test]
    fn exact_suffix_match_beats_sibling_alias() {
        // `{{cssxref("url_value", …)}}` looks up `Values/url_value` — exact
        // match for `Reference/Values/url_value`. The alias for `Values/url`
        // (which both url_value AND url_function insert into) must NOT win,
        // because the alias key is "Values/url", not "Values/url_value".
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, CssReferenceCategory::Values, "url_value"),
            Some("Reference/Values/url_value")
        );
        assert_eq!(
            resolve_from_map(&map, CssReferenceCategory::Values, "url_function"),
            Some("Reference/Values/url_function")
        );
    }

    #[test]
    fn bare_name_with_both_value_and_function_prefers_value() {
        // Both `url_value` and `url_function` alias to the `Values/url` key.
        // Tie-break policy: prefer `_value` — bare `{{cssxref("url")}}` in
        // practice refers to the data type (functions carry `()` syntax that
        // would route the lookup to `Values/url_function` directly).
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, CssReferenceCategory::Values, "url"),
            Some("Reference/Values/url_value")
        );
    }

    #[test]
    fn selector_resolves() {
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, CssReferenceCategory::Selectors, ":hover"),
            Some("Reference/Selectors/:hover")
        );
    }

    #[test]
    fn at_rule_and_descriptor_resolve() {
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, CssReferenceCategory::AtRules, "@media"),
            Some("Reference/At-rules/@media")
        );
        assert_eq!(
            resolve_from_map(&map, CssReferenceCategory::AtRules, "@media/color"),
            Some("Reference/At-rules/@media/color")
        );
    }

    #[test]
    fn case_insensitive_hit() {
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, CssReferenceCategory::Properties, "COLOR"),
            Some("Reference/Properties/color")
        );
    }

    #[test]
    fn bare_name_caller_controls_property_vs_value_precedence() {
        // For a bare name like `color`, `cssxref` looks up `Properties/color`
        // first and falls back to `Values/color` only on miss. Both buckets
        // resolve in isolation, so the chained-`or_else` order in cssxref is
        // what determines which page wins. Lock the contract in: both
        // category-relative keys must resolve to their own canonical page.
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, CssReferenceCategory::Properties, "color"),
            Some("Reference/Properties/color")
        );
        assert_eq!(
            resolve_from_map(&map, CssReferenceCategory::Values, "color"),
            Some("Reference/Values/color_value")
        );
    }

    #[test]
    fn miss_returns_none() {
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, CssReferenceCategory::Properties, "does-not-exist"),
            None
        );
        assert_eq!(
            resolve_from_map(&map, CssReferenceCategory::Values, "does-not-exist"),
            None
        );
    }
}
