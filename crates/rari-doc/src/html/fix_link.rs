use std::borrow::Cow;

use lol_html::HandlerResult;
use lol_html::html_content::Element;
use rari_types::fm_types::PageType;
use rari_types::locale::{Locale, default_locale};
use rari_utils::concat_strs;

use crate::helpers::l10n::l10n_json_data;
use crate::issues::get_issue_counter;
use crate::pages::page::{Page, PageLike};
use crate::redirects::resolve_redirect;
use crate::resolve::{strip_locale_from_url, url_with_locale};

pub fn check_and_fix_link(
    el: &mut Element,
    page: &impl PageLike,
    data_issues: bool,
) -> HandlerResult {
    let templ_link = el.has_attribute("data-templ-link");
    if templ_link {
        el.remove_attribute("data-templ-link");
    }
    let original_href = el.get_attribute("href").expect("href was required");
    let href = original_href.strip_prefix('!').unwrap_or(&original_href);
    if href != original_href {
        el.set_attribute("href", href)?;
    }

    let auto_link = el.has_attribute("data-autolink");
    if auto_link {
        el.remove_attribute("data-autolink");
    }
    if href.starts_with('/') || href.starts_with("https://developer.mozilla.org") {
        handle_internal_link(&original_href, el, page, data_issues, templ_link, auto_link)
    } else {
        if href != original_href && !templ_link {
            report_unnecessary_marker(el, page, &original_href, href, data_issues)?;
        }
        if href.starts_with("http:") || href.starts_with("https:") {
            handle_external_link(el, page.locale())
        } else {
            Ok(())
        }
    }
}

pub fn handle_external_link(el: &mut Element, locale: Locale) -> HandlerResult {
    let class = el.get_attribute("class").unwrap_or_default();
    if !class.split(' ').any(|s| s == "external") {
        el.set_attribute(
            "class",
            &concat_strs!(&class, if class.is_empty() { "" } else { " " }, "external"),
        )?;
    }
    el.remove_attribute("data-autolink");
    if !el.has_attribute("target") {
        el.set_attribute("target", "_blank")?;
    }
    if !el.has_attribute("title")
        && let Ok(title) = l10n_json_data("Template", "external_link", locale)
    {
        el.set_attribute("title", title)?;
    }
    Ok(())
}

