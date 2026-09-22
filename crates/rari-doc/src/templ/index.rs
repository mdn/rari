/// Render the alphabetical navigation shared by index-producing templates.
///
/// Deliberately not a `<nav>`: pages can hold several indexes, and unnamed
/// navigation landmarks are indistinguishable for screen reader users. The
/// wrapper stays an element so Fred's `.index .index-nav ul` styling applies
/// without matching the index's own `.index > ul`.
pub(crate) fn render_index_navigation(
    out: &mut String,
    letters: impl IntoIterator<Item = char>,
    id_prefix: &str,
) {
    let mut letters = letters.into_iter().peekable();
    if letters.peek().is_none() {
        return;
    }

    out.push_str(r#"<div class="index-nav"><ul>"#);
    for letter in letters {
        let (label, id) = index_letter_label_and_id(letter, id_prefix);
        out.extend([r##"<li><a href="#"##, &id, r#"">"#, &label, "</a></li>"]);
    }
    out.push_str("</ul></div>");
}

/// Normalize an index initial. ASCII-only, as index initials come from page
/// titles that are effectively ASCII, and `char::to_uppercase` can expand to
/// multiple chars.
pub(crate) fn index_letter(letter: char) -> char {
    letter.to_ascii_uppercase()
}

/// Returns the (label, fragment ID) for an index initial, both escaped for
/// direct interpolation into HTML: the ID is escaped as an attribute value and
/// reused verbatim in `href="#..."`, where it decodes to the same raw ID.
///
/// Callers must group by letters normalized via [`index_letter`]; this does not
/// normalize again, so mixed-case keys would render duplicate entries and IDs.
pub(crate) fn index_letter_label_and_id(letter: char, id_prefix: &str) -> (String, String) {
    let id = format!("{id_prefix}-{}", letter.to_ascii_lowercase());
    (
        html_escape::encode_safe(&letter.to_string()).into_owned(),
        html_escape::encode_double_quoted_attribute(&id).into_owned(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_index_navigation() {
        let cases = [
            (
                "index",
                vec!['A', 'B'],
                r##"<div class="index-nav"><ul><li><a href="#index-a">A</a></li><li><a href="#index-b">B</a></li></ul></div>"##,
            ),
            ("index", vec![], ""),
            (
                "index-interfaces",
                vec!['C'],
                r##"<div class="index-nav"><ul><li><a href="#index-interfaces-c">C</a></li></ul></div>"##,
            ),
        ];

        for (prefix, letters, expected) in cases {
            let mut out = String::new();
            render_index_navigation(&mut out, letters, prefix);
            assert_eq!(out, expected, "unexpected navigation for prefix `{prefix}`");
        }
    }

    #[test]
    fn test_index_letter_label_and_id_escapes_label() {
        assert_eq!(
            index_letter_label_and_id('<', "index"),
            ("&lt;".to_string(), "index-&lt;".to_string())
        );
    }

    #[test]
    fn test_index_letter_normalizes_ascii_case() {
        assert_eq!(index_letter('c'), 'C');
        assert_eq!(index_letter('C'), 'C');
        assert_eq!(index_letter('-'), '-');
    }

    #[test]
    fn test_index_letter_label_and_id_keeps_letter_case() {
        assert_eq!(
            index_letter_label_and_id('C', "index"),
            ("C".into(), "index-c".into())
        );
        assert_eq!(
            index_letter_label_and_id('c', "index"),
            ("c".into(), "index-c".into())
        );
    }
}
