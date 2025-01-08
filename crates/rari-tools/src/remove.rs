use std::borrow::Cow;
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::PathBuf;

use console::Style;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirm;
use rari_doc::error::DocError;
use rari_doc::helpers::subpages::get_sub_pages;
use rari_doc::pages::page::{self, Page, PageCategory, PageLike};
use rari_doc::pages::types::doc::Doc;
use rari_doc::reader::read_docs_parallel;
use rari_doc::resolve::build_url;
use rari_doc::utils::root_for_locale;
use rari_types::locale::Locale;
use rayon::iter::{once, IntoParallelIterator, ParallelIterator};

use crate::error::ToolError;
use crate::git::exec_git_with_test_fallback;
use crate::redirects::add_redirects;
use crate::sidebars::update_sidebars;
use crate::wikihistory::delete_from_wiki_history;

pub fn remove(
    slug: &str,
    locale: Option<Locale>,
    recursive: bool,
    redirect: Option<&str>,
    assume_yes: bool,
) -> Result<(), ToolError> {
    validate_args(slug)?;
    let locale = locale.unwrap_or_default();

    let green = Style::new().green();
    let red = Style::new().red();
    let yellow = Style::new().yellow();
    let bold = Style::new().bold();
    let changes = do_remove(slug, locale, recursive, redirect, true)?;
    if changes.is_empty() {
        println!("{}", green.apply_to("No changes would be made"));
        return Ok(());
    } else {
        println!(
            "{} {} {}",
            green.apply_to("This will delete"),
            bold.apply_to(changes.len()),
            green.apply_to("documents:"),
        );
        for slug in changes {
            println!("{}", red.apply_to(&slug));
        }
        if let Some(redirect) = redirect {
            println!(
                "{} {} to: {}",
                green.apply_to("Redirecting"),
                green.apply_to(if recursive {
                    "each document"
                } else {
                    "document"
                }),
                green.apply_to(&redirect),
            );
        } else {
            println!("{}", yellow.apply_to("Deleting without a redirect. Consider using the --redirect option with a related page instead."));
        }
    }

    if assume_yes
        || Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Proceed?")
            .default(true)
            .interact()
            .unwrap_or_default()
    {
        let removed = do_remove(slug, locale, recursive, redirect, false)?;
        let removed_urls = removed
            .iter()
            .map(|slug| build_url(slug, locale, PageCategory::Doc))
            .collect::<Result<Vec<_>, DocError>>()?;
        println!(
            "{} {} {}",
            green.apply_to("Deleted"),
            bold.apply_to(removed.len()),
            green.apply_to("documents:"),
        );
        for url in &removed_urls {
            println!("{}", red.apply_to(&url));
        }

        // Find references to deleted documents and
        // list them for manual review
        println!("Checking references to deleted documents...");
        let mut docs_path = PathBuf::from(root_for_locale(locale)?);
        docs_path.push(locale.as_folder_str());

        let docs = read_docs_parallel::<Page, Doc>(&[docs_path], None)?;

        let referencing_docs: BTreeSet<String> = docs
            .into_par_iter()
            .filter_map(|doc| {
                for url in &removed {
                    if doc.content().contains(url) {
                        return Some(doc.url().to_owned());
                    }
                }
                None
            })
            .collect();

        if referencing_docs.is_empty() {
            println!(
                "{}",
                green.apply_to("No file is referring to the deleted document."),
            );
        } else {
            println!(
                "{} {}",
                yellow.apply_to(referencing_docs.len()),
                yellow.apply_to("files are referring to the deleted documents. Please update the following files to remove the links:"),
            );
            for url in &referencing_docs {
                println!("{}", yellow.apply_to(url));
            }
        }
    }
    Ok(())
}

