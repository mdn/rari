use std::error::Error;
use std::path::{Component, Path, PathBuf};

use lol_html::HandlerResult;
use lol_html::html_content::Element;
use percent_encoding::percent_decode_str;
use rari_types::locale::{Locale, default_locale};
use tracing::warn;
use url::{ParseOptions, Url};

use crate::issues::get_issue_counter;
use crate::pages::page::{Page, PageLike};
use crate::resolve::{strip_locale_from_url, url_to_folder_path};
use crate::utils::root_for_locale;

type ImgSize = (Option<String>, Option<String>);

/// Maps an absolute `/<locale>/docs/<slug>/<file>` src to its file under the
/// locale's content root, optionally overriding the locale.
///
/// Returns `None` for non-doc URLs, if the locale's content root is not
/// configured, or if the tail would leave the root (e.g. `..` or empty segments).
fn absolute_src_path(src: &str, locale_override: Option<Locale>) -> Option<PathBuf> {
    let (locale, rest) = strip_locale_from_url(src);
    let tail = rest.strip_prefix("/docs/")?;
    // A src without a locale is not a doc URL, even with an override.
    let locale = locale_override.unwrap_or(locale?);
    // Only the slug is mapped; the filename is kept raw like in relative srcs.
    let (dir, file) = tail.rsplit_once('/').unwrap_or(("", tail));
    let relative = url_to_folder_path(dir).join(file);
    if !relative
        .components()
        .all(|c| matches!(c, Component::Normal(_)))
    {
        return None;
    }
    Some(
        root_for_locale(locale)
            .ok()?
            .join(locale.as_folder_str())
            .join(relative),
    )
}

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
            // Absolute srcs are resolved from the normalised URL path (no query,
            // fragment or dot segments), matching the emitted `src`.
            let trimmed = src.trim();
            let absolute_path = (trimmed.starts_with(['/', '\\']) || Url::parse(trimmed).is_ok())
                .then(|| {
                    percent_decode_str(url.path())
                        .decode_utf8()
                        .map(|p| p.into_owned())
                })
                .transpose()?;
            // Absolute srcs never fall back to the page folder: `Path::join`
            // would discard the base and probe the literal filesystem path.
            let mut file = match absolute_path.as_deref() {
                Some(p) => absolute_src_path(p, None),
                None => Some(
                    page.full_path()
                        .parent()
                        .unwrap()
                        .join(decoded_src.as_ref()),
                ),
            };
            let mut final_url_path = url.path().to_string();

            let exists = |file: &Path| file.try_exists().unwrap_or_default();

            // If file doesn't exist in translated locale, try en-US fallback
            if !file.as_deref().is_some_and(exists) && page.locale() != default_locale() {
                match absolute_path.as_deref() {
                    Some(p) => {
                        if let Some(en_us_file) = absolute_src_path(p, Some(default_locale()))
                            && exists(&en_us_file)
                        {
                            // Rewrite URL to point to en-US asset
                            final_url_path = format!(
                                "/{}{}",
                                default_locale().as_url_str(),
                                strip_locale_from_url(url.path()).1
                            );
                            file = Some(en_us_file);
                        }
                    }
                    None => {
                        if let Ok(en_us_page) =
                            Page::from_url_with_locale_and_fallback(page.url(), default_locale())
                        {
                            let en_us_file = en_us_page
                                .full_path()
                                .parent()
                                .unwrap()
                                .join(decoded_src.as_ref());
                            if exists(&en_us_file) {
                                // Rewrite URL to point to en-US asset
                                let en_us_url = en_us_page.url();
                                final_url_path = format!(
                                    "{}{}{}",
                                    en_us_url,
                                    if en_us_url.ends_with('/') { "" } else { "/" },
                                    src
                                );
                                file = Some(en_us_file);
                            }
                        }
                    }
                }
            }

            el.set_attribute("src", &final_url_path)?;

            // Leave dimensions alone if we have a `width` attribute
            if el.get_attribute("width").is_some() {
                return Ok(());
            }
            let Some(file) = file else {
                let ic = get_issue_counter();
                warn!(
                    source = "image-check",
                    ic = ic,
                    "Cannot resolve {final_url_path}"
                );
                if data_issues {
                    el.set_attribute("data-flaw", &ic.to_string())?;
                }
                return Ok(());
            };
            let (width, height) = img_size(el, &final_url_path, &file, data_issues)?;
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
    use std::path::PathBuf;

    use lol_html::{RewriteStrSettings, element, rewrite_str};
    use rari_types::globals::{content_root, content_translated_root};
    use rari_types::locale::Locale;
    use url::Url;

    use super::{absolute_src_path, handle_img};
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
            RewriteStrSettings::new().append_element_content_handler(element!("img[src]", |el| {
                handle_img(el, page, true, &base, &base_url)
            })),
        )
        .unwrap()
    }

    /// Filenames with non-ASCII characters (e.g. accents, Cyrillic) are
    /// percent-encoded by the Markdown renderer (comrak).  `handle_img` must
    /// decode them before constructing the filesystem path, otherwise the file
    /// is never found and no `width`/`height` attributes are set.
    #[test]
    fn test_accented_filename_gets_dimensions() {
        let tmp = tempfile::TempDir::new().unwrap();
        // Actual filename on disk uses the literal UTF-8 character.
        std::fs::write(tmp.path().join("bézier.gif"), TINY_GIF).unwrap();

        let page = TestPage {
            path: tmp.path().join("index.md"),
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

    // 20x10 viewBox-only SVG.
    const TINY_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 10"></svg>"#;

    /// Removes the fixture folders from the shared test content roots on drop.
    struct Fixtures(Vec<PathBuf>);

    impl Drop for Fixtures {
        fn drop(&mut self) {
            for dir in &self.0 {
                let _ = std::fs::remove_dir_all(dir);
            }
        }
    }

    #[test]
    fn test_src_resolution() {
        struct Case {
            name: &'static str,
            locale: Locale,
            src: String,
            expected_src: String,
            expected_size: Option<(&'static str, &'static str)>,
        }

        // Unique per run, so stale fixtures from an aborted run can't interfere.
        let slug = format!("FixImgTest{}", std::process::id());
        let folder = slug.to_lowercase();
        let en_dir = content_root().join("en-us/web").join(&folder);
        let fr_dir = content_translated_root()
            .expect("translated root configured")
            .join("fr/web")
            .join(&folder);
        assert!(!en_dir.exists() && !fr_dir.exists(), "stale fixtures");
        let _cleanup = Fixtures(vec![en_dir.clone(), fr_dir.clone()]);

        let en_page_dir = en_dir.join("api/child");
        let en_other_dir = en_dir.join("other");
        let fr_page_dir = fr_dir.join("api/child");
        let fr_other_dir = fr_dir.join("other");
        for dir in [&en_page_dir, &en_other_dir, &fr_page_dir, &fr_other_dir] {
            std::fs::create_dir_all(dir).unwrap();
        }
        // The en-US page is needed for the relative-src fallback of translated pages.
        std::fs::write(
            en_page_dir.join("index.md"),
            format!("---\ntitle: Child\nslug: Web/{slug}/Api/Child\n---\n"),
        )
        .unwrap();
        std::fs::write(en_page_dir.join("local.gif"), TINY_GIF).unwrap();
        std::fs::write(en_other_dir.join("tree.svg"), TINY_SVG).unwrap();
        std::fs::write(en_other_dir.join("mixed.gif"), TINY_GIF).unwrap();
        std::fs::write(en_other_dir.join("bézier.gif"), TINY_GIF).unwrap();
        std::fs::write(fr_other_dir.join("own.gif"), TINY_GIF).unwrap();

        let case = |name, locale, src: &str, expected_src: &str, expected_size| Case {
            name,
            locale,
            src: src.replace("{s}", &slug),
            expected_src: expected_src.replace("{s}", &slug).replace("{f}", &folder),
            expected_size,
        };
        let cases = vec![
            case(
                "relative src",
                Locale::EnUs,
                "local.gif",
                "/en-US/docs/Web/{s}/Api/Child/local.gif",
                Some(("1", "1")),
            ),
            case(
                "absolute src to asset outside the page folder",
                Locale::EnUs,
                "/en-US/docs/Web/{s}/Other/tree.svg",
                "/en-us/docs/web/{f}/other/tree.svg",
                Some(("20", "10")),
            ),
            case(
                "absolute src with mixed-case URL",
                Locale::EnUs,
                "/en-US/docs/Web/{s}/Other/Mixed.GIF",
                "/en-us/docs/web/{f}/other/mixed.gif",
                Some(("1", "1")),
            ),
            case(
                "percent-encoded absolute src",
                Locale::EnUs,
                "/en-US/docs/Web/{s}/Other/b%C3%A9zier.gif",
                "/en-us/docs/web/{f}/other/b%c3%a9zier.gif",
                Some(("1", "1")),
            ),
            case(
                "absolute src with query string",
                Locale::EnUs,
                "/en-US/docs/Web/{s}/Other/tree.svg?v=2",
                "/en-us/docs/web/{f}/other/tree.svg",
                Some(("20", "10")),
            ),
            case(
                "absolute src with backslashes and leading space",
                Locale::EnUs,
                " \\en-US\\docs\\Web\\{s}\\Other\\tree.svg",
                "/en-us/docs/web/{f}/other/tree.svg",
                Some(("20", "10")),
            ),
            case(
                "absolute src with encoded parent segments",
                Locale::EnUs,
                "/en-US/docs/Web/{s}/..%2f..%2f..%2f..%2f..%2f..%2fetc/x.png",
                "/en-us/docs/web/{f}/..%2f..%2f..%2f..%2f..%2f..%2fetc/x.png",
                None,
            ),
            case(
                "absolute src with dot segments",
                Locale::EnUs,
                "/en-US/docs/Web/{s}/Api/../Other/tree.svg",
                "/en-us/docs/web/{f}/other/tree.svg",
                Some(("20", "10")),
            ),
            case(
                "translated page, absolute src found in translation",
                Locale::Fr,
                "/fr/docs/Web/{s}/Other/own.gif",
                "/fr/docs/web/{f}/other/own.gif",
                Some(("1", "1")),
            ),
            case(
                "translated page, explicit en-US absolute src",
                Locale::Fr,
                "/en-US/docs/Web/{s}/Other/tree.svg",
                "/en-us/docs/web/{f}/other/tree.svg",
                Some(("20", "10")),
            ),
            case(
                "translated page, absolute src falls back to en-US",
                Locale::Fr,
                "/fr/docs/Web/{s}/Other/tree.svg",
                "/en-US/docs/web/{f}/other/tree.svg",
                Some(("20", "10")),
            ),
            case(
                "translated page, relative src falls back to en-US",
                Locale::Fr,
                "local.gif",
                "/en-US/docs/Web/{s}/Api/Child/local.gif",
                Some(("1", "1")),
            ),
            case(
                "translated page, absolute src missing in both locales",
                Locale::Fr,
                "/fr/docs/Web/{s}/Other/missing.gif",
                "/fr/docs/web/{f}/other/missing.gif",
                None,
            ),
            case(
                "missing file leaves dimensions unset",
                Locale::EnUs,
                "/en-US/docs/Web/{s}/Other/missing.gif",
                "/en-us/docs/web/{f}/other/missing.gif",
                None,
            ),
            case(
                "absolute non-doc path leaves dimensions unset",
                Locale::EnUs,
                "/en-US/blog/missing.gif",
                "/en-us/blog/missing.gif",
                None,
            ),
        ];

        for case in cases {
            let (dir, url) = match case.locale {
                Locale::EnUs => (&en_page_dir, format!("/en-US/docs/Web/{slug}/Api/Child")),
                _ => (&fr_page_dir, format!("/fr/docs/Web/{slug}/Api/Child")),
            };
            let page = TestPage {
                path: dir.join("index.md"),
                url,
                locale: case.locale,
            };
            let output = rewrite_img(&format!(r#"<img src="{}">"#, case.src), &page);
            let src_attr = format!("src=\"{}\"", case.expected_src);
            assert!(
                output.contains(&src_attr),
                "{}: expected {src_attr}; got: {output}",
                case.name
            );
            match case.expected_size {
                Some((w, h)) => assert!(
                    output.contains(&format!("width=\"{w}\""))
                        && output.contains(&format!("height=\"{h}\""))
                        && !output.contains("data-flaw="),
                    "{}: expected {w}x{h} without flaw; got: {output}",
                    case.name
                ),
                None => assert!(
                    !output.contains("width=")
                        && !output.contains("height=")
                        && output.contains("data-flaw="),
                    "{}: expected no dimensions and a flaw; got: {output}",
                    case.name
                ),
            }
        }
    }

    #[test]
    fn test_absolute_src_path_stays_in_root() {
        struct Case {
            name: &'static str,
            src: &'static str,
            expected: Option<&'static str>,
        }

        let cases = vec![
            Case {
                name: "plain doc asset",
                src: "/en-us/docs/web/api/x.png",
                expected: Some("en-us/web/api/x.png"),
            },
            Case {
                name: "decoded parent segments",
                src: "/en-us/docs/web/../../../etc/x.png",
                expected: None,
            },
            Case {
                name: "empty segment",
                src: "/en-us/docs//etc/x.png",
                expected: None,
            },
            Case {
                name: "missing locale",
                src: "/docs/web/x.png",
                expected: None,
            },
            Case {
                name: "filename is not mapped",
                src: "/en-us/docs/web/a:b.png",
                expected: Some("en-us/web/a:b.png"),
            },
        ];

        for case in cases {
            let actual = absolute_src_path(case.src, None);
            let expected = case.expected.map(|p| content_root().join(p));
            assert_eq!(actual, expected, "{}", case.name);
        }

        assert_eq!(
            absolute_src_path("/docs/web/x.png", Some(Locale::EnUs)),
            None,
            "locale override does not apply to non-doc URLs"
        );
    }
}
