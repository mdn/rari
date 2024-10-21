use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;

use console::Style;
use md5::Digest;
use rari_doc::cached_readers::{
    doc_page_from_slug, read_and_cache_doc_pages, STATIC_DOC_PAGE_FILES,
    STATIC_DOC_PAGE_TRANSLATED_FILES,
};
use rari_doc::pages::page::{Page, PageCategory, PageLike, PageWriter};
use rari_doc::pages::types::doc::Doc;
use rari_doc::redirects::resolve_redirect;
use rari_doc::resolve::{build_url, url_to_folder_path};
use rari_doc::utils::root_for_locale;
use rari_types::locale::Locale;
use rari_utils::concat_strs;

use crate::error::ToolError;
use crate::git::exec_git_with_test_fallback;
use crate::redirects::add_redirects;
use crate::wikihistory::update_wiki_history;

pub fn sync_translated_content(
    locales: &[Locale],
    verbose: bool,
) -> Result<HashMap<Locale, SyncTranslatedContentResult>, ToolError> {
    validate_locales(locales)?;

    let green = Style::new().green();
    let dim = Style::new().dim();
    let bold = Style::new().bold();

    if verbose {
        println!(
            "{}",
            green.apply_to(format!(
                "Syncing translated content for locales: {:?}. Reading documents from filesystem.",
                locales
            )),
        );
    }
    let docs = read_and_cache_doc_pages()?;
    if verbose {
        println!(
            "{}",
            dim.apply_to(format!(
                "read {} docs: {} en-us, {} translated",
                docs.len(),
                STATIC_DOC_PAGE_FILES.get().unwrap().len(),
                STATIC_DOC_PAGE_TRANSLATED_FILES.get().unwrap().len()
            ))
        );
    }
    let locales = locales.iter().collect::<HashSet<_>>();
    let results = HashMap::new();

    let res = STATIC_DOC_PAGE_TRANSLATED_FILES
        .get()
        .expect("Is the translated content root set?")
        .iter()
        .filter(|&((locale, _locale_str), _doc)| locales.contains(locale))
        .fold(results, |mut results, ((locale, _locale_str), page)| {
            if let Page::Doc(doc) = page {
                // println!("syncing doc: {:?}", doc.slug());
                let status = sync_translated_document(doc, verbose);
                if let Ok(status) = status {
                    if !results.contains_key(locale) {
                        results.insert(*locale, SyncTranslatedContentResult::default());
                    }
                    let result = results.get_mut(locale).unwrap();
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
                println!("Page is not a doc: {:?}", page);
            }
            results
        });

    // Add redirects contained in the results to the proper locale redirects files
    for (locale, result) in &res {
        let redirect_pairs = result
            .redirects
            .iter()
            .map(|(from, to)| (from.to_string(), to.to_string()))
            .collect::<Vec<_>>();
        add_redirects(*locale, &redirect_pairs)?;
    }

    if verbose {
        for (locale, result) in &res {
            println!(
                "{}",
                green.apply_to(bold.apply_to(format!("Results for locale {}", locale)))
            );
            println!(
                "  {}",
                green.apply_to(format!("Total of {} documents.", result.total_docs))
            );
            println!(
                "  {}",
                green.apply_to(format!("Moved {} documents.", result.moved_docs))
            );
            println!(
                "  {}",
                green.apply_to(format!("Renamed {} documents.", result.renamed_docs))
            );
            println!(
                "  {}",
                green.apply_to(format!(
                    "Conflicting {} documents.",
                    result.conflicting_docs
                ))
            );
            println!(
                "  {}",
                green.apply_to(format!("Orphaned {} documents", result.orphaned_docs))
            );
            println!(
                "  {}",
                green.apply_to(format!(
                    "Fixed {} redirected documents.",
                    result.redirected_docs
                ))
            );
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
    }
}

#[derive(Debug, Default)]
pub struct SyncTranslatedDocumentStatus {
    pub redirect: Option<(String, String)>,
    pub conflicting: bool,
    pub followed: bool,
    pub moved: bool,
    pub orphaned: bool,
    pub renamed: bool,
}

fn sync_translated_document(
    doc: &Doc,
    verbose: bool,
) -> Result<SyncTranslatedDocumentStatus, ToolError> {
    let mut status = SyncTranslatedDocumentStatus::default();

    let dim = Style::new().dim();
    let yellow = Style::new().yellow();
    let blue = Style::new().blue().bright();

    if doc.is_orphaned() || doc.is_conflicting() {
        return Ok(status);
    }

    let mut resolved_slug = resolve(doc.slug());

    status.renamed = doc.slug() != resolved_slug;
    status.moved = status.renamed && doc.slug().to_lowercase() != resolved_slug.to_lowercase();

    if status.moved {
        status.followed = true;
    }

    if let Some((url, _)) = resolved_slug.split_once('#') {
        resolved_slug = Cow::Owned(url.to_string());
    }

    let resolved_doc = doc_page_from_slug(&resolved_slug, Locale::EnUs);
    status.orphaned = !matches!(resolved_doc, Some(Ok(_)));

    if !status.renamed && !status.orphaned {
        return Ok(status);
    }

    if status.orphaned {
        if verbose {
            println!(
                "{}",
                yellow.apply_to(format!("orphaned: {}", doc.path().to_string_lossy()))
            );
        }
        status.followed = false;
        status.moved = true;
        resolved_slug = concat_strs!("orphaned/", &resolved_slug).into();
        let orphaned_doc = doc_page_from_slug(&resolved_slug, doc.locale());
        if let Some(Ok(_)) = orphaned_doc {
            return Err(ToolError::OrphanedDocExists(Cow::Owned(format!(
                "{} → {}",
                doc.slug(),
                resolved_slug
            ))));
        }
    } else if status.moved && md_or_html_exists(&resolved_slug, doc.locale())? {
        if verbose {
            println!(
                "{}",
                dim.apply_to(format!(
                    "unrooting {} (conflicting translation)",
                    doc.path().to_string_lossy()
                ))
            );
        }
        if let Some(Ok(_)) = resolved_doc {
            // set the slug to a /conflicting /... slug. if that already
            // exists (possibly from a previous move on this run),
            // append a md5 hash of the original slug to the slug
            // also set original_slug in metadata
            resolved_slug = concat_strs!("conflicting/", &resolved_slug).into();
            if md_or_html_exists(&resolved_slug, doc.locale())? {
                let mut hasher = md5::Md5::new();
                hasher.update(doc.slug());
                let digest = format!("{:x}", hasher.finalize());
                // println!("digest: {}", digest);
                resolved_slug = concat_strs!(&resolved_slug, "-", &digest).into();
            }

            status.conflicting = true;
        } else {
            return Err(ToolError::Unknown("Conflicting docs not found"));
        }
    }

    status.redirect = Some((
        build_url(doc.slug(), doc.locale(), PageCategory::Doc)?,
        build_url(&resolved_slug, doc.locale(), PageCategory::Doc)?,
    ));
    if verbose {
        println!(
            "{}",
            blue.apply_to(format!("redirecting {} → {}", doc.slug(), resolved_slug))
        )
    }

    // Update the wiki history
    update_wiki_history(
        doc.locale(),
        &[(doc.slug().to_string(), resolved_slug.to_string())],
    )?;

    if status.moved {
        move_doc(doc, &resolved_slug)?;
        // move the doc to the new location
        // set the original slug in metadata of moved doc
    }

    Ok(status)
}

fn move_doc(doc: &Doc, target_slug: &str) -> Result<(), ToolError> {
    let source_path = doc.path();
    let target_directory = root_for_locale(doc.locale())?
        .join(doc.locale().as_folder_str())
        .join(url_to_folder_path(target_slug));
    std::fs::create_dir_all(&target_directory)?;

    // Write the new slug, store the old slug in `original_slug` metadata
    let mut new_doc = doc.clone();
    new_doc.meta.slug = target_slug.to_owned();
    new_doc.meta.original_slug = Some(doc.slug().to_owned());
    new_doc.write()?;

    // Move the file with git
    let output = exec_git_with_test_fallback(
        &[
            OsStr::new("mv"),
            source_path.as_os_str(),
            target_directory.as_os_str(),
        ],
        root_for_locale(doc.locale())?,
    );

    if !output.status.success() {
        return Err(ToolError::GitError(format!(
            "Failed to move files: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    // If the source directory is empty, remove it with the fs api.
    let source_directory = doc.full_path().parent().unwrap();
    if source_directory
        .read_dir()
        .map(|mut dir| dir.next().is_none())
        .unwrap_or(false)
    {
        std::fs::remove_dir(source_directory)?;
    }

    Ok(())
}

fn md_or_html_exists(slug: &str, locale: Locale) -> Result<bool, ToolError> {
    let folder_path = root_for_locale(locale)?
        .join(locale.as_folder_str())
        .join(slug.to_lowercase());
    let md_path = folder_path.join("index.md");
    // Not use the static cache here (`doc_page_from_static_files`),
    // because we maybe have written files to the filesystem
    // after the cache was created.
    Ok(md_path.exists())
}

fn resolve(slug: &str) -> Cow<'_, str> {
    let en_us_url_lc = concat_strs!("/", Locale::EnUs.as_folder_str(), "/docs/", slug);
    // Note: Contrary to the yari original, we skip the fundamental redirects because
    // those have no role to play any more in this use case.
    let resolved_url =
        resolve_redirect(&en_us_url_lc).unwrap_or_else(|| Cow::Borrowed(&en_us_url_lc));

    let page = Page::from_url(&resolved_url);
    if let Ok(page) = page {
        Cow::Owned(page.slug().to_string())
    } else {
        Cow::Borrowed(slug)
    }
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

    use super::*;
    use crate::tests::fixtures::docs::DocFixtures;
    use crate::tests::fixtures::redirects::RedirectFixtures;

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
}
