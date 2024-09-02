use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, OnceLock, RwLock};

use rari_types::globals::{
    blog_root, cache_content, content_root, content_translated_root, curriculum_root,
};
use rari_types::locale::Locale;
use rari_utils::io::read_to_string;
use tracing::error;

use crate::docs::blog::{Author, AuthorFrontmatter, BlogPost, BlogPostBuildMeta};
use crate::docs::curriculum::{CurriculumIndexEntry, CurriculumPage};
use crate::docs::doc::Doc;
use crate::docs::page::{Page, PageLike};
use crate::error::DocError;
use crate::html::sidebar::{MetaSidebar, Sidebar};
use crate::reader::read_docs_parallel;
use crate::sidebars::jsref;
use crate::translations::init_translations_from_static_docs;
use crate::utils::split_fm;
use crate::walker::walk_builder;

pub static STATIC_DOC_PAGE_FILES: OnceLock<HashMap<PathBuf, Page>> = OnceLock::new();
pub static STATIC_DOC_PAGE_TRANSLATED_FILES: OnceLock<HashMap<PathBuf, Page>> = OnceLock::new();
pub static CACHED_DOC_PAGE_FILES: OnceLock<Arc<RwLock<HashMap<PathBuf, Page>>>> = OnceLock::new();
type SidebarFilesCache = Arc<RwLock<HashMap<(String, Locale), Arc<MetaSidebar>>>>;
pub static CACHED_SIDEBAR_FILES: LazyLock<SidebarFilesCache> =
    LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));
pub static CACHED_CURRICULUM: OnceLock<CurriculumFiles> = OnceLock::new();

#[derive(Debug, Default, Clone)]
pub struct BlogFiles {
    pub posts: HashMap<String, Page>,
    pub authors: HashMap<String, Arc<Author>>,
    pub sorted_meta: Vec<BlogPostBuildMeta>,
}
pub static BLOG_FILES: OnceLock<BlogFiles> = OnceLock::new();

#[derive(Debug, Default, Clone)]
pub struct CurriculumFiles {
    pub by_url: HashMap<String, Page>,
    pub by_path: HashMap<PathBuf, Page>,
    pub index: Vec<CurriculumIndexEntry>,
}

pub fn read_sidebar(name: &str, locale: Locale, slug: &str) -> Result<Arc<MetaSidebar>, DocError> {
    let sidebar = match name {
        "jsref" => Arc::new(jsref::sidebar(slug, locale)?),
        _ => {
            let key = (name.to_string(), locale);
            if cache_content() {
                if let Some(sidebar) = CACHED_SIDEBAR_FILES.read()?.get(&key) {
                    return Ok(sidebar.clone());
                }
            }
            let mut file = content_root().to_path_buf();
            file.push("sidebars");
            file.push(name);
            file.set_extension("yaml");
            let raw = read_to_string(&file)?;
            let sidebar: Sidebar = serde_yaml::from_str(&raw)?;
            let sidebar = Arc::new(MetaSidebar::from(sidebar));
            if cache_content() {
                CACHED_SIDEBAR_FILES.write()?.insert(key, sidebar.clone());
            }
            sidebar
        }
    };
    Ok(sidebar)
}

pub fn doc_page_from_static_files(path: &Path) -> Option<Result<Page, DocError>> {
    let cache = if path.starts_with(content_root()) {
        &STATIC_DOC_PAGE_FILES
    } else {
        &STATIC_DOC_PAGE_TRANSLATED_FILES
    };
    cache.get().map(|static_files| {
        if let Some(page) = static_files.get(path) {
            return Ok(page.clone());
        }
        Err(DocError::NotFoundInStaticCache(path.into()))
    })
}

pub fn gather_blog_posts() -> Result<HashMap<String, Page>, DocError> {
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

pub fn gather_curriculum() -> Result<CurriculumFiles, DocError> {
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

pub fn gather_blog_authors() -> Result<HashMap<String, Arc<Author>>, DocError> {
    if let Some(blog_authors_path) = blog_root().map(|br| br.join("authors")) {
        Ok(walk_builder(&[blog_authors_path], None)?
            .build()
            .filter_map(|f| f.ok())
            .filter(|f| f.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .map(|f| {
                let path = f.into_path();
                let raw = read_to_string(&path)?;
                let (fm, _) = split_fm(&raw);
                let frontmatter: AuthorFrontmatter = serde_yaml::from_str(fm.unwrap_or_default())?;
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

pub fn blog_auhtor_by_name(name: &str) -> Option<Arc<Author>> {
    blog_files().authors.get(name).cloned()
}

pub fn blog_from_url(url: &str) -> Option<Page> {
    let _ = blog_root()?;
    blog_files().posts.get(&url.to_ascii_lowercase()).cloned()
}

pub fn curriculum_from_url(url: &str) -> Option<Page> {
    let _ = curriculum_root()?;
    curriculum_files().by_url.get(url).cloned()
}

pub fn curriculum_from_path(path: &Path) -> Option<Page> {
    let _ = curriculum_root()?;
    curriculum_files().by_path.get(path).cloned()
}

pub fn read_and_cache_doc_pages() -> Result<Vec<Page>, DocError> {
    let mut docs = read_docs_parallel::<Doc>(&[content_root()], None)?;
    STATIC_DOC_PAGE_FILES
        .set(
            docs.iter()
                .cloned()
                .map(|doc| (doc.full_path().to_owned(), doc))
                .collect(),
        )
        .unwrap();
    if let Some(translated_root) = content_translated_root() {
        let transted_docs = read_docs_parallel::<Doc>(&[translated_root], None)?;
        STATIC_DOC_PAGE_TRANSLATED_FILES
            .set(
                transted_docs
                    .iter()
                    .cloned()
                    .map(|doc| (doc.full_path().to_owned(), doc))
                    .collect(),
            )
            .unwrap();
        docs.extend(transted_docs)
    }
    init_translations_from_static_docs();
    Ok(docs)
}
