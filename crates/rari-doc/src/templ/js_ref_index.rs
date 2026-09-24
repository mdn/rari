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
use std::sync::{Arc, LazyLock};

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

/// Two parallel maps backing [`resolve_js_ref`]:
///
/// - `primary` is keyed by canonical (case-preserved) sub-paths.
/// - `lowercase` is keyed by case-folded keys, with values unioned across
///   every case-variant in `primary`. Built once so the case-insensitive
///   fallback in [`resolve_from_index`] stays O(1).
///
/// Values are `Arc<str>` so the canonical sub-path heap buffer is shared
/// between the two maps and across the multiple primary keys that point
/// to the same page (e.g. `Statements/const` and the bare `const` leaf).
struct JsRefIndex {
    primary: HashMap<String, IndexSet<Arc<str>>>,
    lowercase: HashMap<String, IndexSet<Arc<str>>>,
}

impl JsRefIndex {
    /// Derive the lowercase fallback map from an already-built primary map.
    fn from_primary(primary: HashMap<String, IndexSet<Arc<str>>>) -> Self {
        let mut lowercase: HashMap<String, IndexSet<Arc<str>>> = HashMap::new();
        for (k, v) in &primary {
            let bucket = lowercase.entry(k.to_lowercase()).or_default();
            for s in v {
                bucket.insert(Arc::clone(s));
            }
        }
        Self { primary, lowercase }
    }
}

static JS_REF_INDEX: LazyLock<JsRefIndex> = LazyLock::new(build_index);

fn build_index() -> JsRefIndex {
    let pages = get_sub_pages(
        "/en-US/docs/Web/JavaScript/Reference",
        None,
        SubPagesSorter::Slug,
    )
    .expect("failed to build jsxref reference index");

    let mut primary: HashMap<String, IndexSet<Arc<str>>> = HashMap::new();
    for page in &pages {
        let Some(sub_slug) = page.slug().strip_prefix(JS_REF_PREFIX) else {
            continue;
        };
        index_one(&mut primary, sub_slug);
    }
    JsRefIndex::from_primary(primary)
}

