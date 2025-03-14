use std::borrow::Cow;

use lol_html::html_content::Element;
use lol_html::HandlerResult;
use rari_types::fm_types::PageType;
use rari_types::locale::default_locale;
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

    let auto_link = el.has_attribute("data-autolink");
    if auto_link {
        el.remove_attribute("data-autolink");
    }
    if original_href.starts_with('/') || original_href.starts_with("https://developer.mozilla.org")
    {
        handle_internal_link(&original_href, el, page, data_issues, templ_link, auto_link)
    } else if original_href.starts_with("http:") || original_href.starts_with("https:") {
        handle_external_link(el)
    } else {
        Ok(())
    }
}

pub fn handle_external_link(el: &mut Element) -> HandlerResult {
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
                if let Some((start, _)) = pos.split_once('-') {
                    if let Some((line, col)) = start.split_once(':') {
                        let line = line
                            .parse::<i64>()
                            .map(|l| l + i64::try_from(page.fm_offset()).unwrap_or(l - 1))
                            .ok()
                            .unwrap_or(-1);
                        let col = col.parse::<i64>().ok().unwrap_or(0);
                        let ic = get_issue_counter();
                        if remove_href {
                            tracing::warn!(
                                source = "broken-link",
                                ic = ic,
                                line = line,
                                col = col,
                                url = original_href,
                            );
                        } else {
                            let source =
                                if original_href.to_lowercase() == resolved_href.to_lowercase() {
                                    "ill-cased-link"
                                } else {
                                    "redirected-link"
                                };
                            tracing::warn!(
                                source = source,
                                ic = ic,
                                line = line,
                                col = col,
                                url = original_href,
                                redirect = resolved_href
                            );
                        }
                        if data_issues {
                            el.set_attribute("data-flaw", &ic.to_string())?;
                        }
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
