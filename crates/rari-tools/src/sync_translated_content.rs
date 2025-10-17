use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::OsStr;

use rari_doc::pages::page::{Page, PageCategory, PageLike, PageWriter};
use rari_doc::pages::types::doc::Doc;
use rari_doc::resolve::{build_url, url_to_folder_path};
use rari_doc::utils::root_for_locale;
use rari_types::locale::Locale;
use rari_utils::concat_strs;
use sha2::{Digest, Sha256};

use crate::error::ToolError;
use crate::git::exec_git_with_test_fallback;
use crate::redirects::{add_redirects, fix_redirects};
use crate::utils::{get_redirects_map, read_all_doc_pages};
use crate::wikihistory::update_wiki_history;

pub fn sync_translated_content(
    locales: &[Locale],
    verbose: bool,
) -> Result<HashMap<Locale, SyncTranslatedContentResult>, ToolError> {
    validate_locales(locales)?;

    if verbose {
        tracing::info!("Syncing translated content for locales: {locales:?}.");
        tracing::info!("Fixing cross-locale redirects.");
    }
    fix_redirects(Some(locales))?;

    if verbose {
        tracing::info!("Reading all documents.");
    }

    let docs = read_all_doc_pages()?;
    let redirects_maps: HashMap<Locale, HashMap<String, String>> = locales
        .iter()
        .chain(std::iter::once(&Locale::EnUs))
        .map(|locale| {
            (
                *locale,
                get_redirects_map(*locale)
                    .iter()
                    .map(|(k, v)| (k.to_lowercase(), v.to_string()))
                    .collect(),
            )
        })
        .collect();

    if verbose {
        let (doc_count, translated_doc_count) =
            docs.iter()
                .fold((0, 0), |(x, y), ((locale, _slug), _page)| {
                    if *locale == Locale::EnUs {
                        (x + 1, y)
                    } else {
                        (x, y + 1)
                    }
                });
        tracing::info!(
            "read {} docs: {} en-Us, {} translated.",
            docs.len(),
            doc_count,
            translated_doc_count
        );
    }
    let results = HashMap::new();

    let res = docs
        .iter()
        .filter(|&((locale, _), _doc)| locales.contains(locale))
        .fold(results, |mut results, ((locale, _), page)| {
            if let Page::Doc(doc) = page {
                let status = sync_translated_document(&docs, &redirects_maps, doc, verbose);
                if let Ok(status) = status {
                    let result: &mut SyncTranslatedContentResult =
                        results.entry(*locale).or_default();
                    result.add_status(status);
                } else {
                    tracing::error!(
                        "Error syncing translated content for {} ({}): {:?}",
                        doc.slug(),
                        locale,
                        status
                    );
                }
            } else {
                tracing::info!("Page is not a doc: {:?}", page);
            }
            results
        });

    // Add redirects contained in the results to the proper locale redirects files
    // and modify wiki history files if needed.
    for (locale, result) in &res {
        let redirect_pairs = result
            .redirects
            .iter()
            .map(|(from, to)| (from.to_string(), to.to_string()))
            .collect::<Vec<_>>();
        add_redirects(*locale, &redirect_pairs)?;
        let wiki_history_pairs = result
            .wiki_history
            .iter()
            .map(|(from, to)| (from.to_string(), to.to_string()))
            .collect::<Vec<_>>();
        update_wiki_history(*locale, &wiki_history_pairs)?;
    }

    if verbose {
        for (locale, result) in &res {
            tracing::info!("Results for locale {locale}");
            tracing::info!("  Total of {} documents.", result.total_docs);
            tracing::info!("  Moved {} documents.", result.moved_docs);
            tracing::info!("  Renamed {} documents.", result.renamed_docs);
            tracing::info!("  Conflicting {} documents.", result.conflicting_docs);
            tracing::info!("  Orphaned {} documents", result.orphaned_docs);
            tracing::info!("  Fixed {} redirected documents.", result.redirected_docs);
        }
    }
    Ok(res)
}