pub fn handle_internal_link(
    original_href: &str,
    el: &mut Element,
    page: &impl PageLike,
    data_issues: bool,
    templ_link: bool,
    auto_link: bool,
) -> HandlerResult {
    let marked_href = original_href;
    let allow_broken = original_href.starts_with('!');
    let original_href = original_href.strip_prefix('!').unwrap_or(original_href);
    // Strip prefix for curriculum links.
    let original_href = if page.page_type() == PageType::Curriculum || auto_link {
        original_href
            .strip_prefix("https://developer.mozilla.org")
            .map(|stripped| if stripped.is_empty() { "/" } else { stripped })
            .unwrap_or(original_href)
    } else {
        original_href
    };

    let href = original_href
        .strip_prefix("https://developer.mozilla.org")
        .map(|href| if href.is_empty() { "/" } else { href })
        .unwrap_or(original_href);
    let href_no_hash = &href[..href.find('#').unwrap_or(href.len())];
    let (href_locale, _) = strip_locale_from_url(href);
    let no_locale = href_locale.is_none();
    if no_locale && Page::ignore_link_check(href_no_hash) {
        if allow_broken && !templ_link {
            report_unnecessary_marker(el, page, marked_href, original_href, data_issues)?;
        }
        return Ok(());
    }
    let maybe_prefixed_href = if no_locale {
        Cow::Owned(concat_strs!("/", page.locale().as_url_str(), href))
    } else {
        Cow::Borrowed(href)
    };
    let mut resolved_href =
        resolve_redirect(&maybe_prefixed_href).unwrap_or(Cow::Borrowed(&maybe_prefixed_href));
    let mut resolved_href_no_hash =
        &resolved_href[..resolved_href.find('#').unwrap_or(resolved_href.len())];
    if resolved_href_no_hash == page.url() {
        el.set_attribute("aria-current", "page")?;
    }
    let en_us_fallback = if !Page::exists(resolved_href_no_hash)
        && !Page::ignore_link_check(href)
        && href_locale != Some(default_locale())
    {
        if let Some(en_us_href) = url_with_locale(&resolved_href, default_locale()) {
            resolved_href = resolve_redirect(&en_us_href).unwrap_or(Cow::Owned(en_us_href));
            resolved_href_no_hash =
                &resolved_href[..resolved_href.find('#').unwrap_or(resolved_href.len())];
        }
        true
    } else {
        false
    };

    let remove_href =
        if !Page::exists_with_fallback(resolved_href_no_hash) && !Page::ignore_link_check(href) {
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
            if let Some(href) = el.get_attribute("href") {
                el.set_attribute("data-href", &href)?;
            }
            el.remove_attribute("href");
            el.set_attribute("title", l10n_json_data("Common", "summary", page.locale())?)?;
            true
        } else {
            false
        };

    if !remove_href && en_us_fallback {
        let class = el.get_attribute("class").unwrap_or_default();
        if !class.split(' ').any(|s| s == "only-in-en-us") {
            el.set_attribute(
                "class",
                &concat_strs!(
                    &class,
                    if class.is_empty() { "" } else { " " },
                    "only-in-en-us"
                ),
            )?;
        }
    }

    if !templ_link {
        let resolved_href = if no_locale {
            strip_locale_from_url(&resolved_href).1
        } else {
            resolved_href.as_ref()
        };
        if allow_broken && !remove_href {
            report_unnecessary_marker(el, page, marked_href, resolved_href, data_issues)?;
        }
        if !allow_broken && (original_href != resolved_href || remove_href) && !en_us_fallback {
            if let Some(pos) = el.get_attribute("data-sourcepos") {
                if let Some((start, end)) = pos.split_once('-')
                    && let Some((line, col)) = start.split_once(':')
                {
                    let line = line
                        .parse::<i64>()
                        .map(|l| l + i64::try_from(page.fm_offset()).unwrap_or(l - 1))
                        .ok()
                        .unwrap_or(-1);
                    let col = col.parse::<i64>().ok().unwrap_or(0);
                    let (end_line, end_col) = end
                        .split_once(':')
                        .map(|(end_line, end_col)| {
                            let end_line = end_line
                                .parse::<i64>()
                                .map(|l| l + i64::try_from(page.fm_offset()).unwrap_or(l - 1))
                                .ok()
                                .unwrap_or(-1);
                            let end_col = end_col.parse::<i64>().ok().unwrap_or(0);
                            (end_line, end_col)
                        })
                        .unwrap_or((-1, -1));
                    let ic = get_issue_counter();
                    if remove_href {
                        tracing::warn!(
                            source = "broken-link",
                            ic = ic,
                            line = line,
                            col = col,
                            end_line = end_line,
                            end_col = end_col,
                            url = original_href,
                        );
                    } else {
                        let source = if original_href.to_lowercase() == resolved_href.to_lowercase()
                        {
                            "ill-cased-link"
                        } else {
                            "redirected-link"
                        };
                        tracing::warn!(
                            source = source,
                            ic = ic,
                            line = line,
                            col = col,
                            end_line = end_line,
                            end_col = end_col,
                            url = original_href,
                            redirect = resolved_href
                        );
                    }
                    if data_issues {
                        el.set_attribute("data-flaw", &ic.to_string())?;
                    }
                }
            } else {
                let ic = get_issue_counter();
                if remove_href {
                    tracing::warn!(source = "broken-link", ic = ic, url = original_href);
                } else {
                    let source = if original_href.to_lowercase() == resolved_href.to_lowercase() {
                        "ill-cased-link"
                    } else {
                        "redirected-link"
                    };
                    tracing::warn!(
                        source = source,
                        ic = ic,
                        url = original_href,
                        redirect = resolved_href
                    );
                }
                if data_issues {
                    el.set_attribute("data-flaw", &ic.to_string())?;
                }
            }
        }
        if remove_href {
            el.remove_attribute("href");
        } else {
            el.set_attribute("href", resolved_href)?;
        }
    }
    Ok(())
}

