use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use rari_tools::redirects::{fix_redirects, validate_redirects};
use rari_tools::sync_translated_content::sync_translated_content;
use rari_types::globals::SETTINGS;
use rari_types::locale::Locale;
use rari_types::settings::{Settings, TranslatedContentSource};

#[test]
fn mapped_only_locale_participates_in_redirects_and_sync() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let fixture = std::env::temp_dir().join(format!("rari-mapped-only-{suffix}"));
    let content = fixture.join("content");
    let dedicated = fixture.join("translated-content-de");
    for (root, locale, title) in [
        (&content, Locale::EnUs, "English"),
        (&dedicated, Locale::De, "German"),
    ] {
        let dir = root.join(locale.as_folder_str());
        fs::create_dir_all(dir.join("web")).unwrap();
        fs::write(
            dir.join("web/index.md"),
            format!("---\ntitle: {title}\nslug: Web\n---\n\n{title}\n"),
        )
        .unwrap();
        fs::write(dir.join("_wikihistory.json"), "{}\n").unwrap();
        fs::write(dir.join("_redirects.txt"), "").unwrap();
    }
    fs::write(
        dedicated.join("de/_redirects.txt"),
        "/de/docs/OldWeb\t/de/docs/Web\n",
    )
    .unwrap();

    let mut settings = Settings {
        content_root: content,
        reader_ignores_gitignore: true,
        ..Settings::default()
    };
    settings.translated_content_sources.insert(
        Locale::De,
        TranslatedContentSource {
            root: dedicated.clone(),
            repository: "translated-content-de".into(),
        },
    );
    SETTINGS.set(settings).unwrap();

    assert!(fix_redirects(Some(&[Locale::Es])).is_err());

    fix_redirects(None).unwrap();
    let redirects = fs::read_to_string(dedicated.join("de/_redirects.txt")).unwrap();
    assert!(redirects.contains("/de/docs/OldWeb\t/de/docs/Web"));
    validate_redirects(Some(&[Locale::De])).unwrap();
    validate_redirects(None).unwrap();
    let result = sync_translated_content(&[Locale::De], false).unwrap();
    assert_eq!(result[&Locale::De].total_docs, 1);
    fs::remove_dir_all(fixture).unwrap();
}
