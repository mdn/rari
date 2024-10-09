use std::borrow::Cow;
use std::str::FromStr;

use console::{style, Style};
use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirm;
use rari_doc::helpers::subpages::get_sub_pages;
use rari_doc::pages::page::{self, PageCategory, PageLike};
use rari_doc::resolve::build_url;
use rari_types::locale::Locale;

use crate::error::ToolError;
use crate::utils::parent_slug;

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
            green.apply_to("Removed"),
            bold.apply_to(removed.len()),
            green.apply_to("documents"),
        );
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
    let redirect_value = if let Some(redirect_str) = redirect {
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
    if !recursive && !subpages.is_empty() {
        return Err(ToolError::HasSubpagesError(Cow::Owned(format!(
            "{0}, use --recursive to delete recursively",
            slug
        ))));
    }

    let slugs_to_remove = [doc.clone()]
        .iter()
        .chain(&subpages)
        .map(|page_ref| page_ref.slug().to_owned())
        .collect::<Vec<_>>();

    if dry_run {
        return Ok(slugs_to_remove);
    }

    let removed: Vec<String> = Vec::new();

    // let new_parent_slug = parent_slug(slug)?;
    // if !page::Page::exists(&build_url(new_parent_slug, locale, PageCategory::Doc)?) {
    //     return Err(ToolError::InvalidSlug(Cow::Owned(format!(
    //         "new parent slug does not exist: {new_parent_slug}"
    //     ))));
    // }
    // // let subpages = get_sub_pages(&old_url, None, Default::default())?;

    return Ok(vec![]);
}

fn validate_args(slug: &str) -> Result<(), ToolError> {
    if slug.is_empty() {
        return Err(ToolError::InvalidSlug(Cow::Borrowed(
            "old slug cannot be empty",
        )));
    }
    if slug.contains("#") {
        return Err(ToolError::InvalidSlug(Cow::Borrowed(
            "old slug cannot contain '#'",
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

    // use std::collections::HashMap;
    // use std::path::Path;

    use super::*;
    use crate::tests::fixtures::docs::DocFixtures;
    // use crate::redirects;
    // use crate::tests::fixtures::docs::DocFixtures;
    // use crate::tests::fixtures::redirects::RedirectFixtures;
    // use crate::tests::fixtures::wikihistory::WikihistoryFixtures;

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
        ];
        let _docs = DocFixtures::new(&slugs, Locale::EnUs);

        // no recursive, we bail because of existing subpages
        let result = do_remove("Web/API/ExampleOne", Locale::EnUs, false, None, true);
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
        assert!(matches!(result, Err(ToolError::DocError(_))));
        assert!(result.is_err());
    }
}
