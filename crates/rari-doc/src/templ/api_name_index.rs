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
        insert_unique(&mut map, canonical.clone(), canonical.clone());

        // Leaf segment key (e.g. `structuredClone`).
        if let Some(leaf) = sub_slug.rsplit('/').next()
            && leaf != sub_slug
        {
            insert_unique(&mut map, leaf.to_string(), canonical);
        }
    }
    map
}

fn insert_unique(map: &mut HashMap<String, Vec<String>>, key: String, value: String) {
    let entry = map.entry(key).or_default();
    if !entry.iter().any(|v| v == &value) {
        entry.push(value);
    }
}

/// Look up an API name in the index. Input must already be normalized
/// (spaces → `_`, `()` stripped, `.prototype.` → `.`, `.` → `/`).
///
/// On ambiguity (multiple Web/API pages share the same key), emits a
/// `templ-ambiguous-arg` warning and returns the first candidate.
pub fn resolve_api_name(normalized: &str) -> Option<&'static str> {
    resolve_from_map(&API_NAME_INDEX, normalized)
}

fn resolve_from_map<'a>(
    map: &'a HashMap<String, Vec<String>>,
    normalized: &str,
) -> Option<&'a str> {
    let candidates = map.get(normalized)?;
    let first = candidates.first()?.as_str();
    if candidates.len() > 1 {
        warn!(
            source = "templ-ambiguous-arg",
            ic = get_issue_counter(),
            api_name = normalized,
            chosen = first,
            candidates = candidates.len(),
        );
    }
    Some(first)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> HashMap<String, Vec<String>> {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        insert_unique(
            &mut map,
            "Window/structuredClone".into(),
            "Window/structuredClone".into(),
        );
        insert_unique(
            &mut map,
            "structuredClone".into(),
            "Window/structuredClone".into(),
        );
        insert_unique(
            &mut map,
            "WorkerGlobalScope/structuredClone".into(),
            "WorkerGlobalScope/structuredClone".into(),
        );
        insert_unique(
            &mut map,
            "structuredClone".into(),
            "WorkerGlobalScope/structuredClone".into(),
        );
        insert_unique(
            &mut map,
            "CSPViolationReport".into(),
            "CSPViolationReport".into(),
        );
        map
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
    fn ambiguous_leaf_returns_first() {
        let map = fixture();
        // First inserted candidate wins.
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