#[derive(Debug, Default)]
pub struct SyncTranslatedContentResult {
    pub moved_docs: usize,
    pub conflicting_docs: usize,
    pub orphaned_docs: usize,
    pub redirected_docs: usize,
    pub renamed_docs: usize,
    pub total_docs: usize,
    pub redirects: HashMap<String, String>,
    pub wiki_history: HashMap<String, String>,
}

impl SyncTranslatedContentResult {
    pub fn add_status(&mut self, status: SyncTranslatedDocumentStatus) {
        self.moved_docs += status.moved as usize;
        self.conflicting_docs += status.conflicting as usize;
        self.orphaned_docs += status.orphaned as usize;
        self.redirected_docs += status.followed as usize;
        self.renamed_docs += status.renamed as usize;
        self.total_docs += 1;
        if let Some((from, to)) = status.redirect {
            self.redirects.insert(from, to);
        }
        if let Some((old, current)) = status.wiki_history {
            self.wiki_history.insert(old, current);
        }
    }
}

#[derive(Debug, Default)]
pub struct SyncTranslatedDocumentStatus {
    pub redirect: Option<(String, String)>,
    pub wiki_history: Option<(String, String)>,
    pub conflicting: bool,
    pub followed: bool,
    pub moved: bool,
    pub orphaned: bool,
    pub renamed: bool,
}

