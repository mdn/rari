//! # Reader Module
//!
//! The `reader` module provides functionality for reading and processing documentation pages
//! in parallel. It includes utilities for walking through directories, reading files, and
//! collecting the resulting pages. The module leverages parallel processing to improve the
//! efficiency of reading large sets of documentation files.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use rari_types::globals::settings;
use tracing::error;

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
) -> Result<Vec<P>, DocError> {
    let (tx, rx) = crossbeam_channel::bounded::<Result<P, DocError>>(100);
    let stdout_thread = std::thread::spawn(move || rx.into_iter().collect());
    // For testing, we do not pay attention to the .gitignore files (walk_builder's
    // default is to obey them). The test configuration has `reader_ignores_gitignore = true`.
    let success = AtomicBool::new(true);
    let ignore_gitignore = !settings().reader_ignores_gitignore;
    walk_builder(paths, glob)?
        .git_ignore(ignore_gitignore)
        .build_parallel()
        .run(|| {
            let tx = tx.clone();
            let success = &success;
            Box::new(move |result| {
                if let Ok(f) = result
                    && f.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                        let p = f.into_path();
                        match T::read(p, None) {
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
