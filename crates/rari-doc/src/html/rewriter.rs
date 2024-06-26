use std::borrow::Cow;
use std::collections::HashSet;

use lol_html::html_content::ContentType;
use lol_html::{element, text, HtmlRewriter, Settings};
use rari_md::bq::NoteCard;
use rari_types::fm_types::PageType;
use rari_types::locale::Locale;

use crate::docs::curriculum::relative_file_to_curriculum_page;
use crate::docs::page::{Page, PageLike};
use crate::error::DocError;
use crate::redirects::resolve_redirect;
use crate::resolve::strip_locale_from_url;

pub fn post_process_html<T: PageLike>(
    input: &str,
    page: &T,
    sidebar: bool,
) -> Result<String, DocError> {
    let mut output = vec![];
    let mut ids = HashSet::new();

    let mut element_content_handlers = vec![
        element!("*[id]", |el| {
            if let Some(id) = el.get_attribute("id") {
                if !ids.contains(id.as_str()) {
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
                    while !ids.insert(id) && count < 666 {
                        count += 1;
                        id = format!("{prefix}_{count}");
                    }
                } else {
                    ids.insert(id);
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
        element!("li > p", |el| {
            el.remove_and_keep_content();
            Ok(())
        }),
        element!("a[href]", |el| {
            let href = el.get_attribute("href").expect("href was required");
            if href.starts_with('/') || href.starts_with("https://developer.mozilla.org") {
                let href = href
                    .strip_prefix("https://developer.mozilla.org")
                    .map(|href| if href.is_empty() { "/" } else { href })
                    .unwrap_or(&href);
                let no_locale = strip_locale_from_url(href).0.is_none();
                let maybe_prefixed_href = if no_locale {
                    Cow::Owned(format!("/{}{href}", Locale::default().as_url_str()))
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
                if !Page::exists(resolved_href_no_hash) && !Page::ignore(href) {
                    tracing::info!("{resolved_href_no_hash} {href}");
                    let class = el.get_attribute("class").unwrap_or_default();
                    el.set_attribute(
                        "class",
                        &format!(
                            "{class}{}page-not-created",
                            if class.is_empty() { "" } else { " " }
                        ),
                    )?;
                    el.set_attribute("title", "This is a link to an unwritten page")?;
                }
                el.set_attribute(
                    "href",
                    if no_locale {
                        strip_locale_from_url(&resolved_href).1
                    } else {
                        &resolved_href
                    },
                )?;
            } else if href.starts_with("http:") || href.starts_with("https:") {
                let class = el.get_attribute("class").unwrap_or_default();
                if !class.split(' ').any(|s| s == "external") {
                    el.set_attribute(
                        "class",
                        &format!("{class}{}external", if class.is_empty() { "" } else { " " }),
                    )?;
                }
                if !el.has_attribute("target") {
                    el.set_attribute("target", "_blank")?;
                }
            }

            Ok(())
        }),
        element!("dt[data-add-link]", |el| {
            el.remove_attribute("data-add-link");
            if let Some(id) = el.get_attribute("id") {
                el.prepend(&format!("<a href=\"#{id}\">"), ContentType::Html);
            }
            Ok(())
        }),
        element!("dt[data-add-link] *:first-child", |el| {
            el.append("</a>", ContentType::Html);
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
              &format!(
                r#"<div class="code-example"><div class='example-header'><span class="language-name">{name}</span></div>"#,
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
        element!("div.notecard.callout > p:first-child", |el| {
            el.prepend(
                &format!(
                    "<strong>{}</strong>",
                    NoteCard::Callout.prefix_for_locale(page.locale())
                ),
                ContentType::Html,
            );
            Ok(())
        }),
        element!("div.notecard.warning > p:first-child", |el| {
            el.prepend(
                &format!(
                    "<strong>{}</strong>",
                    NoteCard::Warning.prefix_for_locale(page.locale())
                ),
                ContentType::Html,
            );
            Ok(())
        }),
        element!("div.notecard.note > p:first-child", |el| {
            el.prepend(
                &format!(
                    "<strong>{}</strong>",
                    NoteCard::Note.prefix_for_locale(page.locale())
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
    ];
    if sidebar {
        element_content_handlers.push(element!("*[data-rewriter=em]", |el| {
            el.prepend("<em>", ContentType::Html);
            el.append("</em>", ContentType::Html);
            el.remove_attribute("data-rewriter");
            Ok(())
        }));
        element_content_handlers.push(element!("html", |el| {
            el.remove_and_keep_content();
            Ok(())
        }));
    }
    if page.page_type() == PageType::Curriculum {
        element_content_handlers = {
            let mut curriculum_links = vec![
                element!("a[href^=\".\"]", |el| {
                    let href = el.get_attribute("href").unwrap_or_default();
                    let split_href = href.split_once('#');
                    if let Ok(page) = relative_file_to_curriculum_page(
                        page.full_path(),
                        split_href.map(|s| s.0).unwrap_or(&href),
                    ) {
                        el.set_attribute(
                            "href",
                            &split_href
                                .map(|s| Cow::Owned(format!("{}#{}", page.url(), s.1)))
                                .unwrap_or(Cow::Borrowed(page.url())),
                        )?;
                    }
                    Ok(())
                }),
                text!("p", |t| {
                    if t.as_str() == "Learning outcomes:" {
                        t.before(
                            "<span class=\"curriculum-outcomes\"></span>",
                            ContentType::Html,
                        )
                    }
                    if t.as_str() == "Resources:" || t.as_str() == "General resources:" {
                        t.before(
                            "<span class=\"curriculum-resources\"></span>",
                            ContentType::Html,
                        )
                    }
                    Ok(())
                }),
            ];

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
