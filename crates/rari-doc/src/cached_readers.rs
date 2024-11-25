//! # Cached Readers Module
//!
//! The `cached_readers` module provides functionality for managing and accessing cached data
//! related to various types of pages. This includes documentation pages, blog posts, curriculum pages,
//! generic pages, contributor spotlights, and more. The module utilizes various caching mechanisms
//! to improve the efficiency of reading and processing documentation content.
//!
//! ## Key Components
//!
//! - **Static Caches**: These caches store pre-loaded documentation pages and are initialized once.
//!   - `STATIC_DOC_PAGE_FILES`: Stores documentation pages indexed by locale and URL.
//!   - `STATIC_DOC_PAGE_TRANSLATED_FILES`: Stores translated documentation pages indexed by locale and URL.
//!   - `STATIC_DOC_PAGE_FILES_BY_PATH`: Stores documentation pages indexed by file path.
//!
//! - **Dynamic Caches**: These caches store documentation pages that can be modified during runtime.
//!   - `CACHED_DOC_PAGE_FILES`: Stores documentation pages indexed by file path.
//!   - `CACHED_SIDEBAR_FILES`: Stores sidebar metadata indexed by name and locale.
//!
//! - **Specialized Caches**: These caches store specific types of documentation content.
//!   - `CACHED_CURRICULUM`: Stores curriculum files, indexed by URL, path, and index,
//!   - `GENERIC_CONTENT_FILES`: Stores generic pages indexed by URL.
//!   - `CONTRIBUTOR_SPOTLIGHT_FILES`: Stores contributor spotlight pages indexed by URL.

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, LazyLock, OnceLock};

use dashmap::DashMap;
use rari_types::globals::{
    blog_root, cache_content, content_root, content_translated_root, contributor_spotlight_root,
    curriculum_root, generic_content_root,
};
use rari_types::locale::Locale;
use rari_utils::concat_strs;
use rari_utils::io::read_to_string;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::contributors::{WikiHistories, WikiHistory};
use crate::error::DocError;
use crate::html::sidebar::{MetaSidebar, Sidebar};
use crate::pages::page::{Page, PageLike};
use crate::pages::types::blog::{Author, AuthorFrontmatter, BlogPost, BlogPostBuildMeta};
use crate::pages::types::contributors::ContributorSpotlight;
use crate::pages::types::curriculum::{CurriculumIndexEntry, CurriculumPage};
use crate::pages::types::doc::Doc;
use crate::pages::types::generic::GenericPage;
use crate::reader::read_docs_parallel;
use crate::sidebars::jsref;
use crate::translations::init_translations_from_static_docs;
use crate::utils::split_fm;
use crate::walker::walk_builder;

