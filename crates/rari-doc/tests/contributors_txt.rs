use std::fs;

use rari_doc::build::build_single_doc_with;
use rari_doc::pages::page::Page;
use rari_types::globals::SETTINGS;
use rari_types::settings::Settings;

#[test]
fn skip_contributors_txt() {
    struct Case {
        name: &'static str,
        slug: &'static str,
        skip: bool,
        expect_contributors_txt: bool,
    }
    let cases = vec![
        Case {
            name: "default writes contributors.txt",
            slug: "Web/Default",
            skip: false,
            expect_contributors_txt: true,
        },
        Case {
            name: "skip omits only contributors.txt",
            slug: "Web/Skipped",
            skip: true,
            expect_contributors_txt: false,
        },
    ];

    let content = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();
    fs::create_dir(content.path().join("en-us")).unwrap();
    fs::write(content.path().join("en-us/_redirects.txt"), "").unwrap();
    for case in &cases {
        let doc_dir = content.path().join("en-us").join(case.slug.to_lowercase());
        fs::create_dir_all(&doc_dir).unwrap();
        fs::write(
            doc_dir.join("index.md"),
            format!("---\ntitle: Example\nslug: {}\n---\n\nHello\n", case.slug),
        )
        .unwrap();
    }
    SETTINGS
        .set(Settings {
            content_root: content.path().to_path_buf(),
            build_out_root: Some(out.path().to_path_buf()),
            reader_ignores_gitignore: true,
            ..Default::default()
        })
        .unwrap();

    for case in &cases {
        let page = Page::from_url(&format!("/en-US/docs/{}", case.slug)).unwrap();
        build_single_doc_with(&page, case.skip).unwrap();

        let built = out.path().join("en-us/docs").join(case.slug.to_lowercase());
        assert!(
            built.join("index.json").exists(),
            "{}: index.json missing",
            case.name
        );
        assert!(
            built.join("metadata.json").exists(),
            "{}: metadata.json missing",
            case.name
        );
        assert_eq!(
            built.join("contributors.txt").exists(),
            case.expect_contributors_txt,
            "{}: contributors.txt presence",
            case.name
        );
    }
}
