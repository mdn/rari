use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use rari_types::HistoryEntry;
use rari_types::globals::{content_root, translated_content_roots};
use rari_utils::git::exec_git;

use crate::error::ToolError;

pub fn gather_history() -> Result<(), ToolError> {
    modification_times(content_root())?;
    for root in translated_content_roots() {
        modification_times(root)?;
    }
    Ok(())
}

fn modification_times(path: &Path) -> Result<(), ToolError> {
    let output = exec_git(&["rev-parse", "--show-toplevel"], path);

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

            if let Some(data) = data.get(2)
                && let Some(parent_hash) = data.split(' ').nth(1)
            {
                parents.insert(parent_hash, HistoryEntry::new(date, hash));
            }
        } else if line.ends_with("index.md")
            && let Ok(rel_path) = PathBuf::from(line).strip_prefix("files")
        {
            history.insert(rel_path.to_path_buf(), HistoryEntry::new(date, hash));
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
