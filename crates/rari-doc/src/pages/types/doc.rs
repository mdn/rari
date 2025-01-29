use std::collections::HashMap;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use pretty_yaml::config::{FormatOptions, LanguageOptions};
use rari_md::m2h;
use rari_types::fm_types::{FeatureStatus, PageType};
use rari_types::locale::{default_locale, Locale};
use rari_types::RariEnv;
use rari_utils::io::read_to_string;
use serde::{Deserialize, Serialize};
use serde_yaml_ng::Value;
use tracing::debug;
use validator::Validate;

use crate::cached_readers::{doc_page_from_static_files, CACHED_DOC_PAGE_FILES};
use crate::error::DocError;
use crate::pages::page::{Page, PageCategory, PageLike, PageReader, PageWriter};
use crate::resolve::{build_url, url_to_folder_path};
use crate::utils::{
    locale_and_typ_from_path, root_for_locale, serialize_t_or_vec, split_fm, t_or_vec,
};

/*
  "attribute-order": [
   "title",
   "short-title",
   "slug",
   "page-type",
   "status",
   "browser-compat",
   "spec-urls"
 ]
*/

fn is_page_type_none(page_type: &PageType) -> bool {
    matches!(page_type, PageType::None)
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, Validate)]
#[serde(default)]
pub struct FrontMatter {
    #[validate(length(max = 120))]
    pub title: String,
    #[serde(rename = "short-title", skip_serializing_if = "Option::is_none")]
    #[validate(length(max = 60))]
    pub short_title: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    pub slug: String,
    #[serde(rename = "page-type", skip_serializing_if = "is_page_type_none")]
    pub page_type: PageType,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub status: Vec<FeatureStatus>,
    #[serde(
        rename = "browser-compat",
        deserialize_with = "t_or_vec",
        serialize_with = "serialize_t_or_vec",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub browser_compat: Vec<String>,
    #[serde(
        rename = "spec-urls",
        deserialize_with = "t_or_vec",
        serialize_with = "serialize_t_or_vec",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub spec_urls: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_slug: Option<String>,
    #[serde(
        deserialize_with = "t_or_vec",
        serialize_with = "serialize_t_or_vec",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub sidebar: Vec<String>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct Meta {
    pub title: String,
    pub short_title: Option<String>,
    pub tags: Vec<String>,
    pub slug: String,
    pub page_type: PageType,
    pub status: Vec<FeatureStatus>,
    pub browser_compat: Vec<String>,
    pub spec_urls: Vec<String>,
    pub original_slug: Option<String>,
    pub sidebar: Vec<String>,
    pub locale: Locale,
    pub full_path: PathBuf,
    pub path: PathBuf,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct Doc {
    pub meta: Meta,
    raw: String,
    content_start: usize,
}

pub type ADoc = Arc<Doc>;

impl Doc {
    pub fn page_from_slug(slug: &str, locale: Locale, fallback: bool) -> Result<Page, DocError> {
        Doc::page_from_slug_path(&url_to_folder_path(slug), locale, fallback)
    }

    pub fn page_from_slug_path(
        path: &Path,
        locale: Locale,
        fallback: bool,
    ) -> Result<Page, DocError> {
        let doc = Self::page_from_slug_path_internal(path, locale);
        if doc.is_err() && locale != default_locale() && fallback {
            Self::page_from_slug_path_internal(path, Default::default())
        } else {
            doc
        }
    }

    fn page_from_slug_path_internal(path: &Path, locale: Locale) -> Result<Page, DocError> {
        let mut file = root_for_locale(locale)?.to_path_buf();
        file.push(locale.as_folder_str());
        file.push(path);
        file.push("index.md");
        Doc::read(file, None)
    }

    fn copy_meta_from_super(&mut self, super_doc: &Doc) {
        let meta = &mut self.meta;
        meta.tags = super_doc.meta.tags.clone();
        meta.page_type = super_doc.meta.page_type;
        meta.status = super_doc.meta.status.clone();
        meta.browser_compat = super_doc.meta.browser_compat.clone();
        meta.spec_urls = super_doc.meta.spec_urls.clone();
        meta.original_slug = super_doc.meta.original_slug.clone();
        meta.sidebar = super_doc.meta.sidebar.clone();
    }

    pub fn is_orphaned(&self) -> bool {
        self.meta.slug.starts_with("orphaned/")
    }

    pub fn is_conflicting(&self) -> bool {
        self.meta.slug.starts_with("conflicting/")
    }
}

impl PageReader<Page> for Doc {
    fn read(path: impl Into<PathBuf>, _: Option<Locale>) -> Result<Page, DocError> {
        let path = path.into();
        if let Ok(doc) = doc_page_from_static_files(&path) {
            return Ok(doc);
        }

        if let Some(cache) = CACHED_DOC_PAGE_FILES.get() {
            if let Some(doc) = cache.get(&path) {
                return Ok(doc.clone());
            }
        }
        debug!("reading doc: {}", &path.display());
        let mut doc = read_doc(&path)?;

        if doc.meta.locale != Default::default() && !doc.is_conflicting() && !doc.is_orphaned() {
            match Doc::page_from_slug(&doc.meta.slug, Default::default(), false) {
                Ok(Page::Doc(super_doc)) => {
                    doc.copy_meta_from_super(&super_doc);
                }
                Err(DocError::PageNotFound(path, _)) => {
                    tracing::error!(
                        "Super doc not found for {}:{} (looked for {})",
                        doc.meta.locale.as_url_str(),
                        doc.meta.slug,
                        path
                    );
                }
                _ => {}
            }
        }

        let page = Page::Doc(Arc::new(doc));
        if let Some(cache) = CACHED_DOC_PAGE_FILES.get() {
            cache.insert(path, page.clone());
        }
        Ok(page)
    }
}

impl PageWriter for Doc {
    fn write(&self) -> Result<(), DocError> {
        write_doc(self)
    }
}

impl PageLike for Doc {
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
        self.meta.short_title.as_deref().or_else(|| {
            if self.meta.title.starts_with('<') {
                if let Some(end) = self.meta.title.find('>') {
                    return Some(&self.meta.title[..end + 1]);
                }
            }
            None
        })
    }

    fn locale(&self) -> Locale {
        self.meta.locale
    }

    fn content(&self) -> &str {
        &self.raw[self.content_start..]
    }

    fn rari_env(&self) -> Option<RariEnv<'_>> {
        Some(RariEnv {
            url: &self.meta.url,
            locale: self.meta.locale,
            title: &self.meta.title,
            tags: &self.meta.tags,
            browser_compat: &self.meta.browser_compat,
            spec_urls: &self.meta.spec_urls,
            page_type: self.meta.page_type,
            slug: &self.meta.slug,
        })
    }

