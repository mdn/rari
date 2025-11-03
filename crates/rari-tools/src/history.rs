use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread::spawn;

use rari_types::HistoryEntry;
use rari_types::globals::{content_root, content_translated_root};

use crate::error::ToolError;
use crate::git::exec_git;

pub fn gather_history() -> Result<(), ToolError> {
    let handle = content_translated_root().map(|translated_root| {
        spawn(|| {
            modification_times(translated_root).unwrap();
        })
    });
    modification_times(content_root())?;
    if let Some(handle) = handle {
        handle.join().expect("Unable to join history thread.");
    }
    Ok(())
}

fn modification_times(path: &Path) -> Result<(), ToolError> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(path)
        .output()
        .expect("failed to execute git rev-parse");

    let repo_root_raw = String::from_utf8_lossy(&output.stdout);
    let repo_root = repo_root_raw.trim();

    let output = exec_git(
        &[
            "log",
            "--name-only",
            "--no-decorate",
            "--format=COMMIT:%H_%cI_%P",
            "--date-order",
            "--reverse",
            "-z",
        ],
        repo_root,
    );

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut history = BTreeMap::new();
    let mut parents = BTreeMap::new();
    let mut date = "";
    let mut hash = "";
    for line in output_str.split(['\0', '\n']) {
        if line.trim().is_empty() {
            continue;
        }
        if line.starts_with("COMMIT:") {
            let data: Vec<&str> = line
                .trim()
                .strip_prefix("COMMIT:")
                .unwrap_or(line)
                .split('_')
                .collect();
            if let [hash_data, date_data, ..] = data.as_slice() {
                hash = *hash_data;
                date = *date_data;
            }

            if let Some(data) = data.get(2) {
                if let Some(parent_hash) = data.split(' ').nth(1) {
                    parents.insert(parent_hash, HistoryEntry::new(date, hash));
                }
            }
        } else if line.ends_with("index.md") {
            if let Ok(rel_path) = PathBuf::from(line).strip_prefix("files") {
                history.insert(rel_path.to_path_buf(), HistoryEntry::new(date, hash));
            }
        }
    }

    // Replace merged commit dates with their parent date.
    let history = history
        .into_iter()
        .map(|(k, v)| {
            if let Some(parent) = parents.get(&&*v.hash).cloned() {
                (k, parent)
            } else {
                (k, v)
            }
        })
        .collect::<BTreeMap<PathBuf, HistoryEntry>>();

    let out_file = path.join("_git_history.json");
    let file = File::create(out_file).unwrap();
    let buffed = BufWriter::new(file);

    serde_json::to_writer_pretty(buffed, &history).unwrap();
    Ok(())
}
