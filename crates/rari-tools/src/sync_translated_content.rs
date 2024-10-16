use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

use rari_doc::cached_readers::doc_page_from_slug;
use rari_doc::pages::page::{Page, PageLike};
use rari_doc::pages::types::doc::{self, Doc};
use rari_doc::reader::read_docs_parallel;
use rari_doc::redirects::resolve_redirect;
use rari_doc::utils::root_for_locale;
use rari_types::locale::Locale;
use rari_utils::concat_strs;

use crate::error::ToolError;

pub fn sync_translated_content(locales: &[Locale], verbose: bool) -> Result<(), ToolError> {
    validate_locales(locales)?;

    let mut results = vec![];

    for locale in locales {
        let result = sync_translated_content_for_locale(*locale)?;
        results.push(result);
    }

    println!("results: {:#?}", results);

    // let green = Style::new().green();
    // let bold = Style::new().bold();

    // println!(
    //     "{} {} {} {}",
    //     green.apply_to("Saved"),
    //     bold.apply_to(from_url),
    //     green.apply_to("→"),
    //     bold.apply_to(to_url),
    // );

    Ok(())
}

#[derive(Debug)]
struct SyncTranslatedContentResult {
    pub locale: Locale,
    pub moved_docs: usize,
    pub conflicting_docs: usize,
    pub orphaned_docs: usize,
    pub redirected_docs: usize,
    pub renamed_docs: usize,
    pub total_docs: usize,
}

fn sync_translated_content_for_locale(
    locale: Locale,
) -> Result<SyncTranslatedContentResult, ToolError> {
    let mut result = SyncTranslatedContentResult {
        locale,
        moved_docs: 0,
        conflicting_docs: 0,
        orphaned_docs: 0,
        redirected_docs: 0,
        renamed_docs: 0,
        total_docs: 0,
    };

    let mut docs_path = PathBuf::from(root_for_locale(locale)?);
    docs_path.push(locale.as_folder_str());
    println!("reading docs for locale: {:?}", docs_path);
    let docs: Vec<Arc<Doc>> = read_docs_parallel::<Doc>(&[docs_path], None)?
        .into_iter()
        .filter_map(|page| {
            if let Page::Doc(doc) = page {
                Some(doc.clone())
            } else {
                None
            }
        })
        .collect();
    println!("read {} docs", docs.len());

    for doc in docs {
        result.total_docs += 1;
        sync_translated_document(&doc)?;
    }

    Ok(result)
}

#[derive(Debug)]
struct SyncTranslatedDocumentStatus {
    pub redirect: Option<String>,
    pub conflicting: bool,
    pub followed: bool,
    pub moved: bool,
    pub orphaned: bool,
    pub renamed: bool,
}

fn sync_translated_document(doc: &Doc) -> Result<SyncTranslatedDocumentStatus, ToolError> {
    let mut status = SyncTranslatedDocumentStatus {
        redirect: None,
        conflicting: false,
        followed: false,
        moved: false,
        orphaned: false,
        renamed: false,
    };

    let debug = if doc.slug() == "Web/Demos" {
        println!("HIT WEB DEMOS");
        true
    } else {
        false
    };

    if doc.is_orphaned() || doc.is_conflicting() {
        return Ok(status);
    }

    let resolved_url = resolve(doc.slug());
    let resolved_page = Page::from_url(&resolved_url);

    let mut resolved_slug = resolved_page
        .map(|resolved_page| Cow::Owned(resolved_page.slug().to_owned()))
        .unwrap_or_else(|_| Cow::Borrowed(doc.slug()));

    status.renamed = doc.slug() != resolved_slug;
    status.moved = status.renamed && doc.slug().to_lowercase() != resolved_slug.to_lowercase();

    if status.moved {
        println!("moved: {} -> {}", doc.slug(), resolved_slug);
        status.followed = true;
    }

    if resolved_slug.contains('#') {
        status.moved = true;
        println!("{resolved_slug} contains #, stripping");
        resolved_slug = Cow::Owned(resolved_slug.split('#').next().unwrap().to_string());
    }

    // this function assumes caching is enabled?
    let resolved_doc = doc_page_from_slug(&resolved_slug, doc.locale());
    status.orphaned = resolved_doc.is_none();

    if debug {
        println!("resolved_doc: {:?} {:?} ", resolved_doc, resolved_slug);
    }

    if !status.renamed && !status.orphaned {
        return Ok(status);
    }

    if status.orphaned {
        println!("orphaned: {:?}", doc.full_path());
    }
    Ok(status)
}

fn resolve(slug: &str) -> Cow<'_, str> {
    let en_us_url_lc = concat_strs!("/", Locale::EnUs.as_folder_str(), "/docs/", slug);
    // Note: Contrary to the yari original, we skip the fundamental redirects because
    // those have no role to play in this use case.
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

        let result = sync_translated_content_for_locale(locale);
        assert!(result.is_ok());
        let sync_result = result.unwrap();
        assert_eq!(sync_result.locale, locale);
        assert_eq!(sync_result.total_docs, 7); // Assuming no docs are read in the test environment
    }
}
