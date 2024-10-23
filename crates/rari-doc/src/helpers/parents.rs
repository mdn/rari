use super::title::transform_title;
use crate::pages::json::Parent;
use crate::pages::page::{Page, PageLike};

pub fn parents<T: PageLike>(doc: &T) -> Vec<Parent> {
    let mut url = doc.url();
    let mut parents = vec![Parent {
        uri: url.into(),
        title: doc
            .short_title()
            .unwrap_or(transform_title(doc.title()))
            .to_string(),
    }];
    while let Some(i) = url.trim_end_matches('/').rfind('/') {
        let parent_url = &url[..if doc.trailing_slash() { i + 1 } else { i }];
        if parent_url.ends_with(doc.base_slug()) {
            break;
        }
        if let Ok(parent) = Page::from_url_with_default_fallback(parent_url) {
            parents.push(Parent {
                uri: parent.url().into(),
                title: parent
                    .short_title()
                    .unwrap_or(transform_title(parent.title()))
                    .to_string(),
            })
        }
        url = parent_url
    }
    parents.reverse();
    parents
}
