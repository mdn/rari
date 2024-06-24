use std::fs::{self, File};
use std::io::BufWriter;
use std::iter::once;

use rari_types::globals::build_out_root;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tracing::{error, span, Level};

use crate::cached_readers::{blog_files, curriculum_files};
use crate::docs::build::{build_blog_post, build_curriculum, build_doc, build_dummy};
use crate::docs::dummy::Dummy;
use crate::docs::page::{Page, PageLike};
use crate::error::DocError;
use crate::resolve::url_to_path_buf;

pub fn build_single_page(page: &Page) {
    let slug = &page.slug();
    let locale = page.locale();
    let span = span!(Level::ERROR, "ctx", "{}:{}", locale, slug);
    let _enter = span.enter();
    let built_page = match page {
        Page::Doc(doc) => build_doc(doc),
        Page::BlogPost(post) => build_blog_post(post),
        Page::Dummy(dummy) => build_dummy(dummy),
        Page::Curriculum(curriculum) => build_curriculum(curriculum),
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
        }
        Err(e) => {
            error!("Error: {e}");
        }
    }
}

pub fn build_docs(docs: Vec<Page>) -> Result<(), DocError> {
    docs.into_par_iter()
        .for_each(|page| build_single_page(&page));
    Ok(())
}

pub fn build_curriculum_pages() -> Result<(), DocError> {
    curriculum_files()
        .by_path
        .iter()
        .for_each(|(_, page)| build_single_page(page));
    Ok(())
}

pub fn build_blog_pages() -> Result<(), DocError> {
    blog_files()
        .posts
        .values()
        .chain(once(&Dummy::from_url("/en-US/blog/").unwrap()))
        .for_each(build_single_page);
    Ok(())
}
