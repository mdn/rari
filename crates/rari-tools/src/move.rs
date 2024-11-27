use std::borrow::Cow;
use std::ffi::OsStr;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::sync::Arc;

use console::{style, Style};
use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirm;
use rari_doc::{
    helpers::subpages::get_sub_pages,
    pages::page::{self, Page, PageCategory, PageLike, PageWriter},
    resolve::{build_url, url_meta_from, UrlMeta}, //  url_path_to_path_buf
    utils::root_for_locale,
};
use rari_types::locale::Locale;

use crate::error::ToolError;
use crate::git::exec_git_with_test_fallback;
use crate::redirects::add_redirects;
use crate::sidebars::update_sidebars;
use crate::utils::parent_slug;
use crate::wikihistory::update_wiki_history;

pub fn r#move(
    old_slug: &str,
    new_slug: &str,
    locale: Option<Locale>,
    assume_yes: bool,
) -> Result<(), ToolError> {
    validate_args(old_slug, new_slug)?;
    let locale = locale.unwrap_or_default();

    // Make a dry run to give some feedback on what would be done
    let green = Style::new().green();
    let red = Style::new().red();
    let bold = Style::new().bold();
    let changes = do_move(old_slug, new_slug, locale, true)?;
    if changes.is_empty() {
        println!("{}", style("No changes would be made").green());
        return Ok(());
    } else {
        println!(
            "{} {} {} {} {} {}",
            green.apply_to("This will move"),
            bold.apply_to(changes.len()),
            green.apply_to("documents from"),
            green.apply_to(old_slug),
            green.apply_to("to"),
            green.apply_to(new_slug)
        );
        for (old_slug, new_slug) in changes {
            println!(
                "{} -> {}",
                red.apply_to(&old_slug),
                green.apply_to(&new_slug)
            );
        }
    }

    if assume_yes
        || Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Proceed?")
            .default(true)
            .interact()
            .unwrap_or_default()
    {
        let moved = do_move(old_slug, new_slug, locale, false)?;
        println!(
            "{} {} {}",
            green.apply_to("Moved"),
            bold.apply_to(moved.len()),
            green.apply_to("documents"),
        );
    } else {
        return Ok(());
    }

    Ok(())
}