fn do_remove(
    slug: &str,
    locale: Locale,
    recursive: bool,
    redirect: Option<&str>,
    dry_run: bool,
) -> Result<Vec<String>, ToolError> {
    let doc = Doc::page_from_slug(slug, locale, false)?;
    let real_slug = doc.slug();

    // If we get a redirect value passed in, it is either a slug or a complete url.
    // If it is a slug, check if it actually exists, otherwise bail.
    let redirect_target = if let Some(redirect_str) = redirect {
        if !redirect_str.starts_with("http") {
            let redirect_url = build_url(redirect_str, locale, PageCategory::Doc)?;
            if !page::Page::exists(&redirect_url) {
                return Err(ToolError::InvalidRedirectToURL(format!(
                    "redirect slug does not exist: {redirect_url}"
                )));
            }
            Some(redirect_url)
        } else {
            Some(redirect_str.to_owned())
        }
    } else {
        None
    };

    let subpages = get_sub_pages(doc.url(), None, Default::default())?;
    if !recursive && !subpages.is_empty() && redirect.is_some() {
        return Err(ToolError::HasSubpagesError(Cow::Owned(format!(
            "{0}, unable to remove and redirect a document with children",
            slug
        ))));
    }

    let slugs_to_remove = if recursive {
        once(&doc)
            .chain(&subpages)
            .map(|page_ref| page_ref.slug().to_string())
            .collect::<Vec<_>>()
    } else {
        vec![real_slug.to_string()]
    };

    if dry_run {
        return Ok(slugs_to_remove);
    }

    // Remove the documents. For single documents, we just remove the `index.md` file and
    // leave the folder structure in place. For recursive removal, we remove the entire
    // folder structure, duplicating the original yari tool behaviour.

    // Conditional command for testing. In testing, we do not use git, because the test
    // fixtures are not under git control. Instead of `git rm …` we use `rm …`.
    let path = doc.path();

    if recursive {
        let parent = path
            .parent()
            .ok_or(ToolError::InvalidSlug(Cow::Owned(format!(
                "{slug} ({}) has no parent directory",
                path.display()
            ))))?;

        // Execute the recursive remove command
        let output = exec_git_with_test_fallback(
            &[OsStr::new("rm"), OsStr::new("-rf"), parent.as_os_str()],
            root_for_locale(locale)?,
        );

        if !output.status.success() {
            return Err(ToolError::GitError(format!(
                "Failed to remove files: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
    } else {
        // Execute the single file remove command
        let output = exec_git_with_test_fallback(
            &[OsStr::new("rm"), path.as_os_str()],
            root_for_locale(locale)?,
        );

        if !output.status.success() {
            return Err(ToolError::GitError(format!(
                "Failed to remove files: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
    }

    // update the wiki history
    delete_from_wiki_history(locale, &slugs_to_remove)?;

    // Update the sidebars, removing links and paths where necessary.
    // But only for the default locale. Translated content cannot change
    // sidebars.
    if locale == Locale::default() {
        let pairs = slugs_to_remove
            .iter()
            .map(|slug| {
                let url = build_url(slug, locale, PageCategory::Doc)?;
                Ok((Cow::Owned(url), None))
            })
            .collect::<Result<Vec<_>, ToolError>>()?;
        update_sidebars(&pairs)?;
    }

    // update the redirects map if needed
    if let Some(new_target) = redirect_target {
        let pairs = slugs_to_remove
            .iter()
            .map(|slug| {
                let old_url = build_url(slug, locale, PageCategory::Doc)?;
                // let new_url = build_url(&new_slug, locale, PageCategory::Doc)?;
                Ok((old_url, new_target.to_owned()))
            })
            .collect::<Result<Vec<_>, ToolError>>()?;
        add_redirects(locale, &pairs)?;
    }
    Ok(slugs_to_remove)
}

fn validate_args(slug: &str) -> Result<(), ToolError> {
    if slug.is_empty() {
        return Err(ToolError::InvalidSlug(Cow::Borrowed(
            "slug cannot be empty",
        )));
    }
    if slug.contains("#") {
        return Err(ToolError::InvalidSlug(Cow::Borrowed(
            "slug cannot contain '#'",
        )));
    }
    Ok(())
}

// These tests use file system fixtures to simulate content and translated content.
// The file system is a shared resource, so we force tests to be run serially,
// to avoid concurrent fixture management issues.
// Using `file_serial` as a synchronization lock, we run all tests using
// the same `key` (here: file_fixtures) to be serialized across modules.
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
    use crate::wikihistory::test_get_wiki_history;

    #[test]
    fn test_validate_args() {
        assert!(validate_args("old").is_ok());
        assert!(validate_args("").is_err());
        assert!(validate_args("old#").is_err());
    }

    #[test]
    fn test_redirect_option() {
        let slugs = vec![
            "Web/API/ExampleOne".to_string(),
            "Web/API/ExampleTwo".to_string(),
        ];
        let _docs = DocFixtures::new(&slugs, Locale::EnUs);

        // valid redirect
        let result = do_remove(
            "Web/API/ExampleTwo",
            Locale::EnUs,
            false,
            Some("Web/API/ExampleOne"),
            true,
        );
        assert!(result.is_ok());

        // invalid redirect
        let result = do_remove(
            "Web/API/ExampleTwo",
            Locale::EnUs,
            false,
            Some("Web/API/ExampleNonExisting"),
            true,
        );
        assert!(matches!(result, Err(ToolError::InvalidRedirectToURL(_))));
        assert!(result.is_err());

        // external URL
        let result = do_remove(
            "Web/API/ExampleTwo",
            Locale::EnUs,
            false,
            Some("https://example.com/"),
            true,
        );
        assert!(result.is_ok());

        // no redirect
        let result = do_remove("Web/API/ExampleTwo", Locale::EnUs, false, None, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_existing_subpages() {
        let slugs = vec![
            "Web/API/ExampleOne".to_string(),
            "Web/API/ExampleOne/Subpage".to_string(),
            "Web/API/RedirectTarget".to_string(),
        ];
        let _docs = DocFixtures::new(&slugs, Locale::EnUs);

        // no recursive, no redirect, ok even with subpages
        let result = do_remove("Web/API/ExampleOne", Locale::EnUs, false, None, true);
        assert!(result.is_ok());

        // no recursive, with redirect, not ok with subpages
        let result = do_remove(
            "Web/API/ExampleOne",
            Locale::EnUs,
            false,
            Some("Web/API/RedirectTarget"),
            true,
        );
        assert!(result.is_err());
        assert!(matches!(result, Err(ToolError::HasSubpagesError(_))));

        // recursive
        let result = do_remove("Web/API/ExampleOne", Locale::EnUs, true, None, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_no_subpages() {
        let slugs = vec!["Web/API/ExampleOne".to_string()];
        let _docs = DocFixtures::new(&slugs, Locale::EnUs);

        // no recursive, but no subpages, so it is ok
        let result = do_remove("Web/API/ExampleOne", Locale::EnUs, false, None, true);
        assert!(result.is_ok());

        // recursive, all the same
        let result = do_remove("Web/API/ExampleOne", Locale::EnUs, true, None, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_nonexisting() {
        // This does not exist
        let result = do_remove("Web/API/ExampleOne", Locale::EnUs, false, None, true);
        assert!(result.is_err());
        assert!(matches!(result, Err(ToolError::DocError(_))));
    }

    #[test]
    fn test_remove_single() {
        let slugs = vec!["Web/API/ExampleOne".to_string()];
        let _docs = DocFixtures::new(&slugs, Locale::EnUs);
        let _wikihistory = WikihistoryFixtures::new(&slugs, Locale::EnUs);
        let _sidebars = SidebarFixtures::default();

        let result = do_remove("Web/API/ExampleOne", Locale::EnUs, false, None, false);
        assert!(result.is_ok());

        let should_exist = vec![];
        let should_not_exist = vec!["en-us/web/api/exampleone/index.md"];
        let root_path = root_for_locale(Locale::EnUs).unwrap();
        check_file_existence(root_path, &should_exist, &should_not_exist);

        let wiki_history = test_get_wiki_history(Locale::EnUs);
        assert!(!wiki_history.contains_key("Web/API/ExampleOne"));
    }

    #[test]
    fn test_remove_single_with_redirect() {
        let slugs = vec![
            "Web/API/ExampleOne".to_string(),
            "Web/API/RedirectTarget".to_string(),
        ];
        let _docs = DocFixtures::new(&slugs, Locale::EnUs);
        let _wikihistory = WikihistoryFixtures::new(&slugs, Locale::EnUs);
        let _redirects = RedirectFixtures::new(&vec![], Locale::EnUs);
        let _sidebars = SidebarFixtures::default();

        let result = do_remove(
            "Web/API/ExampleOne",
            Locale::EnUs,
            false,
            Some("Web/API/RedirectTarget"),
            false,
        );
        assert!(result.is_ok());

        let should_exist = vec!["en-us/web/api/redirecttarget/index.md"];
        let should_not_exist = vec!["en-us/web/api/exampleone/index.md"];
        let root_path = root_for_locale(Locale::EnUs).unwrap();
        check_file_existence(root_path, &should_exist, &should_not_exist);

        let wiki_history = test_get_wiki_history(Locale::EnUs);
        assert!(!wiki_history.contains_key("Web/API/ExampleOne"));
        assert!(wiki_history.contains_key("Web/API/RedirectTarget"));

        let redirects = get_redirects_map(Locale::EnUs);
        assert_eq!(redirects.len(), 1);
        assert!(redirects.contains_key("/en-US/docs/Web/API/ExampleOne"));
        assert_eq!(
            redirects.get("/en-US/docs/Web/API/ExampleOne").unwrap(),
            "/en-US/docs/Web/API/RedirectTarget"
        );
    }

    #[test]
    fn test_remove_single_with_subpages() {
        let slugs = vec![
            "Web/API/ExampleOne".to_string(),
            "Web/API/ExampleOne/Subpage".to_string(),
            "Web/API/RedirectTarget".to_string(),
        ];
        let _docs = DocFixtures::new(&slugs, Locale::EnUs);
        let _wikihistory = WikihistoryFixtures::new(&slugs, Locale::EnUs);
        let _redirects = RedirectFixtures::new(&vec![], Locale::EnUs);
        let _sidebars = SidebarFixtures::default();

        let result = do_remove("Web/API/ExampleOne", Locale::EnUs, false, None, false);
        assert!(result.is_ok());

        let should_exist = vec!["en-us/web/api/exampleone/subpage/index.md"];
        let should_not_exist = vec!["en-us/web/api/exampleone/index.md"];
        let root_path = root_for_locale(Locale::EnUs).unwrap();
        check_file_existence(root_path, &should_exist, &should_not_exist);

        let wiki_history = test_get_wiki_history(Locale::EnUs);
        assert!(!wiki_history.contains_key("Web/API/ExampleOne"));
        assert!(wiki_history.contains_key("Web/API/ExampleOne/Subpage"));
        assert!(wiki_history.contains_key("Web/API/RedirectTarget"));

        let redirects = get_redirects_map(Locale::EnUs);
        assert_eq!(redirects.len(), 0);
    }

    #[test]
    fn test_remove_recursive() {
        let slugs = vec![
            "Web/API/ExampleOne".to_string(),
            "Web/API/ExampleOne/Subpage".to_string(),
        ];
        let _docs = DocFixtures::new(&slugs, Locale::EnUs);
        let _wikihistory = WikihistoryFixtures::new(&slugs, Locale::EnUs);
        let _redirects = RedirectFixtures::new(&vec![], Locale::EnUs);
        let _sidebars = SidebarFixtures::default();

        let result = do_remove("Web/API/ExampleOne", Locale::EnUs, true, None, false);
        assert!(result.is_ok());

        let should_exist = vec![];
        let should_not_exist = vec![
            "en-us/web/api/exampleone/index.md",
            "en-us/web/api/exampleone/subpage/index.md",
        ];
        let root_path = root_for_locale(Locale::EnUs).unwrap();
        check_file_existence(root_path, &should_exist, &should_not_exist);

        let wiki_history = test_get_wiki_history(Locale::EnUs);
        assert!(!wiki_history.contains_key("Web/API/ExampleOne"));
        assert!(!wiki_history.contains_key("Web/API/ExampleOne/Subpage"));

        let redirects = get_redirects_map(Locale::EnUs);
        assert_eq!(redirects.len(), 0);
    }

    #[test]
    fn test_remove_recursive_with_redirect() {
        let slugs = vec![
            "Web/API/ExampleOne".to_string(),
            "Web/API/ExampleOne/Subpage".to_string(),
            "Web/API/RedirectTarget".to_string(),
        ];
        let _docs = DocFixtures::new(&slugs, Locale::EnUs);
        let _wikihistory = WikihistoryFixtures::new(&slugs, Locale::EnUs);
        let _redirects = RedirectFixtures::new(&vec![], Locale::EnUs);
        let _sidebars = SidebarFixtures::default();

        let result = do_remove(
            "Web/API/ExampleOne",
            Locale::EnUs,
            true,
            Some("Web/API/RedirectTarget"),
            false,
        );
        assert!(result.is_ok());

        let should_exist = vec![];
        let should_not_exist = vec![
            "en-us/web/api/exampleone/index.md",
            "en-us/web/api/exampleone/subpage/index.md",
        ];
        let root_path = root_for_locale(Locale::EnUs).unwrap();
        check_file_existence(root_path, &should_exist, &should_not_exist);

        let wiki_history = test_get_wiki_history(Locale::EnUs);
        assert!(!wiki_history.contains_key("Web/API/ExampleOne"));
        assert!(!wiki_history.contains_key("Web/API/ExampleOne/Subpage"));

        let redirects = get_redirects_map(Locale::EnUs);
        assert_eq!(redirects.len(), 2);
        assert_eq!(
            redirects.get("/en-US/docs/Web/API/ExampleOne").unwrap(),
            "/en-US/docs/Web/API/RedirectTarget"
        );
        assert_eq!(
            redirects
                .get("/en-US/docs/Web/API/ExampleOne/Subpage")
                .unwrap(),
            "/en-US/docs/Web/API/RedirectTarget"
        );
    }

    #[test]
    fn test_remove_recursive_with_redirect_translated() {
        let slugs = vec![
            "Web/API/ExampleOne".to_string(),
            "Web/API/ExampleOne/Subpage".to_string(),
            "Web/API/RedirectTarget".to_string(),
        ];
        let _docs = DocFixtures::new(&slugs, Locale::PtBr);
        let _wikihistory = WikihistoryFixtures::new(&slugs, Locale::PtBr);
        let _redirects = RedirectFixtures::new(&vec![], Locale::PtBr);
        let _sidebars = SidebarFixtures::default();

        let result = do_remove(
            "Web/API/ExampleOne",
            Locale::PtBr,
            true,
            Some("Web/API/RedirectTarget"),
            false,
        );
        assert!(result.is_ok());

        let should_exist = vec![];
        let should_not_exist = vec![
            "pt-br/web/api/exampleone/index.md",
            "pt-br/web/api/exampleone/subpage/index.md",
        ];
        let root_path = root_for_locale(Locale::PtBr).unwrap();
        check_file_existence(root_path, &should_exist, &should_not_exist);

        let wiki_history = test_get_wiki_history(Locale::PtBr);
        assert!(!wiki_history.contains_key("Web/API/ExampleOne"));
        assert!(!wiki_history.contains_key("Web/API/ExampleOne/Subpage"));

        let redirects = get_redirects_map(Locale::PtBr);
        assert_eq!(redirects.len(), 2);
        assert_eq!(
            redirects.get("/pt-BR/docs/Web/API/ExampleOne").unwrap(),
            "/pt-BR/docs/Web/API/RedirectTarget"
        );
        assert_eq!(
            redirects
                .get("/pt-BR/docs/Web/API/ExampleOne/Subpage")
                .unwrap(),
            "/pt-BR/docs/Web/API/RedirectTarget"
        );
    }
}
