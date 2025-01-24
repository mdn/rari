use std::borrow::Cow;
use std::collections::HashSet;

use lol_html::html_content::ContentType;
use lol_html::{element, rewrite_str, text, HtmlRewriter, RewriteStrSettings, Settings};
use rari_md::ext::DELIM_START;
use rari_md::node_card::NoteCard;
use rari_types::fm_types::PageType;
use rari_types::globals::settings;
use rari_utils::concat_strs;
use url::Url;

use crate::error::DocError;
use crate::html::fix_img::handle_img;
use crate::html::fix_link::check_and_fix_link;
use crate::pages::page::PageLike;
use crate::pages::types::curriculum::CurriculumPage;

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
    let mut in_pre = false;

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
            handle_img(el, page, data_issues, &base, &base_url)
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
            check_and_fix_link(el, page, data_issues)?;
            Ok(())
        }),
        element!("pre:not(.notranslate)", |el| {
            let mut class = el.get_attribute("class").unwrap_or_default();
            class.push_str(" notranslate");
            el.set_attribute("class", &class)?;
            Ok(())
        }),
        text!("pre[class*=brush]", |text| {
            // trim the first _empty_ line,
            // fixes issue: https://github.com/mdn/yari/issues/12364
            if !in_pre && text.as_str().starts_with('\n') {
                text.as_mut_str().remove(0);
            }
            in_pre = true;
            if text.last_in_text_node() {
                in_pre = false;
            }
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