fn sync_translated_document(
    docs: &HashMap<(Locale, Cow<'_, str>), Page>,
    redirect_maps: &HashMap<Locale, HashMap<String, String>>,
    doc: &Doc,
    verbose: bool,
) -> Result<SyncTranslatedDocumentStatus, ToolError> {
    let mut status = SyncTranslatedDocumentStatus::default();

    if doc.is_orphaned() || doc.is_conflicting() {
        return Ok(status);
    }

    let resolved_slug = resolve(redirect_maps, doc.slug());

    status.renamed = doc.slug() != resolved_slug;
    status.moved = status.renamed && doc.slug().to_lowercase() != resolved_slug.to_lowercase();

    if status.moved {
        status.followed = true;
    }

    let mut resolved_slug = if let Some((url, _)) = resolved_slug.split_once('#') {
        Cow::Borrowed(url)
    } else {
        resolved_slug
    };

    let resolved_doc = docs.get(&(Locale::EnUs, resolved_slug.clone()));
    status.orphaned = resolved_doc.is_none();

    if !status.renamed && !status.orphaned {
        return Ok(status);
    }

    if status.orphaned {
        if verbose {
            tracing::info!("orphaned: {}", doc.path().to_string_lossy());
        }
        status.followed = false;
        status.moved = true;
        resolved_slug = concat_strs!("orphaned/", &resolved_slug).into();
        let orphaned_doc = docs.get(&(doc.locale(), resolved_slug.clone()));
        if orphaned_doc.is_some() {
            return Err(ToolError::OrphanedDocExists(Cow::Owned(format!(
                "{} → {}",
                doc.slug(),
                resolved_slug
            ))));
        }
    } else if status.moved && md_exists(&resolved_slug, doc.locale())? {
        if verbose {
            tracing::info!(
                "unrooting {} (conflicting translation)",
                doc.path().to_string_lossy()
            );
        }
        if resolved_doc.is_some() {
            // Set the slug to a /conflicting /... slug. if that already
            // exists (possibly from a previous move on this run),
            // append a sha256 hash of the original slug to the slug.
            resolved_slug = concat_strs!("conflicting/", &resolved_slug).into();
            if md_exists(&resolved_slug, doc.locale())? {
                let hash = Sha256::digest(doc.slug().as_bytes());
                let digest = format!("{hash:x}");
                resolved_slug = concat_strs!(&resolved_slug, "_", &digest).into();
            }

            status.conflicting = true;
        } else {
            return Err(ToolError::Unknown("Conflicting docs not found"));
        }
    }

    // Add entries to the redirects and wiki history maps
    status.redirect = Some((
        build_url(doc.slug(), doc.locale(), PageCategory::Doc)?,
        build_url(&resolved_slug, doc.locale(), PageCategory::Doc)?,
    ));
    status.wiki_history = Some((doc.slug().to_string(), resolved_slug.to_string()));

    // Write and then move the doc to the new location.
    // Also set `original_slug` in metadata.
    if status.moved {
        write_and_move_doc(doc, &resolved_slug)?;
    } else if status.renamed {
        // Rename only: just update the slug in metadata
        let mut new_doc = doc.clone();
        new_doc.meta.slug = resolved_slug.to_string();
        // Do not serialize these for translated content
        new_doc.meta.page_type = rari_types::fm_types::PageType::None;
        new_doc.meta.spec_urls = vec![];
        new_doc.meta.sidebar = vec![];
        new_doc.write()?;
    }

    Ok(status)
}

fn write_and_move_doc(doc: &Doc, target_slug: &str) -> Result<(), ToolError> {
    let source_directory = doc.full_path().parent().unwrap();
    let target_directory = root_for_locale(doc.locale())?
        .join(doc.locale().as_folder_str())
        .join(url_to_folder_path(target_slug));
    std::fs::create_dir_all(&target_directory)?;

    // Write the new slug, store the old slug in `original_slug` metadata
    let mut new_doc = doc.clone();
    new_doc.meta.slug = target_slug.to_owned();
    new_doc.meta.original_slug = Some(doc.slug().to_owned());
    new_doc.write()?;

    // Move all files and directories with git
    for entry in std::fs::read_dir(source_directory)? {
        let entry = entry?;
        let entry_path = entry.path();

        let output = exec_git_with_test_fallback(
            &[
                OsStr::new("mv"),
                entry_path.as_os_str(),
                target_directory.as_os_str(),
            ],
            root_for_locale(doc.locale())?,
        );

        if !output.status.success() {
            return Err(ToolError::GitError(format!(
                "Failed to move file/directory {}: {}",
                entry_path.display(),
                String::from_utf8_lossy(&output.stderr)
            )));
        }
    }

    // If the source directory is empty, remove it with the fs api.
    if source_directory
        .read_dir()
        .map(|mut dir| dir.next().is_none())
        .unwrap_or(false)
    {
        std::fs::remove_dir(source_directory)?;
    }

    Ok(())
}

/// Check if a markdown file exists for a given slug and locale.
fn md_exists(slug: &str, locale: Locale) -> Result<bool, ToolError> {
    let folder_path = root_for_locale(locale)?
        .join(locale.as_folder_str())
        .join(url_to_folder_path(slug));
    let md_path = folder_path.join("index.md");
    Ok(md_path.exists())
}

fn resolve<'a>(
    redirect_maps: &'a HashMap<Locale, HashMap<String, String>>,
    slug: &'a str,
) -> Cow<'a, str> {
    let en_us_url_lc =
        concat_strs!("/", Locale::EnUs.as_folder_str(), "/docs/", slug).to_lowercase();
    // Note: Contrary to the yari original, we skip the fundamental redirects because
    // those have no role to play any more in this use case.

    let resolved_url = redirect_maps
        .get(&Locale::EnUs)
        .expect("Redirect map for locale not loaded")
        .get(&en_us_url_lc)
        .unwrap_or(&en_us_url_lc);

    let page = Page::from_url(resolved_url);
    if let Ok(page) = page {
        if page.slug() != slug {
            return Cow::Owned(page.slug().to_string());
        }
    }
    Cow::Borrowed(slug)
}

fn validate_locales(locales: &[Locale]) -> Result<(), ToolError> {
    if locales.is_empty() {
        return Err(ToolError::InvalidLocale(Cow::Borrowed(
            "Locales cannot be empty",
        )));
    }
    if locales.contains(&Locale::EnUs) {
        return Err(ToolError::InvalidLocale(Cow::Borrowed(
            "Locales cannot contain en-us",
        )));
    }
    Ok(())
}

