use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use rari_types::globals::{data_dir, deps};
use serde::Deserialize;

use crate::error::DepsError;
use crate::npm::get_package;

#[derive(Debug, Clone, Deserialize)]
pub struct EventTarget {
    pub target: String,
    pub bubbles: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventEntry {
    #[serde(rename = "type")]
    pub event_type: String,
    pub cancelable: Option<bool>,
    #[serde(default)]
    pub targets: Vec<EventTarget>,
}

/// Webref event entries by event type.
pub type EventIndex = HashMap<String, Vec<EventEntry>>;

static EVENTS_INDEX: OnceLock<Option<EventIndex>> = OnceLock::new();

pub fn update_webref_events(base_path: &Path) -> Result<(), DepsError> {
    get_package("@webref/events", &deps().webref_events, base_path)?;
    Ok(())
}

/// Returns the webref event index, downloading the package first if missing.
///
/// Returns `None` (after one warning) if the data can neither be downloaded nor read.
pub fn events_index() -> Option<&'static EventIndex> {
    EVENTS_INDEX
        .get_or_init(|| {
            load_events_index(data_dir(), update_webref_events)
                .inspect_err(|e| {
                    tracing::warn!(
                        source = "webref-events-load",
                        "Failed to load @webref/events data: {e}"
                    );
                })
                .ok()
        })
        .as_ref()
}

pub fn parse_index(json: &str) -> Result<EventIndex, serde_json::Error> {
    let entries: Vec<EventEntry> = serde_json::from_str(json)?;
    let mut index = EventIndex::new();
    for entry in entries {
        index
            .entry(entry.event_type.clone())
            .or_default()
            .push(entry);
    }
    Ok(index)
}

fn load_events_index(
    data_dir: &Path,
    update: impl FnOnce(&Path) -> Result<(), DepsError>,
) -> Result<EventIndex, String> {
    let package_dir = data_dir.join("@webref/events");
    let path = package_dir.join("package/events.json");
    if !path.exists() {
        // A fresh `last_check.json` makes `get_package` skip the download, so drop it.
        let _ = fs::remove_file(package_dir.join("last_check.json"));
        update(data_dir).map_err(|e| format!("download failed: {e}"))?;
    }
    let json = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    parse_index(&json).map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(test)]
mod test {
    use super::*;

    const JSON: &str = r#"[{"type":"a","cancelable":true,"targets":[{"target":"X","bubbles":false}]},{"type":"a","targets":[]}]"#;

    #[test]
    fn test_load_events_index() {
        struct Case {
            name: &'static str,
            cached: bool,
            update_writes: bool,
            expect_ok: bool,
            expected_updates: usize,
        }
        let cases = [
            Case {
                name: "cached skips update",
                cached: true,
                update_writes: false,
                expect_ok: true,
                expected_updates: 0,
            },
            Case {
                name: "missing triggers update",
                cached: false,
                update_writes: true,
                expect_ok: true,
                expected_updates: 1,
            },
            Case {
                name: "failed download falls back to error",
                cached: false,
                update_writes: false,
                expect_ok: false,
                expected_updates: 1,
            },
        ];
        for case in cases {
            let dir = tempfile::tempdir().unwrap();
            let package = dir.path().join("@webref/events/package");
            let path = package.join("events.json");
            if case.cached {
                fs::create_dir_all(&package).unwrap();
                fs::write(&path, JSON).unwrap();
            }
            let mut updates = 0;
            let result = load_events_index(dir.path(), |_| {
                updates += 1;
                if case.update_writes {
                    fs::create_dir_all(&package)?;
                    fs::write(&path, JSON)?;
                }
                Ok(())
            });
            assert_eq!(updates, case.expected_updates, "{}", case.name);
            assert_eq!(result.is_ok(), case.expect_ok, "{}", case.name);
            if let Ok(index) = result {
                assert_eq!(index["a"].len(), 2, "{}", case.name);
                assert_eq!(index["a"][0].cancelable, Some(true), "{}", case.name);
            }
        }
    }
}
