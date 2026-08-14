use rari_doc::html::rewriter::post_process_html;
use rari_doc::pages::page::Page;
use rari_types::locale::Locale;
use serial_test::file_serial;

use super::fixtures::docs::DocFixtures;

const LINKING: &str = "Web/API/Baseline-Linking";
const DISCOURAGED: &str = "Web/API/Baseline-Discouraged";
const FINE: &str = "Web/API/Baseline-Fine";
const OUTSIDE: &str = "Mozilla/Add-ons/Baseline-Deprecated";

fn fixtures() -> DocFixtures {
    let docs = DocFixtures::new(
        &[
            LINKING.to_string(),
            DISCOURAGED.to_string(),
            FINE.to_string(),
            OUTSIDE.to_string(),
        ],
        Locale::EnUs,
    );
    DocFixtures::create_doc_with_status(DISCOURAGED, Locale::EnUs, &["deprecated"], "");
    DocFixtures::create_doc_with_status(OUTSIDE, Locale::EnUs, &["deprecated"], "");
    docs
}

fn rewrite(html: &str) -> String {
    let page = Page::from_url(&format!("/en-US/docs/{LINKING}")).expect("Failed to load page");
    let doc = match &page {
        Page::Doc(doc) => doc.clone(),
        _ => panic!("Expected Doc page"),
    };
    post_process_html(html, &doc, false).expect("Failed to post-process HTML")
}

fn link_to(slug: &str) -> String {
    format!(r#"<p><a href="/en-US/docs/{slug}">thing</a></p>"#)
}

#[test]
#[file_serial(file_fixtures)]
fn a_link_to_a_discouraged_page_is_marked() {
    let _docs = fixtures();

    let out = rewrite(&link_to(DISCOURAGED));
    assert!(out.contains(r#"data-baseline="discouraged""#), "{out}");
    assert!(
        out.contains(r#"title="This feature is discouraged.""#),
        "{out}"
    );
}

#[test]
#[file_serial(file_fixtures)]
fn a_link_which_already_has_a_title_keeps_it() {
    let _docs = fixtures();

    let out = rewrite(&format!(
        r#"<p><a href="/en-US/docs/{DISCOURAGED}" title="&lt;menclose&gt;">thing</a></p>"#
    ));
    assert!(out.contains(r#"data-baseline="discouraged""#), "{out}");
    assert!(out.contains(r#"title="&lt;menclose&gt;""#), "{out}");
    assert!(!out.contains("discouraged.\""), "{out}");
}

#[test]
#[file_serial(file_fixtures)]
fn a_link_to_a_page_without_a_status_is_not_marked() {
    let _docs = fixtures();

    let out = rewrite(&link_to(FINE));
    assert!(!out.contains("data-baseline"), "{out}");
}

#[test]
#[file_serial(file_fixtures)]
fn a_link_outside_the_mocked_sections_is_not_marked() {
    let _docs = fixtures();

    let out = rewrite(&link_to(OUTSIDE));
    assert!(!out.contains("data-baseline"), "{out}");
}

#[test]
#[file_serial(file_fixtures)]
fn a_templ_link_to_a_discouraged_page_is_marked() {
    let _docs = fixtures();

    let out = rewrite(&format!(
        r#"<p><a data-templ-link href="/en-US/docs/{DISCOURAGED}">thing</a></p>"#
    ));
    assert!(out.contains(r#"data-baseline="discouraged""#), "{out}");
}

#[test]
#[file_serial(file_fixtures)]
fn a_link_inside_an_svg_is_not_marked() {
    let _docs = fixtures();

    let out = rewrite(&format!(
        r#"<svg viewBox="0 0 10 10"><a href="/en-US/docs/{DISCOURAGED}"><text>thing</text></a></svg>"#
    ));
    assert!(!out.contains("data-baseline"), "{out}");
}

#[test]
#[file_serial(file_fixtures)]
fn a_link_to_a_page_which_does_not_exist_is_not_marked() {
    let _docs = fixtures();

    let out = rewrite(&link_to("Web/API/Baseline-Missing"));
    assert!(out.contains("page-not-created"), "{out}");
    assert!(!out.contains("data-baseline"), "{out}");
}
