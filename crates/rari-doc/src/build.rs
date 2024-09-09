use std::borrow::Cow;
use std::fs::{self, File};
use std::io::BufWriter;
use std::iter::once;

use rari_types::globals::build_out_root;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tracing::{error, span, Level};

use crate::cached_readers::{blog_files, curriculum_files, generic_pages_files};
use crate::error::DocError;
use crate::pages::build::{
    build_blog_post, build_contributor_spotlight, build_curriculum, build_doc, build_generic_page,
    build_spa, copy_additional_files,
};
use crate::pages::page::{Page, PageLike};
use crate::pages::types::spa::SPA;
use crate::resolve::url_to_path_buf;

pub fn build_single_page(page: &Page) {
    let slug = &page.slug();
    let locale = page.locale();
    let span = span!(Level::ERROR, "page", "{}:{}", locale, slug);
    let _enter = span.enter();
    let built_page = match page {
        Page::Doc(doc) => build_doc(doc),
        Page::BlogPost(post) => build_blog_post(post),
        Page::SPA(spa) => build_spa(spa),
        Page::Curriculum(curriculum) => build_curriculum(curriculum),
        Page::ContributorSpotlight(cs) => build_contributor_spotlight(cs),
        Page::GenericPage(generic) => build_generic_page(generic),
    };
    match built_page {
        Ok(built_page) => {
            let out_path = build_out_root()
                .expect("No BUILD_OUT_ROOT")
                .join(url_to_path_buf(page.url().trim_start_matches('/')));
            fs::create_dir_all(&out_path).unwrap();
            let out_file = out_path.join("index.json");
            let file = File::create(out_file).unwrap();
            let buffed = BufWriter::new(file);

            serde_json::to_writer(buffed, &built_page).unwrap();

            if let Some(in_path) = page.full_path().parent() {
                copy_additional_files(in_path, &out_path).unwrap();
            }
        }
        Err(e) => {
            error!("Error: {e}");
        }
    }
}

pub fn build_docs(docs: &[Page]) -> Result<Vec<Cow<'_, str>>, DocError> {
    Ok(docs
        .into_par_iter()
        .map(|page| {
            build_single_page(page);
            Cow::Borrowed(page.url())
        })
        .collect())
}

pub fn build_curriculum_pages() -> Result<Vec<Cow<'static, str>>, DocError> {
    Ok(curriculum_files()
        .by_path
        .values()
        .map(|page| {
            build_single_page(page);
            Cow::Owned(page.url().to_string())
        })
        .collect())
}

pub fn build_blog_pages() -> Result<Vec<Cow<'static, str>>, DocError> {
    Ok(blog_files()
        .posts
        .values()
        .chain(once(&SPA::from_url("/en-US/blog/").unwrap()))
        .map(|page| {
            build_single_page(page);
            Cow::Owned(page.url().to_string())
        })
        .collect())
}

pub fn build_generic_pages() -> Result<Vec<Cow<'static, str>>, DocError> {
    Ok(generic_pages_files()
        .values()
        .map(|page| {
            build_single_page(page);
            Cow::Owned(page.url().to_string())
        })
        .collect())
}