#[cfg(test)]
use serial_test::file_serial;
#[cfg(test)]
#[file_serial(file_fixtures)]
mod test {

    use rari_types::globals::content_translated_root;

    use super::*;
    use crate::redirects::{read_redirects_raw, redirects_path};
    use crate::tests::fixtures::docs::DocFixtures;
    use crate::tests::fixtures::redirects::RedirectFixtures;
    use crate::tests::fixtures::sidebars::SidebarFixtures;
    use crate::tests::fixtures::wikihistory::WikihistoryFixtures;
    use crate::wikihistory::read_wiki_history;

    #[test]
    fn test_valid_sync_locales() {
        let result = validate_locales(&[Locale::PtBr, Locale::ZhCn, Locale::Ru]);
        assert!(result.is_ok());
        let result = validate_locales(&[]);
        assert!(result.is_err());
        let result = validate_locales(&[Locale::EnUs, Locale::PtBr]);
        assert!(result.is_err());
    }

    #[test]
    fn test_sync_translated_content_no_changes() {
        let en_slugs = vec![
            "Web/API/Other".to_string(),
            "Web/API/ExampleOne".to_string(),
            "Web/API/ExampleTwo".to_string(),
            "Web/API/SomethingElse".to_string(),
        ];
        let en_redirects = vec![(
            "docs/Web/API/OldExampleOne".to_string(),
            "docs/Web/API/ExampleOne".to_string(),
        )];
        let _en_docs = DocFixtures::new(&en_slugs, Locale::EnUs);
        let _en_redirects = RedirectFixtures::new(&en_redirects, Locale::EnUs);
        let _en_wikihistory = WikihistoryFixtures::new(&en_slugs, Locale::EnUs);
        let _sidebars = SidebarFixtures::default();

        let es_slugs = vec![
            "Web/API/Other".to_string(),
            "Web/API/ExampleOne".to_string(),
            "Web/API/ExampleTwo".to_string(),
            "Web/API/SomethingElse".to_string(),
        ];
        let es_redirects = vec![(
            "docs/Web/API/OldExampleOne".to_string(),
            "docs/Web/API/ExampleOne".to_string(),
        )];
        let _es_docs = DocFixtures::new(&es_slugs, Locale::Es);
        let _es_redirects = RedirectFixtures::new(&es_redirects, Locale::Es);
        let _es_wikihistory = WikihistoryFixtures::new(&es_slugs, Locale::Es);

        let _de_redirects = RedirectFixtures::new(&[], Locale::De);
        let _fr_redirects = RedirectFixtures::new(&[], Locale::Fr);
        let _ja_redirects = RedirectFixtures::new(&[], Locale::Ja);
        let _ko_redirects = RedirectFixtures::new(&[], Locale::Ko);
        let _ptbr_redirects = RedirectFixtures::new(&[], Locale::PtBr);
        let _ru_redirects = RedirectFixtures::new(&[], Locale::Ru);
        let _zhcn_redirects = RedirectFixtures::new(&[], Locale::ZhCn);
        let _zhtw_redirects = RedirectFixtures::new(&[], Locale::ZhTw);

        let result = sync_translated_content(&[Locale::Es], false);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.len(), 1);

