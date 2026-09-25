use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use rari_doc::pages::page::{Page, PageLike};
use rari_doc::pages::types::doc::Doc;
use rari_doc::reader::read_docs_parallel;
use rari_doc::utils::root_for_locale;
use rari_tools::sync_translated_content::sync_translated_content;
use rari_types::HistoryEntry;
use rari_types::globals::{SETTINGS, git_history, translated_content_locale_paths};
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
fn sync_reads_and_writes_dedicated_locale_root() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let fixture = std::env::temp_dir().join(format!("rari-multi-root-{suffix}"));
    let content = fixture.join("content");
    let primary = fixture.join("translated-content");
    let dedicated = fixture.join("translated-content-de");
    write_doc(&content, Locale::EnUs, "English", "Web");
    write_doc(&primary, Locale::Fr, "French", "Web");
    write_doc(&primary, Locale::De, "Stale German", "OldWeb");
    write_doc(&dedicated, Locale::De, "German", "OldWeb");
    fs::write(
        content.join("en-us/_redirects.txt"),
        "/en-US/docs/OldWeb\t/en-US/docs/Web\n",
    )
    .unwrap();
    fs::write(
        primary.join("fr/_redirects.txt"),
        "/fr/docs/LegacyWeb\t/fr/docs/Web\n",
    )
    .unwrap();
    fs::write(
        dedicated.join("de/_redirects.txt"),
        "/de/docs/LegacyWeb\t/fr/docs/LegacyWeb\n",
    )
    .unwrap();
    fs::write(
        dedicated.join("de/_wikihistory.json"),
        "{\"OldWeb\":{\"owner\":\"de\"}}\n",
    )
    .unwrap();
    assert!(
        Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(&dedicated)
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
            .current_dir(&dedicated)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE")
            .env_remove("GIT_COMMON_DIR")
            .env_remove("GIT_PREFIX")
            .status()
            .unwrap()
            .success()
    );
    let history = |de_hash: &str, fr_hash: &str| {
        serde_json::to_string(&std::collections::BTreeMap::from([
            (
                "de/oldweb/index.md",
                HistoryEntry::new("2024-01-01T00:00:00Z", de_hash),
            ),
            (
                "fr/web/index.md",
                HistoryEntry::new("2024-01-01T00:00:00Z", fr_hash),
            ),
        ]))
        .unwrap()
    };
    fs::write(
        primary.join("_git_history.json"),
        history("stale-de", "primary-fr"),
    )
    .unwrap();
    fs::write(
        dedicated.join("_git_history.json"),
        history("dedicated-de", "stale-fr"),
    )
    .unwrap();

    let mut settings = Settings {
        content_root: content,
        content_translated_root: Some(primary.clone()),
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

    assert_eq!(root_for_locale(Locale::De).unwrap(), dedicated);
    let paths = translated_content_locale_paths(None);
    let docs = read_docs_parallel::<Page, Doc>(&paths, None).unwrap();
    assert_eq!(docs.len(), 2);
    assert!(
        docs.iter()
            .any(|doc| doc.locale() == Locale::De && doc.title() == "German")
    );
    assert!(docs.iter().any(|doc| doc.locale() == Locale::Fr));
    assert_eq!(
        git_history()[Path::new("de/oldweb/index.md")].hash,
        "dedicated-de"
    );
    assert_eq!(
        git_history()[Path::new("fr/web/index.md")].hash,
        "primary-fr"
    );

    let result = sync_translated_content(&[Locale::De], false).unwrap();
    assert_eq!(result[&Locale::De].total_docs, 1);
    assert_eq!(result[&Locale::De].moved_docs, 1);
    assert!(dedicated.join("de/web/index.md").exists());
    assert!(!dedicated.join("de/oldweb/index.md").exists());
    assert!(primary.join("de/oldweb/index.md").exists());
    let redirects = fs::read_to_string(dedicated.join("de/_redirects.txt")).unwrap();
    assert!(redirects.contains("/de/docs/OldWeb\t/de/docs/Web"));
    assert!(redirects.contains("/de/docs/LegacyWeb\t/fr/docs/Web"));
    let history: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dedicated.join("de/_wikihistory.json")).unwrap())
            .unwrap();
    assert!(history.get("Web").is_some());
    assert!(history.get("OldWeb").is_none());
    assert_eq!(
        fs::read_to_string(primary.join("de/_redirects.txt")).unwrap(),
        ""
    );
    fs::remove_dir_all(fixture).unwrap();
}
