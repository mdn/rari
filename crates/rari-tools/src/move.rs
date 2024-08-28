use std::str::FromStr;

use rari_doc::{
    docs::page::{self, PageCategory, PageLike},
    resolve::build_url,
};
use rari_types::locale::Locale;

use crate::error::ToolError;

pub fn r#move(
    old_slug: &str,
    new_slug: &str,
    locale: Option<&str>,
    _assume_yes: bool,
) -> Result<(), ToolError> {
    validate_args(old_slug, new_slug)?;
    let locale = if let Some(l) = locale {
        Locale::from_str(l)?
    } else {
        Locale::default()
    };
    // println!("Locale: {:?}", locale);

    // Make a dry run to give some feedback on what would be done
    let _changes = do_move(old_slug, new_slug, &locale, true)?;

    Ok(())
}

fn do_move(
    old_slug: &str,
    new_slug: &str,
    locale: &Locale,
    _dry_run: bool,
) -> Result<(), ToolError> {
    let old_url = build_url(old_slug, locale, PageCategory::Doc);
    let doc = page::Page::page_from_url_path(&old_url)?;
    let new_parent_slug = parent_slug(new_slug)?;
    let real_old_slug = doc.slug();
    println!("new_parent_slug: {new_parent_slug} real_old_slug: {real_old_slug}");
    Ok(())
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
    use rari_types::globals::settings;
    use std::env;

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
            "Web/API/AbortController",
            "Web/API/AbortControllerAlternative",
            &Locale::default(),
            true,
        );
        println!("result: {:?}", result);
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
