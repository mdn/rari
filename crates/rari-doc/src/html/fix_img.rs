use std::error::Error;
use std::path::Path;

use lol_html::HandlerResult;
use lol_html::html_content::Element;
use percent_encoding::percent_decode_str;
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
        if url.host() == base.host()
            && !url.path().starts_with("/assets/")
            && !url.path().starts_with("/shared-assets/")
        {
            // MDN content requires all filenames to be lowercase, so we
            // normalise the src path to avoid case-sensitivity issues on Linux.
            let src = src.to_lowercase();
            let url = base_url.parse(&src)?;

            // Check if the file exists in the current locale.
            // The src may be percent-encoded (comrak encodes non-ASCII characters
            // in URLs), so decode it before constructing the filesystem path.
            let decoded_src = percent_decode_str(&src).decode_utf8()?;
            let mut file = page
                .full_path()
                .parent()
                .unwrap()
                .join(decoded_src.as_ref());
            let mut final_url_path = url.path().to_string();

            // If file doesn't exist in translated locale, try en-US fallback
            if !file.try_exists().unwrap_or_default()
                && page.locale() != default_locale()
                && let Ok(en_us_page) =
                    Page::from_url_with_locale_and_fallback(page.url(), default_locale())
            {
                let en_us_file = en_us_page
                    .full_path()
                    .parent()
                    .unwrap()
                    .join(decoded_src.as_ref());
                if en_us_file.try_exists().unwrap_or_default() {
                    // Rewrite URL to point to en-US asset
                    let en_us_url = en_us_page.url();
                    final_url_path = format!(
                        "{}{}{}",
                        en_us_url,
                        if en_us_url.ends_with('/') { "" } else { "/" },
                        src
                    );
                    file = en_us_file;
                }
            }

            el.set_attribute("src", &final_url_path)?;

            // Leave dimensions alone if we have a `width` attribute
            if el.get_attribute("width").is_some() {
                return Ok(());
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

#[cfg(test)]
mod tests {
    use lol_html::{RewriteStrSettings, element, rewrite_str};
    use url::Url;

    use super::handle_img;
    use crate::test_utils::TestPage;

    // Minimal GIF recognised by imagesize: the format detector reads a 12-byte
    // header, then the GIF size parser seeks to byte 6 and reads width/height
    // as little-endian u16 values. 12 bytes is therefore the minimum.
    const TINY_GIF: &[u8] = b"GIF89a\x01\x00\x01\x00\x00\x00";

    fn rewrite_img(html: &str, page: &TestPage) -> String {
        let options = Url::options();
        let base = Url::parse(&format!(
            "http://rari.placeholder{}{}",
            page.url,
            if page.url.ends_with('/') { "" } else { "/" }
        ))
        .unwrap();
        let base_url = options.base_url(Some(&base));
        rewrite_str(
            html,
            RewriteStrSettings {
                element_content_handlers: vec![element!("img[src]", |el| {
                    handle_img(el, page, false, &base, &base_url)
                })],
                ..Default::default()
            },
        )
        .unwrap()
    }

    /// Filenames with non-ASCII characters (e.g. accents, Cyrillic) are
    /// percent-encoded by the Markdown renderer (comrak).  `handle_img` must
    /// decode them before constructing the filesystem path, otherwise the file
    /// is never found and no `width`/`height` attributes are set.
    #[test]
    fn test_accented_filename_gets_dimensions() {
        let tmp = std::env::temp_dir().join("rari-test-accented-img");
        std::fs::create_dir_all(&tmp).unwrap();
        // Actual filename on disk uses the literal UTF-8 character.
        std::fs::write(tmp.join("bézier.gif"), TINY_GIF).unwrap();

        let page = TestPage {
            path: tmp.join("index.md"),
            url: "/en-US/docs/Test".to_string(),
            ..Default::default()
        };

        // comrak encodes `é` (U+00E9, UTF-8 0xC3 0xA9) as `%C3%A9`.
        let output = rewrite_img(r#"<img src="b%C3%A9zier.gif">"#, &page);

        assert!(
            output.contains("width=\"1\""),
            "expected width attribute; got: {output}"
        );
        assert!(
            output.contains("height=\"1\""),
            "expected height attribute; got: {output}"
        );
    }
}
