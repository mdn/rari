use std::borrow::Cow;
use std::collections::HashMap;

use rari_doc::error::DocError;
use rari_doc::pages::page::{Page, PageLike};
use rari_doc::pages::types::doc::Doc;
use rari_doc::reader::read_docs_parallel;
use rari_types::globals::{content_root, content_translated_root};
use rari_types::locale::Locale;

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
