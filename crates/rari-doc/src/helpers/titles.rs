use rari_types::fm_types::PageType;

use crate::docs::page::{Page, PageLike};

pub fn api_page_title(page: &Page) -> &str {
    if let Some(short_title) = page.short_title() {
        return short_title;
    }
    let title = page.title();
    let title = &title[title.rfind('.').map(|i| i + 1).unwrap_or(0)..];
    if matches!(page.page_type(), PageType::WebApiEvent) {
        let title = page.slug();
        let title = &title[title.rfind('/').map(|i| i + 1).unwrap_or(0)..];
        if let Some(title) = title.strip_suffix("_event") {
            title
        } else {
            title
        }
    } else {
        title
    }
}
