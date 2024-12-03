//! # Build Module
//!
//! The `build` module provides functionality for building pages. The module leverages parallel
//! processing for documentzation pages to improve the efficiency of building large sets files.

use std::borrow::Cow;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::iter::once;
use std::path::PathBuf;

use chrono::NaiveDateTime;
use rari_types::globals::{build_out_root, git_history};
use rari_types::locale::Locale;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use sha2::{Digest, Sha256};
use tracing::{span, Level};

use crate::cached_readers::{
    blog_files, contributor_spotlight_files, curriculum_files, generic_content_files,
    wiki_histories,
};
use crate::contributors::contributors_txt;
use crate::error::DocError;
use crate::pages::build::copy_additional_files;
use crate::pages::json::BuiltPage;
use crate::pages::page::{Page, PageBuilder, PageLike};
use crate::pages::types::spa::SPA;
use crate::resolve::url_to_folder_path;

#[derive(Clone, Debug, Default)]
pub struct SitemapMeta<'a> {
    pub url: Cow<'a, str>,
    pub modified: Option<NaiveDateTime>,
    pub locale: Locale,
}

/// Builds a single documentation page and writes the output to a JSON file.
///
/// This function takes a `Page` object, builds the page, and writes the resulting content
/// to a JSON file in the output directory. It also copies additional files from the source
/// directory to the output directory, excluding the markdown source file. The function uses
/// tracing to create a `span` holding context and also log errors.
///
/// # Arguments
///
/// * `page` - A reference to the `Page` object to be built.
///
/// # Panics
///
/// This function will panic if:
/// - The `BUILD_OUT_ROOT` environment variable is not set.
/// - An error occurs while creating the output directory or file.
/// - An error occurs while writing the JSON content to the file.
/// - An error occurs while copying additional files.
pub fn build_single_page(page: &Page) -> Result<(), DocError> {
    let file = page.full_path().to_string_lossy();
    let span = span!(
        Level::ERROR,
        "page",
        locale = page.locale().as_url_str(),
        slug = page.slug(),
        file = file.as_ref()
    );
    let _enter = span.enter();
    let built_page = page.build()?;
    let out_path = build_out_root()
        .expect("No BUILD_OUT_ROOT")
        .join(url_to_folder_path(page.url().trim_start_matches('/')));
    fs::create_dir_all(&out_path)?;
    let out_file = out_path.join("index.json");
    let file = File::create(out_file).unwrap();
    let mut buffed = BufWriter::new(file);

    if let BuiltPage::Doc(json) = built_page {
        let json_str = serde_json::to_string(&json)?;
        buffed.write_all(json_str.as_bytes())?;
        let hash = Sha256::digest(json_str.as_bytes());
        let meta_out_file = out_path.join("metadata.json");
        let meta_file = File::create(meta_out_file).unwrap();
        let meta_buffed = BufWriter::new(meta_file);
        serde_json::to_writer(meta_buffed, &json.doc.as_meta(format!("{hash:x}")))?;
        let wiki_histories = wiki_histories();
        let wiki_history = wiki_histories
            .get(&page.locale())
            .and_then(|wh| wh.get(page.slug()));
        let github_file_url = json.doc.source.github_url.as_str();
        let contributors_txt_str = contributors_txt(wiki_history, github_file_url);
        let contributors_out_file = out_path.join("contributors.txt");
        let contributors_file = File::create(contributors_out_file).unwrap();
        let mut contributors_buffed = BufWriter::new(contributors_file);
        contributors_buffed.write_all(contributors_txt_str.as_bytes())?;
    } else {
        serde_json::to_writer(buffed, &built_page)?;
    }

    if let Some(in_path) = page.full_path().parent() {
        copy_additional_files(in_path, &out_path, page.full_path())?;
    }
    Ok(())
}

/// Builds a collection of documentation pages and returns their URLs.
///
/// This function takes a slice of `Page` objects, builds each page in parallel using the `build_single_page` function,
/// and collects the URLs of the built pages into a vector. The function leverages parallel processing to improve
/// the efficiency of building large sets of documentation files.
///
/// # Arguments
///
/// * `docs` - A slice of `Page` objects representing the documentation pages to be built.
///
/// # Returns
///
/// * `Result<Vec<SitemapMeta<'a>>, DocError>` - Returns a vector of `SitemapMeta` containing the URLs, Locales and
///    optionally the modification time of the built pages if successful, or a `DocError` if an error occurs during
///    the process.
///
/// # Errors
///
/// This function will return an error if:
/// - An error occurs while building any of the documentation pages.
pub fn build_docs<'a, 'b: 'a>(docs: &'b [Page]) -> Result<Vec<SitemapMeta<'a>>, DocError> {
    docs.into_par_iter()
        .map(|page| {
            let history = git_history().get(page.path());
            let modified = history.map(|entry| entry.modified);
            build_single_page(page).map(|_| SitemapMeta {
                url: Cow::Borrowed(page.url()),
                locale: page.locale(),
                modified,
            })
        })
        .collect()
}

