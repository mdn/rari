use rari_doc::html::rewriter::post_process_html;
use rari_doc::pages::page::Page;
use rari_types::locale::Locale;
use serial_test::file_serial;

use super::fixtures::docs::DocFixtures;

/// Tests that image URLs are rewritten to point to en-US when the image
/// doesn't exist in the translated locale but does exist in en-US.
#[test]
#[file_serial(file_fixtures)]
fn test_image_url_fallback_to_en_us() {
    let slug = "Test/Image-Fallback";

    // Create en-US doc (this creates the necessary directory structure)
    let _en_us_docs = DocFixtures::new(&[slug.to_string()], Locale::EnUs);

    // Create an image in the en-US doc folder
    DocFixtures::create_image(slug, Locale::EnUs, "test-image.png");

    // Create French doc with content that references the image
    // but WITHOUT copying the image to the French folder
    let _fr_docs = DocFixtures::new(&[slug.to_string()], Locale::Fr);
    DocFixtures::create_doc_with_content(slug, Locale::Fr, "![Test image](test-image.png)");

    // Load the French page
    let fr_url = format!("/fr/docs/{slug}");
    let page = Page::from_url(&fr_url).expect("Failed to load French page");

    // Get the Doc from the Page enum
    let doc = match &page {
        Page::Doc(d) => d.clone(),
        _ => panic!("Expected Doc page"),
    };

    // Process HTML to trigger the image URL rewriting
    let html = "<img src=\"test-image.png\">";
    let result = post_process_html(html, &doc, false).expect("Failed to post-process HTML");

    // Verify the image src was rewritten to point to en-US
    assert!(
        result.contains("src=\"/en-US/docs/Test/Image-Fallback/test-image.png\""),
        "Expected image src to be rewritten to en-US path, got: {}",
        result
    );
}

/// Tests that image URLs are NOT rewritten when the image exists in the
/// translated locale.
#[test]
#[file_serial(file_fixtures)]
fn test_image_url_not_rewritten_when_exists_in_locale() {
    let slug = "Test/Image-Local";

    // Create en-US doc with image
    let _en_us_docs = DocFixtures::new(&[slug.to_string()], Locale::EnUs);
    DocFixtures::create_image(slug, Locale::EnUs, "test-image.png");

    // Create French doc WITH the image (copied to French folder)
    let _fr_docs = DocFixtures::new(&[slug.to_string()], Locale::Fr);
    DocFixtures::create_image(slug, Locale::Fr, "test-image.png");
    DocFixtures::create_doc_with_content(slug, Locale::Fr, "![Test image](test-image.png)");

    // Load the French page
    let fr_url = format!("/fr/docs/{slug}");
    let page = Page::from_url(&fr_url).expect("Failed to load French page");

    let doc = match &page {
        Page::Doc(d) => d.clone(),
        _ => panic!("Expected Doc page"),
    };

    // Process HTML
    let html = "<img src=\"test-image.png\">";
    let result = post_process_html(html, &doc, false).expect("Failed to post-process HTML");

    // Verify the image src was NOT rewritten (should stay as French path)
    assert!(
        result.contains("src=\"/fr/docs/Test/Image-Local/test-image.png\""),
        "Expected image src to remain as French path, got: {}",
        result
    );
}

/// Tests that en-US pages don't get URL rewriting (no fallback needed).
#[test]
#[file_serial(file_fixtures)]
fn test_image_url_no_fallback_for_en_us() {
    let slug = "Test/Image-EnUs";

    // Create en-US doc with image
    let _en_us_docs = DocFixtures::new(&[slug.to_string()], Locale::EnUs);
    DocFixtures::create_image(slug, Locale::EnUs, "test-image.png");

    // Load the en-US page
    let en_us_url = format!("/en-US/docs/{slug}");
    let page = Page::from_url(&en_us_url).expect("Failed to load en-US page");

    let doc = match &page {
        Page::Doc(d) => d.clone(),
        _ => panic!("Expected Doc page"),
    };

    // Process HTML
    let html = "<img src=\"test-image.png\">";
    let result = post_process_html(html, &doc, false).expect("Failed to post-process HTML");

    // Verify the image src points to en-US (its own locale)
    assert!(
        result.contains("src=\"/en-US/docs/Test/Image-EnUs/test-image.png\""),
        "Expected image src to be en-US path, got: {}",
        result
    );
}
