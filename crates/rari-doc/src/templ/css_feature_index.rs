//! Lazy index of all en-US `Web/CSS/Reference/*` page slugs.
//!
//! Used by the `cssxref` template to resolve CSS feature names (properties,
//! types, functions, selectors, at-rules, descriptors) to their canonical
//! sub-path under `Web/CSS/Reference/`, replacing per-call
//! `RariApi::get_page_nowarn` lookups during URL construction.
//!
//! Also used by `csssyntax` (see `resolve_formal_syntax_ref`) to decide whether
//! a `<type>` in a formal-syntax box is documented at all.

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use css_syntax::syntax::CssRefKind;

use crate::helpers::subpages::{SubPagesSorter, get_sub_pages};
use crate::pages::page::PageLike;

const WEB_CSS_REFERENCE_PREFIX: &str = "Web/CSS/Reference/";

/// One of the four `Web/CSS/Reference/<Category>/` buckets a `cssxref` lookup
/// can target. The `as_str` form is the lowercase path segment used as the
/// first half of an index key.
#[derive(Clone, Copy, Debug)]
pub(crate) enum CssRefCategory {
    Properties,
    Values,
    Selectors,
    AtRules,
}

impl CssRefCategory {
    fn as_str(self) -> &'static str {
        match self {
            Self::Properties => "properties",
            Self::Values => "values",
            Self::Selectors => "selectors",
            Self::AtRules => "at-rules",
        }
    }

    /// Match a `Reference/<Category>/` path segment (e.g. `Properties`,
    /// `At-rules`) — case-insensitive — to its enum variant. Returns `None`
    /// for any segment that isn't one of the four known buckets.
    fn from_segment(segment: &str) -> Option<Self> {
        match segment.to_ascii_lowercase().as_str() {
            "properties" => Some(Self::Properties),
            "values" => Some(Self::Values),
            "selectors" => Some(Self::Selectors),
            "at-rules" => Some(Self::AtRules),
            _ => None,
        }
    }
}

static CSS_FEATURE_INDEX: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(build_index);

fn build_index() -> HashMap<String, Vec<String>> {
    // Slugs are locale-invariant in MDN content, so it's enough to walk the
    // en-US tree — the resulting paths are reused unchanged across locales
    // when constructing URLs.
    let pages = get_sub_pages("/en-US/docs/Web/CSS/Reference", None, SubPagesSorter::Slug)
        .expect("failed to build cssxref CSS feature index from /en-US/docs/Web/CSS/Reference");

    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    let mut unknown_categories: HashSet<String> = HashSet::new();
    for page in pages {
        let Some(after_ref) = page.slug().strip_prefix(WEB_CSS_REFERENCE_PREFIX) else {
            continue;
        };
        // The first segment must be one of the four known categories. Surface
        // any unknown bucket once so a new `Reference/<Category>/` folder
        // doesn't silently fail to resolve via `cssxref`.
        if let Some((category, _)) = after_ref.split_once('/')
            && CssRefCategory::from_segment(category).is_none()
        {
            if unknown_categories.insert(category.to_string()) {
                tracing::warn!(
                    "cssxref index: skipping unknown Reference/{category}/ bucket — \
                     add a CssRefCategory variant if it should be indexed",
                );
            }
            continue;
        }
        index_one(&mut map, after_ref);
    }
    map
}