        let es_result = result.get(&Locale::Es);
        assert!(es_result.is_some());
        let es_result = es_result.unwrap();
        assert_eq!(es_result.total_docs, 6);
        assert_eq!(es_result.moved_docs, 0);
        assert_eq!(es_result.conflicting_docs, 0);
        assert_eq!(es_result.orphaned_docs, 0);
        assert_eq!(es_result.redirected_docs, 0);
        assert_eq!(es_result.renamed_docs, 0);
        assert_eq!(es_result.redirects.len(), 0);
    }

    #[test]
    fn test_sync_translated_content_orphaned() {
        let en_slugs = vec![
            "Web/API/ExampleOne".to_string(),
            "Web/API/ExampleTwo".to_string(),
            "Web/API/SomethingElse".to_string(),
        ];
        let en_redirects = vec![(
            "docs/Web/API/OldExampleOne".to_string(),
            "docs/Web/API/ExampleOne".to_string(),
        )];
        let _en_docs = DocFixtures::new(&en_slugs, Locale::EnUs);
        let _en_redirects = RedirectFixtures::new(&en_redirects, Locale::EnUs);
        let _en_wikihistory = WikihistoryFixtures::new(&en_slugs, Locale::EnUs);

        let es_slugs = vec![
            "Web/API/Other".to_string(),
            "Web/API/ExampleOne".to_string(),
            "Web/API/ExampleTwo".to_string(),
            "Web/API/SomethingElse".to_string(),
        ];
        let es_redirects = vec![(
            "docs/Web/API/OldExampleOne".to_string(),
            "docs/Web/API/ExampleOne".to_string(),
        )];
        let _es_docs = DocFixtures::new(&es_slugs, Locale::Es);
        let _es_redirects = RedirectFixtures::new(&es_redirects, Locale::Es);
        let _es_wikihistory = WikihistoryFixtures::new(&es_slugs, Locale::Es);

        let _de_redirects = RedirectFixtures::new(&[], Locale::De);
        let _fr_redirects = RedirectFixtures::new(&[], Locale::Fr);
        let _ja_redirects = RedirectFixtures::new(&[], Locale::Ja);
        let _ko_redirects = RedirectFixtures::new(&[], Locale::Ko);
        let _ptbr_redirects = RedirectFixtures::new(&[], Locale::PtBr);
        let _ru_redirects = RedirectFixtures::new(&[], Locale::Ru);
        let _zhcn_redirects = RedirectFixtures::new(&[], Locale::ZhCn);
        let _zhtw_redirects = RedirectFixtures::new(&[], Locale::ZhTw);

        let result = sync_translated_content(&[Locale::Es], false);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.len(), 1);

        let es_result = result.get(&Locale::Es);
        assert!(es_result.is_some());
        let es_result = es_result.unwrap();
        assert_eq!(es_result.total_docs, 6);
        assert_eq!(es_result.moved_docs, 1);
        assert_eq!(es_result.conflicting_docs, 0);
        assert_eq!(es_result.orphaned_docs, 1);
        assert_eq!(es_result.redirected_docs, 0);
        assert_eq!(es_result.renamed_docs, 0);
        assert_eq!(es_result.redirects.len(), 1);

        let translated_root = content_translated_root().unwrap();
        let orphaned_original_path = translated_root
            .join(Locale::Es.as_folder_str())
            .join("web")
            .join("api")
            .join("other")
            .join("index.md");
        assert!(!orphaned_original_path.exists());
        let orphaned_path = translated_root
            .join(Locale::Es.as_folder_str())
            .join("orphaned")
            .join("web")
            .join("api")
            .join("other")
            .join("index.md");
        assert!(orphaned_path.exists());

        let mut redirects = HashMap::new();
        redirects
            .extend(read_redirects_raw(redirects_path(Locale::Es).unwrap().as_path()).unwrap());
        assert_eq!(redirects.len(), 2);
        assert_eq!(
            redirects.get("/es/docs/Web/API/Other").unwrap(),
            "/es/docs/orphaned/Web/API/Other"
        );

        let wiki_history = read_wiki_history(Locale::Es).unwrap();
        assert!(wiki_history.contains_key("orphaned/Web/API/Other"));
    }

    #[test]
    fn test_sync_translated_content_moved() {
        let en_slugs = vec![
            "Web/API/OtherMoved".to_string(),
            "Web/API/ExampleOne".to_string(),
        ];
        let en_redirects = vec![(
            "docs/Web/API/Other".to_string(),
            "docs/Web/API/OtherMoved".to_string(),
        )];
        let _en_docs = DocFixtures::new(&en_slugs, Locale::EnUs);
        let _en_redirects = RedirectFixtures::new(&en_redirects, Locale::EnUs);
        let _en_wikihistory = WikihistoryFixtures::new(&en_slugs, Locale::EnUs);

        let es_slugs = vec![
            "Web/API/Other".to_string(),
            "Web/API/ExampleOne".to_string(),
        ];
        let es_redirects = vec![];
        let _es_docs = DocFixtures::new(&es_slugs, Locale::Es);
        // Let's assume there are also assets for this page that have to be moved as well
        DocFixtures::create_assets("Web/API/Other", Locale::Es);
        let _es_redirects = RedirectFixtures::new(&es_redirects, Locale::Es);
        let _es_wikihistory = WikihistoryFixtures::new(&es_slugs, Locale::Es);

        let _de_redirects = RedirectFixtures::new(&[], Locale::De);
        let _fr_redirects = RedirectFixtures::new(&[], Locale::Fr);
        let _ja_redirects = RedirectFixtures::new(&[], Locale::Ja);
        let _ko_redirects = RedirectFixtures::new(&[], Locale::Ko);
        let _ptbr_redirects = RedirectFixtures::new(&[], Locale::PtBr);
        let _ru_redirects = RedirectFixtures::new(&[], Locale::Ru);
        let _zhcn_redirects = RedirectFixtures::new(&[], Locale::ZhCn);
        let _zhtw_redirects = RedirectFixtures::new(&[], Locale::ZhTw);

        let result = sync_translated_content(&[Locale::Es], false);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.len(), 1);

        let es_result = result.get(&Locale::Es);
        assert!(es_result.is_some());
        let es_result = es_result.unwrap();
        assert_eq!(es_result.total_docs, 4);
        assert_eq!(es_result.moved_docs, 1);
        assert_eq!(es_result.conflicting_docs, 0);
        assert_eq!(es_result.orphaned_docs, 0);
        assert_eq!(es_result.redirected_docs, 1);
        assert_eq!(es_result.renamed_docs, 1);
        assert_eq!(es_result.redirects.len(), 1);

        let translated_root = content_translated_root().unwrap();
        let moved_original_path = translated_root
            .join(Locale::Es.as_folder_str())
            .join("web")
            .join("api")
            .join("other")
            .join("index.md");
        assert!(!moved_original_path.exists());
        let moved_path = translated_root
            .join(Locale::Es.as_folder_str())
            .join("web")
            .join("api")
            .join("othermoved")
            .join("index.md");
        assert!(moved_path.exists());

        // Test that assets are also moved to the new location
        let original_asset_path = translated_root
            .join(Locale::Es.as_folder_str())
            .join("web")
            .join("api")
            .join("other")
            .join("asset.txt");
        assert!(!original_asset_path.exists());

        let original_assets_dir = translated_root
            .join(Locale::Es.as_folder_str())
            .join("web")
            .join("api")
            .join("other")
            .join("assets");
        assert!(!original_assets_dir.exists());

        let moved_asset_path = translated_root
            .join(Locale::Es.as_folder_str())
            .join("web")
            .join("api")
            .join("othermoved")
            .join("asset.txt");
        assert!(moved_asset_path.exists());

        let moved_assets_dir = translated_root
            .join(Locale::Es.as_folder_str())
            .join("web")
            .join("api")
            .join("othermoved")
            .join("assets");
        assert!(moved_assets_dir.exists());

        let moved_asset_in_dir = moved_assets_dir.join("asset.txt");
        assert!(moved_asset_in_dir.exists());

        let mut redirects = HashMap::new();
        redirects
            .extend(read_redirects_raw(redirects_path(Locale::Es).unwrap().as_path()).unwrap());
        assert_eq!(redirects.len(), 1);
        assert_eq!(
            redirects.get("/es/docs/Web/API/Other").unwrap(),
            "/es/docs/Web/API/OtherMoved"
        );

        let wiki_history = read_wiki_history(Locale::Es).unwrap();
        assert!(wiki_history.contains_key("Web/API/OtherMoved"));
    }
}
