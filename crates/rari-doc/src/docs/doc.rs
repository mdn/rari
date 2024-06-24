use std::collections::HashMap;
use std::fmt::Display;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rari_md::m2h;
use rari_types::fm_types::{FeatureStatus, PageType};
use rari_types::globals::deny_warnings;
use rari_types::locale::Locale;
use rari_types::RariEnv;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use tracing::warn;
use validator::Validate;

use super::page::{Page, PageCategory, PageLike, PageReader};
use crate::cached_readers::{page_from_static_files, CACHED_PAGE_FILES};
use crate::error::DocError;
use crate::resolve::build_url;
use crate::templ::parser::{decode_ks, encode_ks};
use crate::utils::{locale_and_typ_from_path, root_for_locale, split_fm, t_or_vec};

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

#[derive(Deserialize, Serialize, Clone, Debug, Default, Validate)]
#[serde(default)]
pub struct FrontMatter {
    #[validate(length(max = 120))]
    pub title: String,
    #[serde(rename = "short-title")]
    #[validate(length(max = 60))]
    pub short_title: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub slug: String,
    #[serde(rename = "page-type")]
    pub page_type: PageType,
    #[serde(deserialize_with = "t_or_vec", default)]
    pub status: Vec<FeatureStatus>,
    #[serde(rename = "browser-compat", deserialize_with = "t_or_vec", default)]
    pub browser_compat: Vec<String>,
    #[serde(rename = "spec-urls", deserialize_with = "t_or_vec", default)]
    pub spec_urls: Vec<String>,
    pub original_slug: Option<String>,
    #[serde(deserialize_with = "t_or_vec", default)]
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

impl PageReader for Doc {
    fn read(path: impl Into<PathBuf>) -> Result<Page, DocError> {
        let path = path.into();
        if let Some(doc) = page_from_static_files(&path) {
            return doc;
        }

        if let Some(cache) = CACHED_PAGE_FILES.get() {
            if let Some(doc) = cache.read()?.get(&path) {
                return Ok(doc.clone());
            }
        }
        let page = read_doc(&path).map(Arc::new).map(Page::Doc)?;
        if let Some(cache) = CACHED_PAGE_FILES.get() {
            if let Ok(mut cache) = cache.write() {
                cache.insert(path, page.clone());
            }
        }
        Ok(page)
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
        self.meta.short_title.as_deref()
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
        "/docs"
    }

    fn trailing_slash(&self) -> bool {
        false
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
    } = serde_yaml::from_str(fm)?;
    let url = build_url(&slug, &locale, PageCategory::Doc);
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

pub fn render_md_to_html(
    input: &str,
    locale: Locale,
    path: Option<&impl Display>,
) -> Result<String, DocError> {
    let (encoded, before) = encode_ks(input)?;
    let encoded_html = m2h(&encoded, locale)?;
    let (html, after) = decode_ks(&encoded_html)?;
    if before != after {
        if deny_warnings() {
            return Err(DocError::InvalidTempl(
                path.map(|s| s.to_string()).unwrap_or_default(),
            ));
        }
        warn!(
            "invalid templ: {}",
            path.map(|s| s.to_string()).unwrap_or_default()
        );
    }

    Ok(html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_status_test() {
        let fm = r#"
        status:
          - non-standard
          - experimental
      "#;
        let meta = serde_yaml::from_str::<FrontMatter>(fm).unwrap();
        assert_eq!(meta.status.len(), 2);

        let fm = r#"
        status: experimental
      "#;
        let meta = serde_yaml::from_str::<FrontMatter>(fm).unwrap();
        assert_eq!(meta.status.len(), 1);
    }

    #[test]
    fn browser_compat_test() {
        let fm = r#"
        browser-compat:
          - foo
          - ba
      "#;
        let meta = serde_yaml::from_str::<FrontMatter>(fm).unwrap();
        assert_eq!(meta.browser_compat.len(), 2);

        let fm = r#"
        browser-compat: foo
      "#;
        let meta = serde_yaml::from_str::<FrontMatter>(fm).unwrap();
        assert_eq!(meta.browser_compat.len(), 1);
    }
}
