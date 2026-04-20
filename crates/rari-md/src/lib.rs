use std::fmt::Write;

use comrak::nodes::{AstNode, NodeValue};
use comrak::{Arena, Options, parse_document};
use html::{CustomFormatter, RariContext};
use rari_types::locale::Locale;

use crate::error::MarkdownError;
use crate::p::{fix_p, is_empty_p, is_escaped_templ_p};

pub mod anchor;
pub(crate) mod ctype;
pub(crate) mod dl;
pub mod error;
pub mod ext;
pub(crate) mod html;
pub mod node_card;
pub(crate) mod p;
pub(crate) mod utils;

use dl::{convert_dl, is_dl};

/// Returns the byte offset of the next opening `<a` tag in `bytes` at or after `pos`.
/// Only matches tags where `<a` is followed by whitespace or `>` (not `<abbr>`, `<aside>`, etc.).
fn find_next_opening_a(bytes: &[u8], pos: usize) -> Option<usize> {
    for (offset, &byte) in bytes[pos..].iter().enumerate() {
        if byte != b'<' {
            continue;
        }
        let tag_start = pos + offset;
        // `bytes.get` returns `None` if `<` is the last byte — no room for a tag name.
        let first_tag_byte = bytes.get(tag_start + 1).copied()?;
        // Must be 'a' or 'A', not '/' (closing tag) and not another letter
        if first_tag_byte == b'a' || first_tag_byte == b'A' {
            // Must be followed by whitespace, '>', or end-of-input
            let second_tag_byte = bytes.get(tag_start + 2).copied().unwrap_or(b'>');
            if matches!(second_tag_byte, b' ' | b'\t' | b'\n' | b'\r' | b'>') {
                return Some(tag_start);
            }
        }
    }
    None
}

/// Injects `data-sourcepos` into every `<a` tag in `html`.
fn inject_sourcepos_in_opening_a(html: &mut String, sp: &str) {
    let attr = format!(" data-sourcepos=\"{sp}\"");
    let attr_len = attr.len();
    let mut pos = 0;
    while let Some(lt) = find_next_opening_a(html.as_bytes(), pos) {
        html.insert_str(lt + 2, &attr);
        pos = lt + 2 + attr_len;
    }
}

/// Advances `(line, line_start)` over `bytes[start..end]`, returning the updated pair.
/// `line_start` is the byte offset of the current line's start within the full literal.
fn advance_line_tracking(
    bytes: &[u8],
    start: usize,
    end: usize,
    mut line: usize,
    mut line_start: usize,
) -> (usize, usize) {
    for (offset, &byte) in bytes[start..end].iter().enumerate() {
        if byte == b'\n' {
            line += 1;
            line_start = start + offset + 1;
        }
    }
    (line, line_start)
}

/// Walks an HTML block literal, injecting `data-sourcepos` into every `<a` tag.
/// `block_start_line` is the 1-based line number of the first line of the block in the source.
fn inject_sourcepos_in_html_block(literal: &str, block_start_line: usize) -> String {
    let mut result = String::with_capacity(literal.len() + 64);
    let bytes = literal.as_bytes();
    let mut pos = 0;
    let mut line = block_start_line; // 1-based, tracks current line
    let mut line_start = 0; // byte offset of current line's start within `literal`

    while let Some(lt) = find_next_opening_a(bytes, pos) {
        (line, line_start) = advance_line_tracking(bytes, pos, lt, line, line_start);
        let start_col = literal[line_start..lt].chars().count() + 1;

        // Find the closing '>' of this opening tag (respects quoted attrs)
        let Some(gt) = find_opening_tag_end(bytes, lt) else {
            // Malformed tag with no closing `>` — emit as-is and stop
            result.push_str(&literal[pos..]);
            return result;
        };

        let (end_line, end_line_start) = advance_line_tracking(bytes, lt, gt, line, line_start);
        let end_col = literal[end_line_start..gt].chars().count() + 1;

        result.push_str(&literal[pos..lt + 2]);
        write!(
            result,
            " data-sourcepos=\"{line}:{start_col}-{end_line}:{end_col}\""
        )
        .unwrap();
        pos = lt + 2;
    }
    result.push_str(&literal[pos..]);
    result
}