fn report_unnecessary_marker(
    el: &mut Element,
    page: &impl PageLike,
    original_href: &str,
    href: &str,
    data_issues: bool,
) -> HandlerResult {
    let pos = el.get_attribute("data-sourcepos");
    let mut positions = pos.as_deref().unwrap_or("").split([':', '-']);
    let line = positions
        .next()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);
    let col = positions
        .next()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);
    let end_line = positions
        .next()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);
    let end_col = positions
        .next()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);
    let (line, col, end_line, end_col) = if let Some(destination) =
        rari_md::link_destination_offset(page.content(), line as usize, col as usize, original_href)
    {
        let prefix = &page.content()[..destination];
        let line = prefix.bytes().filter(|byte| *byte == b'\n').count() as i64 + 1;
        let col = (prefix.len() - prefix.rfind('\n').map_or(0, |pos| pos + 1)) as i64 + 1;
        (line, col, line, col + original_href.len() as i64 - 1)
    } else {
        (line, col, end_line, end_col)
    };
    let offset = i64::try_from(page.fm_offset()).unwrap_or(0);
    let ic = get_issue_counter();
    tracing::warn!(
        source = "unnecessary-broken-link-marker",
        ic = ic,
        line = if line == 0 { 0 } else { line + offset },
        col = col,
        end_line = if end_line == 0 { 0 } else { end_line + offset },
        end_col = end_col,
        url = original_href,
        redirect = href,
    );
    if data_issues {
        el.set_attribute("data-flaw", &ic.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod marker_tests {
    use super::*;
    use crate::issues::{DIssue, InMemoryLayer};
    use crate::pages::types::doc::doc_from_raw;
    use lol_html::{RewriteStrSettings, element, rewrite_str};
    use tracing_subscriber::layer::SubscriberExt;

    #[test]
    fn validates_marked_html_and_markdown_links() {
        crate::templ::broken_links::create_test_target();
        let doc = doc_from_raw(
            "---\ntitle: Test\nslug: Test\n---\n".into(),
            rari_types::globals::content_root().join("en-us/test/index.md"),
        )
        .unwrap();
        let page = Page::Doc(std::sync::Arc::new(doc));
        struct Case {
            name: &'static str,
            input: &'static str,
            source: Option<&'static str>,
            suggestion: Option<&'static str>,
            broken: bool,
        }
        let cases = [
            Case {
                name: "marked missing markdown",
                input: "[new](!/en-US/docs/BrandNew)",
                source: None,
                suggestion: None,
                broken: true,
            },
            Case {
                name: "marked missing HTML",
                input: r#"<a href="!/en-US/docs/BrandNew">new</a>"#,
                source: None,
                suggestion: None,
                broken: true,
            },
            Case {
                name: "unmarked missing",
                input: "[new](/en-US/docs/BrandNew)",
                source: Some("broken-link"),
                suggestion: None,
                broken: true,
            },
            Case {
                name: "resolved",
                input: "[target](!/en-US/docs/Glossary/Marker_test_target)",
                source: Some("unnecessary-broken-link-marker"),
                suggestion: Some("/en-US/docs/Glossary/Marker_test_target"),
                broken: false,
            },
            Case {
                name: "resolved raw HTML",
                input: r#"<a href="!/en-US/docs/Glossary/Marker_test_target">target</a>"#,
                source: Some("unnecessary-broken-link-marker"),
                suggestion: Some("/en-US/docs/Glossary/Marker_test_target"),
                broken: false,
            },
            Case {
                name: "fragment",
                input: "[target](!/en-US/docs/Glossary/Marker_test_target#test)",
                source: Some("unnecessary-broken-link-marker"),
                suggestion: Some("/en-US/docs/Glossary/Marker_test_target#test"),
                broken: false,
            },
            Case {
                name: "locale fallback",
                input: "[target](!/fr/docs/Glossary/Marker_test_target)",
                source: Some("unnecessary-broken-link-marker"),
                suggestion: Some("/en-US/docs/Glossary/Marker_test_target"),
                broken: false,
            },
            Case {
                name: "unchecked route",
                input: "[chat](!/discord)",
                source: Some("unnecessary-broken-link-marker"),
                suggestion: Some("/discord"),
                broken: false,
            },
            Case {
                name: "external",
                input: "[spec](!https://example.com)",
                source: Some("unnecessary-broken-link-marker"),
                suggestion: Some("https://example.com"),
                broken: false,
            },
            Case {
                name: "anchor",
                input: "[section](!#section)",
                source: Some("unnecessary-broken-link-marker"),
                suggestion: Some("#section"),
                broken: false,
            },
        ];
        for case in cases {
            let layer = InMemoryLayer::default();
            let subscriber = tracing_subscriber::registry().with(layer.clone());
            let _guard = tracing::subscriber::set_default(subscriber);
            let html = rari_md::m2h(case.input, Locale::EnUs).unwrap();
            let output = rewrite_str(
                &html,
                RewriteStrSettings::new()
                    .append_element_content_handler(element!("a[href]", |el| check_and_fix_link(
                        el, &page, true
                    ))),
            )
            .unwrap();
            assert!(!output.contains("href=\"!"), "{}: {output}", case.name);
            assert_eq!(
                output.contains("page-not-created"),
                case.broken,
                "{}: {output}",
                case.name
            );
            let events = layer.get_events();
            let issues = events.get("");
            let sources: Vec<_> = issues
                .iter()
                .flat_map(|issues| issues.iter())
                .flat_map(|issue| issue.fields.iter())
                .filter(|(key, _)| *key == "source")
                .map(|(_, value)| value.as_str())
                .collect();
            assert_eq!(
                sources,
                case.source.into_iter().collect::<Vec<_>>(),
                "{}",
                case.name
            );
            if let Some(suggestion) = case.suggestion {
                let issue = DIssue::from_issue(
                    issues
                        .as_ref()
                        .unwrap()
                        .iter()
                        .find(|issue| {
                            issue.fields.iter().any(|(key, value)| {
                                *key == "source" && Some(value.as_str()) == case.source
                            })
                        })
                        .unwrap()
                        .clone(),
                    &page,
                )
                .unwrap();
                assert_eq!(
                    issue.display_issue().suggestion.as_deref(),
                    Some(suggestion),
                    "{}",
                    case.name
                );
                assert_eq!(issue.display_issue().fixable, Some(true), "{}", case.name);
            }
        }
    }
    #[test]
    fn marker_flaws_point_to_reference_destinations() {
        crate::templ::broken_links::create_test_target();
        struct Case {
            name: &'static str,
            content: String,
            definition_line: i64,
            issues: usize,
        }
        let href = "!/en-US/docs/Glossary/Marker_test_target";
        let cases = [
            Case {
                name: "before use",
                content: format!("[target]: {href}\n\n[go][target]"),
                definition_line: 1,
                issues: 1,
            },
            Case {
                name: "after use",
                content: format!("[go][target]\n\n[target]: {href}"),
                definition_line: 3,
                issues: 1,
            },
            Case {
                name: "multiple uses",
                content: format!("[target]: {href}\n\n[go][target] and [again][target]"),
                definition_line: 1,
                issues: 2,
            },
        ];
        for case in cases {
            let doc = doc_from_raw(
                format!("---\ntitle: Test\nslug: Test\n---\n{}", case.content),
                rari_types::globals::content_root().join("en-us/test/index.md"),
            )
            .unwrap();
            let page = Page::Doc(std::sync::Arc::new(doc));
            let layer = InMemoryLayer::default();
            let subscriber = tracing_subscriber::registry().with(layer.clone());
            let _guard = tracing::subscriber::set_default(subscriber);
            let html = rari_md::m2h(page.content(), Locale::EnUs).unwrap();
            rewrite_str(
                &html,
                RewriteStrSettings::new()
                    .append_element_content_handler(element!("a[href]", |el| check_and_fix_link(
                        el, &page, true
                    ))),
            )
            .unwrap();
            let events = layer.get_events();
            let issues: Vec<_> = events
                .get("")
                .unwrap()
                .iter()
                .filter(|issue| {
                    issue.fields.iter().any(|(key, value)| {
                        *key == "source" && value == "unnecessary-broken-link-marker"
                    })
                })
                .cloned()
                .collect();
            assert_eq!(issues.len(), case.issues, "{}", case.name);
            for issue in issues {
                let issue = DIssue::from_issue(issue, &page).unwrap();
                let display = issue.display_issue();
                assert_eq!(
                    display.line,
                    Some(case.definition_line + page.fm_offset() as i64),
                    "{}",
                    case.name
                );
                assert_eq!(display.column, Some(11), "{}", case.name);
                assert_eq!(
                    display.end_column,
                    Some(10 + href.len() as i64),
                    "{}",
                    case.name
                );
            }
        }
    }
}
