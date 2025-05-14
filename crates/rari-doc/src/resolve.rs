//! # URL Resolution Module
//!
//! The `resolve` module provides functionality for resolving and manipulating URLs within the documentation system.
//! It includes utilities for converting URLs to folder paths, stripping locales from URLs, and extracting metadata
//! from URLs such as locale, slug, and page category.
//!
//! ## Key Components
//!
//! - **Functions**:
//!   - `url_to_folder_path`: Converts a URL slug to a folder path by replacing certain characters.
//!   - `strip_locale_from_url`: Strips the locale from a URL and returns the locale and the remaining URL.
//!   - `url_meta_from`: Extracts metadata from a URL, including the locale, slug, and page category.
//!
//! - **Structs**:
//!   - `UrlMeta`: A struct that holds metadata extracted from a URL, including the folder path, slug, locale, and page category.

use std::path::PathBuf;
use std::str::FromStr;

use rari_types::locale::Locale;
use rari_utils::concat_strs;

use crate::error::{DocError, UrlError};
use crate::pages::page::{PageCategory, PageLike};
use crate::pages::types::generic::Generic;
use crate::pages::types::spa::SPA;

/// Converts a URL slug to a folder path by replacing certain special characters that are not allowed in path names
/// on certain file systems, i.e. Windows.
///
/// This function takes a URL slug and converts it to a folder path by replacing specific characters
/// with their corresponding string representations. The replacements are as follows:
/// - `*` is replaced with `_star_`
/// - `::` is replaced with `_doublecolon_`
/// - `:` is replaced with `_colon_`
/// - `?` is replaced with `_question_`
///
/// The resulting string is then converted to lowercase and returned as a `PathBuf`.
///
/// # Arguments
///
/// * `slug` - A string slice that holds the URL slug to be converted.
///
/// # Returns
///
/// * `PathBuf` - Returns a `PathBuf` representing the converted folder path.
pub fn url_to_folder_path(slug: &str) -> PathBuf {
    PathBuf::from(
        slug.replace('*', "_star_")
            .replace("::", "_doublecolon_")
            .replace(':', "_colon_")
            .replace('?', "_question_")
            .to_lowercase(),
    )
}

/// Strips the locale from a URL and returns the locale and the remaining URL.
///
/// This function takes a URL and attempts to extract the locale from it. If the URL starts with a locale,
/// the function returns the locale and the remaining part of the URL. If the URL does not contain a locale,
/// it returns `None` for the locale and the original URL.
///
/// # Arguments
///
/// * `url` - A string slice that holds the URL to be processed.
///
/// # Returns
///
/// * `(Option<Locale>, &str)` - Returns a tuple where the first element is an `Option<Locale>` containing the locale
///   if found, and the second element is the remaining part of the URL.
pub(crate) fn strip_locale_from_url(url: &str) -> (Option<Locale>, &str) {
    if url.len() < 2 || !url.starts_with('/') {
        return (None, url);
    }
    let i = url[1..].find('/').map(|i| i + 1).unwrap_or(url.len());
    let locale = Locale::from_str(&url[1..i]).ok();
    (locale, &url[if locale.is_none() { 0 } else { i }..])
}

/// Represents metadata extracted from a URL.
///
/// The `UrlMeta` struct holds various pieces of data that are extracted from a URL,
/// including the folder path, slug, locale, and page category.
///
/// # Fields
///
/// * `folder_path` - A `PathBuf` that holds the folder path corresponding to the URL.
/// * `slug` - A string slice that holds the slug extracted from the URL.
/// * `locale` - A `Locale` that specifies the locale extracted from the URL.
/// * `page_category` - A `PageCategory` that specifies the category of the page extracted from the URL.
pub struct UrlMeta<'a> {
    pub folder_path: PathBuf,
    pub slug: &'a str,
    pub locale: Locale,
    pub page_category: PageCategory,
}

/// Extracts metadata from a URL, including the folder path, slug, locale, and page category.
///
/// This function parses the given URL to extract various pieces of metadata, such as the locale,
/// slug, and page category. It supports different URL structures for documentation pages, blog posts,
/// curriculum pages, community spotlight pages, single-page applications (SPA), and generic pages.
/// If the URL does not match any known patterns, it returns an `UrlError::InvalidUrl` error.
///
/// # Arguments
///
/// * `url` - A string slice that holds the URL to be processed.
///
/// # Returns
///
/// * `Result<UrlMeta<'_>, UrlError>` - Returns a `UrlMeta` struct containing the extracted metadata if successful,
///   or an `UrlError` if the URL is invalid or does not match any known patterns.
///
/// # Errors
///
/// This function will return an error if:
/// - The URL does not contain a recognizable locale.
/// - The URL does not match any known patterns for documentation pages, blog posts, curriculum pages, etc.
pub fn url_meta_from(url: &str) -> Result<UrlMeta<'_>, UrlError> {
    let mut split = url[..url.find('#').unwrap_or(url.len())]
        .splitn(4, '/')
        .skip(1);
    let locale: Locale = Locale::from_str(split.next().unwrap_or_default())?;
    let tail: Vec<_> = split.collect();
    let (page_category, slug) = match tail.as_slice() {
        ["docs", tail] => (PageCategory::Doc, *tail),
        ["blog"] | ["blog", ""] if locale == Default::default() => (PageCategory::SPA, "blog"),
        ["blog", tail] if locale == Default::default() => (PageCategory::BlogPost, *tail),
        ["curriculum", tail] if locale == Default::default() => (PageCategory::Curriculum, *tail),
        ["community", tail] if locale == Default::default() && tail.starts_with("spotlight") => {
            (PageCategory::ContributorSpotlight, *tail)
        }
        _ => {
            let (_, slug) = strip_locale_from_url(url);
            let slug = slug.strip_prefix('/').unwrap_or(slug);
            if SPA::is_spa(slug, locale) {
                (PageCategory::SPA, slug)
            } else if Generic::is_generic(slug, locale) {
                (PageCategory::GenericPage, slug)
            } else {
                return Err(UrlError::InvalidUrl);
            }
        }
    };
    let folder_path = url_to_folder_path(slug);
    Ok(UrlMeta {
        folder_path,
        slug,
        locale,
        page_category,
    })
}

