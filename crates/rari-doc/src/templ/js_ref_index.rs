//! Lazy index of all en-US `Web/JavaScript/Reference/*` page slugs.
//!
//! Used by the `jsxref` template to resolve JavaScript references that may be
//! written as a full sub-path (e.g. `Statements/for...of`), a dotted member
//! expression (e.g. `Array.prototype.map`), or — for members of the
//! class-style namespaces `Intl` and `Temporal` — without the namespace
//! prefix (e.g. `Collator` for `Intl.Collator`).
//!
//! Lookups are **case-sensitive**, mirroring JavaScript's naming convention
//! where casing carries meaning: `Set` is the global class,
//! `set` is the `Reflect.set` static method or the `set` getter syntax;
//! `Function` is the global class, `function` is the keyword. The previous
//! case-insensitive scheme produced spurious "ambiguous" reports for ~826
//! PascalCase class references across content + translated-content.

use std::collections::HashMap;
use std::sync::LazyLock;

use indexmap::IndexSet;

use crate::helpers::subpages::{SubPagesSorter, get_sub_pages};
use crate::issues::get_issue_counter;
use crate::pages::page::PageLike;

const JS_REF_PREFIX: &str = "Web/JavaScript/Reference/";
const GLOBAL_OBJECTS_PREFIX: &str = "Global_Objects/";
const OPERATORS_PREFIX: &str = "Operators/";
const STATEMENTS_PREFIX: &str = "Statements/";

/// Namespaces whose direct children are *classes* (`Intl.Collator`,
/// `Temporal.Instant`, etc.) and should be addressable without the
/// namespace prefix.
///
/// Excludes static-API namespaces like `Reflect`, `Atomics`, `Math`, and
/// `JSON` whose children are *methods* (e.g. `Reflect.set`, `Math.sin`).
/// Stripping the namespace there would alias `set`/`sin`/etc. to a JS
/// reference page, creating spurious collisions with the global `Set`
/// class, the `Functions/set` getter syntax, etc.
const NAMESPACE_PREFIXES: &[&str] = &["Global_Objects/Intl/", "Global_Objects/Temporal/"];

static JS_REF_INDEX: LazyLock<HashMap<String, IndexSet<String>>> = LazyLock::new(build_index);

fn build_index() -> HashMap<String, IndexSet<String>> {
    let pages = get_sub_pages(
        "/en-US/docs/Web/JavaScript/Reference",
        None,
        SubPagesSorter::Slug,
    )
    .expect("failed to build jsxref reference index");

    let mut map: HashMap<String, IndexSet<String>> = HashMap::new();
    for page in &pages {
        let Some(sub_slug) = page.slug().strip_prefix(JS_REF_PREFIX) else {
            continue;
        };
        index_one(&mut map, sub_slug);
    }
    map
}

/// Add the index entries (full sub-path, `Global_Objects/*` strip,
/// `Operators/*` and `Statements/*` leaf shortcuts, and namespace strip
/// for class-style namespaces) for a single
/// `Web/JavaScript/Reference/<sub_slug>` page.
fn index_one(map: &mut HashMap<String, IndexSet<String>>, sub_slug: &str) {
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

    // `Operators/*` and `Statements/*` leaf shortcuts so authors can write
    // bare keywords (e.g. `null`, `typeof`, `const`, `return`). A handful of
    // keywords (`function`, `class`, `import`, `async_function`) exist as
    // both an operator (expression) and a statement (declaration); those
    // buckets end up with two candidates and `resolve_js_ref` refuses to
    // pick one — see [`resolve_from_map`].
    if let Some(rest) = sub_slug.strip_prefix(OPERATORS_PREFIX)
        && !rest.contains('/')
    {
        insert(map, rest, canonical.clone());
    }
    if let Some(rest) = sub_slug.strip_prefix(STATEMENTS_PREFIX)
        && !rest.contains('/')
    {
        insert(map, rest, canonical.clone());
    }

    // Namespace strip: for pages under a class-style namespace (Intl,
    // Temporal), also index by the path with the namespace removed so
    // authors can write `Collator` instead of `Intl/Collator`. The path
    // structure naturally distinguishes class (`Collator`) from constructor
    // (`Collator/Collator`) and members (`Collator/compare`).
    for ns in NAMESPACE_PREFIXES {
        if let Some(rest) = sub_slug.strip_prefix(ns) {
            insert(map, rest, canonical);
            break;
        }
    }
}

fn insert(map: &mut HashMap<String, IndexSet<String>>, key: &str, value: String) {
    map.entry(key.to_string()).or_default().insert(value);
}

/// Look up a JS reference name in the index. Input must already be normalized
/// (`()` stripped, `.prototype.` → `.`, then `.` → `/` if no `/` is present).
///
/// Lookups are **case-sensitive**. When a bucket holds multiple candidates
/// (e.g. `function` → `Operators/function` *and* `Statements/function`),
/// `resolve_js_ref` returns `None` and emits a `templ-invalid-arg` tracing
/// event listing the candidates so the author can add the qualifying
/// category prefix. The caller falls back to a literal URL, which the link
/// validator may additionally flag — that's intentional: the
/// `templ-invalid-arg` flaw explains *why*, the resulting
/// `templ-redirected-link` / `templ-broken-link` flaw points to *where*.
///
/// The returned `&'static str` borrows from `JS_REF_INDEX`, which is a
/// `LazyLock` that lives for the rest of the process.
pub fn resolve_js_ref(normalized: &str) -> Option<&'static str> {
    resolve_from_map(&JS_REF_INDEX, normalized)
}

