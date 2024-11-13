use std::borrow::Cow;
use std::collections::HashSet;

use lol_html::html_content::ContentType;
use lol_html::{element, rewrite_str, HtmlRewriter, RewriteStrSettings, Settings};
use rari_md::ext::DELIM_START;
use rari_md::node_card::NoteCard;
use rari_types::fm_types::PageType;
use rari_types::globals::settings;
use rari_utils::concat_strs;
use tracing::warn;
use url::Url;

use crate::error::DocError;
use crate::helpers::l10n::l10n_json_data;
use crate::issues::get_issue_couter;
use crate::pages::page::{Page, PageLike};
use crate::pages::types::curriculum::CurriculumPage;
use crate::redirects::resolve_redirect;
use crate::resolve::strip_locale_from_url;

pub fn post_process_inline_sidebar(input: &str) -> Result<String, DocError> {
    let element_content_handlers = vec![element!("*[data-rewriter=em]", |el| {
        el.prepend("<em>", ContentType::Html);
        el.append("</em>", ContentType::Html);
        el.remove_attribute("data-rewriter");
        Ok(())
    })];
    Ok(rewrite_str(
        input,
        RewriteStrSettings {
            element_content_handlers,
            ..Default::default()
        },
    )?)
}

pub fn post_process_html<T: PageLike>(
    input: &str,
    page: &T,
    sidebar: bool,
) -> Result<String, DocError> {
    let mut output = vec![];
    let mut ids = HashSet::new();
    let options = Url::options();
    let url = page.url();
    let base = Url::parse(&concat_strs!(
        "http://rari.placeholder",
        url,
        if url.ends_with('/') { "" } else { "/" }
    ))?;
    let base_url = options.base_url(Some(&base));
    let data_issues = settings().data_issues;

    let mut element_content_handlers = vec![
        element!("*[id]", |el| {
            if let Some(id) = el.get_attribute("id") {
                if id.contains(DELIM_START) {
                    el.set_attribute("data-update-id", "")?;
                } else {
                    let id = id.to_lowercase();
                    if ids.contains(id.as_str()) {
                        let (prefix, mut count) =
                            if let Some((prefix, counter)) = id.rsplit_once('_') {
                                if counter.chars().all(|c| c.is_ascii_digit()) {
                                    let count = counter.parse::<i64>().unwrap_or_default() + 1;
                                    (prefix, count)
                                } else {
                                    (id.as_str(), 2)
                                }
                            } else {
                                (id.as_str(), 2)
                            };
                        let mut id = format!("{prefix}_{count}");
                        while ids.contains(&id) && count < 666 {
                            count += 1;
                            id = format!("{prefix}_{count}");
                        }

                        if !ids.contains(&id) && count < 666 {
                            el.set_attribute("id", &id)?;
                            ids.insert(id);
                        }
                    } else {
                        el.set_attribute("id", &id)?;
                        ids.insert(id);
                    }
                }
            }
            Ok(())
        }),
        element!("img[src]", |el| {
            // Leave dimensions alone if we have a `width` attribute
            if el.get_attribute("width").is_some() {
                return Ok(());
            }
            if let Some(src) = el.get_attribute("src") {
                let url = base_url.parse(&src)?;
                if url.host() == base.host() && !url.path().starts_with("/assets/") {
                    el.set_attribute("src", url.path())?;
                    let file = page.full_path().parent().unwrap().join(&src);
                    let (width, height) = if src.ends_with(".svg") {
                        match svg_metadata::Metadata::parse_file(&file) {
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
                                    (Some(w), None, Some(vb)) => {
                                        (Some(w), Some(w * vb.height / vb.width))
                                    }
                                    // Only height and viewbox are given
                                    (None, Some(h), Some(vb)) => {
                                        (Some(h * vb.width / vb.height), Some(h))
                                    }
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
                                    final_width.map(|w| format!("{:.0}", w)),
                                    final_height.map(|h| format!("{:.0}", h)),
                                )
                            }
                            Err(e) => {
                                let ic = get_issue_couter();
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
                        match imagesize::size(&file) {
                            Ok(dim) => (Some(dim.width.to_string()), Some(dim.height.to_string())),
                            Err(e) => {
                                let ic = get_issue_couter();
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
                    if let Some(width) = width {
                        el.set_attribute("width", &width)?;
                    }
                    if let Some(height) = height {
                        el.set_attribute("height", &height)?;
                    }
                }
            }
            Ok(())
        }),
        element!("img:not([loading])", |el| {
            el.set_attribute("loading", "lazy")?;
            Ok(())
        }),
        element!("iframe:not([loading])", |el| {
            el.set_attribute("loading", "lazy")?;
            Ok(())
        }),
        element!("a[href]", |el| {
            let original_href = el.get_attribute("href").expect("href was required");
            if original_href.starts_with('/')
                || original_href.starts_with("https://developer.mozilla.org")
            {
                let href = original_href
                    .strip_prefix("https://developer.mozilla.org")
                    .map(|href| if href.is_empty() { "/" } else { href })
                    .unwrap_or(&original_href);
                let href_no_hash = &href[..href.find('#').unwrap_or(href.len())];
                let no_locale = strip_locale_from_url(href).0.is_none();
                if no_locale && Page::ignore_link_check(href_no_hash) {
                    return Ok(());
                }
                let maybe_prefixed_href = if no_locale {
                    Cow::Owned(concat_strs!("/", page.locale().as_url_str(), href))
                } else {
                    Cow::Borrowed(href)
                };
                let resolved_href = resolve_redirect(&maybe_prefixed_href)
                    .unwrap_or(Cow::Borrowed(&maybe_prefixed_href));
                let resolved_href_no_hash =
                    &resolved_href[..resolved_href.find('#').unwrap_or(resolved_href.len())];
                if resolved_href_no_hash == page.url() {
                    el.set_attribute("aria-current", "page")?;
                }
                let remove_href = if !Page::exists(resolved_href_no_hash)
                    && !Page::ignore_link_check(href)
                {
                    tracing::debug!("{resolved_href_no_hash} {href}");
                    let class = el.get_attribute("class").unwrap_or_default();
                    el.set_attribute(
                        "class",
                        &concat_strs!(
                            &class,
                            if class.is_empty() { "" } else { " " },
                            "page-not-created"
                        ),
                    )?;
                    el.remove_attribute("href");
                    el.set_attribute("title", l10n_json_data("Common", "summary", page.locale())?)?;
                    true
                } else {
                    false
                };
                let resolved_href = if no_locale {
                    strip_locale_from_url(&resolved_href).1
                } else {
                    resolved_href.as_ref()
                };
                if original_href != resolved_href {
                    if let Some(pos) = el.get_attribute("data-sourcepos") {
                        if let Some((start, _)) = pos.split_once('-') {
                            if let Some((line, col)) = start.split_once(':') {
                                let line = line
                                    .parse::<i64>()
                                    .map(|l| l + i64::try_from(page.fm_offset()).unwrap_or(l - 1))
                                    .ok()
                                    .unwrap_or(-1);
                                let col = col.parse::<i64>().ok().unwrap_or(0);
                                let ic = get_issue_couter();
                                tracing::warn!(
                                    source = "redirected-link",
                                    ic = ic,
                                    line = line,
                                    col = col,
                                    url = original_href,
                                    redirect = resolved_href
                                );
                                if data_issues {
                                    el.set_attribute("data-flaw", &ic.to_string())?;
                                }
                            }
                        }
                    } else {
                        let ic = get_issue_couter();
                        tracing::warn!(
                            source = "redirected-link",
                            ic = ic,
                            url = original_href,
                            redirect = resolved_href
                        );
                        if data_issues {
                            el.set_attribute("data-flaw", &ic.to_string())?;
                        }
                    }

                    if !remove_href {
                        el.set_attribute("href", resolved_href)?;
                    }
                }
                if remove_href {
                    el.remove_attribute("href");
                }
            } else if original_href.starts_with("http:") || original_href.starts_with("https:") {
                let class = el.get_attribute("class").unwrap_or_default();
                if !class.split(' ').any(|s| s == "external") {
                    el.set_attribute(
                        "class",
                        &concat_strs!(&class, if class.is_empty() { "" } else { " " }, "external"),
                    )?;
                }
                if !el.has_attribute("target") {
                    el.set_attribute("target", "_blank")?;
                }
            }

            Ok(())
        }),
        element!("pre:not(.notranslate)", |el| {
            let mut class = el.get_attribute("class").unwrap_or_default();
            class.push_str(" notranslate");
            el.set_attribute("class", &class)?;
            Ok(())
        }),
        element!("pre[class*=brush]", |el| {
            let class = el.get_attribute("class");
            let class = class.as_deref().unwrap_or_default();
            let is_hidden = class.split_ascii_whitespace().any(|c| c == "hidden");
            let name = class
                .split_ascii_whitespace()
                .skip_while(|s| *s != "brush:")
                .nth(1)
                .unwrap_or_default();

            if !name.is_empty() && name != "plain" {
                el.prepend("<code>", ContentType::Html);
                el.append("</code>", ContentType::Html);
            }
            if is_hidden {
                el.before(r#"<div class="code-example">"#, ContentType::Html);
                el.after("</div>", ContentType::Html);
            } else if !name.is_empty() && name != "plain" {
                el.before(&concat_strs!(
                  r#"<div class="code-example"><div class='example-header'><span class="language-name">"#,
                  name,
                  "</span></div>"),
                  ContentType::Html
                );
                el.after("</div>", ContentType::Html);
            }
            Ok(())
        }),
        element!(
            "div.notecard.warning[data-add-warning] > p:first-child",
            |el| {
                el.prepend(
                    &concat_strs!(
                        "<strong>",
                        NoteCard::Warning.prefix_for_locale(page.locale()),
                        "</strong>"
                    ),
                    ContentType::Html,
                );
                Ok(())
            }
        ),
        element!("div.notecard.note[data-add-note] > p:first-child", |el| {
            el.prepend(
                &concat_strs!(
                    "<strong>",
                    NoteCard::Note.prefix_for_locale(page.locale()),
                    "</strong>"
                ),
                ContentType::Html,
            );
            Ok(())
        }),
        element!("table", |el| {
            el.before("<figure class=\"table-container\">", ContentType::Html);
            el.after("</figure>", ContentType::Html);
            Ok(())
        }),
        element!("math[display=block]", |el| {
            el.before("<figure class=\"table-container\">", ContentType::Html);
            el.after("</figure>", ContentType::Html);
            Ok(())
        }),
        element!("*[data-rewriter=em]", |el| {
            el.prepend("<em>", ContentType::Html);
            el.append("</em>", ContentType::Html);
            el.remove_attribute("data-rewriter");
            Ok(())
        }),
        element!("*[data-sourcepos]", |el| {
            el.remove_attribute("data-sourcepos");
            Ok(())
        }),
        element!("*[data-add-note]", |el| {
            el.remove_attribute("data-add-note");
            Ok(())
        }),
        element!("*[data-add-warning]", |el| {
            el.remove_attribute("data-add-warning");
            Ok(())
        }),
    ];
    if sidebar {
        element_content_handlers.push(element!("html", |el| {
            el.remove_and_keep_content();
            Ok(())
        }));
    }
    if page.page_type() == PageType::Curriculum {
        element_content_handlers = {
            let mut curriculum_links = vec![element!("a[href^=\".\"]", |el| {
                let href = el.get_attribute("href").unwrap_or_default();
                let split_href = href.split_once('#');
                if let Ok(page) = CurriculumPage::page_from_realitve_file(
                    page.full_path(),
                    split_href.map(|s| s.0).unwrap_or(&href),
                ) {
                    el.set_attribute(
                        "href",
                        &split_href
                            .map(|s| Cow::Owned(concat_strs!(page.url(), "#", s.1)))
                            .unwrap_or(Cow::Borrowed(page.url())),
                    )?;
                }
                Ok(())
            })];

            curriculum_links.append(&mut element_content_handlers);
            curriculum_links
        }
    }

    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers,
            ..Settings::default()
        },
        |c: &[u8]| output.extend_from_slice(c),
    );

    rewriter.write(input.as_bytes())?;
    rewriter.end()?;

    Ok(String::from_utf8(output)?)
}
