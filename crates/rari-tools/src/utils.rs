use std::borrow::Cow;

use crate::error::ToolError;

pub(crate) fn parent_slug(slug: &str) -> Result<&str, ToolError> {
    let slug = slug.trim_end_matches('/');
    if let Some(i) = slug.rfind('/') {
        Ok(&slug[..i])
    } else {
        Err(ToolError::InvalidSlug(Cow::Borrowed("slug has no parent")))
    }
}

#[cfg(test)]
use std::path::Path;
#[cfg(test)]
pub(crate) fn check_file_existence(root: &Path, should_exist: &[&str], should_not_exist: &[&str]) {
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