pub(crate) static STATIC_DOC_PAGE_FILES: OnceLock<HashMap<(Locale, Cow<'_, str>), Page>> =
    OnceLock::new();
pub(crate) static STATIC_DOC_PAGE_TRANSLATED_FILES: OnceLock<
    HashMap<(Locale, Cow<'_, str>), Page>,
> = OnceLock::new();
pub(crate) static STATIC_DOC_PAGE_FILES_BY_PATH: OnceLock<HashMap<PathBuf, Page>> = OnceLock::new();
pub static CACHED_DOC_PAGE_FILES: OnceLock<Arc<DashMap<PathBuf, Page>>> = OnceLock::new();
type SidebarFilesCache = Arc<DashMap<(String, Locale), Arc<MetaSidebar>>>;
pub(crate) static CACHED_SIDEBAR_FILES: LazyLock<SidebarFilesCache> =
    LazyLock::new(|| Arc::new(DashMap::new()));
pub(crate) static CACHED_CURRICULUM: OnceLock<CurriculumFiles> = OnceLock::new();
pub(crate) static GENERIC_CONTENT_FILES: OnceLock<UrlToPageMap> = OnceLock::new();
pub(crate) static CONTRIBUTOR_SPOTLIGHT_FILES: OnceLock<UrlToPageMap> = OnceLock::new();
pub(crate) static WIKI_HISTORY: OnceLock<WikiHistories> = OnceLock::new();

/// Represents the cached files for blog posts.
///
/// The `BlogFiles` struct contains the cached data for blog posts, including the posts themselves,
/// the authors, and the sorted metadata for the blog posts. This is used to efficiently manage
/// and access blog-related data during the build process.
///
/// # Fields
///
/// * `posts` - A `HashMap<String, Page>` that holds the blog posts, where the key is the post identifier and
///   the value is the `Page` representing the blog post.
/// * `authors` - A `HashMap<String, Arc<Author>>` that holds the authors of the blog posts, where the key
///   is the author identifier and the value is an `Arc` pointing to the `Author`.
/// * `sorted_meta` - A `Vec<BlogPostBuildMeta>` that holds the metadata for the blog posts, sorted by `date`, `title`.
#[derive(Debug, Default, Clone)]
pub struct BlogFiles {
    pub posts: HashMap<String, Page>,
    pub authors: HashMap<String, Arc<Author>>,
    pub sorted_meta: Vec<BlogPostBuildMeta>,
}
pub(crate) static BLOG_FILES: OnceLock<BlogFiles> = OnceLock::new();

/// Represents the cached files for curriculum pages.
///
/// The `CurriculumFiles` struct contains the cached data for curriculum pages, including
/// mappings from URLs and file paths to pages, as well as an index of curriculum entries.
/// This is used to efficiently manage and access curriculum-related data during the build process.
///
/// # Fields
///
/// * `by_url` - A `HashMap<String, Page>` that holds the curriculum pages, where the key is the URL of the
///   page and the value is the `Page` representing the curriculum page.
/// * `by_path` - A `HashMap<PathBuf, Page>` that holds the curriculum pages, where the key is the file path
///   of the page and the value is the `Page` representing the curriculum page.
/// * `index` - A `Vec<CurriculumIndexEntry>` that holds the sorted index entries for the horizontal navigation
///   of curriculum pages (prev/next).
#[derive(Debug, Default, Clone)]
pub struct CurriculumFiles {
    pub by_url: HashMap<String, Page>,
    pub by_path: HashMap<PathBuf, Page>,
    pub index: Vec<CurriculumIndexEntry>,
}

/// Reads and returns a sidebar for the given name, locale, and slug.
///
/// This function attempts to read a sidebar based on the provided name, locale, and slug.
/// If the name is "jsref", it generates the sidebar using the `jsref::sidebar` function.
/// For other names, it constructs the file path to the sidebar YAML file, reads the file,
/// and parses it into a `Sidebar` object. The resulting sidebar is then cached for future use
/// if caching is enabled.
///
/// # Arguments
///
/// * `name` - A string slice that holds the name of the sidebar.
/// * `locale` - A `Locale` that specifies the locale of the sidebar.
/// * `slug` - A string slice that holds the slug of the sidebar.
///
/// # Returns
///
/// * `Result<Arc<MetaSidebar>, DocError>` - Returns an `Arc` containing the `MetaSidebar` if successful,
///   or a `DocError` if an error occurs while reading or parsing the sidebar.
///
/// # Errors
///
/// This function will return an error if:
/// - An error occurs while reading the sidebar file.
/// - An error occurs while parsing the sidebar YAML content.
pub fn read_sidebar(name: &str, locale: Locale, slug: &str) -> Result<Arc<MetaSidebar>, DocError> {
    let sidebar = match name {
        "jsref" => Arc::new(jsref::sidebar(slug, locale)?),
        _ => {
            let key = (name.to_string(), locale);
            if cache_content() {
                if let Some(sidebar) = CACHED_SIDEBAR_FILES.get(&key) {
                    return Ok(sidebar.clone());
                }
            }
            let mut file = content_root().to_path_buf();
            file.push("sidebars");
            file.push(name);
            file.set_extension("yaml");
            let raw = read_to_string(&file)?;
            let sidebar: Sidebar = serde_yaml_ng::from_str(&raw)?;
            let sidebar = Arc::new(MetaSidebar::try_from(sidebar)?);
            if cache_content() {
                CACHED_SIDEBAR_FILES.insert(key, sidebar.clone());
            }
            sidebar
        }
    };
    Ok(sidebar)
}

/// Retrieves a documentation page from the cache based on the given slug and locale.
///
/// This function attempts to retrieve a documentation page from the static cache using the provided slug and locale.
/// If the locale is `en-US`, it uses the `STATIC_DOC_PAGE_FILES` cache; otherwise, it uses the
/// `STATIC_DOC_PAGE_TRANSLATED_FILES` cache.
///
/// If the page is found in the cache, it returns `Ok(page)`.
/// If the page is not found, it returns a `DocError::NotFoundInStaticCache` error.
/// If there is aproblem with the cache, it returns a `DocError::FileCacheBroken` error.
///
/// # Arguments
///
/// * `slug` - A string slice that holds the slug of the documentation page.
/// * `locale` - A `Locale` that specifies the locale of the documentation page.
///
/// # Returns
///
/// * `Result<Page, DocError>` - Returns `Ok(Page)` if the page is found in the cache, or
///   `Err(DocError)` if the page is not found..
pub fn doc_page_from_slug(slug: &str, locale: Locale) -> Result<Page, DocError> {
    let cache = if locale == Locale::EnUs {
        &STATIC_DOC_PAGE_FILES
    } else {
        &STATIC_DOC_PAGE_TRANSLATED_FILES
    };
    cache.get().map_or_else(
        || Err(DocError::FileCacheBroken),
        |static_files| {
            if let Some(page) = static_files.get(&(locale, Cow::Borrowed(slug))) {
                return Ok(page.clone());
            }
            Err(DocError::NotFoundInStaticCache(concat_strs!(
                "/",
                locale.as_url_str(),
                "/docs/",
                slug
            )))
        },
    )
}

/// Retrieves a documentation page from the static cache based on the given file path.
///
/// This function attempts to retrieve a documentation page from the static cache using the provided file path.
/// If the cache is not available, it returns a `DocError::FileCacheBroken` error. If the page is found in the
/// cache, it returns the page wrapped in an `Ok` variant. If the page is not found, it returns a
/// `DocError::NotFoundInStaticCache` error.
///
/// # Arguments
///
/// * `path` - A reference to a `Path` that holds the file path of the documentation page.
///
/// # Returns
///
/// * `Result<Page, DocError>` - Returns `Ok(Page)` if the page is found in the cache,
///   or `Err(DocError)` if the page is not found or the cache is not available.
pub fn doc_page_from_static_files(path: &Path) -> Result<Page, DocError> {
    STATIC_DOC_PAGE_FILES_BY_PATH.get().map_or_else(
        || Err(DocError::FileCacheBroken),
        |static_files| {
            if let Some(page) = static_files.get(path) {
                return Ok(page.clone());
            }
            Err(DocError::NotFoundInStaticCache(
                path.to_string_lossy().to_string(),
            ))
        },
    )
}

fn gather_blog_posts() -> Result<HashMap<String, Page>, DocError> {
    if let Some(blog_root) = blog_root() {
        let post_root = blog_root.join("posts");
        Ok(read_docs_parallel::<BlogPost>(&[post_root], None)?
            .into_iter()
            .map(|page| (page.url().to_ascii_lowercase(), page))
            .collect())
    } else {
        Err(DocError::NoBlogRoot)
    }
}

fn gather_generic_content() -> Result<HashMap<String, Page>, DocError> {
    if let Some(root) = generic_content_root() {
        Ok(read_docs_parallel::<GenericPage>(&[root], Some("*.md"))?
            .into_iter()
            .filter_map(|page| {
                if let Page::GenericPage(generic) = page {
                    Some(generic)
                } else {
                    None
                }
            })
            .flat_map(|generic| {
                Locale::for_generic_and_spas()
                    .iter()
                    .map(|locale| Page::GenericPage(Arc::new(generic.as_locale(*locale))))
                    .collect::<Vec<_>>()
            })
            .map(|page| (page.url().to_ascii_lowercase(), page))
            .collect())
    } else {
        Err(DocError::NoGenericContentRoot)
    }
}

fn gather_curriculum() -> Result<CurriculumFiles, DocError> {
    if let Some(curriculum_root) = curriculum_root() {
        let curriculum_root = curriculum_root.join("curriculum");
        let pages: Vec<Page> =
            read_docs_parallel::<CurriculumPage>(&[curriculum_root], Some("*.md"))?
                .into_iter()
                .collect();
        let by_url: HashMap<String, Page> = pages
            .iter()
            .cloned()
            .map(|page| (page.url().to_ascii_lowercase(), page))
            .collect();
        let mut index: Vec<(PathBuf, CurriculumIndexEntry)> = pages
            .iter()
            .filter_map(|c| {
                if let Page::Curriculum(c) = c {
                    Some(c)
                } else {
                    None
                }
            })
            .map(|c| {
                (
                    c.full_path().to_path_buf(),
                    CurriculumIndexEntry {
                        url: c.url().to_string(),
                        title: c.title().to_string(),
                        slug: Some(c.slug().to_string()),
                        children: Vec::new(),
                        summary: c.meta.summary.clone(),
                        topic: c.meta.topic,
                    },
                )
            })
            .collect();
        index.sort_by(|a, b| a.0.cmp(&b.0));
        let index = index.into_iter().map(|(_, entry)| entry).collect();

        let by_path = pages
            .into_iter()
            .map(|page| (page.full_path().to_path_buf(), page))
            .collect();

        Ok(CurriculumFiles {
            by_url,
            by_path,
            index,
        })
    } else {
        Err(DocError::NoCurriculumRoot)
    }
}

fn gather_contributre_spotlight() -> Result<HashMap<String, Page>, DocError> {
    if let Some(root) = contributor_spotlight_root() {
        Ok(read_docs_parallel::<ContributorSpotlight>(&[root], None)?
            .into_iter()
            .filter_map(|page| {
                if let Page::ContributorSpotlight(cs) = page {
                    Some(cs)
                } else {
                    None
                }
            })
            .flat_map(|cs| {
                Locale::for_generic_and_spas()
                    .iter()
                    .map(|locale| Page::ContributorSpotlight(Arc::new(cs.as_locale(*locale))))
                    .collect::<Vec<_>>()
            })
            .map(|page| (page.url().to_ascii_lowercase(), page))
            .collect())
    } else {
        Err(DocError::NoGenericContentRoot)
    }
}

/// Retrieves the all curriculum pages. It uses the `CACHED_CURRICULUM` if caching is enabled.
///
/// This function returns a `Cow<'static, CurriculumFiles>` containing the curriculum pages maps.
/// If caching is enabled (as determined by `cache_content()`), the returned value is cached, otherwise
/// pages are read on every call.
///
/// # Returns
///
/// * `Cow<'static, CurriculumFiles>` - Returns a `Cow::Borrowed` containing the cached curriculum files
///   if caching is enabled. Otherwise, returns a `Cow::Owned` containing the gathered curriculum files.
pub fn curriculum_files() -> Cow<'static, CurriculumFiles> {
    if cache_content() {
        Cow::Borrowed(CACHED_CURRICULUM.get_or_init(|| {
            gather_curriculum()
                .map_err(|e| error!("{e}"))
                .ok()
                .unwrap_or_default()
        }))
    } else {
        Cow::Owned(
            gather_curriculum()
                .map_err(|e| error!("{e}"))
                .unwrap_or_default(),
        )
    }
}

fn gather_blog_authors() -> Result<HashMap<String, Arc<Author>>, DocError> {
    if let Some(blog_authors_path) = blog_root().map(|br| br.join("authors")) {
        Ok(walk_builder(&[blog_authors_path], None)?
            .build()
            .filter_map(|f| f.ok())
            .filter(|f| f.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .map(|f| {
                let path = f.into_path();
                let raw = read_to_string(&path)?;
                let (fm, _) = split_fm(&raw);
                let frontmatter: AuthorFrontmatter =
                    serde_yaml_ng::from_str(fm.unwrap_or_default())?;
                let name = path
                    .parent()
                    .and_then(|p| p.file_name())
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let author = Author { frontmatter, path };
                Ok((name, Arc::new(author)))
            })
            .collect::<Result<HashMap<String, Arc<Author>>, DocError>>()?)
    } else {
        Err(DocError::NoBlogRoot)
    }
}

/// Retrieves all blog pages.
///
/// This function returns a `Cow<'static, BlogFiles>` containing the blog pages maps.
/// If caching is enabled (as determined by `cache_content()`), the returned value is cached, otherwise
/// pages are read on every call.
///
/// # Returns
///
/// * `Cow<'static, BlogFiles>` - Returns a `Cow::Borrowed` containing the cached blog data
///   if caching is enabled. Otherwise, returns it in a `Cow::Owned`.
pub fn blog_files() -> Cow<'static, BlogFiles> {
    fn gather() -> BlogFiles {
        let posts = gather_blog_posts().unwrap_or_else(|e| {
            error!("{e}");
            Default::default()
        });
        let authors = gather_blog_authors().unwrap_or_else(|e| {
            error!("{e}");
            Default::default()
        });
        let mut sorted_meta = posts
            .values()
            .filter_map(|post| match post {
                Page::BlogPost(p) => Some(p.meta.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        sorted_meta.sort_by(|a, b| {
            if a.date != b.date {
                a.date.cmp(&b.date)
            } else {
                // TODO: use proper order
                b.title.cmp(&a.title)
            }
        });
        BlogFiles {
            posts,
            authors,
            sorted_meta,
        }
    }
    if cache_content() {
        Cow::Borrowed(BLOG_FILES.get_or_init(gather))
    } else {
        Cow::Owned(gather())
    }
}

/// Retrieves a blog author by their name from the (cached) blog files.
///
/// This function attempts to retrieve a blog author from the cached blog files using the provided name.
/// If the author is found, it returns `Some(Arc<Author>)` pointing to the `Author` struct.
/// If the author is not found, it returns `None`.
///
/// # Arguments
///
/// * `name` - A string slice that holds the name of the blog author to be retrieved.
///
/// # Returns
///
/// * `Option<Arc<Author>>` - Returns `Some(Arc<Author>)` if the author is found in the cache,
///   or `None` if the author is not found.
pub fn blog_author_by_name(name: &str) -> Option<Arc<Author>> {
    blog_files().authors.get(name).cloned()
}

/// Reads all documentation pages from the content root and translated content root directories, fills the
/// interanl cache structures and returns a vector of `Page` objects.
///
/// This function reads documentation pages in parallel from the content root directory and caches them
/// in the `STATIC_DOC_PAGE_FILES` static variable. If a translated content root directory is available,
/// it also reads translated documentation pages and caches them in the `STATIC_DOC_PAGE_TRANSLATED_FILES`
/// static variable. Additionally, it initializes `TRANSLATIONS_BY_SLUG` fills `STATIC_DOC_PAGE_FILES_BY_PATH`
/// static variable.
///
/// # Returns
///
/// * `Result<Vec<Page>, DocError>` - Returns a vector of `Page` objects if successful,
///   or a `DocError` if an error occurs while reading the documentation pages.
///
/// # Errors
///
/// This function will return an error if:
/// - An error occurs while reading the documentation pages from the content root or translated content root directories.
pub fn read_and_cache_doc_pages() -> Result<Vec<Page>, DocError> {
    let mut docs = read_docs_parallel::<Doc>(&[content_root()], None)?;
    STATIC_DOC_PAGE_FILES
        .set(
            docs.iter()
                .cloned()
                .map(|doc| ((doc.locale(), Cow::Owned(doc.slug().to_string())), doc))
                .collect(),
        )
        .unwrap();
    if let Some(translated_root) = content_translated_root() {
        let translated_docs = read_docs_parallel::<Doc>(&[translated_root], None)?;
        STATIC_DOC_PAGE_TRANSLATED_FILES
            .set(
                translated_docs
                    .iter()
                    .cloned()
                    .map(|doc| ((doc.locale(), Cow::Owned(doc.slug().to_string())), doc))
                    .collect(),
            )
            .unwrap();
        docs.extend(translated_docs)
    }
    init_translations_from_static_docs();
    STATIC_DOC_PAGE_FILES_BY_PATH
        .set(
            docs.iter()
                .cloned()
                .map(|doc| (doc.full_path().to_path_buf(), doc))
                .collect(),
        )
        .unwrap();
    Ok(docs)
}

/// A type alias for a hashmap that maps URLs to pages.
pub type UrlToPageMap = HashMap<String, Page>;

/// Represents the configuration for generic pages.
///
/// # Fields
///
/// * `slug_prefix` - The prefix for the slug, defaults to None.
///
/// * `title_suffix` - A suffix to add to the page title.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GenericPagesConfig {
    pub slug_prefix: Option<String>,
    pub title_suffix: Option<String>,
}
/// Represents the configuration for all generic content.
///
/// # Fields
///
/// * `pages` - A `HashMap<String, GenericPagesConfig>` mapping pages sections to their
///   according `GenericPagesConfig`.
///
/// * `title_suffix` - A suffix to add to the page title.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct GenericContentConfig {
    pub pages: HashMap<String, GenericPagesConfig>,
    pub spas: HashMap<String, BuildSPA>,
}

static GENERIC_CONTENT_CONFIG: OnceLock<GenericContentConfig> = OnceLock::new();

fn read_generic_content_config() -> Result<GenericContentConfig, DocError> {
    if let Some(root) = generic_content_root() {
        let json_str = read_to_string(root.join("config.json"))?;
        let config: GenericContentConfig = serde_json::from_str(&json_str)?;
        Ok(config)
    } else {
        Err(DocError::NoGenericContentConfig)
    }
}

/// Provides access to the generic content configuration.
///
/// This function returns the generic content configuration, either from a
/// cached, lazily-initialized global value (`GENERIC_CONTENT_CONFIG`) or by
/// re-reading the configuration file, depending on the value of `cache_content()`.
///
/// - If `cache_content()` is true, the configuration is cached and re-used
///   across calls.
/// - If `cache_content()` is false, the configuration is re-read from the
///   `config.json` file on each call.
///
/// Any errors encountered during the reading or parsing of the configuration
/// are logged, and a default configuration is returned.
///
/// # Returns
/// A `Cow<'static, GenericContentConfig>` representing the configuration.
/// - If the configuration is cached, a borrowed reference is returned.
/// - If the configuration is re-read, an owned copy is returned.
pub fn generic_content_config() -> Cow<'static, GenericContentConfig> {
    fn gather() -> GenericContentConfig {
        read_generic_content_config().unwrap_or_else(|e| {
            error!("{e}");
            Default::default()
        })
    }
    if cache_content() {
        Cow::Borrowed(GENERIC_CONTENT_CONFIG.get_or_init(gather))
    } else {
        Cow::Owned(gather())
    }
}

/// Retrieves all generic pages, using the cache if it is enabled.
///
/// This function returns a `Cow<'static, UrlToPageMap>` containing the generic pages.
/// If caching is enabled (as determined by `cache_content()`), it attempts to get the cached
/// generic pages from `GENERIC_CONTENT_FILES`, initializing it if needed.
/// If caching is not enabled, it directly reads the generic pages and returns them.
///
/// # Returns
///
/// * `Cow<'static, UrlToPageMap>` - Returns a `Cow::Borrowed` containing the cached generic pages
///   if caching is enabled. Otherwise, returns a `Cow::Owned` containing the read-in generic pages.
pub fn generic_content_files() -> Cow<'static, UrlToPageMap> {
    fn gather() -> UrlToPageMap {
        gather_generic_content().unwrap_or_else(|e| {
            error!("{e}");
            Default::default()
        })
    }
    if cache_content() {
        Cow::Borrowed(GENERIC_CONTENT_FILES.get_or_init(gather))
    } else {
        Cow::Owned(gather())
    }
}