/// Extracts the `Locale` from a given URL path.
///
/// This function takes a URL path as input and attempts to parse the first
/// path segment as a locale. If the first segment corresponds to a valid locale,
/// it returns `Some(Locale)`, otherwise, it returns `None`.
///
/// # Arguments
///
/// * `url` - A string slice that holds the URL path, potentially with a leading `/`.
///
/// # Returns
///
/// * `Option<Locale>` - `Some(Locale)` if the first path segment is a valid locale,
///   or `None` if it isn't.
///
/// # Examples
///
/// ```
/// # use rari_doc::resolve::locale_from_url;
/// # use rari_types::locale::Locale;
///
/// assert_eq!(locale_from_url("/en-US/some/path"), Some(Locale::EnUs));
/// assert_eq!(locale_from_url("fr/page"), Some(Locale::Fr));
/// assert_eq!(locale_from_url("invalid/path"), None);
/// ```
pub fn locale_from_url(url: &str) -> Option<Locale> {
    let url = url.strip_prefix("/").unwrap_or(url);
    url.split_once('/')
        .and_then(|(l, _)| Locale::from_str(l).ok())
}

/// Replaces the `Locale` in a given URL path.
///
/// This function takes a URL path and Locale as input and attempts to parse the first
/// path segment as a locale. If the first segment corresponds to a valid locale,
/// which is different from the provided locales it substitutes the parse locale with
/// the provided locale and returns the new new URL, otherwise, it returns `None`.
///
/// # Arguments
///
/// * `url` - A string slice that holds the URL path.
/// * `locale` - The provided locale to use.
///
/// # Returns
///
/// * `Option<String>` - `Some(url)` if the first path segment is a valid locale, and
///   not the one already in the url, otherwise `None`.
///
/// # Examples
///
/// ```
/// # use rari_doc::resolve::url_with_locale;
/// # use rari_types::locale::Locale;
///
/// assert_eq!(url_with_locale("/en-US/some/path", Locale::Fr).as_deref(), Some("/fr/some/path"));
/// assert_eq!(url_with_locale("/en-US/", Locale::ZhTw).as_deref(), Some("/zh-TW/"));
/// assert_eq!(url_with_locale("/invalid/path", Locale::Ja), None);
/// ```
pub fn url_with_locale(url: &str, locale: Locale) -> Option<String> {
    if let Some(url) = url.strip_prefix("/") {
        let first_slash = url.find('/').unwrap_or(url.len());
        let locale_str = &url[..first_slash];
        let current_locale = Locale::from_str(locale_str).ok();
        if current_locale.is_none() || current_locale == Some(locale) {
            return None;
        }

        return Some(concat_strs!("/", locale.as_url_str(), &url[first_slash..]));
    }

    None
}

/// Builds a URL for a given slug, locale, and page category.
///
/// This function constructs a URL based on the provided slug, locale, and page category.
/// It uses different URL patterns for different page categories, such as documentation pages,
/// blog posts, single-page applications (SPA), curriculum pages, contributor spotlight pages,
/// and generic pages. If the page category is SPA and the slug does not correspond to a valid SPA,
/// it returns a `DocError::PageNotFound` error.
///
/// # Arguments
///
/// * `slug` - A string slice that holds the slug of the page.
/// * `locale` - A `Locale` that specifies the locale of the page.
/// * `typ` - A `PageCategory` that specifies the category of the page.
///
/// # Returns
///
/// * `Result<String, DocError>` - Returns the constructed URL as a `String` if successful,
///   or a `DocError` if an error occurs (e.g., if the SPA slug is not found).
///
/// # Errors
///
/// This function will return an error if:
/// - The page category is SPA and the slug does not correspond to a valid SPA.
pub fn build_url(slug: &str, locale: Locale, typ: PageCategory) -> Result<String, DocError> {
    Ok(match typ {
        PageCategory::Doc => concat_strs!("/", locale.as_url_str(), "/docs/", slug),
        PageCategory::BlogPost => concat_strs!("/", locale.as_url_str(), "/blog/", slug, "/"),
        PageCategory::SPA => SPA::from_slug(slug, locale)
            .ok_or(DocError::PageNotFound(slug.to_string(), PageCategory::SPA))?
            .url()
            .to_owned(),
        PageCategory::Curriculum => {
            concat_strs!("/", locale.as_url_str(), "/curriculum/", slug, "/")
        }
        PageCategory::ContributorSpotlight => {
            concat_strs!("/", locale.as_url_str(), "/community/spotlight/", slug)
        }
        PageCategory::GenericPage => concat_strs!("/", locale.as_url_str(), "/", slug),
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_url_to_path() -> Result<(), UrlError> {
        let url = "/en-US/docs/Web/HTML";
        let UrlMeta {
            folder_path,
            slug,
            locale,
            ..
        } = url_meta_from(url)?;
        assert_eq!(locale, Locale::EnUs);
        assert_eq!(folder_path, PathBuf::from("web/html"));
        assert_eq!(slug, "Web/HTML");
        Ok(())
    }

    #[test]
    fn test_from_url() {
        let url = "/en-US/docs/Web";
        let (locale, url) = strip_locale_from_url(url);
        assert_eq!(Some(Locale::EnUs), locale);
        assert_eq!("/docs/Web", url);
    }
}
