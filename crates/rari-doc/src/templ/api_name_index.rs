//! Lazy index of all en-US `Web/API/*` page slugs.
//!
//! Used by the `domxref` template to resolve API names that may be written as
//! either a leaf segment (e.g. `structuredClone`) or a sub-path under `Web/API/`
//! (e.g. `Window/structuredClone`).

use std::collections::HashMap;
use std::sync::LazyLock;

use tracing::warn;

use crate::helpers::subpages::{SubPagesSorter, get_sub_pages};
use crate::issues::get_issue_counter;
use crate::pages::page::PageLike;

const WEB_API_PREFIX: &str = "Web/API/";

static API_NAME_INDEX: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(build_index);

fn build_index() -> HashMap<String, Vec<String>> {
    let pages = match get_sub_pages("/en-US/docs/Web/API", None, SubPagesSorter::Slug) {
        Ok(pages) => pages,
        Err(e) => {
            warn!("failed to build domxref API name index: {e}");
            return HashMap::new();
        }
    };

    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for page in pages {
        let Some(sub_slug) = page.slug().strip_prefix(WEB_API_PREFIX) else {
            continue;
        };
        if sub_slug.is_empty() {
            continue;
        }
        let canonical = sub_slug.to_string();

        // Full sub-slug key (e.g. `Window/structuredClone`).
        insert_unique(&mut map, &canonical, canonical.clone());

        // Leaf-only key for `Window/*` members (e.g. `fetch` → `Window/fetch`).
        // Other interfaces' members must be referenced by full sub-path.
        if let Some(leaf) = sub_slug.strip_prefix("Window/")
            && !leaf.contains('/')
        {
            insert_unique(&mut map, leaf, canonical);
        }
    }
    map
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
/// `DocumentPictureInPicture/window`). Remaining ties emit a
/// `templ-ambiguous-arg` warning and the first candidate is returned.
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
    let candidates = map.get(&normalized.to_lowercase())?;
    let min_segments = candidates.iter().map(|v| segments(v)).min()?;
    let mut shortest = candidates.iter().filter(|v| segments(v) == min_segments);
    let chosen = shortest.next()?.as_str();
    let remaining = shortest.count();
    if remaining > 0 {
        warn!(
            source = "templ-ambiguous-arg",
            ic = get_issue_counter(),
            api_name = normalized,
            chosen = chosen,
            candidates = remaining + 1,
        );
    }
    Some(chosen)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Simulate what `build_index` would produce for a representative set of
    /// Web/API sub-slugs.
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
        ] {
            insert_unique(&mut map, sub_slug, sub_slug.to_string());
            if let Some(leaf) = sub_slug.strip_prefix("Window/")
                && !leaf.contains('/')
            {
                insert_unique(&mut map, leaf, sub_slug.to_string());
            }
        }
        map
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
    fn miss_returns_none() {
        let map = fixture();
        assert_eq!(resolve_from_map(&map, "DoesNotExist"), None);
    }
}