/// Scans forward from `tag_start` in `bytes` to find the byte offset of the `>` that closes
/// the opening tag, handling double- and single-quoted attribute values.
///
/// Returns `None` if no closing `>` is found (malformed tag).
fn find_opening_tag_end(bytes: &[u8], tag_start: usize) -> Option<usize> {
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    for (offset, &byte) in bytes[tag_start..].iter().enumerate() {
        match byte {
            b'\'' if !in_double_quote => in_single_quote = !in_single_quote,
            b'"' if !in_single_quote => in_double_quote = !in_double_quote,
            b'>' if !in_single_quote && !in_double_quote => return Some(tag_start + offset),
            _ => {}
        }
    }

    None
}

/// Injects `data-sourcepos` attributes into raw HTML `<a>` tags in `HtmlInline` and
/// `HtmlBlock` AST nodes.
/// Allows `fix_link.rs` to report accurate line numbers for ill-cased or
/// redirected links that appear as raw HTML rather than Markdown link syntax.
fn annotate_raw_html_links(node: &AstNode<'_>) {
    let mut data = node.data.borrow_mut();
    let sp = data.sourcepos;
    match &mut data.value {
        NodeValue::HtmlInline(html) if find_next_opening_a(html.as_bytes(), 0).is_some() => {
            inject_sourcepos_in_opening_a(
                html,
                &format!(
                    "{}:{}-{}:{}",
                    sp.start.line, sp.start.column, sp.end.line, sp.end.column
                ),
            );
        }
        NodeValue::HtmlBlock(nhb) if find_next_opening_a(nhb.literal.as_bytes(), 0).is_some() => {
            nhb.literal = inject_sourcepos_in_html_block(&nhb.literal.clone(), sp.start.line);
        }
        _ => {}
    }
}

fn iter_nodes<'a, F>(node: &'a AstNode<'a>, f: &F)
where
    F: Fn(&'a AstNode<'a>),
{
    f(node);
    for c in node.children() {
        iter_nodes(c, f);
    }
}

pub struct M2HOptions {
    pub sourcepos: bool,
}

impl Default for M2HOptions {
    fn default() -> Self {
        Self { sourcepos: true }
    }
}

/// rari's custom markdown parser. This implements the MDN markdown extensions.
/// See [MDN Markdown](https://developer.mozilla.org/en-US/docs/MDN/Writing_guidelines/Howto/Markdown_in_MDN)
pub fn m2h(input: &str, locale: Locale) -> Result<String, MarkdownError> {
    m2h_internal(input, locale, Default::default())
}

