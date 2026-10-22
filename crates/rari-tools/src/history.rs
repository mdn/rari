use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::thread::spawn;

use rari_doc::resolve::url_meta_from;
use rari_types::HistoryEntry;
use rari_types::globals::{content_root, content_translated_root};
use rari_types::locale::Locale;
use rari_utils::git::exec_git;

use crate::error::ToolError;
use crate::utils::get_redirects_map;

pub fn gather_history() -> Result<(), ToolError> {
    let handle = content_translated_root().map(|translated_root| {
        let locales = locales_in_root(translated_root);
        spawn(move || {
            modification_times(translated_root, &locales).unwrap();
        })
    });
    modification_times(content_root(), &[Locale::EnUs])?;
    if let Some(handle) = handle {
        handle.join().expect("Unable to join history thread.");
    }
    Ok(())
}

fn modification_times(path: &Path, locales: &[Locale]) -> Result<(), ToolError> {
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
    let mut history = history
        .into_iter()
        .map(|(k, v)| {
            if let Some(parent) = parents.get(&&*v.hash).cloned() {
                (k, parent)
            } else {
                (k, v)
            }
        })
        .collect::<BTreeMap<PathBuf, HistoryEntry>>();

    let redirects = locales
        .iter()
        .flat_map(|locale| get_redirects_map(*locale))
        .collect::<Vec<_>>();
    loop {
        let mut changed = false;
        for (old_url, new_url) in &redirects {
            let (Some(old_path), Some(new_path)) = (history_key(old_url), history_key(new_url))
            else {
                continue;
            };
            if old_path != new_path
                && !history.contains_key(&new_path)
                && let Some(entry) = history.get(&old_path).cloned()
            {
                history.insert(new_path, entry);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    let out_file = path.join("_git_history.json");
    let file = File::create(out_file).unwrap();
    let buffed = BufWriter::new(file);

    serde_json::to_writer_pretty(buffed, &history).unwrap();
    Ok(())
}

fn history_key(url: &str) -> Option<PathBuf> {
    let meta = url_meta_from(url).ok()?;
    Some(
        PathBuf::from(meta.locale.as_folder_str())
            .join(meta.folder_path)
            .join("index.md"),
    )
}

fn locales_in_root(root: &Path) -> Vec<Locale> {
    root.read_dir()
        .expect("unable to read content root")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| Locale::from_str(entry.file_name().to_str()?).ok())
        .collect()
}