/// Add the index entries (full sub-path, `Global_Objects/*` strip,
/// `Operators/*` and `Statements/*` leaf shortcuts, and namespace strip
/// for class-style namespaces) for a single
/// `Web/JavaScript/Reference/<sub_slug>` page.
fn index_one(map: &mut HashMap<String, IndexSet<Arc<str>>>, sub_slug: &str) {
    if sub_slug.is_empty() {
        return;
    }
    let canonical: Arc<str> = Arc::from(sub_slug);

    // Full sub-path key (e.g. `Statements/for...of`, `Operators/typeof`).
    insert(map, sub_slug, Arc::clone(&canonical));

    // `Global_Objects/*` strip so authors can write the bare global-object
    // name or dotted member (e.g. `Array`, `Array/from`, `undefined`).
    if let Some(rest) = sub_slug.strip_prefix(GLOBAL_OBJECTS_PREFIX) {
        insert(map, rest, Arc::clone(&canonical));
    }

    // `Operators/*` and `Statements/*` leaf shortcuts so authors can write
    // bare keywords (e.g. `null`, `typeof`, `const`, `return`). A handful of
    // keywords (`function`, `class`, `import`, `async_function`) exist as
    // both an operator (expression) and a statement (declaration); those
    // buckets end up with two candidates and `resolve_js_ref` refuses to
    // pick one — see [`resolve_from_index`].
    if let Some(rest) = sub_slug.strip_prefix(OPERATORS_PREFIX)
        && !rest.contains('/')
    {
        insert(map, rest, Arc::clone(&canonical));
    }
    if let Some(rest) = sub_slug.strip_prefix(STATEMENTS_PREFIX)
        && !rest.contains('/')
    {
        insert(map, rest, Arc::clone(&canonical));
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

fn insert(map: &mut HashMap<String, IndexSet<Arc<str>>>, key: &str, value: Arc<str>) {
    map.entry(key.to_string()).or_default().insert(value);
}

/// Normalize a raw `jsxref` name into an index key: strip `()`, collapse
/// `.prototype.` to `.`, then convert member-access `.` to `/`.
///
/// A name that already uses `/` is left untouched. Runs of `.` are preserved
/// so triple-dot statements resolve by their bare leaf (`for...of` stays
/// `for...of`, not `for///of`); only a *lone* `.` (neither neighbor is `.`)
/// is a member separator and becomes `/`.
///
/// Normalization lives here rather than in the `jsxref` caller because the
/// `.` → `/` mapping only makes sense against the `/`-separated sub-paths this
/// index is keyed by; the two evolve together.
fn normalize(name: &str) -> String {
    let collapsed = name.replace("()", "").replace(".prototype.", ".");
    if collapsed.contains('/') {
        return collapsed;
    }
    let bytes = collapsed.as_bytes();
    let mut out = String::with_capacity(collapsed.len());
    for (i, ch) in collapsed.char_indices() {
        // `.` is ASCII, so byte-indexed neighbor checks line up with chars.
        let prev_is_dot = i > 0 && bytes[i - 1] == b'.';
        let next_is_dot = bytes.get(i + 1) == Some(&b'.');
        // A `.` flanked by another `.` is part of a dot-run (e.g. `for...of`)
        // and stays; a lone `.` is a member separator and becomes `/`.
        let in_dot_run = prev_is_dot || next_is_dot;
        out.push(if ch == '.' && !in_dot_run { '/' } else { ch });
    }
    out
}

/// Normalize (see [`normalize`]) and look up a JS reference name in the index.
///
/// Resolution proceeds in two passes:
///
/// 1. **Case-sensitive** match. When a bucket holds multiple candidates
///    (e.g. `function` → `Operators/function` *and* `Statements/function`),
///    the resolver returns `None` and emits a `templ-invalid-arg` tracing
///    event listing the candidates.
/// 2. **Case-insensitive fallback** on miss. If exactly one canonical
///    sub-path matches case-insensitively (e.g. `Undefined` →
///    `Global_Objects/undefined`), the resolver returns it *and* emits a
///    `templ-ill-cased-arg` event so the author can fix the casing.
///    Multiple case-folded matches re-trigger the `templ-invalid-arg`
///    branch.
///
/// The returned `&'static str` borrows from `JS_REF_INDEX`, which is a
/// `LazyLock` that lives for the rest of the process.
pub fn resolve_js_ref(name: &str) -> Option<&'static str> {
    resolve_from_index(&JS_REF_INDEX, &normalize(name))
}

fn resolve_from_index<'a>(idx: &'a JsRefIndex, normalized: &str) -> Option<&'a str> {
    if let Some(candidates) = idx.primary.get(normalized) {
        if candidates.len() > 1 {
            warn_ambiguous(normalized, candidates);
            return None;
        }
        return candidates.iter().next().map(Arc::as_ref);
    }
    let candidates = idx.lowercase.get(&normalized.to_lowercase())?;
    // A present bucket always holds at least one entry (created via
    // `.or_default().insert(…)`), so the only distinction is one vs. many.
    if candidates.len() == 1 {
        let canonical = candidates.iter().next().unwrap();
        warn_ill_cased(normalized, canonical);
        Some(canonical.as_ref())
    } else {
        warn_ambiguous(normalized, candidates);
        None
    }
}

