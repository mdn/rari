use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn fix_flaws_accepts_optional_german_locale_absent_and_present() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let fixture = std::env::temp_dir().join(format!("rari-optional-cli-{suffix}"));
    let content = fixture.join("content");
    fs::create_dir_all(&content).unwrap();
    let translated_content = fixture.join("translated-content");
    fs::create_dir_all(&translated_content).unwrap();
    let source = fixture.join("translated-content-de");
    fs::write(
        fixture.join(".config.toml"),
        format!(
            "content_root = {:?}\noptional_translated_locales = [\"de\"]\n[translated_content_sources.de]\nroot = {:?}\nrepository = \"translated-content-de\"\n",
            content.display().to_string(),
            source.display().to_string()
        ),
    )
    .unwrap();

    let run = |locale: &str| {
        Command::new(env!("CARGO_BIN_EXE_rari"))
            .args(["--skip-updates", "content", "fix-flaws", "--locale", locale])
            .current_dir(&fixture)
            .env_remove("CONTENT_ROOT")
            .env_remove("CONTENT_TRANSLATED_ROOT")
            .env_remove("OPTIONAL_TRANSLATED_LOCALES")
            .env("TESTING_CONTENT_ROOT", &content)
            .env("TESTING_CONTENT_TRANSLATED_ROOT", &translated_content)
            .output()
            .unwrap()
    };
    let german = run("de");
    assert!(
        german.status.success(),
        "{}",
        String::from_utf8_lossy(&german.stderr)
    );
    assert!(
        String::from_utf8_lossy(&german.stderr).contains("Skipping optional locale de")
            || String::from_utf8_lossy(&german.stdout).contains("Skipping optional locale de"),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&german.stdout),
        String::from_utf8_lossy(&german.stderr)
    );
    assert!(!source.exists());

    let unsupported = run("xx");
    assert!(!unsupported.status.success());
    assert!(String::from_utf8_lossy(&unsupported.stderr).contains("invalid locale: xx"));

    let english_doc = content.join("en-us/web/index.md");
    let german_doc = source.join("de/web/index.md");
    fs::create_dir_all(english_doc.parent().unwrap()).unwrap();
    fs::create_dir_all(german_doc.parent().unwrap()).unwrap();
    fs::write(
        &english_doc,
        "---\ntitle: English\nslug: Web\n---\n\nEnglish\n",
    )
    .unwrap();
    let german_source = "---\ntitle: Deutsch\nslug: Web\n---\n\nDeutsch\n";
    fs::write(&german_doc, german_source).unwrap();
    fs::write(content.join("en-us/_redirects.txt"), "").unwrap();
    fs::write(source.join("de/_redirects.txt"), "").unwrap();
    fs::write(content.join("en-us/_wikihistory.json"), "{}\n").unwrap();
    fs::write(source.join("de/_wikihistory.json"), "{}\n").unwrap();
    let present = run("de");
    assert!(
        present.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&present.stdout),
        String::from_utf8_lossy(&present.stderr)
    );
    assert!(
        String::from_utf8_lossy(&present.stdout).contains("reading 2 content pages"),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&present.stdout),
        String::from_utf8_lossy(&present.stderr)
    );
    assert_eq!(fs::read_to_string(german_doc).unwrap(), german_source);
    fs::remove_dir_all(fixture).unwrap();
}