pub fn m2h_internal(
    input: &str,
    locale: Locale,
    m2h_options: M2HOptions,
) -> Result<String, MarkdownError> {
    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.tagfilter = false;
    options.render.sourcepos = m2h_options.sourcepos;
    options.render.r#unsafe = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.header_id_prefix = Some(Default::default());
    let root = parse_document(&arena, input, &options);

    iter_nodes(root, &|node| {
        let (dl, templs_p, empty_p) = match node.data.borrow().value {
            NodeValue::List(_) => (is_dl(node), false, false),
            NodeValue::Paragraph => (false, is_escaped_templ_p(node), is_empty_p(node)),
            _ => (false, false, false),
        };
        if dl {
            convert_dl(node);
        }
        if templs_p || empty_p {
            fix_p(node)
        }
        if m2h_options.sourcepos {
            annotate_raw_html_links(node);
        }
    });

    let mut html = String::new();
    CustomFormatter::format_document(
        root,
        &options,
        &mut html,
        RariContext {
            stack: Vec::new(),
            locale,
        },
    )
    .map_err(|_| MarkdownError::HTMLFormatError)?;
    Ok(html)
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::utils::escape_href;

    #[test]
    fn render_code_tags() -> Result<(), anyhow::Error> {
        let out = m2h("`<select>`", Locale::EnUs)?;
        assert_eq!(
            out,
            "<p data-sourcepos=\"1:1-1:10\"><code data-sourcepos=\"1:1-1:10\">&lt;select&gt;</code></p>\n"
        );
        Ok(())
    }

    #[test]
    fn basic() -> Result<(), anyhow::Error> {
        let out = m2h("{{foo-bar}}", Locale::EnUs)?;
        assert_eq!(out, "<p data-sourcepos=\"1:1-1:11\">{{foo-bar}}</p>\n");
        Ok(())
    }

    #[test]
    fn test_comrak_sourcepos_multibyte() -> Result<(), anyhow::Error> {
        // Test to verify Comrak's sourcepos uses BYTES (1-based) for column positions
        // 🔥 emoji is 4 bytes but 1 character
        let input = "🔥 [link](url)";
        let html = m2h(input, Locale::EnUs)?;

        // Expected: "1:6" means position 6 (1-based) = byte offset 5 (0-based)
        // 🔥 (4 bytes) + space (1 byte) = 5 bytes before "[link]" starts
        // If it were CHARACTERS: would be "1:3" (emoji=1 char + space=1 char = 2 chars before link)

        // Verify Comrak uses byte-based columns (1-based)
        assert!(
            html.contains("data-sourcepos=\"1:6-1:16\""),
            "Comrak should use byte positions (1-based). Got: {}",
            html
        );

        Ok(())
    }

    #[test]
    fn line_break() -> Result<(), anyhow::Error> {
        let out = m2h("- {{foo}}\n  - : bar", Locale::EnUs)?;
        assert_eq!(
            out,
            "<dl data-sourcepos=\"1:1-2:9\">\n<dt data-sourcepos=\"1:1-2:9\">{{foo}}</dt>\n<dd data-sourcepos=\"2:3-2:9\">\n<p data-sourcepos=\"2:5-2:9\">bar</p>\n</dd>\n</dl>\n"
        );
        Ok(())
    }

    #[test]
    fn dt() -> Result<(), anyhow::Error> {
        let out = m2h("- {{foo}}\n  - : bar", Locale::EnUs)?;
        assert_eq!(
            out,
            "<dl data-sourcepos=\"1:1-2:9\">\n<dt data-sourcepos=\"1:1-2:9\">{{foo}}</dt>\n<dd data-sourcepos=\"2:3-2:9\">\n<p data-sourcepos=\"2:5-2:9\">bar</p>\n</dd>\n</dl>\n"
        );
        Ok(())
    }

    #[test]
    fn dt_double() -> Result<(), anyhow::Error> {
        let out = m2h("- foo\n  - : item1\n  - : item2", Locale::EnUs)?;
        assert_eq!(
            out,
            "<dl data-sourcepos=\"1:1-3:11\">\n<dt data-sourcepos=\"1:1-3:11\">foo</dt>\n<dd data-sourcepos=\"2:3-2:11\">\n<p data-sourcepos=\"2:5-2:11\">item1</p>\n</dd>\n<dd data-sourcepos=\"3:3-3:11\">\n<p data-sourcepos=\"3:5-3:11\">item2</p>\n</dd>\n</dl>\n"
        );
        Ok(())
    }

    #[test]
    fn code_macro() -> Result<(), anyhow::Error> {
        let out = m2h(r#"`{{foo}}` bar"#, Locale::EnUs)?;
        assert_eq!(
            out,
            "<p data-sourcepos=\"1:1-1:13\"><code data-sourcepos=\"1:1-1:9\">{{foo}}</code> bar</p>\n"
        );
        Ok(())
    }

    #[test]
    fn code_macro2() -> Result<(), anyhow::Error> {
        let out = m2h(r#"`aaaaaaa`"#, Locale::EnUs)?;
        assert_eq!(
            out,
            "<p data-sourcepos=\"1:1-1:9\"><code data-sourcepos=\"1:1-1:9\">aaaaaaa</code></p>\n"
        );
        Ok(())
    }

    #[test]
    fn macro_nl() -> Result<(), anyhow::Error> {
        let out = m2h("{{bar}}{{foo}}", Locale::EnUs)?;
        assert_eq!(out, "<p data-sourcepos=\"1:1-1:14\">{{bar}}{{foo}}</p>\n");
        Ok(())
    }

    #[test]
    fn li_p() -> Result<(), anyhow::Error> {
        let out = m2h("- foo\n- bar\n", Locale::EnUs)?;
        assert_eq!(
            out,
            "<ul data-sourcepos=\"1:1-2:5\">\n<li data-sourcepos=\"1:1-1:5\">foo</li>\n<li data-sourcepos=\"2:1-2:5\">bar</li>\n</ul>\n"
        );
        let out = m2h("- foo\n\n- bar\n", Locale::EnUs)?;
        assert_eq!(
            out,
            "<ul data-sourcepos=\"1:1-3:5\">\n<li data-sourcepos=\"1:1-1:5\">\n<p data-sourcepos=\"1:3-1:5\">foo</p>\n</li>\n<li data-sourcepos=\"3:1-3:5\">\n<p data-sourcepos=\"3:3-3:5\">bar</p>\n</li>\n</ul>\n"
        );
        Ok(())
    }

    #[test]
    fn callout() -> Result<(), anyhow::Error> {
        let out = m2h("> [!CALLOUT]\n> foobar", Locale::EnUs)?;
        assert_eq!(
            out,
            "<div class=\"callout\" data-sourcepos=\"1:1-2:8\">\n<p data-sourcepos=\"1:3-2:8\">\nfoobar</p>\n</div>\n"
        );
        Ok(())
    }

    #[test]
    fn callout_strong() -> Result<(), anyhow::Error> {
        let out = m2h("> [!CALLOUT]\n> **foobar**", Locale::EnUs)?;
        assert_eq!(
            out,
            "<div class=\"callout\" data-sourcepos=\"1:1-2:12\">\n<p data-sourcepos=\"1:3-2:12\">\n<strong data-sourcepos=\"2:3-2:12\">foobar</strong></p>\n</div>\n"
        );
        Ok(())
    }

    #[test]
    fn note() -> Result<(), anyhow::Error> {
        let out = m2h("> [!NOTE]\n> foobar", Locale::EnUs)?;
        assert_eq!(
            out,
            "<div class=\"notecard note\" data-add-note data-sourcepos=\"1:1-2:8\">\n<p data-sourcepos=\"1:3-2:8\">\nfoobar</p>\n</div>\n"
        );
        Ok(())
    }

    #[test]
    fn note_zh_locale() -> Result<(), anyhow::Error> {
        let out = m2h(
            "> [!NOTE]\n> This paragraph should have no leading spaces",
            Locale::ZhCn,
        )?;
        assert_eq!(
            out,
            "<div class=\"notecard note\" data-add-note data-sourcepos=\"1:1-2:46\">\n<p data-sourcepos=\"1:3-2:46\">This paragraph should have no leading spaces</p>\n</div>\n"
        );
        Ok(())
    }

    #[test]
    fn escape_hrefs() -> Result<(), anyhow::Error> {
        fn eh(s: &str) -> Result<String, anyhow::Error> {
            let mut out = String::with_capacity(s.len());
            escape_href(&mut out, s)?;
            Ok(out)
        }

        assert_eq!(eh("/en-US/foo/bar")?, "/en-US/foo/bar");
        assert_eq!(eh("/en-US/foo/\"")?, "/en-US/foo/&quot;");
        assert_eq!(eh("/en-US/foo<script")?, "/en-US/foo&lt;script");
        assert_eq!(eh("/en-US/foo&bar")?, "/en-US/foo&amp;bar");
        Ok(())
    }

    // ── raw-HTML sourcepos injection ─────────────────────────────────────────

    #[test]
    fn test_find_next_opening_a() {
        assert_eq!(find_next_opening_a(b"<a href=\"/foo\">", 0), Some(0));
        assert_eq!(find_next_opening_a(b"text <a href>", 0), Some(5));
        assert_eq!(find_next_opening_a(b"<A HREF=\"/foo\">", 0), Some(0)); // uppercase
        assert_eq!(find_next_opening_a(b"<a>", 0), Some(0)); // bare anchor
        assert_eq!(find_next_opening_a(b"</a>", 0), None); // closing tag
        assert_eq!(find_next_opening_a(b"<abbr>", 0), None); // not an <a>
        assert_eq!(find_next_opening_a(b"<aside>", 0), None); // not an <a>
        // Skips non-<a> tags, finds <a> further in
        assert_eq!(find_next_opening_a(b"<abbr><a href>", 0), Some(6));
    }

    #[test]
    fn test_inject_sourcepos_in_opening_a() {
        let mut html = String::from("<a href=\"/foo\">");
        inject_sourcepos_in_opening_a(&mut html, "1:1-1:15");
        assert_eq!(html, "<a data-sourcepos=\"1:1-1:15\" href=\"/foo\">");
    }

    #[test]
    fn test_inject_sourcepos_in_opening_a_uppercase() {
        let mut html = String::from("<A HREF=\"/foo\">");
        inject_sourcepos_in_opening_a(&mut html, "1:1-1:15");
        assert_eq!(html, "<A data-sourcepos=\"1:1-1:15\" HREF=\"/foo\">");
    }

    #[test]
    fn test_inject_sourcepos_closing_tag_unchanged() {
        let mut html = String::from("</a>");
        inject_sourcepos_in_opening_a(&mut html, "1:1-1:4");
        assert_eq!(html, "</a>"); // closing tag must not be modified
    }

    #[test]
    fn test_inject_sourcepos_in_html_block_single_line() {
        let result = inject_sourcepos_in_html_block("<a href=\"/foo\">text</a>", 1);
        assert_eq!(
            result,
            "<a data-sourcepos=\"1:1-1:15\" href=\"/foo\">text</a>"
        );
    }

    #[test]
    fn test_inject_sourcepos_in_html_block_multiline_a() {
        // The `>` ending the opening tag is on line 2
        let result = inject_sourcepos_in_html_block("<a\n  href=\"/foo\">text</a>", 1);
        assert_eq!(
            result,
            "<a data-sourcepos=\"1:1-2:14\"\n  href=\"/foo\">text</a>"
        );
    }

    #[test]
    fn test_inject_sourcepos_in_html_block_start_line_offset() {
        // Block starts on line 5 of the source
        let result = inject_sourcepos_in_html_block("<a href=\"/foo\">text</a>", 5);
        assert!(result.contains("data-sourcepos=\"5:1-5:15\""));
    }

    #[test]
    fn test_inject_sourcepos_in_html_block_multiple_links() {
        let input = "<a href=\"/foo\">A</a> <a href=\"/bar\">B</a>";
        let result = inject_sourcepos_in_html_block(input, 1);
        // Both links should get data-sourcepos
        let count = result.matches("data-sourcepos=").count();
        assert_eq!(
            count, 2,
            "Both <a> tags should get data-sourcepos: {result}"
        );
    }

    #[test]
    fn test_inject_sourcepos_in_html_block_skips_abbr() {
        let result = inject_sourcepos_in_html_block("<abbr title=\"x\">y</abbr>", 1);
        assert!(
            !result.contains("data-sourcepos"),
            "<abbr> should not get data-sourcepos: {result}"
        );
    }

    #[test]
    fn test_inject_sourcepos_in_html_block_uppercase_a() {
        // Uppercase <A> tag in block path should also get data-sourcepos
        let result = inject_sourcepos_in_html_block("<A HREF=\"/foo\">text</A>", 1);
        assert_eq!(
            result,
            "<A data-sourcepos=\"1:1-1:15\" HREF=\"/foo\">text</A>"
        );
    }

    #[test]
    fn test_find_next_opening_a_at_end_of_input() {
        // `<a` with nothing after it — c2 falls back to b'>' via unwrap_or
        assert_eq!(find_next_opening_a(b"<a", 0), Some(0));
        assert_eq!(find_next_opening_a(b"text<a", 0), Some(4));
    }

    #[test]
    fn test_inject_sourcepos_in_html_block_malformed_no_closing_gt() {
        // Malformed tag with no closing `>` — emitted as-is without data-sourcepos.
        let result = inject_sourcepos_in_html_block("<a href=\"/foo\"", 1);
        assert!(
            !result.contains("data-sourcepos="),
            "Malformed <a> tag (no closing >) should not get data-sourcepos: {result}"
        );
        assert_eq!(result, "<a href=\"/foo\"");
    }

    #[test]
    fn test_inject_sourcepos_in_html_block_multibyte_char_column() {
        // 'é' (U+00E9) is 2 bytes in UTF-8, so <a> sits at byte offset 2 but
        // character offset 1 (0-based), i.e. column 2 (1-based).
        // The HtmlBlock path currently computes byte-based columns
        // (start_col = lt - line_start + 1 = 2 - 0 + 1 = 3), whereas Comrak's
        // HtmlInline sourcepos uses character-based columns.
        // This test expects character-based columns (matching the inline path)
        // and will fail until the block path is fixed to count chars, not bytes.
        let result = inject_sourcepos_in_html_block("é<a href=\"/foo\">text</a>", 1);
        assert!(
            result.contains("data-sourcepos=\"1:2-1:16\""),
            "Column should be character-based (1:2-1:16), not byte-based (1:3-1:17): {result}"
        );
    }

    #[test]
    fn test_inject_sourcepos_in_opening_a_annotates_all() {
        // `inject_sourcepos_in_opening_a` loops over all `<a>` tags and injects
        // the same sourcepos string into each one. In the normal flow Comrak
        // produces one `HtmlInline` node per tag (so the loop runs once), but
        // the function's contract covers the multi-tag case too.
        let mut html = String::from("<a href=\"/a\"><a href=\"/b\">");
        inject_sourcepos_in_opening_a(&mut html, "1:1-1:13");
        assert_eq!(
            html.matches("data-sourcepos=").count(),
            2,
            "Both <a> tags should get data-sourcepos, got: {html}"
        );
    }

    // ── end-to-end m2h tests ─────────────────────────────────────────────────

    #[test]
    fn html_inline_a_gets_sourcepos() -> Result<(), anyhow::Error> {
        let out = m2h("text <a href=\"/foo\">link</a> end", Locale::EnUs)?;
        assert!(
            out.contains("<a data-sourcepos="),
            "Inline <a> tag should have data-sourcepos injected, got: {out}"
        );
        assert!(
            !out.contains("</a data-sourcepos"),
            "Closing </a> tag must not be modified, got: {out}"
        );
        Ok(())
    }

    #[test]
    fn html_block_a_gets_sourcepos() -> Result<(), anyhow::Error> {
        let out = m2h("<a href=\"/foo\">text</a>", Locale::EnUs)?;
        assert!(
            out.contains("<a data-sourcepos="),
            "Block-level <a> tag should have data-sourcepos injected, got: {out}"
        );
        Ok(())
    }

    #[test]
    fn html_block_multiline_a_gets_sourcepos() -> Result<(), anyhow::Error> {
        let out = m2h("<a\n  href=\"/foo\">text</a>", Locale::EnUs)?;
        assert!(
            out.contains("<a data-sourcepos="),
            "Multi-line block <a> tag should have data-sourcepos injected, got: {out}"
        );
        // sourcepos should span lines 1–2
        assert!(
            out.contains("data-sourcepos=\"1:1-2:"),
            "sourcepos should start on line 1 and end on line 2, got: {out}"
        );
        Ok(())
    }

    #[test]
    fn html_inline_a_sourcepos_disabled() -> Result<(), anyhow::Error> {
        let out = m2h_internal(
            "text <a href=\"/foo\">link</a> end",
            Locale::EnUs,
            M2HOptions { sourcepos: false },
        )?;
        // With sourcepos disabled no data-sourcepos attributes anywhere
        assert!(
            !out.contains("data-sourcepos"),
            "No data-sourcepos expected when sourcepos is disabled, got: {out}"
        );
        Ok(())
    }
}
