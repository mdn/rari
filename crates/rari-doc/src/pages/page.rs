use std::path::{Path, PathBuf};
use std::sync::Arc;

use enum_dispatch::enum_dispatch;
use rari_types::fm_types::{FeatureStatus, PageType};
use rari_types::globals::blog_root;
use rari_types::locale::Locale;
use rari_types::RariEnv;

use super::json::BuiltDocy;
use super::types::contributors::contributor_spotlight_from_url;
use super::types::generic::GenericPage;
use crate::error::DocError;
use crate::pages::types::blog::BlogPost;
use crate::pages::types::contributors::ContributorSpotlight;
use crate::pages::types::curriculum::CurriculumPage;
use crate::pages::types::doc::Doc;
use crate::pages::types::spa::SPA;
use crate::resolve::{strip_locale_from_url, url_meta_from, UrlMeta};
use crate::utils::locale_and_typ_from_path;

#[derive(Debug, Clone)]
#[enum_dispatch]
pub enum Page {
    Doc(Arc<Doc>),
    BlogPost(Arc<BlogPost>),
    SPA(Arc<SPA>),
    Curriculum(Arc<CurriculumPage>),
    ContributorSpotlight(Arc<ContributorSpotlight>),
    GenericPage(Arc<GenericPage>),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PageCategory {
    Doc,
    BlogPost,
    SPA,
    Curriculum,
    ContributorSpotlight,
    GenericPage,
}

impl Page {
    pub fn from_url(url: &str) -> Result<Self, DocError> {
        Self::from_url_with_other_locale_and_fallback(url, None)
    }

    pub fn from_url_with_other_locale_and_fallback(
        url: &str,
        locale: Option<Locale>,
    ) -> Result<Self, DocError> {
        let UrlMeta {
            folder_path,
            slug,
            locale: locale_from_url,
            page_category,
        } = url_meta_from(url)?;
        let locale = locale.unwrap_or(locale_from_url);
        match page_category {
            PageCategory::SPA => SPA::from_slug(slug, locale)
                .ok_or(DocError::PageNotFound(url.to_string(), PageCategory::SPA)),
            PageCategory::Doc => {
                let doc = Doc::page_from_slug_path(&folder_path, locale);
                if doc.is_err() && locale != Default::default() {
                    Doc::page_from_slug_path(&folder_path, Default::default())
                } else {
                    doc
                }
            }
            PageCategory::BlogPost => BlogPost::page_from_url(url).ok_or(DocError::PageNotFound(
                url.to_string(),
                PageCategory::BlogPost,
            )),
            PageCategory::Curriculum => CurriculumPage::page_from_url(url).ok_or(
                DocError::PageNotFound(url.to_string(), PageCategory::Curriculum),
            ),
            PageCategory::ContributorSpotlight => contributor_spotlight_from_url(url, locale)
                .ok_or(DocError::PageNotFound(
                    url.to_string(),
                    PageCategory::ContributorSpotlight,
                )),
            PageCategory::GenericPage => GenericPage::from_slug(slug, locale).ok_or(
                DocError::PageNotFound(url.to_string(), PageCategory::GenericPage),
            ),
        }
    }

    pub fn ignore(url: &str) -> bool {
        if url == "/discord" {
            return true;
        }
        if url == "/en-US/blog/rss.xml" {
            return true;
        }
        if url.starts_with("/users/") {
            return true;
        }
        if url.starts_with("/en-US/observatory") {
            return true;
        }
        if url.starts_with("/en-US/plus") {
            return true;
        }
        if url.starts_with("/en-US/play") {
            return true;
        }

        false
    }
    pub fn exists(url: &str) -> bool {
        if url == "/discord" {
            return true;
        }
        if url.starts_with("/users/") {
            return true;
        }
        if url.starts_with("/en-US/blog") && blog_root().is_none() {
            return true;
        }
        if url.starts_with("/en-US/curriculum") {
            return true;
        }
        if strip_locale_from_url(url).1 == "/" {
            return true;
        }

        Page::from_url(url).is_ok()
    }
}

impl PageReader for Page {
    fn read(path: impl Into<PathBuf>, locale: Option<Locale>) -> Result<Page, DocError> {
        let path = path.into();
        let (_, typ) = locale_and_typ_from_path(&path)?;
        match typ {
            PageCategory::Doc => Doc::read(path, locale),
            PageCategory::BlogPost => BlogPost::read(path, locale),
            PageCategory::SPA => SPA::read(path, locale),
            PageCategory::Curriculum => CurriculumPage::read(path, locale),
            PageCategory::ContributorSpotlight => ContributorSpotlight::read(path, locale),
            PageCategory::GenericPage => GenericPage::read(path, locale),
        }
    }
}

#[enum_dispatch(Page)]
pub trait PageLike {
    fn url(&self) -> &str;
    fn slug(&self) -> &str;
    fn title(&self) -> &str;
    fn short_title(&self) -> Option<&str>;
    fn locale(&self) -> Locale;
    fn content(&self) -> &str;
    fn rari_env(&self) -> Option<RariEnv<'_>>;
    fn render(&self) -> Result<String, DocError>;
    fn title_suffix(&self) -> Option<&str>;
    fn page_type(&self) -> PageType;
    fn status(&self) -> &[FeatureStatus];
    fn full_path(&self) -> &Path;
    fn path(&self) -> &Path;
    fn base_slug(&self) -> &str;
    fn trailing_slash(&self) -> bool;
}

impl<T: PageLike> PageLike for Arc<T> {
    fn url(&self) -> &str {
        (**self).url()
    }

    fn slug(&self) -> &str {
        (**self).slug()
    }

    fn title(&self) -> &str {
        (**self).title()
    }

    fn short_title(&self) -> Option<&str> {
        (**self).short_title()
    }

    fn locale(&self) -> Locale {
        (**self).locale()
    }

    fn content(&self) -> &str {
        (**self).content()
    }

    fn rari_env(&self) -> Option<RariEnv<'_>> {
        (**self).rari_env()
    }

    fn render(&self) -> Result<String, DocError> {
        (**self).render()
    }

    fn title_suffix(&self) -> Option<&str> {
        (**self).title_suffix()
    }

    fn page_type(&self) -> PageType {
        (**self).page_type()
    }

    fn status(&self) -> &[FeatureStatus] {
        (**self).status()
    }

    fn full_path(&self) -> &Path {
        (**self).full_path()
    }

    fn path(&self) -> &Path {
        (**self).path()
    }

    fn base_slug(&self) -> &str {
        (**self).base_slug()
    }

    fn trailing_slash(&self) -> bool {
        (**self).trailing_slash()
    }
}

pub trait PageReader {
    fn read(path: impl Into<PathBuf>, locale: Option<Locale>) -> Result<Page, DocError>;
}

pub trait PageBuilder {
    fn build(&self) -> Result<BuiltDocy, DocError>;
}
