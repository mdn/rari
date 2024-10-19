use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use console::Style;
use md5::Digest;
use rari_doc::cached_readers::{
    doc_page_from_slug, doc_page_from_static_files, read_and_cache_doc_pages,
    STATIC_DOC_PAGE_FILES, STATIC_DOC_PAGE_TRANSLATED_FILES,
};
use rari_doc::pages::page::{Page, PageCategory, PageLike};
use rari_doc::pages::types::doc::Doc;
use rari_doc::redirects::resolve_redirect;
use rari_doc::resolve::build_url;
use rari_doc::utils::root_for_locale;
use rari_types::locale::Locale;
use rari_utils::concat_strs;

use crate::error::ToolError;

pub fn sync_translated_content(locales: &[Locale], verbose: bool) -> Result<(), ToolError> {
    validate_locales(locales)?;

    let green = Style::new().green();
    // let white = Style::new().white();
    // let red = Style::new().red();
    // let yellow = Style::new().yellow();
    let dim = Style::new().dim();
    let bold = Style::new().bold();
    // let default = Style::new();

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

    // add redirects, if there are any

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
                "Fixed {} redirected document.",
                result.redirected_docs
            ))
        );
    }

    // println!("res: {:#?}", res);

    Ok(())
}

#[derive(Debug, Default)]
struct SyncTranslatedContentResult {
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
struct SyncTranslatedDocumentStatus {
    pub redirect: Option<(String, String)>,
    pub conflicting: bool,
    pub followed: bool,
    pub moved: bool,
    pub orphaned: bool,
    pub renamed: bool,
}

fn sync_translated_document(
    doc: &Doc,
    _verbose: bool,
) -> Result<SyncTranslatedDocumentStatus, ToolError> {
    let mut status = SyncTranslatedDocumentStatus::default();

    // let debug = if doc.slug() == "Web/API/Document_object_model/How_to_create_a_DOM_tree" {
    //     println!("HIT THE THING slug: {}", doc.slug());
    //     true
    // } else {
    //     false
    // };

    if doc.is_orphaned() || doc.is_conflicting() {
        return Ok(status);
    }

    let mut resolved_slug = resolve(doc.slug());

    status.renamed = doc.slug() != resolved_slug;
    status.moved = status.renamed && doc.slug().to_lowercase() != resolved_slug.to_lowercase();

    if status.moved {
        // println!("moved: {} → {}", doc.slug(), resolved_slug);
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

    // if debug {
    //     println!("   doc.slug: {}\n   status: {:#?}", doc.slug(), status);
    // }

    if status.orphaned {
        println!("orphaned: {}", doc.path().to_string_lossy());
        status.followed = false;
        status.moved = true;
        let orphaned_slug = concat_strs!("orphaned/", &resolved_slug);
        let orphaned_doc = doc_page_from_slug(&orphaned_slug, doc.locale());
        // USE THIS: doc_page_from_static_files(path: &PathBuf, locale: Locale) -> Option<Result<Page, DocError>
        if let Some(Ok(_)) = orphaned_doc {
            return Err(ToolError::OrphanedDocExists(Cow::Owned(format!(
                "{} → {}",
                doc.slug(),
                resolved_slug
            ))));
        }
    } else if status.moved && md_or_html_exists(&resolved_slug, doc.locale())? {
        println!(
            "unrooting {} (conflicting translation)",
            doc.path().to_string_lossy()
        );
        if let Some(Ok(_)) = resolved_doc {
            // set the slug to a /conflicting /... slug. if that already
            // exists (possibly from a previous move on this run),
            // append a md5 hash of the original slug to the slug
            // also set original_slug in metadata
            let mut conflicting_slug = concat_strs!("conflicting/", &resolved_slug);
            if md_or_html_exists(&conflicting_slug, doc.locale())? {
                let mut hasher = md5::Md5::new();
                hasher.update(doc.slug());
                let digest = format!("{:x}", hasher.finalize());
                println!("digest: {}", digest);

                conflicting_slug = concat_strs!("conflicting/", &resolved_slug, "-", &digest);
            }

            status.conflicting = true;

            status.redirect = Some((
                build_url(doc.slug(), doc.locale(), PageCategory::Doc)?,
                build_url(&conflicting_slug, doc.locale(), PageCategory::Doc)?,
            ));
        } else {
            return Err(ToolError::Unknown("Conflicting docs not found"));
        }

        // if debug {
        //     println!(
        //         "HERE: {} → {} {:#?} {}",
        //         doc.slug(),
        //         resolved_slug,
        //         doc_page_from_slug(&resolved_slug, doc.locale()),
        //         md_or_html_exists(&resolved_slug, doc.locale())?
        //     );
        // }

        // if let Some(Ok(conflicting_doc)) = doc_page_from_slug(&resolved_slug, doc.locale()) {
        //     println!(
        //         "unrooting {} (conflicting translation)",
        //         conflicting_doc.path().to_string_lossy()
        //     );
        // }
        // move the document to the conflicting folder
    }
    Ok(status)
}

fn md_or_html_exists(slug: &str, locale: Locale) -> Result<bool, ToolError> {
    let folder_path = root_for_locale(locale)?
        .join(locale.as_folder_str())
        .join(slug.to_lowercase());
    let md_path = folder_path.join("index.md");
    let html_path = folder_path.join("index.html");
    let md_exists = doc_page_from_static_files(&md_path);
    let html_exists = doc_page_from_static_files(&html_path);
    let ret = matches!(md_exists, Some(Ok(_))) || matches!(html_exists, Some(Ok(_)));
    Ok(ret)
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
    fn test_valid_locales() {
        let result = validate_locales(&[Locale::PtBr, Locale::ZhCn, Locale::Ru]);
        assert!(result.is_ok());
        let result = validate_locales(&[]);
        assert!(result.is_err());
        let result = validate_locales(&[Locale::EnUs, Locale::PtBr]);
        assert!(result.is_err());
    }

    #[test]
    fn test_sync_translated_content_for_locale() {
        let slugs = vec![
            "Web/API/Other".to_string(),
            "Web/API/ExampleOne".to_string(),
            "Web/API/ExampleOne/SubExampleOne".to_string(),
            "Web/API/ExampleOne/SubExampleTwo".to_string(),
            "Web/API/SomethingElse".to_string(),
        ];
        let redirects = vec![(
            "docs/Web/API/ExampleOne".to_string(),
            "docs/Web/API/ExampleOne/SubExampleOne".to_string(),
        )];
        let _redirects = RedirectFixtures::new(&redirects, Locale::PtBr);

        let locale = Locale::PtBr;
        let _docs = DocFixtures::new(&slugs, locale);

        // let result = sync_translated_content_for_locale(locale);
        // assert!(result.is_ok());
        // let sync_result = result.unwrap();
        // assert_eq!(sync_result.total_docs, 7); // Assuming no docs are read in the test environment
    }
}
