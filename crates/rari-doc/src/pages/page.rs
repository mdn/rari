use std::path::{Path, PathBuf};
use std::sync::Arc;

use enum_dispatch::enum_dispatch;
use rari_types::fm_types::{FeatureStatus, PageType};
use rari_types::globals::{
    blog_root, contributor_spotlight_root, curriculum_root, generic_content_root,
};
use rari_types::locale::Locale;
use rari_types::RariEnv;

use super::json::BuiltPage;
use super::types::contributors::contributor_spotlight_from_url;
use super::types::generic::Generic;
use crate::error::DocError;
use crate::pages::types::blog::BlogPost;
use crate::pages::types::contributors::ContributorSpotlight;
use crate::pages::types::curriculum::Curriculum;
use crate::pages::types::doc::Doc;
use crate::pages::types::spa::SPA;
use crate::pages::types::utils::FmTempl;
use crate::resolve::{url_meta_from, UrlMeta};
use crate::utils::locale_and_typ_from_path;

/// Represents a page in the documentation system.
///
/// The `Page` enum is used to define different types of pages that can be part of the documentation.
/// It provides methods to create instances from URLs and to check the existence of pages.
#[derive(Debug, Clone)]
#[enum_dispatch]
pub enum Page {
    Doc(Arc<Doc>),
    BlogPost(Arc<BlogPost>),
    SPA(Arc<SPA>),
    Curriculum(Arc<Curriculum>),
    ContributorSpotlight(Arc<ContributorSpotlight>),
    GenericPage(Arc<Generic>),
}

/// Represents the category of a page in the documentation system.
///
/// The `PageCategory` enum is used to classify different types of pages within the documentation.
/// Each variant corresponds to a specific category of pages.
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
    /// Creates an instance of `Page` from the given URL if it exists.
    ///
    /// # Arguments
    ///
    /// * `url` - A string slice that holds the URL to create the `Page` instance from.
    ///
    /// # Returns
    ///
    /// * `Result<Self, DocError>` - Returns an instance of `Page` on success, or a `DocError` on failure,
    ///   usually if the Page's file cannot be found.
    pub fn from_url(url: &str) -> Result<Self, DocError> {
        Self::internal_from_url(url, None, false)
    }

    /// Creates a `Page` from the given URL with a fallback to the
    /// default `Locale` if the page cannot be found in the given URL's `Locale`.
    ///
    /// # Arguments
    ///
    /// * `url` - A string slice that holds the URL to create the instance from.
    ///
    /// # Returns
    ///
    /// * `Result<Self, DocError>` - Returns an instance of `Self` on success, or a `DocError` on failure.
    pub fn from_url_with_fallback(url: &str) -> Result<Self, DocError> {
        Self::internal_from_url(url, None, true)
    }

    /// Creates a `Page` from the given URL and a specified `Locale`. The page will be searched
    /// for in the passed-in `Locale`, overriding the URL's `Locale`. If not found, the default `Locale`
    /// is searched as a fallback.
    ///
    /// # Arguments
    ///
    /// * `url` - A string slice that holds the URL to create the instance from.
    /// * `locale` - A `Locale` that specifies the `Locale` the URL is searched in.
    ///
    /// # Returns
    ///
    /// * `Result<Self, DocError>` - Returns an instance of `Self` on success, or a `DocError` on failure.
    pub fn from_url_with_locale_and_fallback(url: &str, locale: Locale) -> Result<Self, DocError> {
        Self::internal_from_url(url, Some(locale), true)
    }

    pub fn internal_from_url(
        url: &str,
        locale: Option<Locale>,
        fallback: bool,
    ) -> Result<Self, DocError> {
        let url = &url[..url.find('#').unwrap_or(url.len())];
        let UrlMeta {
            folder_path,
            slug,
            locale: locale_from_url,
            page_category,
        } = url_meta_from(url)?;
        let locale = locale.unwrap_or(locale_from_url);
        match page_category {
            PageCategory::SPA => {
                if locale != Locale::EnUs && slug.starts_with("blog") {
                    // Blog is en-US only.
                    Err(DocError::PageNotFound(url.to_string(), PageCategory::SPA))
                } else {
                    SPA::from_slug(slug, locale)
                        .ok_or(DocError::PageNotFound(url.to_string(), PageCategory::SPA))
                }
            }
            PageCategory::Doc => Doc::page_from_slug_path(&folder_path, locale, fallback)
                .map_err(|_| DocError::PageNotFound(url.to_string(), PageCategory::Doc)),
            PageCategory::BlogPost => {
                if locale != Locale::EnUs {
                    // Blog is en-US only.
                    Err(DocError::PageNotFound(
                        url.to_string(),
                        PageCategory::BlogPost,
                    ))
                } else {
                    BlogPost::page_from_url(url).ok_or(DocError::PageNotFound(
                        url.to_string(),
                        PageCategory::BlogPost,
                    ))
                }
            }
            PageCategory::Curriculum => {
                if locale != Locale::EnUs {
                    // Curriculum is en-US only.
                    Err(DocError::PageNotFound(
                        url.to_string(),
                        PageCategory::Curriculum,
                    ))
                } else {
                    Curriculum::page_from_url(url).ok_or(DocError::PageNotFound(
                        url.to_string(),
                        PageCategory::Curriculum,
                    ))
                }
            }
            PageCategory::ContributorSpotlight => contributor_spotlight_from_url(url, locale)
                .ok_or(DocError::PageNotFound(
                    url.to_string(),
                    PageCategory::ContributorSpotlight,
                )),
            PageCategory::GenericPage => Generic::from_slug(slug, locale).ok_or(
                DocError::PageNotFound(url.to_string(), PageCategory::GenericPage),
            ),
        }
    }

    /// Determines if the given URL should be ignored for link-checking.
    ///
    /// # Arguments
    ///
    /// * `url` - A string slice that holds the URL to be checked.
    ///
    /// # Returns
    ///
    /// * `bool` - Returns `true` if the URL gets a free pass on link-checking, otherwise returns `false`.
    pub fn ignore_link_check(url: &str) -> bool {
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

    /// Checks whether a page exists for the given URL.
    ///
    /// # Arguments
    /// - `url`: A string slice that holds the URL to check.
    ///
    /// # Returns
    /// `true` if the page exists; otherwise, `false`.
    ///
    /// # Examples
    pub fn exists(url: &str) -> bool {
        if let Ok(meta) = url_meta_from(url) {
            match meta.page_category {
                PageCategory::BlogPost if blog_root().is_none() => return true,
                PageCategory::Curriculum if curriculum_root().is_none() => return true,
                PageCategory::ContributorSpotlight if contributor_spotlight_root().is_none() => {
                    return true
                }
                PageCategory::GenericPage if generic_content_root().is_none() => return true,
                _ => {}
            };
            Page::from_url(url).is_ok()
        } else {
            false
        }
    }

    /// Checks whether a page exists for the given URL, with a fallback mechanism.
    ///
    /// This function operates similarly to [`Page::exists`], but it uses a en-US fallback
    /// when determining if a page exists.
    ///
    /// # Arguments
    /// - `url`: A string slice that holds the URL to check.
    ///
    /// # Returns
    /// `true` if the page exists (with or without fallback); otherwise, `false`.
    pub fn exists_with_fallback(url: &str) -> bool {
        if let Ok(meta) = url_meta_from(url) {
            match meta.page_category {
                PageCategory::BlogPost if blog_root().is_none() => return true,
                PageCategory::Curriculum if curriculum_root().is_none() => return true,
                PageCategory::ContributorSpotlight if contributor_spotlight_root().is_none() => {
                    return true
                }
                PageCategory::GenericPage if generic_content_root().is_none() => return true,
                _ => {}
            };
            Page::from_url_with_fallback(url).is_ok()
        } else {
            false
        }
    }
}

