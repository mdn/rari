use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use ignore::types::TypesBuilder;
use rari_types::globals::{content_root, content_translated_root, settings};

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
/// Uses the same `markdown:index.md` type filter and `.gitignore` handling
/// as [`walk_builder`]. Files that cannot be read are logged and skipped.
/// An empty `needle` returns an empty result without walking.
pub fn grep_doc_files(needle: &str) -> Result<Vec<PathBuf>, DocError> {
    if needle.is_empty() {
        return Ok(Vec::new());
    }
    let needle_bytes = needle.as_bytes();
    let paths: Vec<&Path> = if let Some(translated) = content_translated_root() {
        vec![content_root(), translated]
    } else {
        vec![content_root()]
    };
    let (tx, rx) = crossbeam_channel::bounded::<PathBuf>(100);
    let collector = std::thread::spawn(move || rx.into_iter().collect::<Vec<_>>());
    walk_builder(&paths, None)?.build_parallel().run(|| {
        let tx = tx.clone();
        Box::new(move |result| {
            if let Ok(entry) = result
                && entry.file_type().map(|ft| ft.is_file()).unwrap_or(false)
            {
                let path = entry.into_path();
                match std::fs::read(&path) {
                    Ok(bytes) => {
                        if bytes
                            .windows(needle_bytes.len())
                            .any(|w| w.eq_ignore_ascii_case(needle_bytes))
                        {
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
