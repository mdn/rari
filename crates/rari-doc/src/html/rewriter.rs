use std::borrow::Cow;
use std::collections::HashSet;

use lol_html::html_content::{ContentType, Element};
use lol_html::{element, rewrite_str, HtmlRewriter, RewriteStrSettings, Settings};
use rari_md::ext::DELIM_START;
use rari_md::node_card::NoteCard;
use rari_types::fm_types::PageType;
use rari_types::locale::Locale;
use rari_utils::concat_strs;
use tracing::warn;
use url::Url;

use crate::error::DocError;
use crate::helpers::l10n::l10n_json_data;
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
    let open_dt_a = std::rc::Rc::new(std::cell::RefCell::new(false));
    let options = Url::options();
    let url = page.url();
    let base = Url::parse(&concat_strs!(
        "http://rari.placeholder",
        url,
        if url.ends_with('/') { "" } else { "/" }
    ))?;
    let base_url = options.base_url(Some(&base));

    let mut element_content_handlers = vec![
        element!("*[id]", |el| {
            if let Some(id) = el.get_attribute("id") {
                if id.contains(DELIM_START) {
                    el.set_attribute("data-update-id", "")?;
                } else if ids.contains(id.as_str()) {
                    let (prefix, mut count) = if let Some((prefix, counter)) = id.rsplit_once('_') {
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
                    ids.insert(id);
                }
            }
            Ok(())
        }),
        element!("img[src]", |el| {
            if let Some(src) = el.get_attribute("src") {
                let url = base_url.parse(&src)?;
                if url.host() == base.host() && !url.path().starts_with("/assets/") {
                    el.set_attribute("src", url.path())?;
                    let file = page.full_path().parent().unwrap().join(&src);
                    let (width, height) = if src.ends_with(".svg") {
                        match svg_metadata::Metadata::parse_file(&file) {
                            Ok(meta) => (
                                meta.width
                                    .map(|width| width.width)
                                    .or(meta.view_box.map(|vb| vb.width))
                                    .map(|width| format!("{:.0}", width)),
                                meta.height
                                    .map(|height| height.height)
                                    .or(meta.view_box.map(|vb| vb.height))
                                    .map(|height| format!("{:.0}", height)),
                            ),
                            Err(e) => {
                                warn!(
                                    source = "image-check",
                                    "Error parsing {}: {e}",
                                    file.display()
                                );
                                (None, None)
                            }
                        }
                    } else if let Ok(dim) = imagesize::size(&file).inspect_err(|e| {
                        warn!(
                            source = "image-check",
                            "Error opening {}: {e}",
                            file.display()
                        )
                    }) {
                        (Some(dim.width.to_string()), Some(dim.height.to_string()))
                    } else {
                        (None, None)
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
                let no_locale = strip_locale_from_url(href).0.is_none();
                let maybe_prefixed_href = if no_locale {
                    Cow::Owned(concat_strs!("/", Locale::default().as_url_str(), href))
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
                if !Page::exists(resolved_href_no_hash) && !Page::ignore_link_check(href) {
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
                    el.set_attribute("title", l10n_json_data("Common", "summary", page.locale())?)?;
                }
                if original_href != resolved_href {
                    if let Some(pos) = el.get_attribute("data-sourcepos") {
                        if let Some((start, _)) = pos.split_once('-') {
                            if let Some((line, col)) = start.split_once(':') {
                                tracing::warn!(
                                    source = "redirected-link",
                                    line = line,
                                    col = col,
                                    url = original_href,
                                    redirect = resolved_href.as_ref()
                                )
                            }
                        }
                    } else {
                        tracing::warn!(
                            source = "redirected-link",
                            url = original_href,
                            redirect = resolved_href.as_ref()
                        )
                    }
                }
                el.set_attribute(
                    "href",
                    if no_locale {
                        strip_locale_from_url(&resolved_href).1
                    } else {
                        &resolved_href
                    },
                )?;
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
        element!("dt[data-add-link]", |el: &mut Element| {
            el.remove_attribute("data-add-link");
            if let Some(id) = el.get_attribute("id") {
                el.prepend(&concat_strs!("<a href=\"#", &id, "\">"), ContentType::Html);
                let mut s = open_dt_a.borrow_mut();
                *s = true;
                let open_dt_a = open_dt_a.clone();
                // We need this handler if there's only a text node in the dl.
                if let Some(handlers) = el.end_tag_handlers() {
                    handlers.push(Box::new(move |end| {
                        let mut s = open_dt_a.borrow_mut();
                        if *s {
                            end.before("</a>", ContentType::Html);
                            *s = false;
                        }
                        Ok(())
                    }));
                }
            }
            Ok(())
        }),
        element!("dt[data-add-link] *:first-child", |el| {
            let mut s = open_dt_a.borrow_mut();
            if *s {
                el.after("</a>", ContentType::Html);
                *s = false;
            }
            Ok(())
        }),
        element!("pre:not(.notranslate)", |el| {
            let mut class = el.get_attribute("class").unwrap_or_default();
            class.push_str(" notranslate");
            el.set_attribute("class", &class)?;
            Ok(())
        }),
        element!("pre[class*=brush]:not(.hidden)", |el| {
            let class = el.get_attribute("class");
            let class = class.as_deref().unwrap_or_default();
            let name = class
                .split_ascii_whitespace()
                .skip_while(|s| *s != "brush:")
                .nth(1)
                .unwrap_or_default();
            if !name.is_empty() && name != "plain" {
                el.before(
              &concat_strs!(
                r#"<div class="code-example"><div class='example-header'><span class="language-name">"#, name, "</span></div>"
              ),
              ContentType::Html
            );
                el.after("</div>", ContentType::Html);
            }
            Ok(())
        }),
        element!("pre[class*=brush].hidden", |el| {
            el.before(r#"<div class="code-example">"#, ContentType::Html);
            el.after("</div>", ContentType::Html);
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