fn warn_ambiguous(normalized: &str, candidates: &IndexSet<Arc<str>>) {
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

fn warn_ill_cased(normalized: &str, canonical: &str) {
    let ic = get_issue_counter();
    tracing::warn!(
        source = "templ-ill-cased-arg",
        ic = ic,
        arg = normalized,
        canonical = canonical,
        "ill-cased jsxref `{normalized}`: canonical sub-path is `{canonical}`"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build an index from a representative set of `Web/JavaScript/Reference/`
    /// sub-slugs using the real `index_one` per-slug logic.
    fn fixture() -> JsRefIndex {
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

        let mut primary: HashMap<String, IndexSet<Arc<str>>> = HashMap::new();
        for sub_slug in entries {
            index_one(&mut primary, sub_slug);
        }
        JsRefIndex::from_primary(primary)
    }

    /// Normalize a raw name and resolve it, mirroring [`resolve_js_ref`]
    /// against the fixture instead of the real index.
    fn resolve<'a>(idx: &'a JsRefIndex, name: &str) -> Option<&'a str> {
        resolve_from_index(idx, &normalize(name))
    }

    #[test]
    fn full_path_resolves() {
        let idx = fixture();
        assert_eq!(
            resolve_from_index(&idx, "Statements/for...of"),
            Some("Statements/for...of")
        );
        assert_eq!(
            resolve_from_index(&idx, "Operators/typeof"),
            Some("Operators/typeof")
        );
    }

    #[test]
    fn global_object_resolves_by_bare_name() {
        let idx = fixture();
        assert_eq!(
            resolve_from_index(&idx, "Array"),
            Some("Global_Objects/Array")
        );
        assert_eq!(
            resolve_from_index(&idx, "undefined"),
            Some("Global_Objects/undefined")
        );
    }

    #[test]
    fn global_object_member_resolves_by_dotted_path() {
        let idx = fixture();
        // `Array.from` normalizes to `Array/from`.
        assert_eq!(
            resolve(&idx, "Array.from"),
            Some("Global_Objects/Array/from")
        );
        // `Array.prototype.map` collapses to `Array.map`, then `Array/map`.
        assert_eq!(
            resolve(&idx, "Array.prototype.map"),
            Some("Global_Objects/Array/map")
        );
        // A trailing `()` is stripped before lookup.
        assert_eq!(
            resolve(&idx, "Array.prototype.map()"),
            Some("Global_Objects/Array/map")
        );
    }

    #[test]
    fn case_sensitive_match_wins_when_available() {
        let idx = fixture();
        // Exact-case input resolves directly without triggering the fallback.
        assert_eq!(
            resolve_from_index(&idx, "Array"),
            Some("Global_Objects/Array")
        );
    }

    #[test]
    fn case_insensitive_fallback_resolves_wrong_case() {
        let idx = fixture();
        // Wrong case still resolves via the case-insensitive fallback (the
        // resolver also emits a `templ-ill-cased-arg` flaw — not asserted
        // here, but exercised in the build).
        assert_eq!(
            resolve_from_index(&idx, "array"),
            Some("Global_Objects/Array")
        );
        assert_eq!(
            resolve_from_index(&idx, "ARRAY/FROM"),
            Some("Global_Objects/Array/from")
        );
    }

    #[test]
    fn pascal_case_class_does_not_collide_with_lowercase_keyword_or_method() {
        let idx = fixture();
        // `Set` (PascalCase) → the global `Set` class. The case-sensitive
        // match wins without falling back, so `Reflect/set` and
        // `Functions/set` don't compete here.
        assert_eq!(resolve_from_index(&idx, "Set"), Some("Global_Objects/Set"));
        assert_eq!(
            resolve_from_index(&idx, "Functions/set"),
            Some("Functions/set")
        );

        // `Function` (PascalCase) → the global `Function` class without
        // ambiguity from `Operators/function` / `Statements/function`.
        assert_eq!(
            resolve_from_index(&idx, "Function"),
            Some("Global_Objects/Function")
        );
        // `function` (lowercase) is genuinely ambiguous — both Operators
        // and Statements have a case-sensitive bucket. The case-insensitive
        // fallback isn't triggered (case-sensitive matched), but the bucket
        // has two candidates → refused.
        assert_eq!(resolve_from_index(&idx, "function"), None);
    }

    #[test]
    fn namespace_class_resolves_without_namespace_prefix() {
        let idx = fixture();
        // `Intl.Collator` (the class) resolves to the parent page, not the
        // constructor at `Intl/Collator/Collator`.
        assert_eq!(
            resolve_from_index(&idx, "Collator"),
            Some("Global_Objects/Intl/Collator")
        );
        assert_eq!(
            resolve_from_index(&idx, "Instant"),
            Some("Global_Objects/Temporal/Instant")
        );
    }

    #[test]
    fn namespace_member_resolves_via_class_relative_path() {
        let idx = fixture();
        // Constructor: `Collator/Collator` → `Intl/Collator/Collator`.
        assert_eq!(
            resolve_from_index(&idx, "Collator/Collator"),
            Some("Global_Objects/Intl/Collator/Collator")
        );
        // Instance method: `Collator/compare` → `Intl/Collator/compare`.
        assert_eq!(
            resolve_from_index(&idx, "Collator/compare"),
            Some("Global_Objects/Intl/Collator/compare")
        );
    }

    #[test]
    fn static_api_namespace_members_require_full_path() {
        let idx = fixture();
        // `Reflect/set` is not aliased to bare `set` (Reflect isn't a
        // class-style namespace), but the full path still resolves.
        assert_eq!(
            resolve_from_index(&idx, "Reflect/set"),
            Some("Global_Objects/Reflect/set")
        );
    }

    #[test]
    fn namespace_member_also_resolves_via_full_namespace_path() {
        let idx = fixture();
        assert_eq!(
            resolve_from_index(&idx, "Intl/Collator"),
            Some("Global_Objects/Intl/Collator")
        );
        assert_eq!(
            resolve_from_index(&idx, "Intl/Collator/compare"),
            Some("Global_Objects/Intl/Collator/compare")
        );
    }

    #[test]
    fn full_global_objects_prefix_also_resolves() {
        let idx = fixture();
        assert_eq!(
            resolve_from_index(&idx, "Global_Objects/Array"),
            Some("Global_Objects/Array")
        );
        assert_eq!(
            resolve_from_index(&idx, "Global_Objects/Intl/Collator"),
            Some("Global_Objects/Intl/Collator")
        );
    }

    #[test]
    fn operator_leaf_resolves_bare() {
        let idx = fixture();
        assert_eq!(resolve_from_index(&idx, "null"), Some("Operators/null"));
        assert_eq!(resolve_from_index(&idx, "typeof"), Some("Operators/typeof"));
        // The full sub-path still works for `.`-containing leaves.
        assert_eq!(
            resolve_from_index(&idx, "Operators/new.target"),
            Some("Operators/new.target")
        );
    }

    #[test]
    fn statement_leaf_resolves_bare() {
        let idx = fixture();
        assert_eq!(resolve(&idx, "const"), Some("Statements/const"));
        // `for...of` resolves bare: normalization keeps the `.` run intact
        // (rather than mangling it to `for///of`), so the leaf shortcut hits.
        assert_eq!(resolve(&idx, "for...of"), Some("Statements/for...of"));
        // The full `Statements/` sub-path resolves too.
        assert_eq!(
            resolve(&idx, "Statements/for...of"),
            Some("Statements/for...of")
        );
    }

    #[test]
    fn ambiguous_leaf_returns_none() {
        let idx = fixture();
        // `function` (lowercase) exists as both `Operators/function`
        // (expression) and `Statements/function` (declaration). The
        // resolver refuses to pick one; the qualified forms still resolve.
        assert_eq!(resolve_from_index(&idx, "function"), None);
        assert_eq!(
            resolve_from_index(&idx, "Operators/function"),
            Some("Operators/function")
        );
        assert_eq!(
            resolve_from_index(&idx, "Statements/function"),
            Some("Statements/function")
        );
    }

    #[test]
    fn normalize_cases() {
        let cases = [
            ("plain name", "Array", "Array"),
            ("lone dot to slash", "Array.from", "Array/from"),
            ("prototype collapsed", "Array.prototype.map", "Array/map"),
            ("parens stripped", "Array.prototype.map()", "Array/map"),
            ("dot run preserved", "for...of", "for...of"),
            ("dot run with member", "Symbol.for...of", "Symbol/for...of"),
            (
                "existing slash untouched",
                "Statements/for...of",
                "Statements/for...of",
            ),
            ("single-dot leaf still split", "new.target", "new/target"),
        ];
        for (name, input, expected) in cases {
            assert_eq!(normalize(input), expected, "[{name}]");
        }
    }

    #[test]
    fn miss_returns_none() {
        let idx = fixture();
        assert_eq!(resolve_from_index(&idx, "DoesNotExist"), None);
        // A member of a namespace class is NOT addressable by leaf alone —
        // it must be qualified with at least the class.
        assert_eq!(resolve_from_index(&idx, "compare"), None);
    }

    #[test]
    fn case_insensitive_fallback_refuses_when_folds_collide() {
        let idx = fixture();
        // `SET` (uppercase) doesn't match case-sensitive; the case-folded
        // fallback would match both `Set` (the global class) and *would*
        // match `set` if any entry had that key. In our fixture there's
        // no lowercase `set` key, so `SET` resolves cleanly to `Set`.
        assert_eq!(resolve_from_index(&idx, "SET"), Some("Global_Objects/Set"));

        // Constructing a synthetic collision: insert both `Foo` and `foo`
        // mapping to different canonicals — the case-insensitive fallback
        // must refuse.
        let mut primary: HashMap<String, IndexSet<Arc<str>>> = HashMap::new();
        index_one(&mut primary, "Global_Objects/Foo");
        // Manually inject a sibling whose lowercase key collides with `Foo`.
        insert(
            &mut primary,
            "foo",
            Arc::from("Global_Objects/SomethingElse/foo"),
        );
        let idx = JsRefIndex::from_primary(primary);
        // `FOO` matches neither case-sensitively; the fallback merges
        // `Foo` and `foo` buckets → 2 candidates → refusal.
        assert_eq!(resolve_from_index(&idx, "FOO"), None);
    }
}
