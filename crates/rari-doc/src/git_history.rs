//! # Git History Module
//!
//! Loads `_git_history.json` artifacts (produced by `rari git-history`) for the
//! content and translated-content roots and exposes them as a single map keyed
//! by the relative file path used elsewhere in the build pipeline.
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;

use rari_types::HistoryEntry;
use rari_types::globals::{content_root, content_translated_root};

static GIT_HISTORY: LazyLock<HashMap<PathBuf, HistoryEntry>> = LazyLock::new(|| {
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
});

pub fn git_history() -> &'static HashMap<PathBuf, HistoryEntry> {
    &GIT_HISTORY
}
