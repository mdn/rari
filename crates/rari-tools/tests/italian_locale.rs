use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use rari_tools::sync_translated_content::sync_translated_content;
use rari_types::globals::SETTINGS;
use rari_types::locale::Locale;
use rari_types::settings::{Settings, TranslatedContentSource};

fn write_doc(root: &Path, locale: Locale, title: &str, slug: &str) {
    let locale_dir = root.join(locale.as_folder_str());
    let path = locale_dir.join(slug.to_lowercase()).join("index.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        format!("---\ntitle: {title}\nslug: {slug}\n---\n\n{title}\n"),
    )
    .unwrap();
    fs::write(locale_dir.join("_redirects.txt"), "").unwrap();
    fs::write(locale_dir.join("_wikihistory.json"), "{}\n").unwrap();
}

#[test]
fn absent_optional_italian_skips_then_syncs_when_present() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let fixture = std::env::temp_dir().join(format!("rari-optional-sync-{suffix}"));
    let content = fixture.join("content");
    let italian = fixture.join("translated-content-it");
    write_doc(&content, Locale::EnUs, "English", "Web");
    fs::write(
        content.join("en-us/_redirects.txt"),
        "/en-US/docs/OldWeb\t/en-US/docs/Web\n",
    )
    .unwrap();
    let mut settings = Settings {
        content_root: content,
        optional_translated_locales: vec![Locale::It],
        reader_ignores_gitignore: true,
        ..Settings::default()
    };
    settings.translated_content_sources.insert(
        Locale::It,
        TranslatedContentSource {
            root: italian.clone(),
            repository: "translated-content-it".into(),
        },
    );
    SETTINGS.set(settings).unwrap();

    assert!(Locale::translated().contains(&Locale::It));
    assert!(
        sync_translated_content(&[Locale::It], false)
            .unwrap()
            .is_empty()
    );
    assert!(!italian.exists());

    write_doc(&italian, Locale::It, "Italian", "OldWeb");
    fs::write(
        italian.join("it/_wikihistory.json"),
        "{\"OldWeb\":{\"owner\":\"it\"}}\n",
    )
    .unwrap();
    assert!(
        Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(&italian)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE")
            .env_remove("GIT_COMMON_DIR")
            .env_remove("GIT_PREFIX")
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("git")
            .args(["add", "."])
            .current_dir(&italian)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE")
            .env_remove("GIT_COMMON_DIR")
            .env_remove("GIT_PREFIX")
            .status()
            .unwrap()
            .success()
    );

    let result = sync_translated_content(&[Locale::It], false).unwrap();
    assert_eq!(result[&Locale::It].moved_docs, 1);
    assert!(italian.join("it/web/index.md").exists());
    assert!(!italian.join("it/oldweb/index.md").exists());
    assert!(
        fs::read_to_string(italian.join("it/_redirects.txt"))
            .unwrap()
            .contains("/it/docs/OldWeb\t/it/docs/Web")
    );
    let history: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(italian.join("it/_wikihistory.json")).unwrap())
            .unwrap();
    assert!(history.get("Web").is_some());
    fs::remove_dir_all(fixture).unwrap();
}
