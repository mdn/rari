//! Lazy index of all en-US `Web/JavaScript/Reference/*` page slugs.
//!
//! Used by the `jsxref` template to resolve JavaScript references that may be
//! written as a full sub-path (e.g. `Statements/for...of`), a dotted member
//! expression (e.g. `Array.prototype.map`), or — for members of namespace
//! classes like `Intl` and `Temporal` — without the namespace prefix
//! (e.g. `Collator` for `Intl.Collator`).

use std::collections::HashMap;
use std::sync::LazyLock;

use indexmap::IndexSet;
use rari_types::fm_types::PageType;

use crate::helpers::subpages::{SubPagesSorter, get_sub_pages};
use crate::pages::page::PageLike;

const JS_REF_PREFIX: &str = "Web/JavaScript/Reference/";
const GLOBAL_OBJECTS_PREFIX: &str = "Global_Objects/";

static JS_REF_INDEX: LazyLock<HashMap<String, IndexSet<String>>> = LazyLock::new(build_index);

fn build_index() -> HashMap<String, IndexSet<String>> {
    let pages = get_sub_pages(
        "/en-US/docs/Web/JavaScript/Reference",
        None,
        SubPagesSorter::Slug,
    )
    .expect("failed to build jsxref reference index");

    // Identify top-level namespace pages (e.g. `Global_Objects/Intl`,
    // `Global_Objects/Temporal`). Their direct children are full-fledged
    // classes that authors should be able to reference without the
    // namespace prefix.
    let ns_prefixes: Vec<String> = pages
        .iter()
        .filter(|p| p.page_type() == PageType::JavascriptNamespace)
        .filter_map(|p| {
            p.slug()
                .strip_prefix(JS_REF_PREFIX)
                .and_then(|s| s.strip_prefix(GLOBAL_OBJECTS_PREFIX))
                .filter(|rest| !rest.contains('/'))
                .map(|rest| format!("{GLOBAL_OBJECTS_PREFIX}{rest}/"))
        })
        .collect();

    let mut map: HashMap<String, IndexSet<String>> = HashMap::new();
    for page in &pages {
        let Some(sub_slug) = page.slug().strip_prefix(JS_REF_PREFIX) else {
            continue;
        };
        index_one(&mut map, sub_slug, &ns_prefixes);
    }
    map
}

/// Add the index entries (full sub-path, `Global_Objects/*` strip, and
/// namespace strip when applicable) for a single
/// `Web/JavaScript/Reference/<sub_slug>` page.
fn index_one(map: &mut HashMap<String, IndexSet<String>>, sub_slug: &str, ns_prefixes: &[String]) {
    if sub_slug.is_empty() {
        return;
    }
    let canonical = sub_slug.to_string();

    // Full sub-path key (e.g. `Statements/for...of`, `Operators/typeof`).
    insert(map, sub_slug, canonical.clone());

    // `Global_Objects/*` strip so authors can write the bare global-object
    // name or dotted member (e.g. `Array`, `Array/from`, `undefined`).
    if let Some(rest) = sub_slug.strip_prefix(GLOBAL_OBJECTS_PREFIX) {
        insert(map, rest, canonical.clone());
    }

    // Namespace strip: for pages under a namespace class (Intl, Temporal),
    // also index by the path with the namespace removed so authors can write
    // `Collator` instead of `Intl/Collator`. The path structure naturally
    // distinguishes class (`Collator`) from constructor (`Collator/Collator`)
    // and members (`Collator/compare`), so there's no class-vs-descendant
    // clash to resolve.
    for ns in ns_prefixes {
        if let Some(rest) = sub_slug.strip_prefix(ns.as_str()) {
            insert(map, rest, canonical);
            break;
        }
    }
}

fn insert(map: &mut HashMap<String, IndexSet<String>>, key: &str, value: String) {
    map.entry(key.to_lowercase()).or_default().insert(value);
}

/// Look up a JS reference name in the index. Input must already be normalized
/// (`()` stripped, `.prototype.` → `.`, then `.` → `/` if no `/` is present).
///
/// Lookup is case-insensitive. When the bucket contains multiple candidates,
/// the one with the fewest path segments wins, so a top-level page (e.g.
/// `Global_Objects/Intl/Collator`) is preferred over a nested clash.
///
/// The returned `&'static str` borrows from `JS_REF_INDEX`, which is a
/// `LazyLock` that lives for the rest of the process.
pub fn resolve_js_ref(normalized: &str) -> Option<&'static str> {
    resolve_from_map(&JS_REF_INDEX, normalized)
}

fn segments(value: &str) -> usize {
    value.matches('/').count()
}

