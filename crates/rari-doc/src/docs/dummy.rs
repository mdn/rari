use std::path::{Path, PathBuf};
use std::sync::Arc;

use rari_types::fm_types::{FeatureStatus, PageType};
use rari_types::locale::Locale;
use rari_types::RariEnv;
use serde::Serialize;

use super::blog::BlogMeta;
use super::json::{BuiltDocy, HyData, JsonBlogPost, JsonBlogPostDoc};
use super::page::{Page, PageLike, PageReader};
use super::title::page_title;
use crate::cached_readers::blog_files;
use crate::error::DocError;

#[derive(Debug, Clone, Serialize)]
pub struct BlogIndex {
    pub posts: Vec<BlogMeta>,
}

#[derive(Debug, Clone)]
pub enum DummyData {
    BlogIndex(BlogIndex),
}

#[derive(Debug, Clone)]
pub struct Dummy {
    pub title: String,
    pub slug: String,
    pub url: String,
    pub locale: Locale,
    pub content: String,
    pub page_type: PageType,
    pub path: PathBuf,
    pub typ: Option<DummyData>,
    pub base_slug: String,
}

impl Dummy {
    pub fn from_url(url: &str) -> Option<Page> {
        match url {
            "/en-US/blog/" | "/en-us/blog/" => Some(Page::Dummy(Arc::new(Dummy {
                title: "MDN Blog".to_string(),
                slug: "blog".to_string(),
                url: "/en-US/blog/".to_string(),
                locale: Locale::EnUs,
                content: Default::default(),
                page_type: PageType::Dummy,
                path: PathBuf::new(),
                typ: Some(DummyData::BlogIndex(BlogIndex {
                    posts: blog_files()
                        .sorted_meta
                        .iter()
                        .rev()
                        .map(BlogMeta::from)
                        .map(|mut m| {
                            m.links = Default::default();
                            m
                        })
                        .collect(),
                })),
                base_slug: "/en-US/".to_string(),
            }))),
            _ => None,
        }
    }
    pub fn from_sulg(slug: &str, locale: Locale) -> Page {
        Dummy::from_url(match (slug, locale) {
            ("blog" | "blog/", Locale::EnUs) => "/en-US/blog/",
            _ => "",
        })
        .unwrap()
    }

    pub fn as_built_doc(&self) -> Result<BuiltDocy, DocError> {
        match self.url() {
            "/en-US/blog/" | "/en-us/blog/" => Ok(BuiltDocy::BlogPost(Box::new(JsonBlogPost {
                doc: JsonBlogPostDoc {
                    title: self.title().to_string(),
                    mdn_url: self.url().to_owned(),
                    native: self.locale().into(),
                    page_title: page_title(self, true)?,
                    locale: self.locale(),
                    ..Default::default()
                },
                url: self.url().to_owned(),
                locale: self.locale(),
                blog_meta: None,
                hy_data: self
                    .typ
                    .as_ref()
                    .map(|DummyData::BlogIndex(b)| HyData::BlogIndex(b.clone())),
                page_title: self.title().to_owned(),
                ..Default::default()
            }))),
            url => Err(DocError::PageNotFound(
                url.to_string(),
                super::page::PageCategory::Dummy,
            )),
        }
    }
}

impl PageReader for Dummy {
    fn read(_: impl Into<PathBuf>) -> Result<Page, DocError> {
        todo!()
    }
}

impl PageLike for Dummy {
    fn url(&self) -> &str {
        &self.url
    }

    fn slug(&self) -> &str {
        &self.slug
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn short_title(&self) -> Option<&str> {
        None
    }

    fn locale(&self) -> Locale {
        self.locale
    }

    fn content(&self) -> &str {
        &self.content
    }

    fn rari_env(&self) -> Option<RariEnv<'_>> {
        None
    }

    fn render(&self) -> Result<String, DocError> {
        todo!()
    }

    fn title_suffix(&self) -> Option<&str> {
        Some("MDN")
    }

    fn page_type(&self) -> PageType {
        self.page_type
    }

    fn status(&self) -> &[FeatureStatus] {
        &[]
    }

    fn full_path(&self) -> &Path {
        &self.path
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn base_slug(&self) -> &str {
        &self.base_slug
    }

    fn trailing_slash(&self) -> bool {
        self.url().ends_with('/')
    }
}
