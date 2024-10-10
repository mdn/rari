use std::borrow::Cow;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

use console::Style;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirm;
use rari_doc::helpers::subpages::get_sub_pages;
use rari_doc::pages::page::{self, PageCategory, PageLike};
use rari_doc::resolve::{build_url, url_meta_from, UrlMeta};
use rari_doc::utils::root_for_locale;
use rari_types::locale::Locale;

use crate::error::ToolError;
use crate::redirects::add_redirects;
use crate::wikihistory::delete_from_wiki_history;

pub fn remove(
    slug: &str,
    locale: Option<&str>,
    recursive: bool,
    redirect: Option<&str>,
    assume_yes: bool,
) -> Result<(), ToolError> {
    validate_args(slug)?;
    let locale = if let Some(l) = locale {
        Locale::from_str(l)?
    } else {
        Locale::default()
    };

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
        println!(
            "{} {} {}",
            green.apply_to("Deleted"),
            bold.apply_to(removed.len()),
            green.apply_to("documents"),
        );

        // Find references to deleted documents and
        // list them for manual review
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
    let url = build_url(slug, locale, PageCategory::Doc)?;
    let doc = page::Page::from_url(&url)?;
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

    let subpages = get_sub_pages(&url, None, Default::default())?;
    if !recursive && !subpages.is_empty() && redirect.is_some() {
        return Err(ToolError::HasSubpagesError(Cow::Owned(format!(
            "{0}, unable to remove and redirect a document with children",
            slug
        ))));
    }

    let slugs_to_remove = if recursive {
        [doc.clone()]
            .iter()
            .chain(&subpages)
            .map(|page_ref| page_ref.slug().to_owned())
            .collect::<Vec<_>>()
    } else {
        vec![real_slug.to_owned()]
    };

    if dry_run {
        return Ok(slugs_to_remove);
    }

    // Remove the documents. For single documents, we just remove the `index.md` file and
    // leave the folder structure in place. For recursive removal, we remove the entire
    // folder structure, duplicating the original yari tool behaviour.

    // Conditional command for testing. In testing, we do not use git, because the test
    // fixtures are not under git control. Instead of `git rm …` we use `rm …`.
    let mut path = PathBuf::from(locale.as_folder_str());
    let url = build_url(real_slug, locale, PageCategory::Doc)?;
    let UrlMeta { folder_path, .. } = url_meta_from(&url)?;
    path.push(folder_path);

    let command = if cfg!(test) { "rm" } else { "git" };
    if recursive {
        let args = if cfg!(test) {
            vec![OsStr::new("-rf"), path.as_os_str()]
        } else {
            vec![OsStr::new("rm"), OsStr::new("-rf"), path.as_os_str()]
        };

        // Execute the recursive remove command
        let output = Command::new(command)
            .args(args)
            .current_dir(root_for_locale(locale)?)
            .output()
            .expect("failed to execute process");

        if !output.status.success() {
            return Err(ToolError::GitError(format!(
                "Failed to remove files: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
    } else {
        path.push("index.md");
        let args = if cfg!(test) {
            vec![path.as_os_str()]
        } else {
            vec![OsStr::new("rm"), &path.as_os_str()]
        };

        // Execute the single file remove command
        let output = Command::new(command)
            .args(args)
            .current_dir(root_for_locale(locale)?)
            .output()
            .expect("failed to execute process");

        if !output.status.success() {
            return Err(ToolError::GitError(format!(
                "Failed to remove files: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
    }

    // update the wiki history
    delete_from_wiki_history(locale, &slugs_to_remove)?;

    // update the redirects map if needed
    if let Some(new_slug) = redirect_target {
        let pairs = slugs_to_remove
            .iter()
            .map(|slug| {
                let old_url = build_url(slug, locale, PageCategory::Doc)?;
                // let new_url = build_url(&new_slug, locale, PageCategory::Doc)?;
                Ok((old_url, new_slug.to_owned()))
            })
            .collect::<Result<Vec<_>, ToolError>>()?;
        add_redirects(locale, &pairs)?;
    }

    // check references to the removed documents (do it in the main method)

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
// Using `file_serial` as a synchonization lock, we run all tests using
// the same `key` (here: file_fixtures) to be serialized across modules.
#[cfg(test)]
use serial_test::file_serial;
#[cfg(test)]
#[file_serial(file_fixtures)]
mod test {

    use super::*;
    use crate::tests::fixtures::docs::DocFixtures;
    use crate::tests::fixtures::redirects::RedirectFixtures;
    use crate::tests::fixtures::wikihistory::WikihistoryFixtures;
    use crate::utils::test_utils::{check_file_existence, get_redirects_map};
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
