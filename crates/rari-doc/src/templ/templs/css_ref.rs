use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};

use itertools::Itertools;
use rari_templ_func::rari_f;
use rari_types::fm_types::{FeatureStatus, PageType};
use rari_types::locale::Locale;
use rari_utils::concat_strs;

use crate::error::DocError;
use crate::helpers::subpages::get_sub_pages;
use crate::helpers::title::{TitleFormat, render_title};
use crate::pages::page::{Page, PageLike};
use crate::pages::types::doc::Doc;
use crate::templ::api::RariApi;

/// Private-use placeholders to smuggle `<code>` tags through `RariApi::link`,
/// which would otherwise re-encode them as `&lt;code&gt;`.
const CODE_OPEN_PLACEHOLDER: &str = "\u{E000}";
const CODE_CLOSE_PLACEHOLDER: &str = "\u{E001}";

#[rari_f(register = "crate::Templ")]
pub fn css_ref() -> Result<String, DocError> {
    let mut index = BTreeMap::<char, HashMap<&str, (String, String)>>::new();

    let css_pages = get_sub_pages("/en-US/docs/Web/CSS", None, Default::default())?;
    for page in css_pages
        .iter()
        .filter(|&page| is_indexed_css_ref_page(page))
    {
        // Use the translated page's title if available, falling back to the
        // en-US page already in hand. `fallback: false` avoids re-loading the
        // en-US doc when no translation exists.
        let translated = (env.locale != Locale::EnUs)
            .then(|| Doc::page_from_slug(page.slug(), env.locale, false).ok())
            .flatten();
        let label_page = translated.as_ref().unwrap_or(page);
        let (html_label, plain_label) = labels_from_page(label_page);
        let initial = initial_letter(&plain_label);
        let entry = index.entry(initial).or_default();
        entry
            .entry(page.slug())
            .or_insert((html_label, plain_label));
    }

    let mut out = String::new();
    out.push_str(r#"<div class="index">"#);
    for (letter, items) in index {
        out.push_str("<h3>");
        out.push_str(&html_escape::encode_safe(letter.encode_utf8(&mut [0; 4])));
        out.push_str("</h3><ul>");
        for (url, (html_label, _)) in items
            .into_iter()
            .sorted_by(|(_, (_, a)), (_, (_, b))| compare_items(a, b))
        {
            let placeholder_label = html_label
                .replace("<code>", CODE_OPEN_PLACEHOLDER)
                .replace("</code>", CODE_CLOSE_PLACEHOLDER);
            let link = RariApi::link(
                &concat_strs!("/", url),
                Some(env.locale),
                Some(&placeholder_label),
                false,
                None,
                false,
            )?;
            out.extend([
                "<li>",
                &link
                    .replace(CODE_OPEN_PLACEHOLDER, "<code>")
                    .replace(CODE_CLOSE_PLACEHOLDER, "</code>"),
                "</li>",
            ]);
        }
        out.push_str("</ul>");
    }
    out.push_str(r#"</div>"#);

    Ok(out)
}

fn is_indexed_css_ref_page(page: &Page) -> bool {
    matches!(
        page.page_type(),
        PageType::CssType
            | PageType::CssAtRule
            | PageType::CssKeyword
            | PageType::CssFunction
            | PageType::CssSelector
            | PageType::CssProperty
            | PageType::CssPseudoElement
            | PageType::CssPseudoClass
            | PageType::CssShorthandProperty
            | PageType::CssAtRuleDescriptor
    ) && !page
        .status()
        .iter()
        .any(|s| matches!(s, FeatureStatus::Deprecated | FeatureStatus::NonStandard))
}

fn compare_items(a: &str, b: &str) -> Ordering {
    let ord = a
        .trim_matches(|c: char| !c.is_ascii_alphabetic() && c != '(' && c != ')' && c != '-')
        .cmp(
            b.trim_matches(|c: char| !c.is_ascii_alphabetic() && c != '(' && c != ')' && c != '-'),
        );
    if ord == Ordering::Equal {
        a.cmp(b)
    } else {
        ord
    }
}

fn initial_letter(s: &str) -> char {
    s.chars()
        .find(|&c| c.is_ascii_alphabetic() || c == '-')
        .unwrap_or('?')
        .to_ascii_uppercase()
}

/// Returns the (HTML, plain) labels for a CSS reference page.
/// Backticks in the raw title are rendered as `<code>` tags in the HTML form.
/// For at-rule descriptors, the at-rule name is appended in parentheses
/// (e.g., "<code>font-family</code> (<code>@font-face</code>)").
fn labels_from_page(page: &Page) -> (String, String) {
    let title_raw = match page {
        Page::Doc(doc) => doc.meta.title_raw.as_str(),
        _ => page.title(),
    };
    compute_labels(title_raw, page.page_type(), page.slug())
}

fn compute_labels(title_raw: &str, page_type: PageType, slug: &str) -> (String, String) {
    let mut html = render_title(title_raw, TitleFormat::Html);
    let mut plain = render_title(title_raw, TitleFormat::Plain);

    if page_type == PageType::CssAtRuleDescriptor
        && let Some(at_rule) = slug.rsplit('/').nth(1).filter(|s| s.starts_with('@'))
    {
        html.push_str(" (<code>");
        html.push_str(&html_escape::encode_text(at_rule));
        html.push_str("</code>)");
        plain.push_str(" (");
        plain.push_str(at_rule);
        plain.push(')');
    }
    (html, plain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_labels_plain_title() {
        let (html, plain) = compute_labels(
            "background-color",
            PageType::CssProperty,
            "Web/CSS/background-color",
        );
        assert_eq!(html, "background-color");
        assert_eq!(plain, "background-color");
    }

    #[test]
    fn test_compute_labels_backticks() {
        let (html, plain) = compute_labels(
            "`background-color`",
            PageType::CssProperty,
            "Web/CSS/background-color",
        );
        assert_eq!(html, "<code>background-color</code>");
        assert_eq!(plain, "background-color");
    }

    #[test]
    fn test_compute_labels_partial_backticks() {
        let (html, plain) = compute_labels(
            "`<input>`: The Input element",
            PageType::CssSelector,
            "Web/CSS/whatever",
        );
        assert_eq!(html, "<code>&lt;input&gt;</code>: The Input element");
        assert_eq!(plain, "<input>: The Input element");
    }

    #[test]
    fn test_compute_labels_at_rule_descriptor() {
        let (html, plain) = compute_labels(
            "`font-family`",
            PageType::CssAtRuleDescriptor,
            "Web/CSS/Reference/At-rules/@font-face/font-family",
        );
        assert_eq!(html, "<code>font-family</code> (<code>@font-face</code>)");
        assert_eq!(plain, "font-family (@font-face)");
    }

    #[test]
    fn test_compute_labels_at_rule_descriptor_no_at_rule_in_slug() {
        let (html, plain) = compute_labels(
            "font-family",
            PageType::CssAtRuleDescriptor,
            "Web/CSS/font-family",
        );
        assert_eq!(html, "font-family");
        assert_eq!(plain, "font-family");
    }

    #[test]
    fn test_initial_letter() {
        assert_eq!(initial_letter("background-color"), 'B');
        assert_eq!(initial_letter("`font-family`"), 'F');
        assert_eq!(initial_letter("-webkit-foo"), '-');
        assert_eq!(initial_letter("@font-face"), 'F');
        assert_eq!(initial_letter(""), '?');
    }

    #[test]
    fn test_compare_items() {
        assert_eq!(compare_items("apple", "banana"), Ordering::Less);
        assert_eq!(compare_items("banana", "apple"), Ordering::Greater);
        assert_eq!(compare_items("apple", "apple"), Ordering::Equal);
        // Leading non-alphabetics are trimmed before comparing; when the
        // trimmed forms are equal, fall back to raw comparison.
        assert_eq!(compare_items("@apple", "apple"), Ordering::Less);
        assert_eq!(compare_items("apple", "@apple"), Ordering::Greater);
    }
}
