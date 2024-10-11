use std::borrow::Cow;

use rari_doc::pages::page;
use rari_doc::resolve::{url_meta_from, UrlMeta};

use crate::error::ToolError;
use crate::redirects::add_redirects;

pub fn add_redirect(from_url: &str, to_url: &str) -> Result<(), ToolError> {
    do_add_redirect(from_url, to_url)
    // console output
}

fn do_add_redirect(from_url: &str, to_url: &str) -> Result<(), ToolError> {
    validate_args(from_url, to_url)?;
    let UrlMeta {
        locale: from_locale,
        ..
    } = url_meta_from(from_url)?;
    if !to_url.starts_with("http") {
        if from_url == to_url {
            return Err(ToolError::InvalidRedirectToURL(format!(
                "redirect url is the same as the from url: {to_url}"
            )));
        }

        if !page::Page::exists(to_url) {
            return Err(ToolError::InvalidRedirectToURL(format!(
                "redirect url does not exist: {to_url}"
            )));
        }
        let UrlMeta {
            locale: to_locale, ..
        } = url_meta_from(to_url)?;
        if from_locale != to_locale {
            return Err(ToolError::InvalidRedirectToURL(format!(
                "redirect url locales do not match: {from_locale} != {to_locale}"
            )));
        }
    }
    add_redirects(from_locale, &[(from_url.to_owned(), to_url.to_owned())])?;
    Ok(())
}

fn validate_args(from_url: &str, to_url: &str) -> Result<(), ToolError> {
    if from_url.is_empty() {
        return Err(ToolError::InvalidUrl(Cow::Borrowed(
            "from_url cannot be empty",
        )));
    }
    if to_url.is_empty() {
        return Err(ToolError::InvalidUrl(Cow::Borrowed(
            "to_url cannot be empty",
        )));
    }
    if from_url.contains("#") {
        return Err(ToolError::InvalidUrl(Cow::Borrowed(
            "from_url cannot contain '#'",
        )));
    }
    Ok(())
}

// These tests use file system fixtures to simulate content and translated content.
// The file system is a shared resource, so we force tests to be run serially,
// to avoid concurrent fixture management issues.
// Using `file_serial` as a synchonization lock, we run all tests using
// the same `key` (here: file_fixtures) to be serialized across modules.
#[cfg(test)]
use serial_test::file_serial;
#[cfg(test)]
#[file_serial(file_fixtures)]
mod test {
    use rari_types::locale::Locale;

    use super::*;
    use crate::tests::fixtures::docs::DocFixtures;
    use crate::tests::fixtures::redirects::RedirectFixtures;
    use crate::utils::test_utils::get_redirects_map;

    #[test]
    fn test_add_redirect() {
        let slugs = vec!["Web/API/ExampleOne".to_string()];
        let _docs = DocFixtures::new(&slugs, Locale::EnUs);
        let _redirects = RedirectFixtures::new(&vec![], Locale::EnUs);

        let result = do_add_redirect(
            "/en-US/docs/Web/API/ExampleGone",
            "/en-US/docs/Web/API/ExampleOne",
        );
        println!("{:?}", result);
        assert!(result.is_ok());

        let redirects = get_redirects_map(Locale::EnUs);
        assert_eq!(redirects.len(), 1);
        assert!(redirects.contains_key("/en-US/docs/Web/API/ExampleGone"));
        assert_eq!(
            redirects.get("/en-US/docs/Web/API/ExampleGone").unwrap(),
            "/en-US/docs/Web/API/ExampleOne"
        );
    }
}