/// Add the index entries for a single `Web/CSS/Reference/<after_ref>` page.
///
/// Each page is indexed under:
/// - the category-relative key (e.g. `Properties/color`, `Values/color_value`,
///   `Selectors/:hover`, `At-rules/@media`, `At-rules/@media/color`)
/// - for pages whose name ends in `_value` or `_function`, an alias without
///   the suffix (e.g. `Values/color_value` is also indexed under
///   `Values/color`, so `{{cssxref("<color>")}}` resolves correctly after
///   the macro strips the brackets; `Selectors/:host_function` is also
///   indexed under `Selectors/:host`, so `{{cssxref(":host()")}}` resolves).
///   The convention is Values-centric, but the alias applies regardless of
///   category for cases like the selector-function pages.
/// - for nested pages whose name contains `/` (e.g.
///   `Values/gradient/radial-gradient`), an alias under the leaf segment
///   (`Values/radial-gradient`) so callers don't need to know the
///   grouping. The `_value` / `_function` suffix-stripping also applies to
///   the leaf alias.
fn index_one(map: &mut HashMap<String, Vec<String>>, after_ref: &str) {
    if after_ref.is_empty() {
        return;
    }

    map.entry(after_ref.to_lowercase())
        .or_default()
        .push(after_ref.to_string());

    let Some((category, name)) = after_ref.split_once('/') else {
        return;
    };

    // `_value` and `_function` suffixes disambiguate types/functions from
    // properties of the same name (Values pages) and also appear on
    // selector-function pages (e.g. `Selectors/:host_function`). Index a
    // suffix-less alias so callers don't need to know the convention.
    add_suffix_aliases(map, category, name, after_ref);

    // Nested pages like `Values/gradient/radial-gradient` are otherwise only
    // reachable by callers that already know the grouping. Index the leaf
    // segment too so `{{cssxref("radial-gradient")}}` resolves.
    if let Some((_, leaf)) = name.rsplit_once('/') {
        let alias = format!("{category}/{leaf}").to_lowercase();
        map.entry(alias).or_default().push(after_ref.to_string());
        add_suffix_aliases(map, category, leaf, after_ref);
    }
}

fn add_suffix_aliases(
    map: &mut HashMap<String, Vec<String>>,
    category: &str,
    name: &str,
    after_ref: &str,
) {
    for suffix in ["_value", "_function"] {
        if let Some(without) = name.strip_suffix(suffix) {
            let alias = format!("{category}/{without}").to_lowercase();
            map.entry(alias).or_default().push(after_ref.to_string());
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
pub(crate) fn resolve_css_feature(category: CssRefCategory, slug: &str) -> Option<&'static str> {
    resolve_from_map(&CSS_FEATURE_INDEX, category, slug)
}

/// Resolve a formal-syntax `<type>` / `<'property'>` reference to the
/// `Web/CSS/Reference/` sub-path documenting it, or `None` when none does (many
/// webref productions are spec-internal). The lookup chain mirrors `cssxref`'s,
/// so both land on the same page for a given name.
pub(crate) fn resolve_formal_syntax_ref(kind: CssRefKind, slug: &str) -> Option<&'static str> {
    resolve_formal_syntax_ref_from_map(&CSS_FEATURE_INDEX, kind, slug)
}

fn resolve_formal_syntax_ref_from_map<'a>(
    map: &'a HashMap<String, Vec<String>>,
    kind: CssRefKind,
    slug: &str,
) -> Option<&'a str> {
    match kind {
        CssRefKind::Property => resolve_from_map(map, CssRefCategory::Properties, slug),
        CssRefKind::AtRule => resolve_from_map(map, CssRefCategory::AtRules, slug),
        // `_function` first, to prefer it over a same-named `_value` page.
        // `Properties/` last, for functions documented under a property
        // (`<palette-mix()>` under `font-palette`).
        CssRefKind::Type if slug.ends_with("()") => {
            let bare = slug.trim_end_matches("()");
            resolve_from_map(map, CssRefCategory::Values, &format!("{bare}_function"))
                .or_else(|| resolve_from_map(map, CssRefCategory::Values, bare))
                .or_else(|| resolve_from_map(map, CssRefCategory::Properties, bare))
        }
        // No `Properties/` fallback: `rect()`'s `<top>`/`<right>`/`<bottom>`/
        // `<left>` are positional argument types, not those properties.
        CssRefKind::Type => resolve_from_map(map, CssRefCategory::Values, slug),
    }
}

