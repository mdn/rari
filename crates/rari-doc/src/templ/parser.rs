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
    /// 0-based `(row, column)` of the macro start, where `column` is a byte
    /// offset from the start of the line.
    pub pos: (usize, usize),
    /// 0-based exclusive `(row, column)` of the macro end, in the same units
    /// as `pos`.
    pub end_pos: (usize, usize),
    pub args: Vec<Option<Arg>>,
}

fn from_node<'a>(
    value: tree_sitter::Node<'a>,
    content: &str,
    cursor: &mut TreeCursor<'a>,
) -> Option<MacroToken> {
    let ident_node = value.named_child(0).unwrap();
    let ident = content[ident_node.start_byte()..ident_node.end_byte()].to_string();
    let args = if let Some(args_node) = value.named_child(1) {
        args_node
            .named_children(cursor)
            .map(|arg| ts_to_arg(arg, content))
            .collect()
    } else {
        vec![]
    };
    let start = value.start_byte();
    let end = value.end_byte();
    let start_position = value.start_position();
    let pos = (start_position.row, start_position.column);
    let end_position = value.end_position();
    let end_pos = (end_position.row, end_position.column);
    Some(MacroToken {
        start,
        end,
        pos,
        end_pos,
        ident,
        args,
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

    /// Macro positions come from tree-sitter and must cover the whole macro,
    /// including macros that span lines and malformed ones.
    #[test]
    fn test_macro_positions() {
        struct Case {
            name: &'static str,
            input: String,
            pos: (usize, usize),
            end_pos: (usize, usize),
        }

        // A malformed macro that occurred in mdn/content (`userScripts`, since
        // fixed), where a stray comma and quote break the argument list.
        let malformed = r#"{{WebExtAPIRef("userScripts.,"execute()", "execute()"}}"#;
        let cases = vec![
            Case {
                name: "single line",
                input: r#"a {{Compat("api.Foo", 0)}} b"#.to_string(),
                pos: (0, 2),
                end_pos: (0, 26),
            },
            Case {
                name: "spans two lines",
                input: "a {{Compat(\"api.Foo\",\n  0)}} b".to_string(),
                pos: (0, 2),
                end_pos: (1, 6),
            },
            Case {
                name: "malformed macro",
                input: format!("{}{malformed}", "x".repeat(59)),
                pos: (0, 59),
                end_pos: (0, 59 + malformed.len()),
            },
        ];

        for case in cases {
            let tokens = parse(&case.input).expect("parse must succeed");
            let mac = tokens
                .iter()
                .find_map(|token| match token {
                    Token::Macro(mac) => Some(mac),
                    Token::Text(_) => None,
                })
                .unwrap_or_else(|| panic!("{}: expected a macro token", case.name));
            assert_eq!(mac.pos, case.pos, "{}: pos", case.name);
            assert_eq!(mac.end_pos, case.end_pos, "{}: end_pos", case.name);
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
