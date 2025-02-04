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
use crate::pages::page::{Page, PageLike};

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
                .filter(|dir| dir.path().is_dir())
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

/// Resolves a given URL to a redirect URL if one exists.
///
/// Takes a URL string as input and returns an Option containing either:
/// - A `Cow<str>` with the redirect URL if a redirect exists
/// - None if no redirect is found
///
/// The function handles hash fragments in URLs and preserves them in the redirect.
/// It also normalizes URLs and can resolve both explicit redirects from the redirects file
/// as well as implicit redirects based on page URL normalization.
pub fn resolve_redirect<'a>(url: impl AsRef<str>) -> Option<Cow<'a, str>> {
    let url = url.as_ref();
    let hash_index = url.find('#').unwrap_or(url.len());
    let (url_no_hash, hash) = (&url[..hash_index], &url[hash_index..]);
    let redirect = match REDIRECTS
        .get(&url_no_hash.to_lowercase())
        .map(|s| s.as_str())
    {
        Some(redirect) if redirect.starts_with("/") => Some(
            Page::from_url(redirect)
                .ok()
                .and_then(|page| {
                    if url != page.url() {
                        Some(Cow::Owned(page.url().to_string()))
                    } else {
                        None
                    }
                })
                .unwrap_or(Cow::Borrowed(redirect)),
        ),
        Some(redirect) => Some(Cow::Borrowed(redirect)),
        None if url.starts_with("/") => Page::from_url(url).ok().and_then(|page| {
            if url != page.url() {
                Some(Cow::Owned(page.url().to_string()))
            } else {
                None
            }
        }),
        None => None,
    };
    match (redirect, hash) {
        (None, _) => None,
        (Some(url), hash) if url.contains('#') || hash.is_empty() => Some(url),
        (Some(url), hash) => Some(Cow::Owned(format!("{url}{hash}"))),
    }
}
