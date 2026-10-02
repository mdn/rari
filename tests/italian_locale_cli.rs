use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn fix_flaws_accepts_optional_italian_locale_absent_and_present() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let fixture = std::env::temp_dir().join(format!("rari-optional-cli-{suffix}"));
    let content = fixture.join("content");
    fs::create_dir_all(&content).unwrap();
    let translated_content = fixture.join("translated-content");
    fs::create_dir_all(&translated_content).unwrap();
    let source = fixture.join("translated-content-it");
    fs::write(
        fixture.join(".config.toml"),
        format!(
            "content_root = {:?}\noptional_translated_locales = [\"it\"]\n[translated_content_sources.it]\nroot = {:?}\nrepository = \"translated-content-it\"\n",
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
    let italian = run("it");
    assert!(
        italian.status.success(),
        "{}",
        String::from_utf8_lossy(&italian.stderr)
    );
    assert!(
        String::from_utf8_lossy(&italian.stderr).contains("Skipping optional locale it")
            || String::from_utf8_lossy(&italian.stdout).contains("Skipping optional locale it"),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&italian.stdout),
        String::from_utf8_lossy(&italian.stderr)
    );
    assert!(!source.exists());

    let unsupported = run("xx");
    assert!(!unsupported.status.success());
    assert!(String::from_utf8_lossy(&unsupported.stderr).contains("invalid locale: xx"));

    let english_doc = content.join("en-us/web/index.md");
    let italian_doc = source.join("it/web/index.md");
    fs::create_dir_all(english_doc.parent().unwrap()).unwrap();
    fs::create_dir_all(italian_doc.parent().unwrap()).unwrap();
    fs::write(
        &english_doc,
        "---\ntitle: English\nslug: Web\n---\n\nEnglish\n",
    )
    .unwrap();
    let italian_source = "---\ntitle: Italiano\nslug: Web\n---\n\nItaliano\n";
    fs::write(&italian_doc, italian_source).unwrap();
    fs::write(content.join("en-us/_redirects.txt"), "").unwrap();
    fs::write(source.join("it/_redirects.txt"), "").unwrap();
    fs::write(content.join("en-us/_wikihistory.json"), "{}\n").unwrap();
    fs::write(source.join("it/_wikihistory.json"), "{}\n").unwrap();
    let present = run("it");
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
    assert_eq!(fs::read_to_string(italian_doc).unwrap(), italian_source);
    fs::remove_dir_all(fixture).unwrap();
}