/// Builds curriculum pages and returns their URLs.
///
/// This function retrieves the (cached) curriculum pages, builds each page using the `build_single_page` function,
/// and collects the URLs of the built pages into a vector.
///
/// # Returns
///
/// * `Result<Vec<SitemapMeta<'a>>, DocError>` - Returns a vector of `SitemapMeta` containing the URLs, Locales and
///    optionally the modification time of the built pages if successful, or a `DocError` if an error occurs during
///    the process.
///
/// # Errors
///
/// This function will return an error if:
/// - An error occurs while building any of the curriculum pages.
pub fn build_curriculum_pages<'a>() -> Result<Vec<SitemapMeta<'a>>, DocError> {
    curriculum_files()
        .by_path
        .values()
        .map(|page| {
            build_single_page(page).map(|_| SitemapMeta {
                url: Cow::Owned(page.url().to_string()),
                locale: page.locale(),
                ..Default::default()
            })
        })
        .collect()
}

fn copy_blog_author_avatars() -> Result<(), DocError> {
    for (slug, author) in &blog_files().authors {
        if let Some(avatar) = &author.frontmatter.avatar {
            let out_path = build_out_root()?.join(
                [
                    Locale::default().as_folder_str(),
                    "blog",
                    "author",
                    slug.as_str(),
                ]
                .iter()
                .collect::<PathBuf>(),
            );

            fs::create_dir_all(&out_path)?;
            let from = author.path.with_file_name(avatar);
            let to = out_path.join(avatar);
            fs::copy(from, to)?;
        }
    }
    Ok(())
}

/// Builds blog pages and returns their URLs.
///
/// This function first copies blog author avatar images as referenced in the blog files' frontmatter `authors` field
/// if available. It then retrieves the (cached) blog pages and the SPA for the blog, builds each page using the
/// `build_single_page` function, and collects the URLs of the built pages into a vector.
///
/// # Returns
///
/// * `Result<Vec<SitemapMeta<'a>>, DocError>` - Returns a vector of `SitemapMeta` containing the URLs, Locales and
///    optionally the modification time of the built pages if successful, or a `DocError` if an error occurs during
///    the process.
///
/// # Errors
///
/// This function will return an error if:
/// - An error occurs while copying blog author avatars.
/// - An error occurs while building any of the blog pages.
pub fn build_blog_pages<'a>() -> Result<Vec<SitemapMeta<'a>>, DocError> {
    copy_blog_author_avatars()?;
    blog_files()
        .posts
        .values()
        .chain(once(&SPA::from_url("/en-US/blog/").unwrap()))
        .map(|page| {
            build_single_page(page).map(|_| SitemapMeta {
                url: Cow::Owned(page.url().to_string()),
                locale: page.locale(),
                ..Default::default()
            })
        })
        .collect()
}

/// Builds generic pages and returns their URLs.
///
/// This function retrieves the cached generic pages, builds each page using the `build_single_page` function,
/// and collects the URLs of the built pages into a vector.
///
/// # Returns
///
/// * `Result<Vec<SitemapMeta<'a>>, DocError>` - Returns a vector of `SitemapMeta` containing the URLs, Locales and
///    optionally the modification time of the built pages if successful, or a `DocError` if an error occurs during
///    the process.
///
/// # Errors
///
/// This function will return an error if:
/// - An error occurs while building any of the generic pages.
pub fn build_generic_pages<'a>() -> Result<Vec<SitemapMeta<'a>>, DocError> {
    generic_content_files()
        .values()
        .map(|page| {
            build_single_page(page).map(|_| SitemapMeta {
                url: Cow::Owned(page.url().to_string()),
                locale: page.locale(),
                ..Default::default()
            })
        })
        .collect()
}

/// Builds contributor spotlight pages and returns their URLs.
///
/// This function retrieves the cached contributor spotlight pages, builds each page using the `build_single_page`
/// function, and collects the URLs of the built pages into a vector.
///
/// # Returns
///
/// * `Result<Vec<SitemapMeta<'a>>, DocError>` - Returns a vector of `SitemapMeta` containing the URLs, Locales and
///    optionally the modification time of the built pages if successful, or a `DocError` if an error occurs during
///    the process.
///
/// # Errors
///
/// This function will return an error if:
/// - An error occurs while building any of the contributor spotlight pages.
pub fn build_contributor_spotlight_pages<'a>() -> Result<Vec<SitemapMeta<'a>>, DocError> {
    contributor_spotlight_files()
        .values()
        .map(|page| {
            build_single_page(page).map(|_| SitemapMeta {
                url: Cow::Owned(page.url().to_string()),
                locale: page.locale(),
                ..Default::default()
            })
        })
        .collect()
}

/// Builds single-page applications (SPAs) and returns their URLs.
///
/// This function retrieves all SPAs, builds each SPA using the `build_single_page` function,
/// and collects the URLs of the built SPAs into a vector.
///
/// # Returns
///
/// * `Result<Vec<SitemapMeta<'a>>, DocError>` - Returns a vector of `SitemapMeta` containing the URLs, Locales and
///    optionally the modification time of the built pages if successful, or a `DocError` if an error occurs during
///    the process.
///
/// # Errors
///
/// This function will return an error if:
/// - An error occurs while building any of the SPAs.
pub fn build_spas<'a>() -> Result<Vec<SitemapMeta<'a>>, DocError> {
    SPA::all()
        .iter()
        .filter_map(|(slug, locale)| SPA::from_slug(slug, *locale))
        .map(|page| {
            build_single_page(&page).map(|_| SitemapMeta {
                url: Cow::Owned(page.url().to_string()),
                locale: page.locale(),
                ..Default::default()
            })
        })
        .collect()
}
