use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use ignore::types::TypesBuilder;
use ignore::WalkBuilder;
use rari_doc::error::DocError;
use rari_doc::pages::page::{Page, PageLike};
use rari_doc::pages::types::doc::Doc;
use rari_doc::reader::read_docs_parallel;
use rari_types::globals::{content_root, content_translated_root, settings};
use rari_types::locale::Locale;
use tracing::error;

use crate::error::ToolError;
use crate::redirects::{self, redirects_path};

pub(crate) fn parent_slug(slug: &str) -> Result<&str, ToolError> {
    let slug = slug.trim_end_matches('/');
    if let Some(i) = slug.rfind('/') {
        Ok(&slug[..i])
    } else {
        Err(ToolError::InvalidSlug(Cow::Borrowed("slug has no parent")))
    }
}

/// Read all en-US and translated documents into a hash, with a key of `(locale, slug)`.
/// This is similar to the `cached_reader` functionality, but not wrapped in a `onceLock`.
pub(crate) fn read_all_doc_pages() -> Result<HashMap<(Locale, Cow<'static, str>), Page>, DocError> {
    let docs = read_docs_parallel::<Doc>(&[content_root()], None)?;
    let mut docs_hash: HashMap<(Locale, Cow<'_, str>), Page> = docs
        .iter()
        .cloned()
        .map(|doc| ((doc.locale(), Cow::Owned(doc.slug().to_string())), doc))
        .collect();

    if let Some(translated_root) = content_translated_root() {
        let translated_docs = read_docs_parallel::<Doc>(&[translated_root], None)?;
        docs_hash.extend(
            translated_docs
                .iter()
                .cloned()
                .map(|doc| ((doc.locale(), Cow::Owned(doc.slug().to_string())), doc)),
        )
    }
    Ok(docs_hash)
}

// read raw file contents into strings
pub(crate) fn read_files_parallel(
    paths: &[impl AsRef<Path>],
) -> Result<Vec<(String, String)>, DocError> {
    let (tx, rx) = crossbeam_channel::bounded::<Result<(String, String), DocError>>(100);
    let stdout_thread = std::thread::spawn(move || rx.into_iter().collect());
    let ignore_gitignore = !settings().reader_ignores_gitignore;
    md_walk_builder(paths)?
        .git_ignore(ignore_gitignore)
        .build_parallel()
        .run(|| {
            let tx = tx.clone();
            Box::new(move |result| {
                if let Ok(f) = result {
                    if f.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                        let p = f.into_path();

                        match fs::read_to_string(&p) {
                            Ok(doc) => {
                                tx.send(Ok((p.to_string_lossy().to_string(), doc))).unwrap();
                            }
                            Err(e) => {
                                error!("{e}");
                            }
                        }
                    }
                }
                ignore::WalkState::Continue
            })
        });

    drop(tx);
    stdout_thread.join().unwrap()
}

pub(crate) fn md_walk_builder(paths: &[impl AsRef<Path>]) -> Result<WalkBuilder, ignore::Error> {
    let mut types = TypesBuilder::new();
    types.add_def(&format!("markdown:{}", "index.md"))?;
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

pub(crate) fn get_redirects_map(locale: Locale) -> HashMap<String, String> {
    let redirects_path = redirects_path(locale).unwrap();
    let mut redirects = HashMap::new();
    redirects::read_redirects_raw(&redirects_path, &mut redirects).unwrap();
    redirects
}

#[cfg(test)]
pub mod test_utils {
    use std::path::Path;

    pub(crate) fn check_file_existence(
        root: &Path,
        should_exist: &[&str],
        should_not_exist: &[&str],
    ) {
        use std::path::PathBuf;

        for relative_path in should_exist {
            let parts = relative_path.split('/').collect::<Vec<&str>>();
            let mut path = PathBuf::from(root);
            for part in parts {
                path.push(part);
            }
            assert!(path.exists(), "{} should exist", path.display());
        }

        for relative_path in should_not_exist {
            let parts = relative_path.split('/').collect::<Vec<&str>>();
            let mut path = PathBuf::from(root);
            for part in parts {
                path.push(part);
            }
            assert!(!path.exists(), "{} should not exist", path.display());
        }
    }
}