fn do_move(
    old_slug: &str,
    new_slug: &str,
    locale: Locale,
    dry_run: bool,
) -> Result<Vec<(String, String)>, ToolError> {
    let old_url = build_url(old_slug, locale, PageCategory::Doc)?;
    let doc = page::Page::from_url_with_fallback(&old_url)?;
    let real_old_slug = doc.slug();

    let new_parent_slug = parent_slug(new_slug)?;
    if !page::Page::exists(&build_url(new_parent_slug, locale, PageCategory::Doc)?) {
        return Err(ToolError::InvalidSlug(Cow::Owned(format!(
            "new parent slug does not exist: {new_parent_slug}"
        ))));
    }
    let subpages = get_sub_pages(&old_url, None, Default::default())?;

    let is_new_slug = real_old_slug != new_slug;

    // Return early if we move onto ourselves.
    if !is_new_slug {
        return Ok(vec![]);
    }

    let pairs = [doc.clone()]
        .iter()
        .chain(&subpages)
        .map(|page_ref| {
            let slug = page_ref.slug().to_owned();
            let new_slug = slug.replace(real_old_slug, new_slug);
            (slug, new_slug)
        })
        .collect::<Vec<_>>();

    // Return early for a dry run.
    if dry_run {
        return Ok(pairs);
    }

    // No dry run, so build a vec of pairs of `(old_page, Option<new_doc>)`.
    let doc_pairs = [&doc].into_iter().chain(&subpages).filter_map(|page_ref| {
        let slug = page_ref.slug().to_owned();
        let new_slug = slug.replace(real_old_slug, new_slug);
        let new_page = page_ref.clone();
        if let Page::Doc(doc) = new_page {
            let mut cloned_doc = doc.clone();
            let doc = Arc::make_mut(&mut cloned_doc);
            doc.meta.slug = new_slug.to_string();
            Some(doc.to_owned())
        } else {
            println!("This does not look like a document");
            None
        }
    });

    // Now iterate through the vec and write the new frontmatter
    // (the changed slug) to all affected documents (root + children).
    // The docs are all still in their old location at this time.
    for new_doc in doc_pairs {
        new_doc.write()?;
    }

    // Now we use the git command to move the whole parent directory
    // to a new location. This will move all children as well and
    // makes sure that we get a proper "file moved" in the git history.

    let mut old_folder_path = PathBuf::from(locale.as_folder_str());
    let url = build_url(real_old_slug, locale, PageCategory::Doc)?;
    let UrlMeta { folder_path, .. } = url_meta_from(&url)?;
    old_folder_path.push(folder_path);

    let mut new_folder_path = PathBuf::from(locale.as_folder_str());
    let url = build_url(new_slug, locale, PageCategory::Doc)?;
    let UrlMeta { folder_path, .. } = url_meta_from(&url)?;
    new_folder_path.push(folder_path);

    // Make sure the target parent directory exists.
    if let Some(target_parent_path) = new_folder_path.parent() {
        let absolute_target_parent_path = root_for_locale(locale)?.join(target_parent_path);
        create_dir_all(absolute_target_parent_path)?;
    } else {
        return Err(ToolError::Unknown(
            "Could not determine parent path for new folder",
        ));
    }

    // Execute the git move.
    let output = exec_git_with_test_fallback(
        &[
            OsStr::new("mv"),
            old_folder_path.as_os_str(),
            new_folder_path.as_os_str(),
        ],
        root_for_locale(locale)?,
    );

    if !output.status.success() {
        return Err(ToolError::GitError(format!(
            "Failed to move files: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    // Update Wiki history for entries that have an entry for the old slug.
    update_wiki_history(locale, &pairs)?;

    // Update the sidebars, changing links and paths where necessary.
    // But only for the default locale. Translated content cannot change
    // sidebars. Map the pairs from (String, String) to (String, Option<String>)
    // to match the function signature.
    if locale == Locale::default() {
        update_sidebars(
            &pairs
                .iter()
                .map(|(from, to)| {
                    (
                        Cow::Borrowed(from.as_str()),
                        Some(Cow::Borrowed(to.as_str())),
                    )
                })
                .collect::<Vec<_>>(),
        )?;
    }

    // Update the redirect map. Create pairs of URLs from the slug pairs.
    let url_pairs = pairs
        .iter()
        .map(|(old_slug, new_slug)| {
            let old_url = build_url(old_slug, locale, PageCategory::Doc)?;
            let new_url = build_url(new_slug, locale, PageCategory::Doc)?;
            Ok((old_url, new_url))
        })
        .collect::<Result<Vec<_>, ToolError>>()?;
    add_redirects(locale, &url_pairs)?;

    // finally, return the pairs of old and new slugs
    Ok(pairs)
}

fn validate_args(old_slug: &str, new_slug: &str) -> Result<(), ToolError> {
    if old_slug.is_empty() {
        return Err(ToolError::InvalidSlug(Cow::Borrowed(
            "old slug cannot be empty",
        )));
    }
    if new_slug.is_empty() {
        return Err(ToolError::InvalidSlug(Cow::Borrowed(
            "new slug cannot be empty",
        )));
    }
    if old_slug.contains("#") {
        return Err(ToolError::InvalidSlug(Cow::Borrowed(
            "old slug cannot contain '#'",
        )));
    }
    if new_slug.contains("#") {
        return Err(ToolError::InvalidSlug(Cow::Borrowed(
            "new slug cannot contain '#'",
        )));
    }
    Ok(())
}

// These tests use file system fixtures to simulate content and translated content.
// The file system is a shared resource, so we force tests to be run serially,
// to avoid concurrent fixture management issues.
// Using `file_serial` as a synchonization lock, we should be able to run all tests
// using the same `key` (here: file_fixtures) to be serialized across modules.
#[cfg(test)]
use serial_test::file_serial;
#[cfg(test)]
#[file_serial(file_fixtures)]
mod test {

    use super::*;
    use crate::tests::fixtures::docs::DocFixtures;
    use crate::tests::fixtures::redirects::RedirectFixtures;
    use crate::tests::fixtures::sidebars::SidebarFixtures;
    use crate::tests::fixtures::wikihistory::WikihistoryFixtures;
    use crate::utils::get_redirects_map;
    use crate::utils::test_utils::check_file_existence;

    fn s(s: &str) -> String {
        s.to_string()
    }

    #[test]
    fn test_validate_args() {
        assert!(validate_args("old", "new").is_ok());
        assert!(validate_args("old", "").is_err());
        assert!(validate_args("", "new").is_err());
        assert!(validate_args("old#", "new").is_err());
        assert!(validate_args("old", "new#").is_err());
    }

    #[test]
    fn test_parent_slug() {
        assert_eq!(parent_slug("a/b/c").unwrap(), "a/b");
        assert_eq!(parent_slug("a/b").unwrap(), "a");
        assert!(parent_slug("a").is_err());
        assert_eq!(parent_slug("a/b/c/").unwrap(), "a/b");
    }

    #[test]
    fn test_do_move_dry_run() {
        let slugs = vec![
            "Web/API/ExampleOne".to_string(),
            "Web/API/ExampleOne/SubExampleOne".to_string(),
            "Web/API/ExampleOne/SubExampleTwo".to_string(),
        ];
        let _docs = DocFixtures::new(&slugs, Locale::EnUs);
        let _wikihistory = WikihistoryFixtures::new(&slugs, Locale::EnUs);
        let redirects = vec![(
            "Web/API/Something".to_string(),
            "Web/API/SomethingElse".to_string(),
        )];
        let _redirects = RedirectFixtures::new(&redirects, Locale::EnUs);

        let result = do_move(
            "Web/API/ExampleOne",
            "Web/API/ExampleOneNewLocation",
            Locale::EnUs,
            true,
        );
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.len() == 3);
        assert_eq!(
            result[0],
            (s("Web/API/ExampleOne"), s("Web/API/ExampleOneNewLocation"))
        );
        assert_eq!(
            result[1],
            (
                s("Web/API/ExampleOne/SubExampleOne"),
                s("Web/API/ExampleOneNewLocation/SubExampleOne")
            )
        );
        assert_eq!(
            result[2],
            (
                s("Web/API/ExampleOne/SubExampleTwo"),
                s("Web/API/ExampleOneNewLocation/SubExampleTwo")
            )
        );
    }

    #[test]
    fn test_do_move() {
        let slugs = vec![
            "Web/API/Other".to_string(),
            "Web/API/ExampleOne".to_string(),
            "Web/API/ExampleOne/SubExampleOne".to_string(),
            "Web/API/ExampleOne/SubExampleTwo".to_string(),
            "Web/API/SomethingElse".to_string(),
        ];
        let redirects = vec![
            (
                "docs/Web/API/Something".to_string(),
                "docs/Web/API/SomethingElse".to_string(),
            ),
            (
                "docs/Web/API/SomethingThatPointsToAMovedDoc".to_string(),
                "docs/Web/API/ExampleOne/SubExampleOne".to_string(),
            ),
        ];
        let _docs = DocFixtures::new(&slugs, Locale::EnUs);
        let _wikihistory = WikihistoryFixtures::new(&slugs, Locale::EnUs);
        let _redirects = RedirectFixtures::new(&redirects, Locale::EnUs);
        let _sidebars = SidebarFixtures::default();

        let root_path = root_for_locale(Locale::EnUs).unwrap();
        let should_exist = vec![
            "en-us/web/api/other",
            "en-us/web/api/exampleone",
            "en-us/web/api/exampleone/subexampleone",
            "en-us/web/api/exampleone/subexampletwo",
        ];
        let should_not_exist = vec![
            "en-us/web/api/exampleonenewlocation",
            "en-us/web/api/exampleonenewlocation/subexampleone",
            "en-us/web/api/exampleonenewlocation/subexampletwo",
        ];
        check_file_existence(root_path, &should_exist, &should_not_exist);

        let result = do_move(
            "Web/API/ExampleOne",
            "Web/API/ExampleOneNewLocation",
            Locale::EnUs,
            false,
        );
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.len() == 3);
        assert_eq!(
            result[0],
            (s("Web/API/ExampleOne"), s("Web/API/ExampleOneNewLocation"))
        );
        assert_eq!(
            result[1],
            (
                s("Web/API/ExampleOne/SubExampleOne"),
                s("Web/API/ExampleOneNewLocation/SubExampleOne")
            )
        );
        assert_eq!(
            result[2],
            (
                s("Web/API/ExampleOne/SubExampleTwo"),
                s("Web/API/ExampleOneNewLocation/SubExampleTwo")
            )
        );

        let should_exist = vec![
            "en-us/web/api/other",
            "en-us/web/api/exampleonenewlocation",
            "en-us/web/api/exampleonenewlocation/subexampleone",
            "en-us/web/api/exampleonenewlocation/subexampletwo",
        ];
        let should_not_exist = vec![
            "en-us/web/api/exampleone",
            "en-us/web/api/exampleone/subexampleone",
            "en-us/web/api/exampleone/subexampletwo",
        ];
        check_file_existence(root_path, &should_exist, &should_not_exist);

        // check redirects
        let redirects = get_redirects_map(Locale::EnUs);
        assert_eq!(
            redirects.get("/en-US/docs/Web/API/ExampleOne").unwrap(),
            "/en-US/docs/Web/API/ExampleOneNewLocation"
        );
        assert_eq!(
            redirects
                .get("/en-US/docs/Web/API/ExampleOne/SubExampleOne")
                .unwrap(),
            "/en-US/docs/Web/API/ExampleOneNewLocation/SubExampleOne"
        );
        assert_eq!(
            redirects
                .get("/en-US/docs/Web/API/ExampleOne/SubExampleTwo")
                .unwrap(),
            "/en-US/docs/Web/API/ExampleOneNewLocation/SubExampleTwo"
        );
        // The entry that pointed to a moved doc should now point to the new location
        assert_eq!(
            redirects
                .get("/en-US/docs/Web/API/SomethingThatPointsToAMovedDoc")
                .unwrap(),
            "/en-US/docs/Web/API/ExampleOneNewLocation/SubExampleOne"
        );
        // Other entries should be unharmed
        assert_eq!(
            redirects.get("/en-US/docs/Web/API/Something").unwrap(),
            "/en-US/docs/Web/API/SomethingElse"
        );
    }

    #[test]
    fn test_do_move_translated() {
        let slugs = vec![
            "Web/API/Other".to_string(),
            "Web/API/ExampleOne".to_string(),
            "Web/API/ExampleOne/SubExampleOne".to_string(),
            "Web/API/ExampleOne/SubExampleTwo".to_string(),
            "Web/API/SomethingElse".to_string(),
        ];
        let redirects = vec![
            (
                "docs/Web/API/Something".to_string(),
                "docs/Web/API/SomethingElse".to_string(),
            ),
            (
                "docs/Web/API/SomethingThatPointsToAMovedDoc".to_string(),
                "docs/Web/API/ExampleOne/SubExampleOne".to_string(),
            ),
        ];
        let _docs = DocFixtures::new(&slugs, Locale::PtBr);
        let _wikihistory = WikihistoryFixtures::new(&slugs, Locale::PtBr);
        let _redirects = RedirectFixtures::new(&redirects, Locale::PtBr);
        let _sidebars = SidebarFixtures::default();

        let root_path = root_for_locale(Locale::PtBr).unwrap();
        let should_exist = vec![
            "pt-br/web/api/other",
            "pt-br/web/api/exampleone",
            "pt-br/web/api/exampleone/subexampleone",
            "pt-br/web/api/exampleone/subexampletwo",
        ];
        let should_not_exist = vec![
            "pt-br/web/api/exampleonenewlocation",
            "pt-br/web/api/exampleonenewlocation/subexampleone",
            "pt-br/web/api/exampleonenewlocation/subexampletwo",
        ];
        check_file_existence(root_path, &should_exist, &should_not_exist);

        let result = do_move(
            "Web/API/ExampleOne",
            "Web/API/ExampleOneNewLocation",
            Locale::PtBr,
            false,
        );
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.len() == 3);
        assert_eq!(
            result[0],
            (s("Web/API/ExampleOne"), s("Web/API/ExampleOneNewLocation"))
        );
        assert_eq!(
            result[1],
            (
                s("Web/API/ExampleOne/SubExampleOne"),
                s("Web/API/ExampleOneNewLocation/SubExampleOne")
            )
        );
        assert_eq!(
            result[2],
            (
                s("Web/API/ExampleOne/SubExampleTwo"),
                s("Web/API/ExampleOneNewLocation/SubExampleTwo")
            )
        );

        let should_exist = vec![
            "pt-br/web/api/other",
            "pt-br/web/api/exampleonenewlocation",
            "pt-br/web/api/exampleonenewlocation/subexampleone",
            "pt-br/web/api/exampleonenewlocation/subexampletwo",
        ];
        let should_not_exist = vec![
            "pt-br/web/api/exampleone",
            "pt-br/web/api/exampleone/subexampleone",
            "pt-br/web/api/exampleone/subexampletwo",
        ];
        check_file_existence(root_path, &should_exist, &should_not_exist);

        // check redirects
        let redirects = get_redirects_map(Locale::PtBr);
        assert_eq!(
            redirects.get("/pt-BR/docs/Web/API/ExampleOne").unwrap(),
            "/pt-BR/docs/Web/API/ExampleOneNewLocation"
        );
        assert_eq!(
            redirects
                .get("/pt-BR/docs/Web/API/ExampleOne/SubExampleOne")
                .unwrap(),
            "/pt-BR/docs/Web/API/ExampleOneNewLocation/SubExampleOne"
        );
        assert_eq!(
            redirects
                .get("/pt-BR/docs/Web/API/ExampleOne/SubExampleTwo")
                .unwrap(),
            "/pt-BR/docs/Web/API/ExampleOneNewLocation/SubExampleTwo"
        );
        // The entry that pointed to a moved doc should now point to the new location
        assert_eq!(
            redirects
                .get("/pt-BR/docs/Web/API/SomethingThatPointsToAMovedDoc")
                .unwrap(),
            "/pt-BR/docs/Web/API/ExampleOneNewLocation/SubExampleOne"
        );
        // Other entries should be unharmed
        assert_eq!(
            redirects.get("/pt-BR/docs/Web/API/Something").unwrap(),
            "/pt-BR/docs/Web/API/SomethingElse"
        );
    }
}
