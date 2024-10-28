//! # Redirects Module
//!
//! The `redirects` module provides functionality for managing URL redirects.
//! It includes utilities for reading redirect mappings from files and storing them in a hashmap for efficient lookup.
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::str::FromStr;
use std::sync::LazyLock;

use rari_types::globals::{content_root, content_translated_root};
use rari_types::locale::Locale;
use rari_utils::error::RariIoError;
use tracing::error;

use crate::error::DocError;

static REDIRECTS: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    if let Some(ctr) = content_translated_root() {
        for locale in ctr
            .read_dir()
            .expect("unable to read translated content root")
            .filter_map(|dir| {
                dir.map_err(|e| {
                    error!("Error: reading translated content root: {e}");
                })
                .ok()
                .and_then(|dir| {
                    Locale::from_str(
                        dir.file_name()
                            .as_os_str()
                            .to_str()
                            .expect("invalid folder"),
                    )
                    .map_err(|e| error!("Invalid folder {:?}: {e}", dir.file_name()))
                    .ok()
                })
            })
        {
            if let Err(e) = read_redirects(
                &ctr.to_path_buf()
                    .join(locale.as_folder_str())
                    .join("_redirects.txt"),
                &mut map,
            ) {
                error!("Error reading redirects: {e}");
            }
        }
    }
    if let Err(e) = read_redirects(
        &content_root()
            .to_path_buf()
            .join(Locale::EnUs.as_folder_str())
            .join("_redirects.txt"),
        &mut map,
    ) {
        error!("Error reading redirects: {e}");
    }
    map
});

fn read_redirects(path: &Path, map: &mut HashMap<String, String>) -> Result<(), DocError> {
    let lines = read_lines(path)?;
    map.extend(lines.map_while(Result::ok).filter_map(|line| {
        if line.starts_with('#') {
            return None;
        }
        let mut from_to = line.splitn(2, '\t');
        if let (Some(from), Some(to)) = (from_to.next(), from_to.next()) {
            Some((from.to_lowercase(), to.into()))
        } else {
            None
        }
    }));
    Ok(())
}

fn read_lines<P>(filename: P) -> Result<io::Lines<io::BufReader<File>>, RariIoError>
where
    P: AsRef<Path>,
{
    let file = File::open(filename.as_ref()).map_err(|e| RariIoError {
        source: e,
        path: filename.as_ref().to_path_buf(),
    })?;
    Ok(io::BufReader::new(file).lines())
}

/// Resolves a URL redirect based on the configured redirects.
///
/// This function attempts to resolve a redirect for the given URL by looking it up in the `REDIRECTS` hashmap.
/// If a redirect is found, it returns the target URL, optionally appending any hash fragment from the original URL.
/// If no redirect is found, it returns `None`.
///
/// # Arguments
///
/// * `url` - A string slice that holds the URL to be resolved.
///
/// # Returns
///
/// * `Option<Cow<'_, str>>` - Returns `Some(Cow::Borrowed(target_url))` if a redirect is found and the target URL
///   does not contain a hash fragment, or `Some(Cow::Owned(format!("{target_url}{hash}")))` if the target URL
///   contains a hash fragment or the original URL has a hash fragment. Returns `None` if no redirect is found.
pub(crate) fn resolve_redirect(url: &str) -> Option<Cow<'_, str>> {
    let hash_index = url.find('#').unwrap_or(url.len());
    let (url_no_hash, hash) = (&url[..hash_index], &url[hash_index..]);
    match (
        REDIRECTS
            .get(&url_no_hash.to_lowercase())
            .map(|s| s.as_str()),
        hash,
    ) {
        (None, _) => None,
        (Some(url), hash) if url.contains('#') || hash.is_empty() => Some(Cow::Borrowed(url)),
        (Some(url), hash) => Some(Cow::Owned(format!("{url}{hash}"))),
    }
}
