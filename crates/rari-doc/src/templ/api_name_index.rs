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

        // Leaf segment key (e.g. `structuredClone`).
        if let Some(leaf) = sub_slug.rsplit('/').next()
            && leaf != sub_slug
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

    fn fixture() -> HashMap<String, Vec<String>> {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        insert_unique(
            &mut map,
            "Window/structuredClone",
            "Window/structuredClone".into(),
        );
        insert_unique(&mut map, "structuredClone", "Window/structuredClone".into());
        insert_unique(
            &mut map,
            "WorkerGlobalScope/structuredClone",
            "WorkerGlobalScope/structuredClone".into(),
        );
        insert_unique(
            &mut map,
            "structuredClone",
            "WorkerGlobalScope/structuredClone".into(),
        );
        insert_unique(&mut map, "CSPViolationReport", "CSPViolationReport".into());
        insert_unique(&mut map, "Window", "Window".into());
        insert_unique(
            &mut map,
            "DocumentPictureInPicture/window",
            "DocumentPictureInPicture/window".into(),
        );
        insert_unique(&mut map, "window", "DocumentPictureInPicture/window".into());
        map
    }

    #[test]
    fn case_insensitive_prefers_top_level_interface() {
        let map = fixture();
        assert_eq!(resolve_from_map(&map, "Window"), Some("Window"));
        // Lowercase input also resolves to the top-level interface, not the
        // nested `DocumentPictureInPicture/window` leaf.
        assert_eq!(resolve_from_map(&map, "window"), Some("Window"));
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
    fn resolves_full_sub_slug() {
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, "Window/structuredClone"),
            Some("Window/structuredClone")
        );
    }

    #[test]
    fn resolves_single_segment() {
        let map = fixture();
        assert_eq!(
            resolve_from_map(&map, "CSPViolationReport"),
            Some("CSPViolationReport")
        );
    }

    #[test]
    fn ambiguous_leaf_returns_first_at_min_depth() {
        let map = fixture();
        // Two candidates at the same depth (1 slash); first inserted wins.
        assert_eq!(
            resolve_from_map(&map, "structuredClone"),
            Some("Window/structuredClone")
        );
    }

    #[test]
    fn miss_returns_none() {
        let map = fixture();
        assert_eq!(resolve_from_map(&map, "DoesNotExist"), None);
    }
}
