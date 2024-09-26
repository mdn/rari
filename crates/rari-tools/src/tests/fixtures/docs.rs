use rari_doc::{
    pages::page::PageCategory,
    resolve::{build_url, url_path_to_path_buf},
    utils::root_for_locale,
};
use rari_types::locale::Locale;
use std::path::PathBuf;

pub(crate) struct DocFixtures {
    files: Vec<String>,
}

impl DocFixtures {
    pub fn new(slugs: &Vec<String>, locale: &Locale) -> Self {
        // create doc file for each slug in the vector, in the configured root directory for the locale
        // Get the root directory from the locale

        // Iterate over each slug and create a file in the root directory
        for slug in slugs {
            let mut folder_path = PathBuf::new();
            folder_path.push(locale.as_folder_str());
            let url = build_url(slug, &locale, PageCategory::Doc).unwrap();
            let (path, _, _, _) = url_path_to_path_buf(&url).unwrap();
            folder_path.push(path);
            let path = root_for_locale(*locale).unwrap().join(folder_path);
            println!("Creating file: {:?}", path);
            // match File::create(&file_path) {
            //     Ok(_) => println!("Created file: {:?}", file_path),
            //     Err(e) => eprintln!("Failed to create file: {:?}, error: {}", file_path, e),
            // }
        }

        DocFixtures { files: vec![] }
    }
    // add helper methods go here
}

impl Drop for DocFixtures {
    fn drop(&mut self) {
        // Perform cleanup actions
        println!("Cleaned up doc fixtures.");
    }
}