    fn render(&self) -> Result<String, DocError> {
        Ok(m2h(self.content(), self.meta.locale)?)
    }

    fn title_suffix(&self) -> Option<&str> {
        Some("MDN")
    }

    fn page_type(&self) -> PageType {
        self.meta.page_type
    }

    fn status(&self) -> &[FeatureStatus] {
        &self.meta.status
    }

    fn full_path(&self) -> &Path {
        &self.meta.full_path
    }

    fn path(&self) -> &Path {
        &self.meta.path
    }

    fn base_slug(&self) -> &str {
        self.meta.url.split_inclusive("/docs").next().unwrap_or("/")
    }

    fn trailing_slash(&self) -> bool {
        false
    }

    fn fm_offset(&self) -> usize {
        self.raw[..self.content_start].lines().count()
    }
}

fn read_doc(path: impl Into<PathBuf>) -> Result<Doc, DocError> {
    let full_path = path.into();
    let (locale, _) = locale_and_typ_from_path(&full_path)?;
    let raw = read_to_string(&full_path)?;
    let (fm, content_start) = split_fm(&raw);
    let fm = fm.ok_or(DocError::NoFrontmatter)?;
    let FrontMatter {
        title,
        short_title,
        tags,
        slug,
        page_type,
        status,
        browser_compat,
        spec_urls,
        original_slug,
        sidebar,
        ..
    } = serde_yaml_ng::from_str(fm)?;
    let url = build_url(&slug, locale, PageCategory::Doc)?;
    let path = full_path
        .strip_prefix(root_for_locale(locale)?)?
        .to_path_buf();

    Ok(Doc {
        meta: Meta {
            title,
            short_title,
            tags,
            slug,
            page_type,
            status,
            browser_compat,
            spec_urls,
            original_slug,
            sidebar,
            locale,
            full_path,
            path,
            url,
        },
        raw,
        content_start,
    })
}

