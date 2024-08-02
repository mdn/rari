use comrak::nodes::{AstNode, NodeValue};
use comrak::{parse_document, Arena, ComrakOptions};
use rari_types::locale::Locale;

use crate::error::MarkdownError;
use crate::p::{fix_p, is_empty_p, is_escaped_templ_p};

pub mod anchor;
pub(crate) mod ctype;
pub(crate) mod dl;
pub mod error;
pub(crate) mod ext;
pub(crate) mod html;
pub(crate) mod li;
pub mod node_card;
pub(crate) mod p;

use dl::{convert_dl, is_dl};
use html::format_document;

use self::li::remove_p;

fn iter_nodes<'a, F>(node: &'a AstNode<'a>, f: &F)
where
    F: Fn(&'a AstNode<'a>),
{
    f(node);
    for c in node.children() {
        iter_nodes(c, f);
    }
}

/// rari's custom markdown parser. This implements the MDN markdown extensions.
/// See [MDN Markdown](https://developer.mozilla.org/en-US/docs/MDN/Writing_guidelines/Howto/Markdown_in_MDN)
pub fn m2h(input: &str, locale: Locale) -> Result<String, MarkdownError> {
    let arena = Arena::new();
    let mut options = ComrakOptions::default();
    options.extension.tagfilter = false;
    options.render.unsafe_ = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.header_ids = Some(Default::default());
    let root = parse_document(&arena, input, &options);

    iter_nodes(root, &|node| {
        let (dl, li, templs_p, empty_p) = match node.data.borrow().value {
            NodeValue::List(_) => (is_dl(node), true, false, false),
            NodeValue::Paragraph => (false, false, is_escaped_templ_p(node), is_empty_p(node)),
            _ => (false, false, false, false),
        };
        if dl {
            convert_dl(node);
        } else if li {
            remove_p(node);
        }
        if templs_p || empty_p {
            fix_p(node)
        }
    });

    let mut html = vec![];
    format_document(root, &options, &mut html, locale)
        .map_err(|_| MarkdownError::HTMLFormatError)?;
    let encoded_html = String::from_utf8(html).map_err(|_| MarkdownError::HTMLFormatError)?;
    Ok(encoded_html)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn render_code_tags() -> Result<(), anyhow::Error> {
        let out = m2h("`<select>`", Locale::EnUs)?;
        assert_eq!(out, "<p><code>&lt;select&gt;</code></p>\n");
        Ok(())
    }

    #[test]
    fn basic() -> Result<(), anyhow::Error> {
        let out = m2h("{{foo-bar}}", Locale::EnUs)?;
        assert_eq!(out, "<p>{{foo-bar}}</p>\n");
        Ok(())
    }

    #[test]
    fn line_break() -> Result<(), anyhow::Error> {
        let out = m2h("{{foo}} bar", Locale::EnUs)?;
        assert_eq!(out, "<p>{{foo}} bar</p>\n");
        let out = m2h("- {{foo}}\n  - : bar", Locale::EnUs)?;
        assert_eq!(
            out,
            "<dl>\n<dt id=\"foo\" data-add-link>{{foo}}</dt>\n<dd>\n<p>bar</p>\n</dd>\n</dl>\n"
        );
        Ok(())
    }

    #[test]
    fn dt() -> Result<(), anyhow::Error> {
        let out = m2h("{{foo}} bar", Locale::EnUs)?;
        assert_eq!(out, "<p>{{foo}} bar</p>\n");
        let out = m2h("- {{foo}}\n  - : bar", Locale::EnUs)?;
        assert_eq!(
            out,
            "<dl>\n<dt id=\"foo\" data-add-link>{{foo}}</dt>\n<dd>\n<p>bar</p>\n</dd>\n</dl>\n"
        );
        Ok(())
    }
    #[test]
    fn code_macro() -> Result<(), anyhow::Error> {
        let out = m2h(r#"`{{foo}}` bar"#, Locale::EnUs)?;
        assert_eq!(out, "<p><code>{{foo}}</code> bar</p>\n");
        Ok(())
    }

    #[test]
    fn macro_nl() -> Result<(), anyhow::Error> {
        let out = m2h("{{bar}}{{foo}}", Locale::EnUs)?;
        assert_eq!(out, "<p>{{bar}}{{foo}}</p>\n");
        Ok(())
    }

    #[test]
    fn li_p() -> Result<(), anyhow::Error> {
        let out = m2h("- foo\n\n- bar\n", Locale::EnUs)?;
        assert_eq!(out, "<ul>\n<li>foo</li>\n<li>bar</li>\n</ul>\n");
        Ok(())
    }

    #[test]
    fn callout() -> Result<(), anyhow::Error> {
        let out = m2h("> **Callout:** foobar", Locale::EnUs)?;
        assert_eq!(out, "<div class=\"callout\">\n<p> foobar</p>\n</div>\n");
        Ok(())
    }

    #[test]
    fn callout_strong() -> Result<(), anyhow::Error> {
        let out = m2h("> **Callout:** **foobar**", Locale::EnUs)?;
        assert_eq!(
            out,
            "<div class=\"callout\">\n<p> <strong>foobar</strong></p>\n</div>\n"
        );
        Ok(())
    }

    #[test]
    fn note() -> Result<(), anyhow::Error> {
        let out = m2h("> **Note:** foobar", Locale::EnUs)?;
        assert_eq!(
            out,
            "<div class=\"notecard note\">\n<p> foobar</p>\n</div>\n"
        );
        Ok(())
    }
}
