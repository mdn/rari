//! Runs in its own test binary: `rari_doc::redirects` reads `_redirects.txt`
//! into a process-wide static on first use, so the fixture must exist before
//! any other test resolves a redirect.

use std::fs;
use std::path::PathBuf;

use indoc::formatdoc;
use rari_doc::issues::IN_MEMORY;
use rari_doc::pages::page::{Page, PageLike};
use rari_doc::utils::root_for_locale;
use rari_tools::fix::issues::{fix_page, get_fixable_issues};
use rari_types::locale::Locale;
use serial_test::file_serial;
use tracing_subscriber::layer::SubscriberExt;

/// en-US docs and redirects under the test content root, removed on drop.
struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new(docs: &[(&str, &str)], redirects: &[(&str, &str)]) -> Self {
        let root = root_for_locale(Locale::EnUs)
            .unwrap()
            .join(Locale::EnUs.as_folder_str());
        for (slug, body) in docs {
            // Ancestors are needed for breadcrumbs; the leaf gets the body.
            let parts = slug.split('/').collect::<Vec<_>>();
            for (i, title) in parts.iter().enumerate() {
                let slug = parts[..=i].join("/");
                let dir = root.join(slug.to_lowercase());
                let path = dir.join("index.md");
                let leaf = i == parts.len() - 1;
                if path.exists() && !leaf {
                    continue;
                }
                let body = if leaf { body } else { "Stub." };
                fs::create_dir_all(&dir).unwrap();
                fs::write(
                    path,
                    formatdoc! {"
                        ---
                        title: {title}
                        slug: {slug}
                        ---

                        {body}
                    "},
                )
                .unwrap();
            }
        }
        let redirects = redirects
            .iter()
            .map(|(from, to)| format!("/en-US/docs/{from}\t/en-US/docs/{to}\n"))
            .collect::<String>();
        fs::write(root.join("_redirects.txt"), redirects).unwrap();
        Self { root }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).ok();
    }
}

#[test]
#[file_serial(file_fixtures)]
fn skips_redirects_to_unrooted_pages() {
    let _fixture = Fixture::new(
        &[
            ("Web/API/Target", "Target."),
            ("orphaned/Web/API/Gone", "Orphaned."),
            (
                "Web/API/Source",
                "[Target](/en-US/docs/Web/API/OldTarget) and [Gone](/en-US/docs/Web/API/OldGone)",
            ),
        ],
        &[
            ("Web/API/OldTarget", "Web/API/Target"),
            ("Web/API/OldGone", "orphaned/Web/API/Gone"),
        ],
    );
    tracing::subscriber::set_global_default(tracing_subscriber::registry().with(IN_MEMORY.clone()))
        .unwrap();

    let page = Page::from_url("/en-US/docs/Web/API/Source").unwrap();

    let issues = get_fixable_issues(&page).unwrap();
    let suggestions = issues
        .iter()
        .map(|issue| issue.display_issue().suggestion.as_deref())
        .collect::<Vec<_>>();
    assert_eq!(suggestions, vec![Some("/en-US/docs/Web/API/Target")]);

    assert!(fix_page(&page).unwrap());
    let fixed = fs::read_to_string(page.full_path()).unwrap();
    assert!(
        fixed.contains("[Target](/en-US/docs/Web/API/Target)"),
        "{fixed}"
    );
    assert!(
        fixed.contains("[Gone](/en-US/docs/Web/API/OldGone)"),
        "{fixed}"
    );
}
