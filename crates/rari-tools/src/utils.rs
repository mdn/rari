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
pub mod test_utils {
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    use rari_doc::utils::root_for_locale;
    use rari_types::locale::Locale;

    use crate::redirects;
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

    pub(crate) fn get_redirects_map(locale: Locale) -> HashMap<String, String> {
        let root_path = root_for_locale(locale).unwrap();

        let mut redirects_path = PathBuf::from(root_path);
        redirects_path.push(locale.as_folder_str());
        redirects_path.push("_redirects.txt");
        let mut redirects = HashMap::new();
        redirects::read_redirects_raw(&redirects_path, &mut redirects).unwrap();
        redirects
    }
}
