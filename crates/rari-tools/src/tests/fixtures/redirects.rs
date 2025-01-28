use std::fs;
use std::path::PathBuf;

use rari_doc::utils::root_for_locale;
use rari_types::locale::Locale;

pub(crate) struct RedirectFixtures {
    path: PathBuf,
    do_not_remove: bool,
}

impl RedirectFixtures {
    pub fn new(entries: &Vec<(String, String)>, locale: Locale) -> Self {
        Self::new_internal(entries, locale, false)
    }
    #[allow(dead_code)]
    pub fn debug_new(entries: &Vec<(String, String)>, locale: Locale) -> Self {
        Self::new_internal(entries, locale, true)
    }

    fn new_internal(entries: &Vec<(String, String)>, locale: Locale, do_not_remove: bool) -> Self {
        // create wiki history file for each slug in the vector, in the configured root directory for the locale
        let mut folder_path = PathBuf::new();
        folder_path.push(root_for_locale(locale).unwrap());
        folder_path.push(locale.as_folder_str());
        fs::create_dir_all(&folder_path).unwrap();
        folder_path.push("_redirects.txt");

        let mut content = String::new();
        for (from, to) in entries {
            content.push_str(
                format!(
                    "/{}/{}\t/{}/{}\n",
                    locale.as_url_str(),
                    from,
                    locale.as_url_str(),
                    to
                )
                .as_str(),
            );
        }
        content.push('\n');

        fs::write(&folder_path, content).unwrap();

        RedirectFixtures {
            path: folder_path,
            do_not_remove,
        }
    }
}

impl Drop for RedirectFixtures {
    fn drop(&mut self) {
        if self.do_not_remove {
            tracing::info!(
                "Leaving redirects fixture {} in place for debugging",
                self.path.display()
            );
            return;
        }

        fs::remove_file(&self.path).unwrap();
    }
}
