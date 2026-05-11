//! Lazy index of all en-US `Web/API/*` page slugs.
//!
//! Used by the `domxref` template to resolve API names that may be written as
//! either a leaf segment (e.g. `structuredClone`) or a sub-path under `Web/API/`
//! (e.g. `Window/structuredClone`).

use std::collections::HashMap;
use std::sync::LazyLock;

use tracing::error;

use crate::helpers::subpages::{SubPagesSorter, get_sub_pages};
use crate::pages::page::PageLike;

const WEB_API_PREFIX: &str = "Web/API/";

static API_NAME_INDEX: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(build_index);

fn build_index() -> HashMap<String, Vec<String>> {
    let pages = match get_sub_pages("/en-US/docs/Web/API", None, SubPagesSorter::Slug) {
        Ok(pages) => pages,
        Err(e) => {
            error!("failed to build domxref API name index: {e}");
            return HashMap::new();
        }
    };

    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for page in pages {
        let Some(sub_slug) = page.slug().strip_prefix(WEB_API_PREFIX) else {
            continue;
        };
        index_one(&mut map, sub_slug);
    }
    map
}

/// Add the index entries (full key, plus optional `_static` and `Window/*`
/// leaf aliases) for a single `Web/API/<sub_slug>` page.
fn index_one(map: &mut HashMap<String, Vec<String>>, sub_slug: &str) {
    if sub_slug.is_empty() {
        return;
    }
    let canonical = sub_slug.to_string();

    // After the Web API reorg (https://github.com/orgs/mdn/discussions/796),
    // pages live at `Web/API/<Group>_API/Reference/<Name>`. The grouping
    // segments aren't part of how users reference these pages in templates,
    // so index by the portion after `/Reference/` when present.
    let indexable = sub_slug
        .split_once("/Reference/")
        .map(|(_, after)| after)
        .unwrap_or(sub_slug);

    // Full indexable key (e.g. `Window/structuredClone` or `SyncEvent`).
    insert_unique(map, indexable, canonical.clone());

    // Static methods/properties live at `<Interface>/<Name>_static` but are
    // commonly referenced without the suffix (e.g. `VideoDecoder.isConfigSupported()`).
    // Index an alias without the `_static` suffix pointing to the same canonical slug.
    //
    // We assume any slug ending in `_static` follows the static-member naming
    // convention. There are no Web/API pages today whose name legitimately
    // ends in `_static` outside that convention; if one is ever added it would
    // get a spurious suffix-stripped alias.
    if let Some(without_static) = indexable.strip_suffix("_static") {
        insert_unique(map, without_static, canonical.clone());
    }

    // Leaf-only key for `Window/*` members (e.g. `fetch` → `Window/fetch`).
    // Other interfaces' members must be referenced by full sub-path.
    if let Some(leaf) = indexable.strip_prefix("Window/")
        && !leaf.contains('/')
    {
        insert_unique(map, leaf, canonical);
    }
}

fn insert_unique(map: &mut HashMap<String, Vec<String>>, key: &str, value: String) {
    let entry = map.entry(key.to_lowercase()).or_default();
    if !entry.iter().any(|v| v == &value) {
        entry.push(value);
    }
}

/// Look up an API name in the index. Input must already be normalized
/// (spaces → `_`, `()` stripped, `.prototype.` → `.`, `.` → `/`).
///
/// Lookup is case-insensitive. When the bucket contains multiple candidates,
/// the one with the fewest path segments wins (so a top-level interface like
/// `Window` is preferred over a nested leaf like
/// `DocumentPictureInPicture/window`).
pub fn resolve_api_name(normalized: &str) -> Option<&'static str> {
    resolve_from_map(&API_NAME_INDEX, normalized)
}

fn segments(value: &str) -> usize {
    value.matches('/').count()
}

fn resolve_from_map<'a>(
    map: &'a HashMap<String, Vec<String>>,
    normalized: &str,
) -> Option<&'a str> {
    let key = normalized.to_lowercase();
    let candidates = map.get(&key)?;
    // Prefer fewer segments, then an exact (case-insensitive) match over an
    // alias — so `Response/json` resolves to `Response/json`, not the
    // `Response/json_static` slug that also aliases under the same key.
    //
    // When both keys above tie (e.g. `Response/json` vs `Response/json_static`
    // looked up by `response/json`), `min_by_key` returns the first candidate
    // in iteration order. Correctness then relies on `build_index` iterating
    // pages in ascending slug order (`SubPagesSorter::Slug`) so the canonical
    // entry is inserted before its `_static`-suffixed sibling.
    candidates
        .iter()
        .min_by_key(|v| (segments(v), v.to_lowercase() != key))
        .map(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build an index from a representative set of Web/API sub-slugs using
    /// the real `index_one` per-slug logic.
    fn fixture() -> HashMap<String, Vec<String>> {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        for sub_slug in [
            "Window",
            "Window/structuredClone",
            "Window/fetch",
            "WorkerGlobalScope/structuredClone",
            "BackgroundFetchManager/fetch",
            "DocumentPictureInPicture/window",
            "CSPViolationReport",
            "Background_Synchronization_API/Reference/SyncEvent",
            "VideoDecoder/isConfigSupported_static",
            "Response/json",
            "Response/json_static",
        ] {
            index_one(&mut map, sub_slug);
        }
        map
    }

    #[test]
    fn reference_path_is_indexed_by_post_reference_segment() {
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, "SyncEvent"),
            Some("Background_Synchronization_API/Reference/SyncEvent")
        );
    }

    #[test]
    fn case_insensitive_prefers_top_level_window() {
        let map = fixture();
        assert_eq!(resolve_from_map(&map, "Window"), Some("Window"));
        // Lowercase `window` resolves to the top-level `Window` interface, not
        // the nested `DocumentPictureInPicture/window` (which requires its full
        // sub-path).
        assert_eq!(resolve_from_map(&map, "window"), Some("Window"));
    }

    #[test]
    fn window_member_resolves_by_leaf() {
        let map = fixture();
        // `fetch` resolves to `Window/fetch`, not `BackgroundFetchManager/fetch`
        // (only `Window/*` members get a leaf-only shortcut).
        assert_eq!(resolve_from_map(&map, "fetch"), Some("Window/fetch"));
        assert_eq!(
            resolve_from_map(&map, "structuredClone"),
            Some("Window/structuredClone")
        );
    }

    #[test]
    fn non_window_member_requires_full_path() {
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, "BackgroundFetchManager/fetch"),
            Some("BackgroundFetchManager/fetch")
        );
        assert_eq!(
            resolve_from_map(&map, "DocumentPictureInPicture/window"),
            Some("DocumentPictureInPicture/window")
        );
    }

    #[test]
    fn case_insensitive_hit() {
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, "cspviolationreport"),
            Some("CSPViolationReport")
        );
    }

    #[test]
    fn static_member_resolves_with_and_without_suffix() {
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, "VideoDecoder/isConfigSupported"),
            Some("VideoDecoder/isConfigSupported_static")
        );
        assert_eq!(
            resolve_from_map(&map, "VideoDecoder/isConfigSupported_static"),
            Some("VideoDecoder/isConfigSupported_static")
        );
    }

    #[test]
    fn instance_and_static_with_same_name_are_distinguishable() {
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, "Response/json"),
            Some("Response/json")
        );
        assert_eq!(
            resolve_from_map(&map, "Response/json_static"),
            Some("Response/json_static")
        );
    }

    #[test]
    fn miss_returns_none() {
        let map = fixture();
        assert_eq!(resolve_from_map(&map, "DoesNotExist"), None);
    }
}
