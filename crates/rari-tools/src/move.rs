use std::str::FromStr;

use rari_doc::{
    build,
    docs::{
        doc,
        page::{self, PageCategory},
    },
    resolve::build_url,
};
use rari_types::locale::Locale;

use crate::error::ToolError;

pub fn r#move(
    old_slug: &str,
    new_slug: &str,
    locale: Option<&str>,
    assume_yes: bool,
) -> Result<(), ToolError> {
    println!(
        "Hello, world! {} {} {:?} {}",
        old_slug, new_slug, locale, assume_yes
    );
    validate_args(old_slug, new_slug)?;
    let locale = if let Some(l) = locale {
        Locale::from_str(l)?
    } else {
        Locale::default()
    };
    // println!("Locale: {:?}", locale);

    // Make a dry run to give some feedback on what would be done
    let changes = do_move(old_slug, new_slug, &locale, true)?;

    Ok(())
}

fn do_move(
    old_slug: &str,
    new_slug: &str,
    locale: &Locale,
    dry_run: bool,
) -> Result<(), ToolError> {
    let old_url = build_url(old_slug, locale, PageCategory::Doc);
    let doc = page::Page::page_from_url_path(&old_url)?;
    let new_parent_slug = println!("doc: {:?}", doc);

    Ok(())
}

fn parent_slug(slug: &str) -> Option<&str> {
    let slug = slug.trim_end_matches('/');
    if let Some(i) = slug.rfind('/') {
        Some(&slug[..i])
    } else {
        None
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
        println!("{:?}", validate_args("old", "new#"))
    }

    #[test]
    fn test_parent_slug() {
        assert_eq!(parent_slug("a/b/c"), Some("a/b"));
        assert_eq!(parent_slug("a/b"), Some("a"));
        assert_eq!(parent_slug("a"), None);
        assert_eq!(parent_slug("a/b/c/"), Some("a/b"));
    }
}
