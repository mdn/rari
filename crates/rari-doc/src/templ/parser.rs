use rari_types::{Arg, Quotes};
use tree_sitter::TreeCursor;

use crate::error::DocError;

#[derive(Debug)]
pub struct TextToken {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug)]
pub struct MacroToken {
    pub start: usize,
    pub end: usize,
    pub ident: String,
    pub pos: (usize, usize),
    pub args: Vec<Option<Arg>>,
    pub expect_missing: bool,
}

fn from_node<'a>(
    value: tree_sitter::Node<'a>,
    content: &str,
    cursor: &mut TreeCursor<'a>,
) -> Option<MacroToken> {
    let mut ident = String::new();
    let mut args = vec![];
    let mut expect_missing = false;
    let mut walker = value.walk();
    for child in value.named_children(&mut walker) {
        match child.kind() {
            "ident" => {
                ident = content[child.start_byte()..child.end_byte()].to_string();
            }
            "expect_missing" => {
                expect_missing = true;
            }
            "args" => {
                args = child
                    .named_children(cursor)
                    .map(|arg| ts_to_arg(arg, content))
                    .collect();
            }
            _ => {}
        }
    }
    let start = value.start_byte();
    let end = value.end_byte();
    let start_position = value.start_position();
    let pos = (start_position.row, start_position.column);
    Some(MacroToken {
        start,
        end,
        pos,
        ident,
        args,
        expect_missing,
    })
}

fn ts_to_arg(value: tree_sitter::Node<'_>, content: &str) -> Option<Arg> {
    match value.kind() {
        "string" => {
            if let Some(child) = value.child(0) {
                ts_to_arg(child, content)
            } else {
                None
            }
        }
        "single_quoted_string" => {
            let s = &content[value.start_byte() + 1..value.end_byte() - 1];
            Some(Arg::String(
                unescaper::unescape(s).unwrap_or_else(|e| {
                    tracing::error!(source = "templ_parser", "{}", e);
                    s.to_string()
                }),
                Quotes::Single,
            ))
        }
        "double_quoted_string" => {
            let s = &content[value.start_byte() + 1..value.end_byte() - 1];
            Some(Arg::String(
                unescaper::unescape(s).unwrap_or_else(|e| {
                    tracing::error!(source = "templ_parser", "{}", e);
                    s.to_string()
                }),
                Quotes::Double,
            ))
        }
        "backquoted_quoted_string" => {
            let s = &content[value.start_byte() + 1..value.end_byte() - 1];
            Some(Arg::String(
                unescaper::unescape(s).unwrap_or_else(|e| {
                    tracing::error!(source = "templ_parser", "{}", e);
                    s.to_string()
                }),
                Quotes::Back,
            ))
        }

        "int" => Some(Arg::Int(
            content[value.start_byte()..value.end_byte()]
                .parse()
                .unwrap_or_default(),
        )),
        "float" => Some(Arg::Float(
            content[value.start_byte()..value.end_byte()]
                .parse()
                .unwrap_or_default(),
        )),
        "boolean" => Some(Arg::Bool(
            content[value.start_byte()..value.end_byte()]
                .parse()
                .unwrap_or_default(),
        )),
        _ => None,
    }
}

#[derive(Debug)]
pub enum Token {
    Text(TextToken),
    Macro(MacroToken),
}

pub fn parse(input: &str) -> Result<Vec<Token>, DocError> {
    let mut parser = tree_sitter::Parser::new();
    let language = tree_sitter_mdn::LANGUAGE;
    parser
        .set_language(&language.into())
        .expect("Error loading MDN parser");
    let tree = parser.parse(input, None).unwrap();
    let mut cursor = tree.walk();
    Ok(tree
        .root_node()
        .children(&mut cursor)
        .filter_map(|child| match child.kind() {
            "text" => Some(Token::Text(TextToken {
                start: child.start_byte(),
                end: child.end_byte(),
            })),
            "macro_tag" => from_node(child, input, &mut child.walk()).map(Token::Macro),
            _ => None,
        })
        .collect())
}

#[cfg(test)]
mod test {
    use {tree_sitter, tree_sitter_mdn};

    use super::*;

    #[test]
    fn test_tree_sitter() {
        let md =
            r#"attribute of an `{{HTMLElement("input","&lt;input type=\"file\"&gt;")}}` element"#;
        let mut parser = tree_sitter::Parser::new();
        let language = tree_sitter_mdn::LANGUAGE;
        parser
            .set_language(&language.into())
            .expect("Error loading MDN parser");
        let tree = parser.parse(md, None).unwrap();
        let mut cursor = tree.walk();
        for node in tree.root_node().children(&mut cursor) {
            println!("{}", node.grammar_name());
            println!("{node:?}");
        }
    }

    #[test]
    fn expect_missing_flag() {
        let p = parse(r#"{{_cssxref("foo")}}"#).unwrap();
        match p.first() {
            Some(Token::Macro(m)) => {
                assert_eq!(m.ident, "cssxref");
                assert!(m.expect_missing);
                assert_eq!(m.args.len(), 1);
            }
            _ => panic!("expected macro token"),
        }
        let p = parse(r#"{{cssxref("foo")}}"#).unwrap();
        match p.first() {
            Some(Token::Macro(m)) => assert!(!m.expect_missing),
            _ => panic!("expected macro token"),
        }
    }

    #[test]
    fn with_empty_string_arg() {
        let p = parse(r#"{{foo("")}}"#);
        assert!(matches!(
            p.unwrap().first(),
            Some(Token::Macro(macro_token)) if macro_token.args.first() == Some(&None)
        ));
    }
}
