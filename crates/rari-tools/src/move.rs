use console::{style, Style};
use dialoguer::{theme::ColorfulTheme, Confirm};

use rari_doc::{
    helpers::subpages::get_sub_pages,
    pages::page::{self, Page, PageCategory, PageLike, PageWriter},
    resolve::build_url,
};
use rari_types::locale::Locale;
use std::{str::FromStr, sync::Arc};

use crate::error::ToolError;

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
    // println!("Locale: {:?}", locale);

    // Make a dry run to give some feedback on what would be done
    let green = Style::new().green();
    let red = Style::new().red();
    let bold = Style::new().bold();
    let changes = do_move(old_slug, new_slug, &locale, true)?;
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
        let moved = do_move(old_slug, new_slug, &locale, false)?;
        println!(
            "{} {} {}",
            green.apply_to("Moved"),
            bold.apply_to(moved.len()),
            green.apply_to("documents"),
        );
        println!("Done");
    } else {
        return Ok(());
    }

    Ok(())
}

fn do_move(
    old_slug: &str,
    new_slug: &str,
    locale: &Locale,
    dry_run: bool,
) -> Result<Vec<(String, String)>, ToolError> {
    let old_url = build_url(old_slug, locale, PageCategory::Doc)?;
    let doc = page::Page::page_from_url_path(&old_url)?;

    let new_parent_slug = parent_slug(new_slug)?;
    let real_old_slug = doc.slug();
    // println!("new_parent_slug: {new_parent_slug} real_old_slug: {real_old_slug}");
    let subpages = get_sub_pages(&old_url, None, Default::default())?;
    // println!("subpages: {subpages:?}");
    let pairs = vec![doc.clone()]
        .iter()
        .chain(&subpages)
        .map(|page_ref| {
            let slug = page_ref.slug().to_owned();
            let new_slug = slug.replace(&real_old_slug, new_slug);
            (slug, new_slug)
        })
        .collect::<Vec<_>>();
    // println!("pairs: {:?}", &pairs);
    if dry_run {
        return Ok(pairs);
    }

    if let Page::Doc(doc) = doc {
        if let Ok(mut doc) = Arc::try_unwrap(doc) {
            doc.meta.slug = new_slug.to_string();
            doc.meta.path = build_url(new_slug, locale, PageCategory::Doc)?.into();
            println!(
                "doc.meta.slug: {} doc.meta.path: {:?}",
                doc.meta.slug, doc.meta.path
            );
            doc.write()?;
        } else {
            return Err(ToolError::Unknown("Failed to unwrap Arc".to_string()));
        }
    }
    // await update(oldUrl, doc.rawBody, doc.metadata);
    // test writing to file
    // Now, go through the pairs and move the files
    // for (old_slug, new_slug) in pairs {
    // let old_path = urlToFilePath(old_slug);
    // let new_path = urlToFilePath(new_slug);
    // println!("old_path: {old_path} new_path: {new_path}");
    // fs.renameSync(old_path, new_path);
    // }

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

// fn find_children(url: &str, recursive: bool) -> Vec<String> {
//   let locale = url.split("/")[1];
//   let root = getRoot(locale);
//   let folder = urlToFolderPath(url);

//   let childPaths = childrenFoldersForPath(root, folder, recursive);
//   return childPaths.map((folder) => read(folder));
// }

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
            &Locale::default(),
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