impl PageReader<Page> for Page {
    fn read(path: impl Into<PathBuf>, locale: Option<Locale>) -> Result<Page, DocError> {
        let path = path.into();
        let (_, typ) = locale_and_typ_from_path(&path)?;
        match typ {
            PageCategory::Doc => Doc::read(path, locale),
            PageCategory::BlogPost => BlogPost::read(path, locale),
            PageCategory::SPA => SPA::read(path, locale),
            PageCategory::Curriculum => Curriculum::read(path, locale),
            PageCategory::ContributorSpotlight => ContributorSpotlight::read(path, locale),
            PageCategory::GenericPage => Generic::read(path, locale),
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
    fn fm_offset(&self) -> usize;
    fn raw_content(&self) -> &str;
    fn banners(&self) -> Option<&[FmTempl]>;
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

    fn fm_offset(&self) -> usize {
        (**self).fm_offset()
    }

    fn raw_content(&self) -> &str {
        (**self).raw_content()
    }

    fn banners(&self) -> Option<&[FmTempl]> {
        (**self).banners()
    }
}

/// A trait for reading pages in the documentation system.
///
/// The `PageReader` trait defines a method for reading pages from a specified path,
/// optionally considering a locale.
pub trait PageReader<T> {
    /// Reads a page from the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - An implementation of `Into<PathBuf>` that specifies the path to the page.
    /// * `locale` - An optional `Locale` that specifies the locale to be considered while reading the page.
    ///
    /// # Returns
    ///
    /// * `Result<Page, DocError>` - Returns a `Page` on success, or a `DocError` on failure.
    fn read(path: impl Into<PathBuf>, locale: Option<Locale>) -> Result<T, DocError>;
}

/// A trait for writing pages in the documentation system.
///
/// The `PageWriter` trait defines a method for writing the current state of a page
/// to the file system.
pub trait PageWriter {
    /// Writes the current state of the page to the file system.
    ///
    /// # Returns
    ///
    /// * `Result<(), DocError>` - Returns `Ok(())` if the write operation is successful,
    ///   or a `DocError` if an error occurs during the write process.
    fn write(&self) -> Result<(), DocError>;
}

/// A trait for building pages in the documentation system.
///
/// The `PageBuilder` trait defines a method for constructing a page and returning
/// a `BuiltDocy` (A JSON representation of the build artifact). Implementors of
/// this trait are responsible for handling the specifics of the build process,
/// which could involve compiling content, applying templates, and performing
/// any necessary transformations.
pub trait PageBuilder {
    /// Builds the page and returns the built `BuiltDocy` artifact.
    ///
    /// # Returns
    ///
    /// * `Result<BuiltDocy, DocError>` - Returns `Ok(BuiltDocy)` if the build process is successful,
    ///   or a `DocError` if an error occurs during the build process.
    fn build(&self) -> Result<BuiltPage, DocError>;
}
