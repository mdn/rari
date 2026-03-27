use std::borrow::Cow;
use std::sync::LazyLock;

use regex::Regex;

/// Extracts a custom heading ID from `{#id}` syntax at the end of heading text.
/// Returns the custom ID if found.
pub fn extract_heading_id(content: &str) -> Option<&str> {
    static HEADING_ID_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\s*\{#([\w-]+)\}\s*$").unwrap());
    HEADING_ID_RE
        .captures(content)
        .map(|c| c.get(1).unwrap().as_str())
}

pub fn anchorize(content: &str) -> Cow<'_, str> {
    static REJECTED_CHARS: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"[*<>"$#%&+,/:;=?@\[\]^`{|}~')(\\]"#).unwrap());

    let id = REJECTED_CHARS.replace_all(content, "");
    let mut id = id.trim().to_lowercase();
    let mut prev = ' ';
    id.retain(|c| {
        let result = !c.is_ascii_whitespace() || !prev.is_ascii_whitespace();
        prev = c;
        result
    });
    let id = id.replace(' ', "_");
    if !id.is_empty() {
        if id == content {
            Cow::Borrowed(content)
        } else {
            Cow::Owned(id)
        }
    } else {
        Cow::Borrowed("sect")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn extract_simple_id() {
        assert_eq!(
            extract_heading_id("Heading {#custom-id}"),
            Some("custom-id")
        );
    }

    #[test]
    fn extract_id_with_underscores() {
        assert_eq!(
            extract_heading_id("Heading {#my_custom_id}"),
            Some("my_custom_id")
        );
    }

    #[test]
    fn extract_id_trailing_whitespace() {
        assert_eq!(
            extract_heading_id("Heading {#custom-id}  "),
            Some("custom-id")
        );
    }

    #[test]
    fn no_id_when_not_at_end() {
        assert_eq!(
            extract_heading_id("Heading {#custom-id} trailing text"),
            None
        );
    }

    #[test]
    fn no_id_without_hash() {
        assert_eq!(extract_heading_id("Heading {custom-id}"), None);
    }

    #[test]
    fn no_id_plain_heading() {
        assert_eq!(extract_heading_id("Plain heading text"), None);
    }

    #[test]
    fn no_id_empty_braces() {
        assert_eq!(extract_heading_id("Heading {#}"), None);
    }
}
