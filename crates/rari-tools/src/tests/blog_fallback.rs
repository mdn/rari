use rari_doc::error::DocError;
use rari_doc::pages::page::{Page, PageCategory};
use rari_types::locale::Locale;
use serial_test::file_serial;

use super::fixtures::blog::BlogFixtures;

const SLUG: &str = "test-post";
const URL: &str = "/en-US/blog/test-post/";

#[test]
#[file_serial(blog_fixtures)]
fn blog_post_no_fallback_returns_page_not_found_for_non_en_us() {
    let _blog = BlogFixtures::new(&[SLUG.to_string()]);

    let err = Page::internal_from_url(URL, Some(Locale::Fr), false)
        .expect_err("non-en-US locale without fallback should fail");

    match err {
        DocError::PageNotFound(_, PageCategory::BlogPost) => {}
        other => panic!("expected PageNotFound(_, BlogPost), got {other:?}"),
    }
}

#[test]
#[file_serial(blog_fixtures)]
fn blog_post_falls_back_to_en_us_for_non_en_us_locale() {
    let _blog = BlogFixtures::new(&[SLUG.to_string()]);

    let page = Page::from_url_with_locale_and_fallback(URL, Locale::Fr)
        .expect("fallback should locate the en-US blog post");

    assert!(matches!(page, Page::BlogPost(_)));
}

#[test]
#[file_serial(blog_fixtures)]
fn blog_post_en_us_still_resolves() {
    let _blog = BlogFixtures::new(&[SLUG.to_string()]);

    let page = Page::from_url(URL).expect("en-US lookup should succeed");

    assert!(matches!(page, Page::BlogPost(_)));
}
