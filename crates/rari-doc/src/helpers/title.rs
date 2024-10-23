use crate::error::DocError;
use crate::pages::page::{Page, PageLike};

pub fn transform_title(title: &str) -> &str {
    match title {
        "Web technology for developers" => "References",
        "Learn web development" => "Guides",
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

    if let Some(root_url) = root_doc_url(doc.url()) {
        if root_url != doc.url() {
            let root_doc = Page::from_url_with_fallback(root_url)?;
            out.push_str(" - ");
            out.push_str(root_doc.title());
        }
    }
    if with_suffix {
        if let Some(suffix) = doc.title_suffix() {
            out.push_str(" | ");
            out.push_str(suffix);
        }
    }
    Ok(out)
}

pub fn root_doc_url(url: &str) -> Option<&str> {
    let m = url.match_indices('/').map(|(i, _)| i).collect::<Vec<_>>();
    if m.len() < 3 {
        return None;
    }
    if url[m[1]..].starts_with("/blog") || url[m[1]..].starts_with("/curriculum") {
        return None;
    }
    if url[m[1]..].starts_with("/docs/Web/") {
        return Some(&url[..*m.get(4).unwrap_or(&url.len())]);
    }
    if url[m[1]..].starts_with("/docs/conflicting/") || url[m[1]..].starts_with("/docs/orphaned/") {
        return None;
    }
    Some(&url[..*m.get(3).unwrap_or(&url.len())])
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_root_doc_url() {
        assert_eq!(
            root_doc_url("/en-US/docs/Web/CSS/border"),
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