fn resolve_from_map<'a>(
    map: &'a HashMap<String, IndexSet<String>>,
    normalized: &str,
) -> Option<&'a str> {
    let candidates = map.get(normalized)?;
    if candidates.len() > 1 {
        warn_ambiguous(normalized, candidates);
        return None;
    }
    candidates.iter().next().map(String::as_str)
}

fn warn_ambiguous(normalized: &str, candidates: &IndexSet<String>) {
    let ic = get_issue_counter();
    let options = candidates
        .iter()
        .map(|s| format!("`{s}`"))
        .collect::<Vec<_>>()
        .join(" or ");
    tracing::warn!(
        source = "templ-invalid-arg",
        ic = ic,
        arg = normalized,
        "ambiguous jsxref `{normalized}`: matches {options}; qualify with the full sub-path"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build an index from a representative set of `Web/JavaScript/Reference/`
    /// sub-slugs using the real `index_one` per-slug logic.
    fn fixture() -> HashMap<String, IndexSet<String>> {
        let entries = [
            // Top-level reference categories
            "Statements/for...of",
            "Statements/try...catch",
            "Statements/const",
            "Statements/function",
            "Operators/typeof",
            "Operators/null",
            "Operators/new.target",
            "Operators/function",
            // Global objects and members
            "Global_Objects/Array",
            "Global_Objects/Array/from",
            "Global_Objects/Array/map",
            "Global_Objects/undefined",
            "Global_Objects/Set",
            "Global_Objects/Function",
            "Functions/set",
            // Class-style namespace (in NAMESPACE_PREFIXES)
            "Global_Objects/Intl",
            "Global_Objects/Intl/Collator",
            "Global_Objects/Intl/Collator/Collator",
            "Global_Objects/Intl/Collator/compare",
            "Global_Objects/Temporal",
            "Global_Objects/Temporal/Instant",
            // Static-API namespace (NOT in NAMESPACE_PREFIXES) — `Reflect.set`
            // must not alias to `set`, since `set` collides semantically with
            // `Functions/set` and the global `Set` class.
            "Global_Objects/Reflect",
            "Global_Objects/Reflect/set",
        ];

        let mut map: HashMap<String, IndexSet<String>> = HashMap::new();
        for sub_slug in entries {
            index_one(&mut map, sub_slug);
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
    fn lookups_are_case_sensitive() {
        let map = fixture();
        // Wrong case → miss. Authors are expected to match the canonical
        // sub-path casing.
        assert_eq!(resolve_from_map(&map, "array"), None);
        assert_eq!(resolve_from_map(&map, "ARRAY/FROM"), None);
    }

    #[test]
    fn pascal_case_class_does_not_collide_with_lowercase_keyword_or_method() {
        let map = fixture();
        // `Set` (PascalCase) → the global `Set` class. `set` (lowercase)
        // resolves to nothing since `Reflect/set` is not aliased
        // (Reflect isn't in `NAMESPACE_PREFIXES`) and `Functions/set` is
        // only addressable by its full sub-path.
        assert_eq!(resolve_from_map(&map, "Set"), Some("Global_Objects/Set"));
        assert_eq!(resolve_from_map(&map, "set"), None);
        assert_eq!(
            resolve_from_map(&map, "Functions/set"),
            Some("Functions/set")
        );

        // `Function` (PascalCase) → the global `Function` class.
        // `function` (lowercase) → ambiguous between the operator and the
        // statement; refused.
        assert_eq!(
            resolve_from_map(&map, "Function"),
            Some("Global_Objects/Function")
        );
        assert_eq!(resolve_from_map(&map, "function"), None);
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
    fn static_api_namespace_members_require_full_path() {
        let map = fixture();
        // `Reflect/set` is not aliased to bare `set` (Reflect isn't a
        // class-style namespace), but the full path still resolves.
        assert_eq!(
            resolve_from_map(&map, "Reflect/set"),
            Some("Global_Objects/Reflect/set")
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
    fn operator_leaf_resolves_bare() {
        let map = fixture();
        assert_eq!(resolve_from_map(&map, "null"), Some("Operators/null"));
        assert_eq!(resolve_from_map(&map, "typeof"), Some("Operators/typeof"));
        // The full sub-path still works for `.`-containing leaves.
        assert_eq!(
            resolve_from_map(&map, "Operators/new.target"),
            Some("Operators/new.target")
        );
    }

    #[test]
    fn statement_leaf_resolves_bare() {
        let map = fixture();
        assert_eq!(resolve_from_map(&map, "const"), Some("Statements/const"));
        // `for...of` still resolves via the full `Statements/` sub-path
        // (its `.` makes a bare-leaf shortcut ambiguous with paths).
        assert_eq!(
            resolve_from_map(&map, "Statements/for...of"),
            Some("Statements/for...of")
        );
    }

    #[test]
    fn ambiguous_leaf_returns_none() {
        let map = fixture();
        // `function` (lowercase) exists as both `Operators/function`
        // (expression) and `Statements/function` (declaration). The
        // resolver refuses to pick one; the qualified forms still resolve.
        assert_eq!(resolve_from_map(&map, "function"), None);
        assert_eq!(
            resolve_from_map(&map, "Operators/function"),
            Some("Operators/function")
        );
        assert_eq!(
            resolve_from_map(&map, "Statements/function"),
            Some("Statements/function")
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
