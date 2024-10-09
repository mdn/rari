use std::borrow::Cow;

use crate::error::ToolError;

pub(crate) fn parent_slug(slug: &str) -> Result<&str, ToolError> {
    let slug = slug.trim_end_matches('/');
    if let Some(i) = slug.rfind('/') {
        Ok(&slug[..i])
    } else {
        Err(ToolError::InvalidSlug(Cow::Borrowed("slug has no parent")))
    }
}
