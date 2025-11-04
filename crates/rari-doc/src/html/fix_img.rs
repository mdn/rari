use std::error::Error;
use std::path::Path;

use lol_html::html_content::Element;
use lol_html::HandlerResult;
use rari_types::locale::default_locale;
use tracing::warn;
use url::{ParseOptions, Url};

use crate::issues::get_issue_counter;
use crate::pages::page::{Page, PageLike};

type ImgSize = (Option<String>, Option<String>);

pub fn handle_img(
    el: &mut Element,
    page: &impl PageLike,
    data_issues: bool,
    base: &Url,
    base_url: &ParseOptions,
) -> HandlerResult {
    if let Some(src) = el.get_attribute("src") {
        let url = base_url.parse(&src)?;
        if url.host() == base.host() && !url.path().starts_with("/assets/") {
            el.set_attribute("src", url.path())?;
            // Leave dimensions alone if we have a `width` attribute
            if el.get_attribute("width").is_some() {
                return Ok(());
            }
            let mut file = page.full_path().parent().unwrap().join(&src);
            if !file.try_exists().unwrap_or_default() {
                if let Ok(en_us_page) =
                    Page::from_url_with_locale_and_fallback(page.url(), default_locale())
                {
                    file = en_us_page.full_path().parent().unwrap().join(&src);
                }
            }
            let (width, height) = img_size(el, &src, &file, data_issues)?;
            if let Some(width) = width {
                el.set_attribute("width", &width)?;
            }
            if let Some(height) = height {
                el.set_attribute("height", &height)?;
            }
        }
    }
    Ok(())
}

pub fn img_size(
    el: &mut Element,
    src: &str,
    file: &Path,
    data_issues: bool,
) -> Result<ImgSize, Box<dyn Error + Send + Sync>> {
    let (width, height) = if src.ends_with(".svg") {
        match svg_metadata::Metadata::parse_file(file) {
            // If only width and viewbox are given, use width and scale
            // the height according to the viewbox size ratio.
            // If width and height are given, use these.
            // If only a viewbox is given, use the viewbox values.
            // If only height and viewbox are given, use height and scale
            // the height according to the viewbox size ratio.
            Ok(meta) => {
                let width = meta.width.map(|w| w.width);
                let height = meta.height.map(|h| h.height);
                let view_box = meta.view_box;

                let (final_width, final_height) = match (width, height, view_box) {
                    // Both width and height are given
                    (Some(w), Some(h), _) => (Some(w), Some(h)),
                    // Only width and viewbox are given
                    (Some(w), None, Some(vb)) => (Some(w), Some(w * vb.height / vb.width)),
                    // Only height and viewbox are given
                    (None, Some(h), Some(vb)) => (Some(h * vb.width / vb.height), Some(h)),
                    // Only viewbox is given
                    (None, None, Some(vb)) => (Some(vb.width), Some(vb.height)),
                    // Only width is given
                    (Some(w), None, None) => (Some(w), None),
                    // Only height is given
                    (None, Some(h), None) => (None, Some(h)),
                    // Neither width, height, nor viewbox are given
                    (None, None, None) => (None, None),
                };

                (
                    final_width.map(|w| format!("{w:.0}")),
                    final_height.map(|h| format!("{h:.0}")),
                )
            }
            Err(e) => {
                let ic = get_issue_counter();
                warn!(
                    source = "image-check",
                    ic = ic,
                    "Error parsing {}: {e}",
                    file.display()
                );
                if data_issues {
                    el.set_attribute("data-flaw", &ic.to_string())?;
                }
                (None, None)
            }
        }
    } else {
        match imagesize::size(file) {
            Ok(dim) => (Some(dim.width.to_string()), Some(dim.height.to_string())),
            Err(e) => {
                let ic = get_issue_counter();
                warn!(
                    source = "image-check",
                    ic = ic,
                    "Error opening {}: {e}",
                    file.display()
                );
                if data_issues {
                    el.set_attribute("data-flaw", &ic.to_string())?;
                }

                (None, None)
            }
        }
    };
    Ok((width, height))
}
