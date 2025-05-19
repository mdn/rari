use pest::iterators::Pair;
use pest::Parser;
use rari_types::{Arg, Quotes};
use tree_sitter::TreeCursor;

use crate::error::DocError;

#[derive(pest_derive::Parser)]
#[grammar = "templ/rari-templ.pest"]
pub struct RariTempl;

#[derive(Debug)]
pub struct TextToken {
    pub start: usize,
    pub end: usize,
}

impl From<Pair<'_, Rule>> for TextToken {
    fn from(pair: Pair<'_, Rule>) -> Self {
        Self {
            start: pair.as_span().start(),
            end: pair.as_span().end(),
        }
    }
}

#[derive(Debug)]
pub struct MacroToken {
    pub start: usize,
    pub end: usize,
    pub ident: String,
    pub pos: (usize, usize),
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
    Some(MacroToken {
        start,
        end,
        pos,
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
impl From<Pair<'_, Rule>> for MacroToken {
    fn from(pair: Pair<'_, Rule>) -> Self {
        let start = pair.as_span().start();
        let end = pair.as_span().end();
        let pos = pair.line_col();
        let m = pair.into_inner().next().unwrap();
        let (ident, args) = match m.as_rule() {
            Rule::fn_call => {
                let mut inner = m.into_inner();
                let ident = inner.next().unwrap().as_span().as_str().to_string();
                let args = inner
                    .next()
                    .map(|args| args.into_inner().map(to_arg).collect::<Vec<_>>())
                    .unwrap_or_default();
                (ident, args)
            }
            Rule::ident => {
                let ident = m.as_span().as_str().to_string();
                (ident, vec![])
            }
            _ => ("noop".to_string(), vec![]),
        };

        Self {
            start,
            end,
            pos,
            ident,
            args,
        }
    }
}

#[derive(Debug)]
pub enum Token {
    Text(TextToken),
    Macro(MacroToken),
}

fn to_arg(pair: Pair<'_, Rule>) -> Option<Arg> {
    match pair.as_rule() {
        Rule::single_quoted_string => pair.into_inner().next().and_then(to_arg),
        Rule::double_quoted_string => pair.into_inner().next().and_then(to_arg),
        Rule::backquoted_quoted_string => pair.into_inner().next().and_then(to_arg),
        Rule::empty_string => None,
        Rule::sq_string => {
            let s = pair.as_span().as_str();
            Some(Arg::String(
                unescaper::unescape(s).unwrap_or_else(|e| {
                    tracing::error!(source = "templ_parser", "{}", e);
                    s.to_string()
                }),
                Quotes::Single,
            ))
        }
        Rule::dq_string => {
            let s = pair.as_span().as_str();
            Some(Arg::String(
                unescaper::unescape(s).unwrap_or_else(|e| {
                    tracing::error!(source = "templ_parser", "{}", e);
                    s.to_string()
                }),
                Quotes::Double,
            ))
        }
        Rule::bq_string => {
            let s = pair.as_span().as_str();
            Some(Arg::String(
                unescaper::unescape(s).unwrap_or_else(|e| {
                    tracing::error!(source = "templ_parser", "{}", e);
                    s.to_string()
                }),
                Quotes::Back,
            ))
        }

        Rule::int => Some(Arg::Int(
            pair.as_span().as_str().parse().unwrap_or_default(),
        )),
        Rule::float => Some(Arg::Float(
            pair.as_span().as_str().parse().unwrap_or_default(),
        )),
        Rule::boolean => Some(Arg::Bool(
            pair.as_span().as_str().parse().unwrap_or_default(),
        )),
        _ => None,
    }
}

pub fn parse(input: &str) -> Result<Vec<Token>, DocError> {
    let mut p =
        RariTempl::parse(Rule::doc, input).map_err(|e| DocError::PestError(e.to_string()))?;
    let tokens = p
        .next()
        .unwrap()
        .into_inner()
        .filter_map(|t| match t.as_rule() {
            Rule::text => Some(Token::Text(t.into())),
            Rule::macro_tag => Some(Token::Macro(t.into())),
            _ => None,
        })
        .collect();
    Ok(tokens)
}

pub fn parse_ts(input: &str) -> Result<Vec<Token>, DocError> {
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
    fn basic() {
        let p = RariTempl::parse(Rule::doc, r#"Foo {{jsxref("Array") }}bar {{ foo }}"#);
        println!("{:#?}", p);
    }

    #[test]
    fn custom() {
        let p = parse_ts(r#"Foo \{{jsxref("Array",,1,true, ' ') }}bar {{ foo }}"#);
        println!("{:#?}", p);
    }

    #[test]
    fn weird() {
        let p = parse(
            r#"attribute of an `{{HTMLElement("input","&lt;input type=\"file\"&gt;")}}` element"#,
        );
        println!("{:#?}", p);
    }

    #[test]
    fn weird2() {
        let p = parse(r#"dasd \{{foo}} 200 {{bar}}"#);
        println!("{:#?}", p);
    }

    #[test]
    fn weird3() {
        let p = parse(r#"foo {{foo(0.1)}} bar"#);
        println!("{:#?}", p);
    }

    #[test]
    fn weird4() {
        let p = parse(r#"dasd \\{{foo}} 200 {{bar}}"#);
        println!("{:#?}", p);
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
