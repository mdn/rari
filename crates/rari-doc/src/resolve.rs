use std::path::PathBuf;
use std::str::FromStr;

use rari_types::locale::Locale;
use rari_utils::concat_strs;

use crate::error::{DocError, UrlError};
use crate::pages::page::{PageCategory, PageLike};
use crate::pages::types::generic::GenericPage;
use crate::pages::types::spa::SPA;

pub fn url_to_folder_path(slug: &str) -> PathBuf {
    PathBuf::from(
        slug.replace('*', "_star_")
            .replace("::", "_doublecolon_")
            .replace(':', "_colon_")
            .replace('?', "_question_")
            .to_lowercase(),
    )
}

pub fn strip_locale_from_url(url: &str) -> (Option<Locale>, &str) {
    if url.len() < 2 || !url.starts_with('/') {
        return (None, url);
    }
    let i = url[1..].find('/').map(|i| i + 1).unwrap_or(url.len());
    let locale = Locale::from_str(&url[1..i]).ok();
    (locale, &url[i..])
}

pub struct UrlMeta<'a> {
    pub folder_path: PathBuf,
    pub slug: &'a str,
    pub locale: Locale,
    pub page_category: PageCategory,
}

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
        ["community", ..] => return Err(UrlError::InvalidUrl),
        _ => {
            let (_, slug) = strip_locale_from_url(url);
            let slug = slug.strip_prefix('/').unwrap_or(slug);
            if SPA::is_spa(slug, locale) {
                (PageCategory::SPA, slug)
            } else if GenericPage::is_generic(slug, locale) {
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
        assert_eq!(slug, "web/html");
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
