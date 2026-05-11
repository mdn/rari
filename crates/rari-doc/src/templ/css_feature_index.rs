//! Lazy index of all en-US `Web/CSS/*` page slugs.
//!
//! Used by the `cssxref` template to resolve CSS feature names (properties,
//! types, functions, selectors, at-rules, descriptors) to their canonical
//! sub-path under `Web/CSS/`, replacing per-call `RariApi::get_page_nowarn`
//! lookups during URL construction.

use std::collections::HashMap;
use std::sync::LazyLock;

use tracing::error;

use crate::helpers::subpages::{SubPagesSorter, get_sub_pages};
use crate::pages::page::PageLike;

const WEB_CSS_PREFIX: &str = "Web/CSS/";

static CSS_FEATURE_INDEX: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(build_index);

fn build_index() -> HashMap<String, Vec<String>> {
    let pages = match get_sub_pages("/en-US/docs/Web/CSS", None, SubPagesSorter::Slug) {
        Ok(pages) => pages,
        Err(e) => {
            error!("failed to build cssxref CSS feature index: {e}");
            return HashMap::new();
        }
    };

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
/// - its full sub-path (e.g. `Reference/Properties/color`)
/// - if under `Reference/`, the category-relative key (e.g. `Properties/color`,
///   `Values/color_value`, `Selectors/:hover`, `At-rules/@media`,
///   `At-rules/@media/color`)
/// - for Values pages whose name ends in `_value` or `_function`, an alias
///   without the suffix (e.g. `Values/color_value` is also indexed under
///   `Values/color`, so `{{cssxref("<color>")}}` resolves correctly after
///   the macro strips the brackets).
fn index_one(map: &mut HashMap<String, Vec<String>>, sub_slug: &str) {
    if sub_slug.is_empty() {
        return;
    }
    let canonical = sub_slug.to_string();

    insert_unique(map, sub_slug, canonical.clone());

    let Some(after_ref) = sub_slug.strip_prefix("Reference/") else {
        return;
    };
    insert_unique(map, after_ref, canonical.clone());

    let Some((category, name)) = after_ref.split_once('/') else {
        return;
    };

    // Values pages use `_value` and `_function` suffixes to disambiguate
    // types and functions from properties of the same name. Index a
    // suffix-less alias so callers don't need to know the convention.
    for suffix in ["_value", "_function"] {
        if let Some(without) = name.strip_suffix(suffix) {
            let alias = format!("{category}/{without}");
            insert_unique(map, &alias, canonical.clone());
        }
    }
}

fn insert_unique(map: &mut HashMap<String, Vec<String>>, key: &str, value: String) {
    let entry = map.entry(key.to_lowercase()).or_default();
    if !entry.iter().any(|v| v == &value) {
        entry.push(value);
    }
}

/// Look up a CSS feature by its category-relative path (e.g. `Properties/color`,
/// `Values/color_value`, `Values/color`, `Selectors/:hover`, `At-rules/@media`)
/// or full sub-path (e.g. `Reference/Properties/color`).
///
/// Lookup is case-insensitive. When the bucket contains multiple candidates,
/// the one whose post-`Reference/` portion is an exact (case-insensitive)
/// match for the key wins — so `Values/color` (alias) loses to `Values/color`
/// (exact) if both exist. Otherwise the candidate with the fewest path
/// segments wins.
///
/// The returned `&'static str` borrows from `CSS_FEATURE_INDEX`, which is a
/// `LazyLock` that lives for the rest of the process.
pub fn resolve_css_feature(category_path: &str) -> Option<&'static str> {
    resolve_from_map(&CSS_FEATURE_INDEX, category_path)
}

fn resolve_from_map<'a>(
    map: &'a HashMap<String, Vec<String>>,
    category_path: &str,
) -> Option<&'a str> {
    let key = category_path.to_lowercase();
    let candidates = map.get(&key)?;
    candidates
        .iter()
        .min_by_key(|v| {
            let after_ref = v.strip_prefix("Reference/").unwrap_or(v.as_str());
            (after_ref.to_lowercase() != key, v.matches('/').count())
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
            resolve_from_map(&map, "Properties/color"),
            Some("Reference/Properties/color")
        );
        assert_eq!(
            resolve_from_map(&map, "Properties/background-color"),
            Some("Reference/Properties/background-color")
        );
    }

    #[test]
    fn type_resolves_via_value_suffix_alias() {
        let map = fixture();
        // `<color>` is normalized to slug `color` and looked up as `Values/color`.
        assert_eq!(
            resolve_from_map(&map, "Values/color"),
            Some("Reference/Values/color_value")
        );
    }

    #[test]
    fn function_resolves_by_exact_match() {
        let map = fixture();
        // `calc()` is normalized to `calc` and looked up as `Values/calc`.
        assert_eq!(
            resolve_from_map(&map, "Values/calc"),
            Some("Reference/Values/calc")
        );
    }

    #[test]
    fn function_resolves_via_function_suffix_alias() {
        let map = fixture();
        // `fit-content()` is special-cased to slug `fit-content_function`
        // and looked up as `Values/fit-content_function` — exact match.
        assert_eq!(
            resolve_from_map(&map, "Values/fit-content_function"),
            Some("Reference/Values/fit-content_function")
        );
        // But the suffix-less alias also resolves (to the same page).
        assert_eq!(
            resolve_from_map(&map, "Values/fit-content"),
            Some("Reference/Values/fit-content_function")
        );
    }

    #[test]
    fn ambiguous_suffix_alias_picks_lowest_segment_count() {
        // `url()` could be either `url_function` or `url_value`. Both alias
        // under `Values/url`; tie-break gives a deterministic answer.
        let map = fixture();
        let resolved = resolve_from_map(&map, "Values/url").unwrap();
        assert!(
            resolved == "Reference/Values/url_function" || resolved == "Reference/Values/url_value",
            "unexpected resolution: {resolved}"
        );
    }

    #[test]
    fn selector_resolves() {
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, "Selectors/:hover"),
            Some("Reference/Selectors/:hover")
        );
    }

    #[test]
    fn at_rule_and_descriptor_resolve() {
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, "At-rules/@media"),
            Some("Reference/At-rules/@media")
        );
        assert_eq!(
            resolve_from_map(&map, "At-rules/@media/color"),
            Some("Reference/At-rules/@media/color")
        );
    }

    #[test]
    fn full_sub_path_resolves() {
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, "Reference/Properties/color"),
            Some("Reference/Properties/color")
        );
    }

    #[test]
    fn case_insensitive_hit() {
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, "properties/COLOR"),
            Some("Reference/Properties/color")
        );
    }

    #[test]
    fn miss_returns_none() {
        let map = fixture();
        assert_eq!(resolve_from_map(&map, "Properties/does-not-exist"), None);
        assert_eq!(resolve_from_map(&map, "Values/does-not-exist"), None);
    }
}
