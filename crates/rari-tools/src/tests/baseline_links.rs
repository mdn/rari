use rari_doc::html::rewriter::post_process_html;
use rari_doc::pages::page::Page;
use rari_types::locale::Locale;
use serial_test::file_serial;

use super::fixtures::docs::DocFixtures;

const LINKING_PAGE: &str = "Web/API/Baseline-Linking";
const NO_STATUS: &str = "Web/API/Baseline-No-Status";
const LOW: &str = "Web/API/Baseline-Low";
const HIGH: &str = "Web/API/Baseline-High";
const DEPRECATED: &str = "Web/API/Baseline-Deprecated";
const REMOVING: &str = "Web/API/Baseline-Removing";

fn fixtures() -> DocFixtures {
    let docs = DocFixtures::new(
        &[
            LINKING_PAGE.to_string(),
            NO_STATUS.to_string(),
            LOW.to_string(),
            HIGH.to_string(),
            DEPRECATED.to_string(),
            REMOVING.to_string(),
        ],
        Locale::EnUs,
    );
    DocFixtures::create_doc_with_front_matter(LOW, Locale::EnUs, "browser-compat: api.low\n", "");
    DocFixtures::create_doc_with_front_matter(HIGH, Locale::EnUs, "browser-compat: api.high\n", "");
    DocFixtures::create_doc_with_front_matter(
        DEPRECATED,
        Locale::EnUs,
        "status:\n  - deprecated\n",
        "",
    );
    DocFixtures::create_doc_with_front_matter(
        REMOVING,
        Locale::EnUs,
        "browser-compat: api.removing\n",
        "",
    );
    docs
}

fn rewrite(html: &str) -> String {
    let page = Page::from_url(&format!("/en-US/docs/{LINKING_PAGE}")).expect("Failed to load page");
    let Page::Doc(doc) = page else {
        panic!("Expected Doc page");
    };
    post_process_html(html, &doc, false).expect("Failed to post-process HTML")
}

#[test]
#[file_serial(file_fixtures)]
fn a_link_is_marked_with_its_baseline_status() {
    let _docs = fixtures();

    let out = rewrite(&format!(r#"<p><a href="/en-US/docs/{LOW}">thing</a></p>"#));
    assert!(out.contains(r#"data-baseline="low""#));
}

#[test]
#[file_serial(file_fixtures)]
fn a_link_to_a_widely_available_feature_is_not_marked() {
    let _docs = fixtures();

    let out = rewrite(&format!(r#"<p><a href="/en-US/docs/{HIGH}">thing</a></p>"#));
    assert!(!out.contains("data-baseline"));
}

#[test]
#[file_serial(file_fixtures)]
fn a_link_to_a_deprecated_page_is_marked_discouraged() {
    let _docs = fixtures();

    let out = rewrite(&format!(
        r#"<p><a href="/en-US/docs/{DEPRECATED}">thing</a></p>"#
    ));
    assert!(out.contains(r#"data-baseline="discouraged""#));
}

#[test]
#[file_serial(file_fixtures)]
fn a_link_to_a_feature_being_removed_is_marked_removing() {
    let _docs = fixtures();

    let out = rewrite(&format!(
        r#"<p><a href="/en-US/docs/{REMOVING}">thing</a></p>"#
    ));
    assert!(out.contains(r#"data-baseline="removing""#));
}

#[test]
#[file_serial(file_fixtures)]
fn a_link_to_a_page_without_a_status_is_not_marked() {
    let _docs = fixtures();

    let out = rewrite(&format!(
        r#"<p><a href="/en-US/docs/{NO_STATUS}">thing</a></p>"#
    ));
    assert!(!out.contains("page-not-created"));
    assert!(!out.contains("data-baseline"));
}

#[test]
#[file_serial(file_fixtures)]
fn a_link_to_a_missing_page_is_not_marked() {
    let _docs = fixtures();

    let out = rewrite(r#"<p><a href="/en-US/docs/Web/API/Baseline-Missing">thing</a></p>"#);
    assert!(out.contains("page-not-created"));
    assert!(!out.contains("data-baseline"));
}
