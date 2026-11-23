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

/// Parses a comrak `data-sourcepos` value (`line:col-end_line:end_col`),
/// shifting lines by the frontmatter offset.
pub(crate) fn parse_sourcepos(pos: &str, fm_offset: usize) -> Option<(i64, i64, i64, i64)> {
    let (start, end) = pos.split_once('-')?;
    let (line, col) = start.split_once(':')?;
    let shift = |l: i64| l + i64::try_from(fm_offset).unwrap_or(l - 1);
    let line = line.parse::<i64>().map(shift).ok().unwrap_or(-1);
    let col = col.parse::<i64>().ok().unwrap_or(0);
    let (end_line, end_col) = end
        .split_once(':')
        .map(|(end_line, end_col)| {
            let end_line = end_line.parse::<i64>().map(shift).ok().unwrap_or(-1);
            let end_col = end_col.parse::<i64>().ok().unwrap_or(0);
            (end_line, end_col)
        })
        .unwrap_or((-1, -1));
    Some((line, col, end_line, end_col))
}

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

    let auto_link = el.has_attribute("data-autolink");
    if auto_link {
        el.remove_attribute("data-autolink");
    }
    if original_href.starts_with('/') || original_href.starts_with("https://developer.mozilla.org")
    {
        handle_internal_link(&original_href, el, page, data_issues, templ_link, auto_link)
    } else if original_href.starts_with("http:") || original_href.starts_with("https:") {
        handle_external_link(el, page.locale())
    } else {
        Ok(())
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
        if (original_href != resolved_href || remove_href) && !en_us_fallback {
            if let Some(pos) = el.get_attribute("data-sourcepos") {
                if let Some((line, col, end_line, end_col)) =
                    parse_sourcepos(&pos, page.fm_offset())
                {
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

#[cfg(test)]
mod tests {
    use super::parse_sourcepos;

    struct Case {
        name: &'static str,
        pos: &'static str,
        fm_offset: usize,
        expected: Option<(i64, i64, i64, i64)>,
    }

    #[test]
    fn parses_sourcepos() {
        let cases = vec![
            Case {
                name: "full",
                pos: "3:5-3:20",
                fm_offset: 4,
                expected: Some((7, 5, 7, 20)),
            },
            Case {
                name: "multi-line",
                pos: "1:1-2:9",
                fm_offset: 0,
                expected: Some((1, 1, 2, 9)),
            },
            Case {
                name: "no end position",
                pos: "3:5-x",
                fm_offset: 1,
                expected: Some((4, 5, -1, -1)),
            },
            Case {
                name: "unparsable numbers",
                pos: "a:b-c:d",
                fm_offset: 1,
                expected: Some((-1, 0, -1, 0)),
            },
            Case {
                name: "no dash",
                pos: "3:5",
                fm_offset: 0,
                expected: None,
            },
            Case {
                name: "no colon",
                pos: "3-4:5",
                fm_offset: 0,
                expected: None,
            },
        ];
        for case in cases {
            assert_eq!(
                parse_sourcepos(case.pos, case.fm_offset),
                case.expected,
                "{}",
                case.name
            );
        }
    }
}