fn resolve_from_map<'a>(
    map: &'a HashMap<String, Vec<String>>,
    category: CssRefCategory,
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
            // ASCII-only comparison: CSS slugs never contain non-ASCII
            // characters, and the bucket key is already lowercase.
            (!v.eq_ignore_ascii_case(&key), !v.ends_with("_value"))
        })
        .map(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build an index from a representative set of Web/CSS/Reference sub-slugs
    /// using the real `index_one` per-slug logic.
    fn fixture() -> HashMap<String, Vec<String>> {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        for after_ref in [
            "At-rules/@media",
            "At-rules/@media/color",
            "Properties/background-color",
            "Properties/color",
            "Selectors/:hover",
            "Selectors/:host_function",
            "Values/calc",
            "Values/color_value",
            "Values/fit-content_function",
            "Values/gradient/radial-gradient",
            "Values/url_function",
            "Values/url_value",
        ] {
            index_one(&mut map, after_ref);
        }
        map
    }

    /// The real layout for slugs `csssyntax` emits.
    fn formal_syntax_fixture() -> HashMap<String, Vec<String>> {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        for after_ref in [
            "Properties/box-shadow",
            "Properties/font-palette/palette-mix",
            "Properties/padding-top",
            "Values/color_value",
            "Values/color_value/contrast-color",
            "Values/cross-fade",
            "Values/flex_value",
            "Values/image/image-set",
            "Values/length",
            "Values/url_function",
            "Values/url_value",
        ] {
            index_one(&mut map, after_ref);
        }
        map
    }

    #[test]
    fn formal_syntax_refs_resolve_to_documented_pages_only() {
        let map = formal_syntax_fixture();
        // (label, node kind, slug as `render_node` normalizes it, expected sub-path)
        let cases = vec![
            (
                "plain type",
                CssRefKind::Type,
                "length",
                Some("Values/length"),
            ),
            (
                "type aliased by `_value`, function sibling exists",
                CssRefKind::Type,
                "url",
                Some("Values/url_value"),
            ),
            (
                "type aliased by `_value`, no function sibling",
                CssRefKind::Type,
                "flex",
                Some("Values/flex_value"),
            ),
            (
                "function type nested under its parent type",
                CssRefKind::Type,
                "image-set()",
                Some("Values/image/image-set"),
            ),
            (
                "function type at the top level",
                CssRefKind::Type,
                "cross-fade()",
                Some("Values/cross-fade"),
            ),
            (
                "function type nested under a property",
                CssRefKind::Type,
                "palette-mix()",
                Some("Properties/font-palette/palette-mix"),
            ),
            // `rect()`'s `<top>` is an argument type, not the `top` property.
            (
                "type with only a same-named property page",
                CssRefKind::Type,
                "box-shadow",
                None,
            ),
            (
                "type pre-normalized to a nested slug by `render_node`",
                CssRefKind::Type,
                "color_value/contrast-color",
                Some("Values/color_value/contrast-color"),
            ),
            (
                "property longhand",
                CssRefKind::Property,
                "padding-top",
                Some("Properties/padding-top"),
            ),
            ("syntax production", CssRefKind::Type, "any-value", None),
            ("tokenizer type", CssRefKind::Type, "number-token", None),
            (
                "undocumented function type",
                CssRefKind::Type,
                "url-set()",
                None,
            ),
            (
                "undocumented spec longhand",
                CssRefKind::Property,
                "box-shadow-color",
                None,
            ),
            // `<'flex'>` is the shorthand, not the `<flex>` data type.
            (
                "property with only a same-named value page",
                CssRefKind::Property,
                "flex",
                None,
            ),
        ];
        for (label, kind, slug, expected) in cases {
            assert_eq!(
                resolve_formal_syntax_ref_from_map(&map, kind, slug),
                expected,
                "{label}: {kind:?} `{slug}`"
            );
        }
    }

    #[test]
    fn property_resolves_by_category_key() {
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, CssRefCategory::Properties, "color"),
            Some("Properties/color")
        );
        assert_eq!(
            resolve_from_map(&map, CssRefCategory::Properties, "background-color"),
            Some("Properties/background-color")
        );
    }

    #[test]
    fn type_resolves_via_value_suffix_alias() {
        let map = fixture();
        // `<color>` is normalized to slug `color` and looked up as `Values/color`.
        assert_eq!(
            resolve_from_map(&map, CssRefCategory::Values, "color"),
            Some("Values/color_value")
        );
    }

    #[test]
    fn function_resolves_by_exact_match() {
        let map = fixture();
        // `calc()` is normalized to `calc` and looked up as `Values/calc`.
        assert_eq!(
            resolve_from_map(&map, CssRefCategory::Values, "calc"),
            Some("Values/calc")
        );
    }

    #[test]
    fn function_resolves_via_function_suffix_alias() {
        let map = fixture();
        // `fit-content()` is special-cased to slug `fit-content_function`
        // and looked up as `Values/fit-content_function` — exact match.
        assert_eq!(
            resolve_from_map(&map, CssRefCategory::Values, "fit-content_function"),
            Some("Values/fit-content_function")
        );
        // But the suffix-less alias also resolves (to the same page).
        assert_eq!(
            resolve_from_map(&map, CssRefCategory::Values, "fit-content"),
            Some("Values/fit-content_function")
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
            resolve_from_map(&map, CssRefCategory::Values, "url_value"),
            Some("Values/url_value")
        );
        assert_eq!(
            resolve_from_map(&map, CssRefCategory::Values, "url_function"),
            Some("Values/url_function")
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
            resolve_from_map(&map, CssRefCategory::Values, "url"),
            Some("Values/url_value")
        );
    }

    #[test]
    fn selector_resolves() {
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, CssRefCategory::Selectors, ":hover"),
            Some("Selectors/:hover")
        );
    }

    #[test]
    fn selector_function_resolves_via_function_suffix_alias() {
        // `:host()` is special-cased to slug `:host_function` and looked up
        // as `Selectors/:host_function` — exact match.
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, CssRefCategory::Selectors, ":host_function"),
            Some("Selectors/:host_function")
        );
        // The suffix-less alias also resolves (to the same page), so
        // `{{cssxref(":host")}}` lands on the function page when no bare
        // `:host` selector page exists.
        assert_eq!(
            resolve_from_map(&map, CssRefCategory::Selectors, ":host"),
            Some("Selectors/:host_function")
        );
    }

    #[test]
    fn at_rule_and_descriptor_resolve() {
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, CssRefCategory::AtRules, "@media"),
            Some("At-rules/@media")
        );
        assert_eq!(
            resolve_from_map(&map, CssRefCategory::AtRules, "@media/color"),
            Some("At-rules/@media/color")
        );
    }

    #[test]
    fn case_insensitive_hit() {
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, CssRefCategory::Properties, "COLOR"),
            Some("Properties/color")
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
            resolve_from_map(&map, CssRefCategory::Properties, "color"),
            Some("Properties/color")
        );
        assert_eq!(
            resolve_from_map(&map, CssRefCategory::Values, "color"),
            Some("Values/color_value")
        );
    }

    #[test]
    fn nested_page_resolves_by_leaf_segment() {
        // `Reference/Values/gradient/radial-gradient` is indexed under both
        // its full category-relative key and the leaf-segment alias so
        // `{{cssxref("radial-gradient")}}` (a bare name) resolves.
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, CssRefCategory::Values, "gradient/radial-gradient"),
            Some("Values/gradient/radial-gradient")
        );
        assert_eq!(
            resolve_from_map(&map, CssRefCategory::Values, "radial-gradient"),
            Some("Values/gradient/radial-gradient")
        );
    }

    #[test]
    fn miss_returns_none() {
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, CssRefCategory::Properties, "does-not-exist"),
            None
        );
        assert_eq!(
            resolve_from_map(&map, CssRefCategory::Values, "does-not-exist"),
            None
        );
    }
}
