use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use ignore::types::TypesBuilder;
use rari_types::globals::{content_root, content_translated_root, settings};
use rari_types::locale::{Locale, LocaleFilter};

use crate::cached_readers::translated_locale_paths;
use crate::error::DocError;

/// Creates a `WalkBuilder` for walking through the specified paths globbing "index.md" files. The glob can be overridden.
///
/// This function initializes a `WalkBuilder` for traversing the file system starting from the given paths.
/// It supports optional glob patterns to filter the files being walked, but by default, it looks for "index.md" files.
/// For running in the test environment, the function also configures the `WalkBuilder` to respect or ignore `.gitignore`
/// files based on the settings `reader_ignores_gitignore`. Testing usually does not follow `.gitignore` files' rules.
///
/// # Arguments
///
/// * `paths` - A slice of paths to start the walk from. Each path should implement `AsRef<Path>`.
/// * `glob` - An optional string slice that holds the glob pattern to filter the files. If `None`, defaults to "index.md".
///
/// # Returns
///
/// * `Result<WalkBuilder, ignore::Error>` - Returns a configured `WalkBuilder` if successful,
///   or an `ignore::Error` if an error occurs while building the file types.
///
/// # Errors
///
/// This function will return an error if:
/// - An error occurs while adding the glob pattern to the `TypesBuilder`.
/// - An error occurs while building the file types.
pub(crate) fn walk_builder(
    paths: &[impl AsRef<Path>],
    glob: Option<&str>,
) -> Result<WalkBuilder, ignore::Error> {
    let mut types = TypesBuilder::new();
    types.add_def(&format!("markdown:{}", glob.unwrap_or("index.md")))?;
    types.select("markdown");
    let mut paths_iter = paths.iter();
    let mut builder = if let Some(path) = paths_iter.next() {
        let mut builder = ignore::WalkBuilder::new(path);
        for path in paths_iter {
            builder.add(path);
        }
        builder
    } else {
        let mut builder = ignore::WalkBuilder::new(content_root());
        if let Some(root) = content_translated_root() {
            builder.add(root);
        }
        builder
    };
    builder.git_ignore(!settings().reader_ignores_gitignore);
    builder.types(types.build()?);
    Ok(builder)
}

/// Walks `content_root()` (and `content_translated_root()` when configured)
/// and returns the paths of every `index.md` whose raw bytes contain
/// `needle` as an ASCII case-insensitive substring.
///
/// `filter` restricts the walk to the selected locales:
/// - `LocaleFilter::All` walks every configured root.
/// - `LocaleFilter::Only(locales)` walks `content_root` only if `en-US`
///   is in the set, plus one translated subdirectory per non-en-US locale.
///
/// Uses the same `markdown:index.md` type filter and `.gitignore` handling
/// as [`walk_builder`]. Files that cannot be read are logged and skipped.
///
/// Callers must validate `filter` upstream: passing
/// `LocaleFilter::Only(&[non-en-US])` without a configured
/// `content_translated_root` produces an empty path set and falls
/// through to [`walk_builder`]'s default (en-US only).
pub fn grep_doc_files(needle: &str, filter: LocaleFilter<'_>) -> Result<Vec<PathBuf>, DocError> {
    match filter {
        LocaleFilter::All => grep_doc_files_in(&[] as &[&Path], needle),
        LocaleFilter::Only(locales) => {
            let mut paths: Vec<PathBuf> = Vec::new();
            if locales.contains(&Locale::EnUs) {
                paths.push(content_root().to_path_buf());
            }
            if let Some(translated_root) = content_translated_root() {
                paths.extend(translated_locale_paths(translated_root, filter));
            }
            grep_doc_files_in(&paths, needle)
        }
    }
}

/// Inner walker shared by [`grep_doc_files`] and tests. Returns every
/// `index.md` under `paths` (or [`walk_builder`]'s default roots if `paths`
/// is empty) whose raw bytes contain `needle` as an ASCII case-insensitive
/// substring. An empty `needle` returns an empty result without walking.
pub(crate) fn grep_doc_files_in(
    paths: &[impl AsRef<Path>],
    needle: &str,
) -> Result<Vec<PathBuf>, DocError> {
    if needle.is_empty() {
        return Ok(Vec::new());
    }
    let needle_lower = needle.to_ascii_lowercase();
    let finder = memchr::memmem::Finder::new(needle_lower.as_bytes());
    let (tx, rx) = crossbeam_channel::bounded::<PathBuf>(100);
    let collector = std::thread::spawn(move || rx.into_iter().collect::<Vec<_>>());
    walk_builder(paths, None)?.build_parallel().run(|| {
        let tx = tx.clone();
        let finder = finder.clone();
        Box::new(move |result| {
            let entry = match result {
                Ok(entry) => entry,
                Err(e) => {
                    tracing::warn!("walker error during --grep: {e}");
                    return ignore::WalkState::Continue;
                }
            };
            if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                let path = entry.into_path();
                match std::fs::read(&path) {
                    Ok(mut bytes) => {
                        bytes.make_ascii_lowercase();
                        if finder.find(&bytes).is_some() {
                            let _ = tx.send(path);
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            file = %path.display(),
                            "failed to read for --grep: {e}",
                        );
                    }
                }
            }
            ignore::WalkState::Continue
        })
    });
    drop(tx);
    Ok(collector.join().unwrap())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn write(dir: &Path, rel: &str, contents: &[u8]) -> PathBuf {
        let path = dir.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn empty_needle_returns_empty() {
        let result = grep_doc_files_in(&[] as &[&Path], "").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn ascii_case_folding_matches_mixed_case() {
        let dir = tempfile::tempdir().unwrap();
        let upper = write(dir.path(), "a/index.md", b"contains SVGRef token");
        let lower = write(dir.path(), "b/index.md", b"contains svgref token");
        let _miss = write(dir.path(), "c/index.md", b"no match here");

        let mut matches = grep_doc_files_in(&[dir.path()], "svgref").unwrap();
        matches.sort();
        let mut expected = vec![upper, lower];
        expected.sort();
        assert_eq!(matches, expected);

        let mut matches = grep_doc_files_in(&[dir.path()], "SVGREF").unwrap();
        matches.sort();
        assert_eq!(matches, expected);
    }

    #[test]
    fn non_ascii_needle_matches_exact_utf8_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let lowercase = write(dir.path(), "a/index.md", "café au lait".as_bytes());
        let _uppercase = write(dir.path(), "b/index.md", "CAFÉ au lait".as_bytes());

        let matches = grep_doc_files_in(&[dir.path()], "café").unwrap();
        assert_eq!(matches, vec![lowercase]);
    }
}
