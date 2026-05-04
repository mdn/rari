use std::fs;
use std::path::PathBuf;

use indoc::formatdoc;
use rari_types::globals::blog_root;

pub(crate) struct BlogFixtures {
    slugs: Vec<String>,
    do_not_remove: bool,
}

impl BlogFixtures {
    pub fn new(slugs: &[String]) -> Self {
        Self::new_internal(slugs, false)
    }

    #[allow(dead_code)]
    pub fn debug_new(slugs: &[String]) -> Self {
        Self::new_internal(slugs, true)
    }

    fn new_internal(slugs: &[String], do_not_remove: bool) -> Self {
        for slug in slugs {
            Self::create_post_file(slug);
        }
        BlogFixtures {
            slugs: slugs.to_vec(),
            do_not_remove,
        }
    }

    fn posts_root() -> PathBuf {
        blog_root()
            .expect("BLOG_ROOT must be set for blog fixtures")
            .join("posts")
    }

    fn create_post_file(slug: &str) {
        let folder = Self::posts_root().join(slug);
        fs::create_dir_all(&folder).unwrap();
        let content = formatdoc! {
            r#"---
            title: {slug}
            slug: {slug}
            date: 2024-01-01
            author: test-author
            ---

            Test content for {slug}.
            "#,
            slug = slug,
        };
        fs::write(folder.join("index.md"), content).unwrap();
    }
}

impl Drop for BlogFixtures {
    fn drop(&mut self) {
        if self.do_not_remove {
            tracing::info!("Leaving blog fixtures in place for debugging");
            return;
        }
        for slug in &self.slugs {
            let folder = Self::posts_root().join(slug);
            if folder.exists() {
                fs::remove_dir_all(&folder).unwrap();
            }
        }
    }
}
