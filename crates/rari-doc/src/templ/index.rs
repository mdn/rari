/// Render the alphabetical navigation shared by index-producing templates.
pub(crate) fn render_index_navigation(
    out: &mut String,
    letters: impl IntoIterator<Item = char>,
    id_prefix: &str,
) {
    out.push_str(r#"<nav class="index-nav"><ul>"#);
    for letter in letters {
        let (label, id) = index_letter_label_and_id(letter, id_prefix);
        out.extend([r##"<li><a href="#"##, &id, r#"">"#, &label, "</a></li>"]);
    }
    out.push_str("</ul></nav>");
}

pub(crate) fn index_letter(letter: char) -> char {
    letter.to_ascii_uppercase()
}

pub(crate) fn index_letter_label_and_id(letter: char, id_prefix: &str) -> (String, String) {
    let letter = index_letter(letter);
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
                r##"<nav class="index-nav"><ul><li><a href="#index-a">A</a></li><li><a href="#index-b">B</a></li></ul></nav>"##,
            ),
            (
                "index-interfaces",
                vec!['C'],
                r##"<nav class="index-nav"><ul><li><a href="#index-interfaces-c">C</a></li></ul></nav>"##,
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
        assert_eq!(
            index_letter_label_and_id('c', "index"),
            ("C".into(), "index-c".into())
        );
    }
}
