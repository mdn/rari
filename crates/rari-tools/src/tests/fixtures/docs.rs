use fake::{faker::lorem::en::Paragraph, Fake};
use indoc::formatdoc;
use rari_doc::{
    pages::page::PageCategory,
    resolve::{build_url, url_meta_from, UrlMeta},
    utils::root_for_locale,
};
use rari_types::locale::Locale;
use std::{fs, path::PathBuf};

pub(crate) struct DocFixtures {
    // files: Vec<String>,
    locale: Locale,
    do_not_remove: bool,
}

impl DocFixtures {
    pub fn new(slugs: &Vec<String>, locale: &Locale) -> Self {
        Self::new_internal(slugs, locale, false)
    }

    #[allow(dead_code)]
    pub fn debug_new(slugs: &Vec<String>, locale: &Locale) -> Self {
        Self::new_internal(slugs, locale, true)
    }

    fn new_internal(slugs: &Vec<String>, locale: &Locale, do_not_remove: bool) -> Self {
        // create doc file for each slug in the vector, in the configured root directory for the locale

        // Iterate over each slug and create a file in the root directory
        let _files: Vec<String> = slugs
            .iter()
            .map(|slug| Self::create_doc_file(&slug, locale))
            .collect();

        DocFixtures {
            // files,
            locale: locale.clone(),
            do_not_remove,
        }
    }

    fn capitalize(s: &str) -> String {
        if s.is_empty() {
            return String::new();
        }
        let mut chars = s.chars();
        let first = chars.next().unwrap().to_uppercase().to_string();
        let rest: String = chars.collect::<String>();
        first + &rest
    }

    fn path_from_slug(slug: &str, locale: &Locale) -> PathBuf {
        let mut folder_path = PathBuf::new();
        folder_path.push(locale.as_folder_str());
        let url = build_url(slug, &locale, PageCategory::Doc).unwrap();
        let UrlMeta {
            folder_path: path, ..
        } = url_meta_from(&url).unwrap();
        folder_path.push(path);
        folder_path
    }

    fn create_doc_file(slug: &str, locale: &Locale) -> String {
        let slug_components = slug.split('/').collect::<Vec<&str>>();

        let mut current_slug = String::new();
        let locale_root = root_for_locale(*locale).unwrap();

        for slug_component in slug_components {
            current_slug.push_str(slug_component);

            let folder_path = Self::path_from_slug(current_slug.as_str(), locale);
            let abs_folder_path = locale_root.join(&folder_path);

            let title = Self::capitalize(current_slug.split("/").last().unwrap());
            let content = formatdoc! {
                r#"---
                title: {}
                slug: {}
                ---

                {}
                "#,
                title,
                current_slug,
                Paragraph(1..3).fake::<String>()
            };
            // first create the parent path
            fs::create_dir_all(&abs_folder_path).unwrap();
            let path = abs_folder_path.join("index.md");
            // overwrite file if it exists
            if fs::exists(&path).unwrap() {
                if path.is_dir() {
                    println!(
                        "File path is a directory - replacing with file: {}",
                        path.to_string_lossy()
                    );
                    fs::remove_dir_all(&path).unwrap();
                    fs::write(&path, content).unwrap();
                }
            } else {
                fs::write(&path, content).unwrap();
            }
            current_slug.push_str("/");
        }

        let path = locale_root
            .join(Self::path_from_slug(current_slug.as_str(), locale))
            .join("index.md");
        path.to_string_lossy().to_string()
    }
}

impl Drop for DocFixtures {
    fn drop(&mut self) {
        if self.do_not_remove {
            println!("Leaving doc fixtures in place for debugging");
            return;
        }
        // Perform cleanup actions, recursively remove all files
        // in the locale folder
        let path = root_for_locale(self.locale)
            .unwrap()
            .join(self.locale.as_folder_str());
        let entries = fs::read_dir(&path).unwrap();

        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                fs::remove_dir_all(&path).unwrap();
            } else {
                fs::remove_file(&path).unwrap();
            }
        }
        fs::remove_dir_all(&path).unwrap();
    }
}
