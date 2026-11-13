use std::fs;

use rari_doc::build::build_single_doc;
use rari_doc::pages::page::Page;
use rari_types::globals::SETTINGS;
use rari_types::settings::Settings;

#[test]
fn skip_contributors_txt_omits_only_that_file() {
    let content = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();
    fs::create_dir(content.path().join("en-us")).unwrap();
    fs::write(content.path().join("en-us/_redirects.txt"), "").unwrap();
    let doc_dir = content.path().join("en-us/web/example");
    fs::create_dir_all(&doc_dir).unwrap();
    fs::write(
        doc_dir.join("index.md"),
        "---\ntitle: Example\nslug: Web/Example\n---\n\nHello\n",
    )
    .unwrap();
    SETTINGS
        .set(Settings {
            content_root: content.path().to_path_buf(),
            build_out_root: Some(out.path().to_path_buf()),
            reader_ignores_gitignore: true,
            skip_contributors_txt: true,
            ..Default::default()
        })
        .unwrap();

    let page = Page::from_url("/en-US/docs/Web/Example").unwrap();
    build_single_doc(&page).unwrap();

    let built = out.path().join("en-us/docs/web/example");
    assert!(built.join("index.json").exists(), "index.json missing");
    assert!(
        built.join("metadata.json").exists(),
        "metadata.json missing"
    );
    assert!(
        !built.join("contributors.txt").exists(),
        "contributors.txt should be skipped"
    );
}