/// Retrieves the contributor spotlight pages, using the cacche if it is enabled.
///
/// This function returns a `Cow<'static, UrlToPageMap>` containing the contributor spotlight pages.
/// If caching is enabled (as determined by `cache_content()`), it attempts to get the cached
/// contributor spotlight pages from `CONTRIBUTOR_SPOTLIGHT_FILES`, initializing it if needed.
/// If caching is not enabled, it directly reads the contributor spotlight pages and returns them.
///
/// # Returns
///
/// * `Cow<'static, UrlToPageMap>` - Returns a `Cow::Borrowed` containing the cached contributor spotlight pages
///   if caching is enabled. Otherwise, returns a `Cow::Owned` containing the read-in contributor spotlight pages.
pub fn contributor_spotlight_files() -> Cow<'static, UrlToPageMap> {
    fn gather() -> UrlToPageMap {
        gather_contributre_spotlight().unwrap_or_else(|e| {
            error!("{e}");
            Default::default()
        })
    }
    if cache_content() {
        Cow::Borrowed(CONTRIBUTOR_SPOTLIGHT_FILES.get_or_init(gather))
    } else {
        Cow::Owned(gather())
    }
}

/// Retrieves the wiki histories for all supported locales, including English (en-US).
///
/// This function gathers historical data about the MDN wiki pages from `_wikihistory.json` files
/// stored in content and translated-content. It returns a `Cow<'static, WikiHistories>` to allow
/// for efficient caching and re-use of data when possible.
///
/// ### Behavior
///
/// - **Caching**: If content caching is enabled (via `cache_content()`), the function retrieves
///   the wiki histories from a global, static cache (`WIKI_HISTORY`) or initializes the cache
///   if it hasn't been loaded yet.
/// - **Dynamic Loading**: If caching is disabled, the function reads the histories directly
///   from disk at runtime.
///
/// ### File Structure
///
/// - Each locale's wiki history is stored in a `_wikihistory.json` file, located in the
///   corresponding locale's directory under the translated content root.
/// - The English (en-US) wiki history is always loaded from the default content root.
///
/// ### Error Handling
///
/// - Errors encountered while reading directories or JSON files are logged, and the function
///   defaults to returning an empty `WikiHistories` map in these cases.
///
/// ### Returns
///
/// - A `Cow<'static, WikiHistories>`:
///   - If caching is enabled, a borrowed reference to the cached `WikiHistories` is returned.
///   - If caching is disabled, an owned instance of `WikiHistories` is returned.
///
/// ### Example
///
/// ```
/// let wiki_histories = wiki_histories();
/// if let Some(en_us_history) = wiki_histories.get(&Locale::EnUs) {
///     println!("Loaded en-US wiki history: {:?}", en_us_history);
/// } else {
///     println!("No en-US wiki history found.");
/// }
/// ```
///
/// ### Panics
///
/// - Panics if the translated content root or individual directory names are invalid.
/// - Panics if the `_wikihistory.json` file is missing or contains malformed JSON.
pub fn wiki_histories() -> Cow<'static, WikiHistories> {
    fn gather() -> Result<WikiHistories, DocError> {
        let mut map = HashMap::new();
        if let Some(ctr) = content_translated_root() {
            for locale in ctr
                .read_dir()
                .expect("unable to read translated content root")
                .filter_map(|dir| {
                    dir.map_err(|e| {
                        error!("Error: reading translated content root: {e}");
                    })
                    .ok()
                    .filter(|dir| dir.path().is_dir())
                    .and_then(|dir| {
                        Locale::from_str(
                            dir.file_name()
                                .as_os_str()
                                .to_str()
                                .expect("invalid folder"),
                        )
                        .map_err(|e| error!("Invalid folder {:?}: {e}", dir.file_name()))
                        .ok()
                    })
                })
            {
                let json_str =
                    read_to_string(ctr.join(locale.as_folder_str()).join("_wikihistory.json"))?;
                let history: WikiHistory = serde_json::from_str(&json_str)?;
                map.insert(locale, history);
            }
        }
        let json_str = read_to_string(
            content_root()
                .join(Locale::EnUs.as_folder_str())
                .join("_wikihistory.json"),
        )?;
        let history: WikiHistory = serde_json::from_str(&json_str)?;
        map.insert(Locale::EnUs, history);
        Ok(map)
    }
    if cache_content() {
        Cow::Borrowed(WIKI_HISTORY.get_or_init(|| {
            gather()
                .inspect_err(|e| tracing::error!("Error reading wiki histories: {e}"))
                .unwrap_or_default()
        }))
    } else {
        Cow::Owned(
            gather()
                .inspect_err(|e| tracing::error!("Error reading wiki histories: {e}"))
                .unwrap_or_default(),
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BasicSPA {
    pub only_follow: bool,
    pub no_indexing: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SPAData {
    BlogIndex,
    HomePage,
    NotFound,
    #[serde(untagged)]
    BasicSPA(BasicSPA),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildSPA {
    pub slug: Cow<'static, str>,
    pub page_title: Cow<'static, str>,
    pub page_description: Option<Cow<'static, str>>,
    pub trailing_slash: bool,
    pub en_us_only: bool,
    pub data: SPAData,
}
