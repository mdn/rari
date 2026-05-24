//! # Reader Module
//!
//! The `reader` module provides functionality for reading and processing documentation pages
//! in parallel. It includes utilities for walking through directories, reading files, and
//! collecting the resulting pages. The module leverages parallel processing to improve the
//! efficiency of reading large sets of documentation files.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use rari_types::globals::settings;
use tracing::{Level, error, span};

use crate::error::DocError;
use crate::pages::page::PageReader;
use crate::walker::walk_builder;

/// Reads documentation pages in parallel from the specified paths and collects them into a vector.
///
/// This function walks through the given paths, reads documentation pages in parallel, and collects
/// the resulting pages into a vector. It leverages parallel processing to improve the efficiency of
/// reading large sets of documentation files. The function respects `.gitignore` files by default,
/// but this behavior can be configured through the `reader_ignores_gitignore` setting for testing.
///
/// # Type Parameters
///
/// * `T` - The type of the page reader. Must implement the `PageReader` trait.
///
/// # Arguments
///
/// * `paths` - A slice of paths to start the walk from. Each path should implement `AsRef<Path>`.
/// * `glob` - An optional string slice that holds the glob pattern to filter the files. If `None`, defaults to "index.md".
/// * `ignore_dirs` - Directory names to exclude at the root of each walk path. Only top-level
///   directories with these names are excluded; deeper directories with the same name are kept.
///   Pass `None` to exclude nothing.
///
/// # Returns
///
/// * `Result<Vec<Page>, DocError>` - Returns a vector of `Page` objects if successful,
///   or a `DocError` if an error occurs during the process.
///
/// # Errors
///
/// This function will return an error if:
/// - An error occurs while building the walk builder.
/// - An error occurs while reading a page.
pub fn read_docs_parallel<P: 'static + Send, T: PageReader<P>>(
    paths: &[impl AsRef<Path>],
    glob: Option<&str>,
    ignore_dirs: Option<&[&str]>,
) -> Result<Vec<P>, DocError> {
    let (tx, rx) = crossbeam_channel::bounded::<Result<P, DocError>>(100);
    let stdout_thread = std::thread::spawn(move || rx.into_iter().collect());
    // For testing, we do not pay attention to the .gitignore files (walk_builder's
    // default is to obey them). The test configuration has `reader_ignores_gitignore = true`.
    let success = AtomicBool::new(true);
    let ignore_gitignore = !settings().reader_ignores_gitignore;
    let mut builder = walk_builder(paths, glob)?;
    builder.git_ignore(ignore_gitignore);
    if let Some(dirs) = ignore_dirs {
        let excluded: Vec<std::path::PathBuf> = paths
            .iter()
            .flat_map(|path| dirs.iter().map(move |dir| path.as_ref().join(dir)))
            .collect();
        builder.filter_entry(move |entry| !excluded.iter().any(|ex| ex == entry.path()));
    }
    builder.build_parallel().run(|| {
        let tx = tx.clone();
        let success = &success;
        Box::new(move |result| {
            if let Ok(f) = result
                && f.file_type().map(|ft| ft.is_file()).unwrap_or(false)
            {
                let span = span!(
                    Level::ERROR,
                    "page",
                    file = f.path().to_string_lossy().as_ref(),
                );
                let _enter = span.enter();
                let p = f.into_path();
                match T::read(&p, None) {
                    Ok(doc) => {
                        if let Err(e) = tx.send(Ok(doc)) {
                            error!("{e}");
                        }
                    }
                    Err(e) => {
                        error!("{e}");
                        success.store(false, Ordering::Relaxed);
                        //tx.send(Err(e.into())).unwrap();
                    }
                }
            }
            ignore::WalkState::Continue
        })
    });

    drop(tx);
    let docs = stdout_thread.join().unwrap();
    if success.load(Ordering::Relaxed) {
        docs
    } else {
        Err(DocError::DocsReadError)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rari_types::locale::Locale;

    use super::*;

    struct PathReader;
    impl PageReader<PathBuf> for PathReader {
        fn read(path: impl Into<PathBuf>, _locale: Option<Locale>) -> Result<PathBuf, DocError> {
            Ok(path.into())
        }
    }

    fn create_file(path: &std::path::Path) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "").unwrap();
    }

    fn read_paths(roots: &[&std::path::Path], ignore_dirs: Option<&[&str]>) -> Vec<PathBuf> {
        let mut paths =
            read_docs_parallel::<PathBuf, PathReader>(roots, None, ignore_dirs).unwrap();
        paths.sort();
        paths
    }

    #[test]
    fn test_ignore_dirs_excludes_root_directory() {
        let tmp = tempfile::TempDir::new().unwrap();
        let doc = tmp.path().join("en-us/web/index.md");
        let template = tmp.path().join("templates/test/index.md");
        for f in [&doc, &template] {
            create_file(f);
        }

        assert_eq!(read_paths(&[tmp.path()], Some(&["templates"])), vec![doc]);
    }

    #[test]
    fn test_ignore_dirs_does_not_exclude_nested_directory() {
        let tmp = tempfile::TempDir::new().unwrap();
        let doc = tmp.path().join("en-us/web/index.md");
        let nested = tmp.path().join("en-us/web/templates/index.md");
        for f in [&doc, &nested] {
            create_file(f);
        }

        assert_eq!(
            read_paths(&[tmp.path()], Some(&["templates"])),
            vec![doc, nested]
        );
    }

    #[test]
    fn test_ignore_dirs_excludes_across_multiple_paths() {
        let tmp1 = tempfile::TempDir::new().unwrap();
        let tmp2 = tempfile::TempDir::new().unwrap();
        let doc1 = tmp1.path().join("en-us/web/index.md");
        let template = tmp1.path().join("templates/test/index.md");
        let doc2 = tmp2.path().join("fr/web/index.md");
        let ignored = tmp2.path().join("ignored/test/index.md");
        for f in [&doc1, &template, &doc2, &ignored] {
            create_file(f);
        }

        let mut expected = vec![doc1, doc2];
        expected.sort();
        assert_eq!(
            read_paths(&[tmp1.path(), tmp2.path()], Some(&["templates", "ignored"])),
            expected
        );
    }

    #[test]
    fn test_none_ignore_dirs_includes_all() {
        let tmp = tempfile::TempDir::new().unwrap();
        let doc = tmp.path().join("en-us/web/index.md");
        let template = tmp.path().join("templates/test/index.md");
        for f in [&doc, &template] {
            create_file(f);
        }

        assert_eq!(read_paths(&[tmp.path()], None), vec![doc, template]);
    }
}