fn write_doc(doc: &Doc) -> Result<(), DocError> {
    let path = doc.path();
    let locale = doc.meta.locale;

    let mut file_path = root_for_locale(locale)?.to_path_buf();
    file_path.push(path);

    let (fm, content_start) = split_fm(&doc.raw);
    let fm = fm.ok_or(DocError::NoFrontmatter)?;
    // Read original frontmatter to pass additional fields along,
    // overwrite fields from meta
    let mut frontmatter: FrontMatter = serde_yaml_ng::from_str(fm)?;
    frontmatter = FrontMatter {
        title: doc.meta.title.clone(),
        short_title: doc.meta.short_title.clone(),
        tags: doc.meta.tags.clone(),
        slug: doc.meta.slug.clone(),
        page_type: doc.meta.page_type,
        status: doc.meta.status.clone(),
        browser_compat: doc.meta.browser_compat.clone(),
        spec_urls: doc.meta.spec_urls.clone(),
        original_slug: doc.meta.original_slug.clone(),
        sidebar: doc.meta.sidebar.clone(),
        ..frontmatter
    };

    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let fm_str = fm_to_string(&frontmatter)?;

    let file = fs::File::create(&file_path)?;
    let mut buffer = BufWriter::new(file);
    buffer.write_all(b"---\n")?;
    buffer.write_all(fm_str.as_bytes())?;
    buffer.write_all(b"---\n")?;

    buffer.write_all(doc.raw[content_start..].as_bytes())?;

    Ok(())
}

fn fm_to_string(fm: &FrontMatter) -> Result<String, DocError> {
    let fm_str = serde_yaml_ng::to_string(fm)?;
    Ok(pretty_yaml::format_text(
        &fm_str,
        &FormatOptions {
            language: LanguageOptions {
                quotes: pretty_yaml::config::Quotes::ForceDouble,
                indent_block_sequence_in_map: true,
                ..Default::default()
            },
            ..Default::default()
        },
    )?)
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;

    #[test]
    fn feature_status_test() {
        let fm = r#"
        status:
          - non-standard
          - experimental
      "#;
        let meta = serde_yaml_ng::from_str::<FrontMatter>(fm).unwrap();
        assert_eq!(meta.status.len(), 2);

        let fm = r#"
        status:
          - experimental
      "#;
        let meta = serde_yaml_ng::from_str::<FrontMatter>(fm).unwrap();
        assert_eq!(meta.status.len(), 1);
    }

    #[test]
    fn browser_compat_test() {
        let fm = indoc!(
            r#"
            title: "007"
            slug: foo
            browser-compat:
              - foo
              - bar
            foo:
              - bar
      "#
        );
        let meta = serde_yaml_ng::from_str::<FrontMatter>(fm).unwrap();
        assert_eq!(meta.browser_compat.len(), 2);

        assert_eq!(fm, fm_to_string(&meta).unwrap());

        let fm = r#"
        browser-compat: foo
      "#;
        let meta = serde_yaml_ng::from_str::<FrontMatter>(fm).unwrap();
        assert_eq!(meta.browser_compat.len(), 1);
    }
}
