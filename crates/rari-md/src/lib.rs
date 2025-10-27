use comrak::nodes::{AstNode, NodeValue};
use comrak::{parse_document, Arena, ComrakOptions};
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
//use html::format_document;

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
    let mut options = ComrakOptions::default();
    options.extension.tagfilter = false;
    options.render.sourcepos = m2h_options.sourcepos;
    options.render.unsafe_ = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.header_ids = Some(Default::default());
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
    });

    let mut html = vec![];
    CustomFormatter::format_document(
        root,
        &options,
        &mut html,
        RariContext {
            stack: Vec::new(),
            locale,
        },
    )
    //format_document(root, &options, &mut html, locale)
    .map_err(|_| MarkdownError::HTMLFormatError)?;
    let encoded_html = String::from_utf8(html).map_err(|_| MarkdownError::HTMLFormatError)?;
    Ok(encoded_html)
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::utils::escape_href;

    #[test]
    fn render_code_tags() -> Result<(), anyhow::Error> {
        let out = m2h("`<select>`", Locale::EnUs)?;
        assert_eq!(out,
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
        assert_eq!(out, "<p data-sourcepos=\"1:1-1:13\"><code data-sourcepos=\"1:1-1:9\">{{foo}}</code> bar</p>\n");
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
        assert_eq!(out, "<ul data-sourcepos=\"1:1-2:5\">\n<li data-sourcepos=\"1:1-1:5\">foo</li>\n<li data-sourcepos=\"2:1-2:5\">bar</li>\n</ul>\n");
        let out = m2h("- foo\n\n- bar\n", Locale::EnUs)?;
        assert_eq!(out, "<ul data-sourcepos=\"1:1-3:5\">\n<li data-sourcepos=\"1:1-2:0\">\n<p data-sourcepos=\"1:3-1:5\">foo</p>\n</li>\n<li data-sourcepos=\"3:1-3:5\">\n<p data-sourcepos=\"3:3-3:5\">bar</p>\n</li>\n</ul>\n");
        Ok(())
    }

    #[test]
    fn callout() -> Result<(), anyhow::Error> {
        let out = m2h("> **Callout:** foobar", Locale::EnUs)?;
        assert_eq!(out, "<div class=\"callout\" data-sourcepos=\"1:1-1:21\">\n<p data-sourcepos=\"1:3-1:21\"> foobar</p>\n</div>\n");
        Ok(())
    }

    #[test]
    fn callout_strong() -> Result<(), anyhow::Error> {
        let out = m2h("> **Callout:** **foobar**", Locale::EnUs)?;
        assert_eq!(
            out,
            "<div class=\"callout\" data-sourcepos=\"1:1-1:25\">\n<p data-sourcepos=\"1:3-1:25\"> <strong data-sourcepos=\"1:16-1:25\">foobar</strong></p>\n</div>\n"
        );
        Ok(())
    }

    #[test]
    fn note() -> Result<(), anyhow::Error> {
        let out = m2h("> **Note:** foobar", Locale::EnUs)?;
        assert_eq!(
            out,
            "<div class=\"notecard note\" data-add-note data-sourcepos=\"1:1-1:18\">\n<p data-sourcepos=\"1:3-1:18\"> foobar</p>\n</div>\n"
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
            let mut out = Vec::with_capacity(s.len());
            escape_href(&mut out, s.as_bytes())?;
            Ok(String::from_utf8(out)?)
        }

        assert_eq!(eh("/en-US/foo/bar")?, "/en-US/foo/bar");
        assert_eq!(eh("/en-US/foo/\"")?, "/en-US/foo/&quot;");
        assert_eq!(eh("/en-US/foo<script")?, "/en-US/foo&lt;script");
        assert_eq!(eh("/en-US/foo&bar")?, "/en-US/foo&amp;bar");
        Ok(())
    }
}
