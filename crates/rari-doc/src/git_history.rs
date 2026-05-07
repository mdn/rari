//! # Git History Module
//!
//! Loads `_git_history.json` artifacts (produced by `rari git-history`) for the
//! content and translated-content roots and exposes them as a single map keyed
//! by the relative file path used elsewhere in the build pipeline.
//!
//! Redirects from `_redirects.txt` are applied at load time so that pages whose
//! source file was moved (locally by `sync-translated-content`, or in a build
//! that ran before `git-history` could pick up a `content move`) still resolve
//! to a real history entry instead of the 1970-01-01 default. See
//! <https://github.com/mdn/rari/issues/247>.
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;

use rari_types::HistoryEntry;
use rari_types::globals::{content_root, content_translated_root};

use crate::redirects::REDIRECTS;
use crate::resolve::{UrlMeta, url_meta_from};

static GIT_HISTORY: LazyLock<HashMap<PathBuf, HistoryEntry>> = LazyLock::new(|| {
    let mut map = load_history_files();
    apply_redirects(
        &mut map,
        REDIRECTS.iter().map(|(k, v)| (k.as_str(), v.as_str())),
    );
    map
});

pub fn git_history() -> &'static HashMap<PathBuf, HistoryEntry> {
    &GIT_HISTORY
}

fn load_history_files() -> HashMap<PathBuf, HistoryEntry> {
    let f = content_root().join("_git_history.json");
    let mut map = if let Ok(json_str) = fs::read_to_string(f) {
        serde_json::from_str(&json_str).expect("unable to parse l10n json")
    } else {
        HashMap::new()
    };
    if let Some(translated_root) = content_translated_root() {
        let f = translated_root.join("_git_history.json");
        if let Ok(json_str) = fs::read_to_string(f) {
            let translated: HashMap<PathBuf, HistoryEntry> =
                serde_json::from_str(&json_str).expect("unable to parse l10n json");
            map.extend(translated);
        };
    }
    map
}

/// Propagate history entries from redirected paths to their current paths.
///
/// For each `(old_url, new_url)` pair, if the corresponding old file path has
/// a history entry and the new path doesn't, insert the entry under the new
/// path. Entries that already exist at the new path (because git itself
/// recorded the move) are preserved.
fn apply_redirects<'a>(
    map: &mut HashMap<PathBuf, HistoryEntry>,
    redirects: impl Iterator<Item = (&'a str, &'a str)>,
) {
    for (old_url, new_url) in redirects {
        let Some(old_path) = url_to_history_key(old_url) else {
            continue;
        };
        let Some(new_path) = url_to_history_key(new_url) else {
            continue;
        };
        if old_path == new_path || map.contains_key(&new_path) {
            continue;
        }
        if let Some(entry) = map.get(&old_path).cloned() {
            map.insert(new_path, entry);
        }
    }
}

/// Convert a doc URL into the relative path used as a `_git_history.json` key,
/// e.g. `/fr/docs/Web/HTTP/Status/303` → `fr/web/http/status/303/index.md`.
fn url_to_history_key(url: &str) -> Option<PathBuf> {
    let UrlMeta {
        folder_path,
        locale,
        ..
    } = url_meta_from(url).ok()?;
    Some(
        PathBuf::from(locale.as_folder_str())
            .join(folder_path)
            .join("index.md"),
    )
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    fn entry(year: i32, hash: &str) -> HistoryEntry {
        HistoryEntry {
            modified: NaiveDate::from_ymd_opt(year, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            hash: hash.to_string(),
        }
    }

    #[test]
    fn propagates_entry_from_old_to_new_path() {
        let old: PathBuf = "fr/web/http/status/303/index.md".into();
        let new: PathBuf = "fr/web/http/reference/status/303/index.md".into();
        let mut map = HashMap::from([(old.clone(), entry(2024, "deadbeef"))]);

        apply_redirects(
            &mut map,
            std::iter::once((
                "/fr/docs/Web/HTTP/Status/303",
                "/fr/docs/Web/HTTP/Reference/Status/303",
            )),
        );

        assert_eq!(map.get(&new), Some(&entry(2024, "deadbeef")));
        // Old entry is left in place; lookups happen via the new key.
        assert!(map.contains_key(&old));
    }

    #[test]
    fn does_not_overwrite_existing_new_path_entry() {
        let old: PathBuf = "fr/a/index.md".into();
        let new: PathBuf = "fr/b/index.md".into();
        let mut map = HashMap::from([
            (old.clone(), entry(2020, "old-hash")),
            (new.clone(), entry(2024, "new-hash")),
        ]);

        apply_redirects(&mut map, std::iter::once(("/fr/docs/A", "/fr/docs/B")));

        assert_eq!(map.get(&new), Some(&entry(2024, "new-hash")));
    }

    #[test]
    fn skips_redirect_when_old_path_has_no_history() {
        let new: PathBuf = "fr/b/index.md".into();
        let mut map: HashMap<PathBuf, HistoryEntry> = HashMap::new();

        apply_redirects(&mut map, std::iter::once(("/fr/docs/A", "/fr/docs/B")));

        assert!(!map.contains_key(&new));
    }

    #[test]
    fn skips_self_redirect() {
        // Pure case-only URL renames produce equal lowercase paths.
        let path: PathBuf = "fr/web/api/example/index.md".into();
        let mut map = HashMap::from([(path.clone(), entry(2022, "abc"))]);

        apply_redirects(
            &mut map,
            std::iter::once(("/fr/docs/Web/api/Example", "/fr/docs/Web/API/Example")),
        );

        assert_eq!(map.len(), 1);
        assert_eq!(map.get(&path), Some(&entry(2022, "abc")));
    }

    #[test]
    fn skips_unparseable_urls() {
        let mut map: HashMap<PathBuf, HistoryEntry> = HashMap::new();

        apply_redirects(
            &mut map,
            std::iter::once(("not-a-url", "https://example.com/external")),
        );

        assert!(map.is_empty());
    }
}
