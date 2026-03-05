use crate::error::DocError;
use crate::pages::page::{Page, PageLike};

pub enum TitleFormat {
    Plain,
    Html,
}

pub fn process_backticks(title: &str, format: TitleFormat) -> String {
    let html = matches!(format, TitleFormat::Html);
    let mut out = String::with_capacity(title.len() * 2);
    // swap escaped backticks for a unicode noncharacter placeholder:
    let normalized = title.replace("\\`", "\u{FFFE}");
    let parts: Vec<&str> = normalized.split('`').collect();

    for (i, s) in parts.iter().enumerate() {
        let is_odd = i % 2 == 1;
        let is_last = i == parts.len() - 1;
        let is_unmatched = is_odd && is_last;
        let is_code = is_odd && !is_last;
        let s = s.replace('\u{FFFE}', "`");

        if is_unmatched {
            out.push('`');
        }
        if html {
            if is_code {
                out.push_str("<code>");
            }
            out.push_str(&html_escape::encode_text(&s));
            if is_code {
                out.push_str("</code>");
            }
        } else {
            out.push_str(&s);
        }
    }
    out
}

pub fn transform_title(title: &str) -> &str {
    match title {
        "Web technology for developers" => "References",
        "Learn web development" => "Learn",
        "HTML: HyperText Markup Language" => "HTML",
        "CSS: Cascading Style Sheets" => "CSS",
        "Graphics on the Web" => "Graphics",
        "HTML elements reference" => "Elements",
        "JavaScript reference" => "Reference",
        "JavaScript Guide" => "Guide",
        "Structuring the web with HTML" => "HTML",
        "Learn to style HTML using CSS" => "CSS",
        "Web forms — Working with user data" => "Forms",
        _ => title,
    }
}

pub fn page_title(doc: &impl PageLike, with_suffix: bool) -> Result<String, DocError> {
    let mut out = String::new();

    out.push_str(doc.title());

    if let Some(root_url) = root_doc_url(doc.url())
        && root_url != doc.url()
    {
        let root_doc = Page::from_url_with_fallback(root_url)?;
        out.push_str(" - ");
        out.push_str(root_doc.short_title().unwrap_or(root_doc.title()));
    }
    if with_suffix && let Some(suffix) = doc.title_suffix() {
        out.push_str(" | ");
        out.push_str(suffix);
    }
    Ok(out)
}

pub fn root_doc_url(url: &str) -> Option<&str> {
    let m = url
        .match_indices('/')
        .map(|(i, _)| i)
        .zip(url.split('/').skip(1))
        .collect::<Vec<_>>();
    if m.len() < 3 {
        return None;
    }
    if matches!(m[1].1, "blog" | "curriculum") {
        return None;
    }
    if m[1].1 == "docs" {
        if m[2].1 == "Web" {
            if let Some(base) = m.iter().rfind(|p| matches!(p.1, "Guides" | "Reference")) {
                return Some(&url[..base.0]);
            }
            return Some(&url[..*m.get(4).map(|(i, _)| i).unwrap_or(&url.len())]);
        }
        if matches!(m[2].1, "conflicting" | "orphaned") {
            return None;
        }
    }
    Some(&url[..*m.get(3).map(|(i, _)| i).unwrap_or(&url.len())])
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_backtick_processing() {
        let cases: &[(&str, &str, &str)] = &[
            (
                "no backticks here",
                "no backticks here",
                "no backticks here",
            ),
            ("", "", ""),
            (
                "`<input>`: The Input element",
                "<input>: The Input element",
                "<code>&lt;input&gt;</code>: The Input element",
            ),
            (
                "`foo` and `bar`",
                "foo and bar",
                "<code>foo</code> and <code>bar</code>",
            ),
            (
                "The `<input>` & `<output>` elements",
                "The <input> & <output> elements",
                "The <code>&lt;input&gt;</code> &amp; <code>&lt;output&gt;</code> elements",
            ),
            ("foo `bar", "foo `bar", "foo `bar"),
            (
                "`foo` bar `baz",
                "foo bar `baz",
                "<code>foo</code> bar `baz",
            ),
            ("`foo``bar`", "foobar", "<code>foo</code><code>bar</code>"),
            ("``", "", "<code></code>"),
            ("foo `` bar", "foo  bar", "foo <code></code> bar"),
            ("\\`", "`", "`"),
            ("\\`foo\\`", "`foo`", "`foo`"),
            ("\\`foo` bar", "`foo` bar", "`foo` bar"),
            ("`foo\\`bar`", "foo`bar", "<code>foo`bar</code>"),
        ];

        for (input, expected_plain, expected_html) in cases {
            assert_eq!(
                process_backticks(input, TitleFormat::Plain),
                *expected_plain,
            );
            assert_eq!(process_backticks(input, TitleFormat::Html), *expected_html,);
        }
    }

    #[test]
    fn test_root_doc_url() {
        assert_eq!(
            root_doc_url("/en-US/docs/Web/CSS/Reference/Properties/border"),
            Some("/en-US/docs/Web/CSS")
        );
        assert_eq!(
            root_doc_url("/en-US/docs/Web/CSS"),
            Some("/en-US/docs/Web/CSS")
        );
        assert_eq!(
            root_doc_url("/en-US/docs/Learn/foo"),
            Some("/en-US/docs/Learn")
        );
        assert_eq!(root_doc_url("/en-US/blog/foo"), None);
        assert_eq!(root_doc_url("/en-US/curriculum/foo"), None);
    }
}
