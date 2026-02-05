use crate::error::DocError;
use crate::pages::page::{Page, PageLike};

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
