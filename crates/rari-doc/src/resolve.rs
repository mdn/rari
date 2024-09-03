use std::path::PathBuf;
use std::str::FromStr;

use rari_types::locale::Locale;

use crate::error::UrlError;
use crate::pages::page::{PageCategory, PageLike};
use crate::pages::types::dummy::Dummy;

pub fn url_to_path_buf(slug: &str) -> PathBuf {
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

pub fn url_path_to_path_buf(url_path: &str) -> Result<(PathBuf, Locale, PageCategory), UrlError> {
    let mut split = url_path[..url_path.find('#').unwrap_or(url_path.len())]
        .splitn(4, '/')
        .skip(1);
    let locale: Locale = Locale::from_str(split.next().unwrap_or_default())?;
    let typ = match split.next() {
        Some("docs") => PageCategory::Doc,
        Some("blog") => PageCategory::BlogPost,
        Some("curriculum") => PageCategory::Curriculum,
        Some("community") => match split.next() {
            Some(slug) if slug.starts_with("spotlight/") => PageCategory::ContributorSpotlight,
            _ => return Err(UrlError::InvalidUrl),
        },
        _ => return Err(UrlError::InvalidUrl),
    };
    let path = url_to_path_buf(split.last().unwrap_or_default());
    Ok((path, locale, typ))
}

pub fn build_url(slug: &str, locale: &Locale, typ: PageCategory) -> String {
    match typ {
        PageCategory::Doc => format!("/{}/docs/{}", locale.as_url_str(), slug),
        PageCategory::BlogPost => format!("/{}/blog/{}/", locale.as_url_str(), slug),
        PageCategory::Dummy => Dummy::from_sulg(slug, *locale).url().to_owned(),
        PageCategory::Curriculum => format!("/{}/curriculum/{}/", locale.as_url_str(), slug),
        PageCategory::ContributorSpotlight => {
            format!("/{}/community/spotlight/{}", locale.as_url_str(), slug)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_url_to_path() -> Result<(), UrlError> {
        let url = "/en-US/docs/Web/HTML";
        let (path, locale, _typ) = url_path_to_path_buf(url)?;
        assert_eq!(locale, Locale::EnUs);
        assert_eq!(path, PathBuf::from("web/html"));
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
