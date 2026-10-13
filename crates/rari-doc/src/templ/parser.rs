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
    /// 0-based `(row, byte column)` of the macro start.
    pub pos: (usize, usize),
    /// 0-based exclusive `(row, byte column)` of the macro end.
    pub end_pos: (usize, usize),
    pub args: Vec<Option<Arg>>,
    pub malformed: bool,
    pub allow_broken_links: bool,
}

fn from_node<'a>(
    value: tree_sitter::Node<'a>,
    content: &str,
    cursor: &mut TreeCursor<'a>,
) -> Option<MacroToken> {
    let ident_node = value.named_child(0).unwrap();
    let ident = content[ident_node.start_byte()..ident_node.end_byte()].to_string();
    // Look the arguments up by kind: a syntax error inserts an `ERROR` node,
    // which would otherwise be mistaken for them.
    let args = if let Some(args_node) = value
        .named_children(&mut value.walk())
        .find(|child| child.kind() == "args")
    {
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
        malformed: value.has_error(),
        allow_broken_links: content.as_bytes().get(ident_node.end_byte()) == Some(&b'!'),
    })
}

/// Returns `None` for an unparseable argument (a tree-sitter `ERROR` node), but
/// an empty [`Arg::String`] for a blank one (`""` or `{{foo(,"bar")}}`).
fn ts_to_arg(value: tree_sitter::Node<'_>, content: &str) -> Option<Arg> {
    match value.kind() {
        "none" => Some(Arg::String(
            String::new(),
            match content[value.start_byte()..].chars().next() {
                Some('\'') => Quotes::Single,
                Some('`') => Quotes::Back,
                _ => Quotes::Double,
            },
        )),
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
    let mut tree = parser.parse(input, None).unwrap();
    let mut normalized = None;
    for node in tree.root_node().children(&mut tree.walk()) {
        if node.kind() == "macro_tag"
            && let Some(ident) = node.named_child(0)
            && ident.kind() == "ident"
            && input.as_bytes().get(ident.end_byte()) == Some(&b'!')
        {
            // Let the existing grammar parse markers without changing source offsets.
            normalized.get_or_insert_with(|| input.as_bytes().to_vec())[ident.end_byte()] = b' ';
        }
    }
    if let Some(normalized) = normalized {
        tree = parser.parse(&normalized, None).unwrap();
    }
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
    fn test_macro_positions() {
        struct Case {
            name: &'static str,
            input: String,
            pos: (usize, usize),
            end_pos: (usize, usize),
        }

        // A malformed macro from mdn/content, where a stray comma breaks the arguments.
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
    fn arg_shapes() {
        let blank = Some(Arg::String(String::new(), Quotes::Double));
        let cases: Vec<(&str, &str, Vec<Option<Arg>>)> = vec![
            (
                "empty string literal",
                r#"{{foo("")}}"#,
                vec![blank.clone()],
            ),
            (
                "empty string literal among others",
                r#"{{foo('', 'CSS')}}"#,
                vec![
                    Some(Arg::String(String::new(), Quotes::Single)),
                    Some(Arg::String("CSS".into(), Quotes::Single)),
                ],
            ),
            (
                "omitted argument",
                r#"{{foo(,"CSS")}}"#,
                vec![
                    blank.clone(),
                    Some(Arg::String("CSS".into(), Quotes::Double)),
                ],
            ),
            ("no parentheses", "{{foo}}", vec![]),
            ("empty parentheses", "{{foo()}}", vec![]),
            (
                "unparseable argument",
                "{{foo(\u{300c}label\u{300d})}}",
                vec![None],
            ),
        ];
        for (name, input, expected) in cases {
            let tokens = parse(input).unwrap();
            let args = match tokens.first() {
                Some(Token::Macro(m)) => m.args.clone(),
                other => panic!("{name}: expected a macro token, got {other:?}"),
            };
            assert_eq!(args, expected, "{name}");
        }
    }

    #[test]
    fn malformed_macros() {
        let cases: Vec<(&str, &str, bool, Vec<Option<Arg>>)> = vec![
            (
                "well-formed",
                r#"{{cssxref("color")}}"#,
                false,
                vec![Some(Arg::String("color".into(), Quotes::Double))],
            ),
            (
                "unparseable argument",
                "{{htmlelement(\u{300c}label\u{300d})}}",
                true,
                vec![None],
            ),
            (
                // The stray `(` inserts an `ERROR` node before the arguments,
                // which are still recovered by looking them up by kind.
                "stray opening parenthesis",
                r#"{{cssxref(("color")}}"#,
                true,
                vec![Some(Arg::String("color".into(), Quotes::Double))],
            ),
            (
                "unbalanced quotes",
                r#"{{WebExtAPIRef("userScripts.,"execute()", "execute()"}}"#,
                true,
                vec![],
            ),
        ];
        for (name, input, malformed, expected_args) in cases {
            let tokens = parse(input).unwrap();
            let m = match tokens.first() {
                Some(Token::Macro(m)) => m,
                other => panic!("{name}: expected a macro token, got {other:?}"),
            };
            assert_eq!(m.malformed, malformed, "{name}: malformed");
            assert_eq!(m.args, expected_args, "{name}: args");
        }
    }
}

#[cfg(test)]
mod marker_tests {
    use super::*;

    #[test]
    fn parses_marked_macro_names() {
        struct Case {
            name: &'static str,
            input: &'static str,
            marked: bool,
            malformed: bool,
        }
        let cases = [
            Case {
                name: "marked arguments",
                input: r#"{{domxref!("BrandNewInterface")}}"#,
                marked: true,
                malformed: false,
            },
            Case {
                name: "marked no arguments",
                input: "{{foo!}}",
                marked: true,
                malformed: false,
            },
            Case {
                name: "whitespace",
                input: r#"{{ DOMxref! ("BrandNewInterface") }}"#,
                marked: true,
                malformed: false,
            },
            Case {
                name: "multiline",
                input: "{{domxref!(\n  \"BrandNewInterface\")}}",
                marked: true,
                malformed: false,
            },
            Case {
                name: "literal exclamation",
                input: r#"{{echo("!")}}"#,
                marked: false,
                malformed: false,
            },
            Case {
                name: "literal macro",
                input: r#"{{echo("{{foo!(x)}}")}}"#,
                marked: false,
                malformed: false,
            },
            Case {
                name: "double marker",
                input: r#"{{domxref!!("Foo")}}"#,
                marked: true,
                malformed: true,
            },
            Case {
                name: "invalid arguments",
                input: r#"{{domxref!(("Foo")}}"#,
                marked: true,
                malformed: true,
            },
        ];
        for case in cases {
            let tokens = parse(case.input).unwrap();
            let Token::Macro(mac) = &tokens[0] else {
                panic!("{}: expected macro", case.name)
            };
            assert_eq!(mac.allow_broken_links, case.marked, "{}", case.name);
            assert_eq!(mac.malformed, case.malformed, "{}", case.name);
            assert_eq!(mac.end, case.input.len(), "{}: end", case.name);
        }
    }
}
