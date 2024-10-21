use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, OnceLock, RwLock};

use rari_types::globals::{
    blog_root, cache_content, content_root, content_translated_root, contributor_spotlight_root,
    curriculum_root, generic_pages_root,
};
use rari_types::locale::Locale;
use rari_utils::concat_strs;
use rari_utils::io::read_to_string;
use tracing::error;

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

pub static STATIC_DOC_PAGE_FILES: OnceLock<HashMap<(Locale, Cow<'_, str>), Page>> = OnceLock::new();
pub static STATIC_DOC_PAGE_TRANSLATED_FILES: OnceLock<HashMap<(Locale, Cow<'_, str>), Page>> =
    OnceLock::new();
pub static STATIC_DOC_PAGE_FILES_BY_PATH: OnceLock<HashMap<PathBuf, Page>> = OnceLock::new();
pub static CACHED_DOC_PAGE_FILES: OnceLock<Arc<RwLock<HashMap<PathBuf, Page>>>> = OnceLock::new();
type SidebarFilesCache = Arc<RwLock<HashMap<(String, Locale), Arc<MetaSidebar>>>>;
pub static CACHED_SIDEBAR_FILES: LazyLock<SidebarFilesCache> =
    LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));
pub static CACHED_CURRICULUM: OnceLock<CurriculumFiles> = OnceLock::new();
pub static GENERIC_PAGES_FILES: OnceLock<UrlToPageMap> = OnceLock::new();
pub static CONTRIBUTOR_SPOTLIGHT_FILES: OnceLock<UrlToPageMap> = OnceLock::new();

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
            let sidebar: Sidebar = serde_yaml_ng::from_str(&raw)?;
            let sidebar = Arc::new(MetaSidebar::from(sidebar));
            if cache_content() {
                CACHED_SIDEBAR_FILES.write()?.insert(key, sidebar.clone());
            }
            sidebar
        }
    };
    Ok(sidebar)
}

pub fn doc_page_from_slug(slug: &str, locale: Locale) -> Option<Result<Page, DocError>> {
    let cache = if locale == Locale::EnUs {
        &STATIC_DOC_PAGE_FILES
    } else {
        &STATIC_DOC_PAGE_TRANSLATED_FILES
    };
    cache.get().map(|static_files| {
        if let Some(page) = static_files.get(&(locale, Cow::Borrowed(slug))) {
            return Ok(page.clone());
        }
        Err(DocError::NotFoundInStaticCache(concat_strs!(
            "/",
            locale.as_url_str(),
            "/docs/",
            slug
        )))
    })
}

pub fn doc_page_from_static_files(path: &Path) -> Option<Result<Page, DocError>> {
    STATIC_DOC_PAGE_FILES_BY_PATH.get().map(|static_files| {
        if let Some(page) = static_files.get(path) {
            return Ok(page.clone());
        }
        Err(DocError::NotFoundInStaticCache(
            path.to_string_lossy().to_string(),
        ))
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

pub fn gather_generic_pages() -> Result<HashMap<String, Page>, DocError> {
    if let Some(root) = generic_pages_root() {
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
        Err(DocError::NoGenericPagesRoot)
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

pub fn gather_contributre_spotlight() -> Result<HashMap<String, Page>, DocError> {
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
        Err(DocError::NoGenericPagesRoot)
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

pub type UrlToPageMap = HashMap<String, Page>;

pub fn generic_pages_files() -> Cow<'static, UrlToPageMap> {
    fn gather() -> UrlToPageMap {
        gather_generic_pages().unwrap_or_else(|e| {
            error!("{e}");
            Default::default()
        })
    }
    if cache_content() {
        Cow::Borrowed(GENERIC_PAGES_FILES.get_or_init(gather))
    } else {
        Cow::Owned(gather())
    }
}

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