fn resolve_from_map<'a>(
    map: &'a HashMap<String, IndexSet<String>>,
    normalized: &str,
) -> Option<&'a str> {
    let key = normalized.to_lowercase();
    let candidates = map.get(&key)?;
    candidates
        .iter()
        .min_by_key(|v| segments(v))
        .map(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build an index from a representative set of `Web/JavaScript/Reference/`
    /// sub-slugs using the real `index_one` per-slug logic. Namespace
    /// detection is reproduced here from the per-fixture page-type metadata.
    fn fixture() -> HashMap<String, IndexSet<String>> {
        let entries: &[(&str, PageType)] = &[
            // Top-level reference categories
            ("Statements/for...of", PageType::JavascriptStatement),
            ("Statements/try...catch", PageType::JavascriptStatement),
            ("Operators/typeof", PageType::JavascriptOperator),
            // Global objects and members
            ("Global_Objects/Array", PageType::JavascriptClass),
            (
                "Global_Objects/Array/from",
                PageType::JavascriptStaticMethod,
            ),
            (
                "Global_Objects/Array/map",
                PageType::JavascriptInstanceMethod,
            ),
            (
                "Global_Objects/undefined",
                PageType::JavascriptLanguageFeature,
            ),
            // Namespace and members
            ("Global_Objects/Intl", PageType::JavascriptNamespace),
            ("Global_Objects/Intl/Collator", PageType::JavascriptClass),
            (
                "Global_Objects/Intl/Collator/Collator",
                PageType::JavascriptConstructor,
            ),
            (
                "Global_Objects/Intl/Collator/compare",
                PageType::JavascriptInstanceMethod,
            ),
            ("Global_Objects/Temporal", PageType::JavascriptNamespace),
            ("Global_Objects/Temporal/Instant", PageType::JavascriptClass),
        ];

        let ns_prefixes: Vec<String> = entries
            .iter()
            .filter(|(_, pt)| *pt == PageType::JavascriptNamespace)
            .filter_map(|(slug, _)| {
                slug.strip_prefix(GLOBAL_OBJECTS_PREFIX)
                    .filter(|rest| !rest.contains('/'))
                    .map(|rest| format!("{GLOBAL_OBJECTS_PREFIX}{rest}/"))
            })
            .collect();

        let mut map: HashMap<String, IndexSet<String>> = HashMap::new();
        for (sub_slug, _) in entries {
            index_one(&mut map, sub_slug, &ns_prefixes);
        }
        map
    }

    #[test]
    fn full_path_resolves() {
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, "Statements/for...of"),
            Some("Statements/for...of")
        );
        assert_eq!(
            resolve_from_map(&map, "Operators/typeof"),
            Some("Operators/typeof")
        );
    }

    #[test]
    fn global_object_resolves_by_bare_name() {
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, "Array"),
            Some("Global_Objects/Array")
        );
        assert_eq!(
            resolve_from_map(&map, "undefined"),
            Some("Global_Objects/undefined")
        );
    }

    #[test]
    fn global_object_member_resolves_by_dotted_path() {
        let map = fixture();
        // Caller normalizes `Array.from` → `Array/from`.
        assert_eq!(
            resolve_from_map(&map, "Array/from"),
            Some("Global_Objects/Array/from")
        );
        // Caller normalizes `Array.prototype.map` → `Array/map`.
        assert_eq!(
            resolve_from_map(&map, "Array/map"),
            Some("Global_Objects/Array/map")
        );
    }

    #[test]
    fn case_insensitive_lookup() {
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, "array"),
            Some("Global_Objects/Array")
        );
        assert_eq!(
            resolve_from_map(&map, "ARRAY/FROM"),
            Some("Global_Objects/Array/from")
        );
    }

    #[test]
    fn namespace_class_resolves_without_namespace_prefix() {
        let map = fixture();
        // `Intl.Collator` (the class) resolves to the parent page, not the
        // constructor at `Intl/Collator/Collator`.
        assert_eq!(
            resolve_from_map(&map, "Collator"),
            Some("Global_Objects/Intl/Collator")
        );
        assert_eq!(
            resolve_from_map(&map, "Instant"),
            Some("Global_Objects/Temporal/Instant")
        );
    }

    #[test]
    fn namespace_member_resolves_via_class_relative_path() {
        let map = fixture();
        // Constructor: `Collator/Collator` → `Intl/Collator/Collator`.
        assert_eq!(
            resolve_from_map(&map, "Collator/Collator"),
            Some("Global_Objects/Intl/Collator/Collator")
        );
        // Instance method: `Collator/compare` → `Intl/Collator/compare`.
        assert_eq!(
            resolve_from_map(&map, "Collator/compare"),
            Some("Global_Objects/Intl/Collator/compare")
        );
    }

    #[test]
    fn namespace_member_also_resolves_via_full_namespace_path() {
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, "Intl/Collator"),
            Some("Global_Objects/Intl/Collator")
        );
        assert_eq!(
            resolve_from_map(&map, "Intl/Collator/compare"),
            Some("Global_Objects/Intl/Collator/compare")
        );
    }

    #[test]
    fn full_global_objects_prefix_also_resolves() {
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, "Global_Objects/Array"),
            Some("Global_Objects/Array")
        );
        assert_eq!(
            resolve_from_map(&map, "Global_Objects/Intl/Collator"),
            Some("Global_Objects/Intl/Collator")
        );
    }

    #[test]
    fn miss_returns_none() {
        let map = fixture();
        assert_eq!(resolve_from_map(&map, "DoesNotExist"), None);
        // A member of a namespace class is NOT addressable by leaf alone —
        // it must be qualified with at least the class.
        assert_eq!(resolve_from_map(&map, "compare"), None);
    }
}
