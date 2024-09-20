use console::{style, Style};
use dialoguer::{theme::ColorfulTheme, Confirm};

use rari_doc::{
    helpers::subpages::get_sub_pages,
    pages::page::{self, Page, PageCategory, PageLike, PageWriter},
    redirects::add_redirects,
    resolve::{build_url, url_path_to_path_buf},
    utils::root_for_locale,
};
use rari_types::locale::Locale;
use std::{fs::create_dir_all, path::PathBuf, process::Command, str::FromStr, sync::Arc};

use crate::{error::ToolError, wikihistory::update_wiki_history};

pub fn r#move(
    old_slug: &str,
    new_slug: &str,
    locale: Option<&str>,
    assume_yes: bool,
) -> Result<(), ToolError> {
    validate_args(old_slug, new_slug)?;
    let locale = if let Some(l) = locale {
        Locale::from_str(l)?
    } else {
        Locale::default()
    };

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
            .unwrap()
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
    let old_url = build_url(old_slug, &locale, PageCategory::Doc)?;
    let doc = page::Page::page_from_url_path(&old_url)?;
    let real_old_slug = doc.slug();

    let _new_parent_slug = parent_slug(new_slug)?;
    let subpages = get_sub_pages(&old_url, None, Default::default())?;

    let is_new_slug = real_old_slug != new_slug;

    // Return early if we move onto ourselves.
    if !is_new_slug {
        return Ok(vec![]);
    }

    let pairs = vec![doc.clone()]
        .iter()
        .chain(&subpages)
        .map(|page_ref| {
            let slug = page_ref.slug().to_owned();
            let new_slug = slug.replace(&real_old_slug, new_slug);
            (slug, new_slug)
        })
        .collect::<Vec<_>>();

    // Return early for a dry run.
    if dry_run {
        return Ok(pairs);
    }

    // No dry run, so build a vec of pairs of `(old_page, Option<new_doc>)`.
    let dv = vec![doc.clone()];
    let doc_pairs = dv
        .iter()
        .chain(&subpages)
        .map(|page_ref| {
            let slug = page_ref.slug().to_owned();
            let new_slug = slug.replace(&real_old_slug, new_slug);
            let new_page = page_ref.clone();
            if let Page::Doc(doc) = new_page {
                let mut cloned_doc = doc.clone();
                let doc = Arc::make_mut(&mut cloned_doc);
                doc.meta.slug = new_slug.to_string();
                (page_ref, Some(doc.to_owned()))
            } else {
                println!("This does not look like a document");
                (page_ref, None)
            }
        })
        .collect::<Vec<_>>();

    // Now iterate through the vec and write the new frontmatter
    // (the changed slug) to all affected documents (root + children).
    // The docs are all still in their old location at this time.
    doc_pairs.iter().try_for_each(|(_page_ref, new_doc)| {
        if let Some(new_doc) = new_doc {
            new_doc.write()?;
        }
        Ok::<(), ToolError>(())
    })?;

    // Now we use the git command to move the whole parent directory
    // to a new location. This will move all children as well and
    // makes sure that we get a proper "file moved" in the git history.

    let mut old_folder_path = PathBuf::new();
    old_folder_path.push(locale.as_folder_str());
    let url = build_url(real_old_slug, &locale, PageCategory::Doc)?;
    let (path, _, _, _) = url_path_to_path_buf(&url)?;
    old_folder_path.push(path);

    let mut new_folder_path = PathBuf::new();
    new_folder_path.push(locale.as_folder_str());
    let url = build_url(new_slug, &locale, PageCategory::Doc)?;
    let (path, _, _, _) = url_path_to_path_buf(&url)?;
    new_folder_path.push(path);

    // Make sure the target parent directory exists.
    if let Some(target_parent_path) = new_folder_path.as_path().parent() {
        let absolute_target_parent_path = root_for_locale(locale)?.join(target_parent_path);
        create_dir_all(absolute_target_parent_path)?;
    } else {
        return Err(ToolError::Unknown(
            "Could not determine parent path for new folder".to_string(),
        ));
    }

    // Execute the git move.
    let output = Command::new("git")
        .args([
            "mv",
            &old_folder_path.to_string_lossy(),
            &new_folder_path.to_string_lossy(),
        ])
        .current_dir(root_for_locale(locale)?)
        .output()
        .expect("failed to execute process");

    let output_str = String::from_utf8_lossy(&output.stdout);
    let err_str = String::from_utf8_lossy(&output.stderr);
    println!(
        "cd {} && git mv {} {}\noutput_str: {} err_str: {} status: {}",
        root_for_locale(locale)?.display(),
        &old_folder_path.to_string_lossy(),
        &new_folder_path.to_string_lossy(),
        output_str,
        err_str,
        output.status
    );

    update_wiki_history(locale, &pairs)?;

    // Update the redirect map.
    add_redirects(locale, &pairs)?;
    // finally, return the pairs of old and new slugs
    Ok(pairs)
}

fn parent_slug(slug: &str) -> Result<&str, ToolError> {
    let slug = slug.trim_end_matches('/');
    if let Some(i) = slug.rfind('/') {
        Ok(&slug[..i])
    } else {
        Err(ToolError::InvalidSlug("slug has no parent".to_string()))
    }
}

fn validate_args(old_slug: &str, new_slug: &str) -> Result<(), ToolError> {
    if old_slug.is_empty() {
        return Err(ToolError::InvalidSlug(
            "old slug cannot be empty".to_string(),
        ));
    }
    if new_slug.is_empty() {
        return Err(ToolError::InvalidSlug(
            "new slug cannot be empty".to_string(),
        ));
    }
    if old_slug.contains("#") {
        return Err(ToolError::InvalidSlug(
            "old slug cannot contain '#'".to_string(),
        ));
    }
    if new_slug.contains("#") {
        return Err(ToolError::InvalidSlug(
            "new slug cannot contain '#'".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod test {

    use super::*;

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
    fn test_do_move() {
        // Test case where old_slug and new_slug are valid
        let result = do_move(
            "Web/API/ExampleOne",
            "Web/API/ExampleOneNewLocation",
            Locale::default(),
            true,
        );
        println!("result: {:?}", result);
        assert!(result.is_ok())
        // assert!(do_move(
        //     "Web/API/AbortController",
        //     "Web/API/AbortControllerAlternative",
        //     &Locale::default(),
        //     true
        // )
        // .is_err());

        // // Test case where old_slug is empty
        // assert!(do_move("", "new", &Locale::default(), true).is_err());

        // // Test case where new_slug is empty
        // assert!(do_move("old", "", &Locale::default(), true).is_err());

        // // Test case where old_slug contains '#'
        // assert!(do_move("old#", "new", &Locale::default(), true).is_err());

        // // Test case where new_slug contains '#'
        // assert!(do_move("old", "new#", &Locale::default(), true).is_err());
    }
}
