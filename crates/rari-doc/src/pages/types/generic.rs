use std::path::{Path, PathBuf};
use std::sync::Arc;

use rari_types::fm_types::{FeatureStatus, PageType};
use rari_types::globals::generic_pages_root;
use rari_types::locale::Locale;
use rari_types::RariEnv;
use rari_utils::concat_strs;
use rari_utils::io::read_to_string;
use serde::Deserialize;

use crate::cached_readers::generic_pages_files;
use crate::error::DocError;
use crate::pages::page::{Page, PageLike, PageReader};
use crate::utils::split_fm;

#[derive(Debug, Clone, Deserialize)]
pub struct GenericPageFrontmatter {
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct GenericPageMeta {
    pub title: String,
    pub locale: Locale,
    pub slug: String,
    pub url: String,
    pub full_path: PathBuf,
    pub path: PathBuf,
    pub title_suffix: String,
    pub page: String,
}

impl GenericPageMeta {
    pub fn from_fm(
        fm: GenericPageFrontmatter,
        full_path: PathBuf,
        path: PathBuf,
        locale: Locale,
        slug: String,
        title_suffix: &str,
        page: String,
    ) -> Result<Self, DocError> {
        let url = concat_strs!(
            "/",
            locale.as_url_str(),
            "/",
            slug.as_str(),
            "/",
            page.as_str()
        );
        Ok(GenericPageMeta {
            title: fm.title,
            locale,
            slug,
            url,
            path,
            full_path,
            title_suffix: title_suffix.to_string(),
            page,
        })
    }
}

impl PageReader for GenericPage {
    fn read(
        path: impl Into<PathBuf>,
        locale: Option<Locale>,
    ) -> Result<crate::pages::page::Page, DocError> {
        let path = path.into();
        let root = generic_pages_root().ok_or(DocError::NoGenericPagesRoot)?;
        let without_root: &Path = path.strip_prefix(root)?;
        let (slug, title_suffix, root) = if without_root.starts_with("plus/") {
            ("plus/docs", "MDN Plus", root.join("plus"))
        } else if without_root.starts_with("observatory/") {
            (
                "observatory/docs",
                "HTTP Observatory",
                root.join("observatory"),
            )
        } else {
            return Err(DocError::PageNotFound(
                path.to_string_lossy().to_string(),
                crate::pages::page::PageCategory::GenericPage,
            ));
        };
        read_generic_page(path, locale.unwrap_or_default(), slug, title_suffix, &root)
            .map(|g| Page::GenericPage(Arc::new(g)))
    }
}
#[derive(Debug, Clone)]
pub struct GenericPage {
    pub meta: GenericPageMeta,
    raw: String,
    content_start: usize,
}

impl GenericPage {
    pub fn from_slug(slug: &str, locale: Locale) -> Option<Page> {
        let url = concat_strs!("/", locale.as_url_str(), "/", slug).to_ascii_lowercase();
        generic_pages_files().get(&url).cloned()
    }

    pub fn as_locale(&self, locale: Locale) -> Self {
        let Self {
            mut meta,
            raw,
            content_start,
        } = self.clone();
        meta.locale = locale;
        meta.url = concat_strs!("/", locale.as_url_str(), "/", meta.slug.as_str());
        Self {
            meta,
            raw,
            content_start,
        }
    }

    pub fn is_generic(slug: &str, locale: Locale) -> bool {
        let url = concat_strs!("/", locale.as_url_str(), "/", slug).to_ascii_lowercase();
        generic_pages_files().contains_key(&url)
    }
}

impl PageLike for GenericPage {
    fn url(&self) -> &str {
        &self.meta.url
    }

    fn slug(&self) -> &str {
        &self.meta.slug
    }

    fn title(&self) -> &str {
        &self.meta.title
    }

    fn short_title(&self) -> Option<&str> {
        None
    }

    fn locale(&self) -> Locale {
        Default::default()
    }

    fn content(&self) -> &str {
        &self.raw[self.content_start..]
    }

    fn rari_env(&self) -> Option<RariEnv<'_>> {
        None
    }

    fn render(&self) -> Result<String, DocError> {
        todo!()
    }

    fn title_suffix(&self) -> Option<&str> {
        Some("MDN Curriculum")
    }

    fn page_type(&self) -> PageType {
        PageType::GenericPage
    }

    fn status(&self) -> &[FeatureStatus] {
        &[]
    }

    fn full_path(&self) -> &Path {
        &self.meta.full_path
    }

    fn path(&self) -> &Path {
        &self.meta.path
    }

    fn base_slug(&self) -> &str {
        &self.meta.url[..self
            .meta
            .url
            .match_indices('/')
            .nth(1)
            .map(|(i, _)| i)
            .unwrap_or(self.meta.url.len())]
    }

    fn trailing_slash(&self) -> bool {
        true
    }
}

fn read_generic_page(
    path: impl Into<PathBuf>,
    locale: Locale,
    slug: &str,
    title_suffix: &str,
    root: &Path,
) -> Result<GenericPage, DocError> {
    let full_path: PathBuf = path.into();
    let raw = read_to_string(&full_path)?;
    let (fm, content_start) = split_fm(&raw);
    let fm = fm.ok_or(DocError::NoFrontmatter)?;
    let fm: GenericPageFrontmatter = serde_yaml::from_str(fm)?;
    let path = full_path.strip_prefix(root)?.to_path_buf();
    let page = path.with_extension("");
    let page = page.to_string_lossy();
    let slug = concat_strs!(slug, "/", page.as_ref());

    Ok(GenericPage {
        meta: GenericPageMeta::from_fm(
            fm,
            full_path,
            path,
            locale,
            slug.to_string(),
            title_suffix,
            page.to_string(),
        )?,
        raw,
        content_start,
    })
}
