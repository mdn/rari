use pest::iterators::Pair;
use pest::Parser;
use rari_types::{Arg, Quotes};

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
    pub args: Vec<Option<Arg>>,
}

impl From<Pair<'_, Rule>> for MacroToken {
    fn from(pair: Pair<'_, Rule>) -> Self {
        let start = pair.as_span().start();
        let end = pair.as_span().end();
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
        Rule::sq_string => Some(Arg::String(
            pair.as_span().as_str().to_string(),
            Quotes::Single,
        )),
        Rule::dq_string => Some(Arg::String(
            pair.as_span().as_str().to_string(),
            Quotes::Double,
        )),
        Rule::bq_string => Some(Arg::String(
            pair.as_span().as_str().to_string(),
            Quotes::Back,
        )),
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

pub(crate) fn parse(input: &str) -> Result<Vec<Token>, DocError> {
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

fn _strip_escape_residues(s: &str) -> &str {
    let s = s.strip_prefix("&gt;").or(s.strip_prefix('>')).unwrap_or(s);
    let s = s
        .strip_suffix("!&lt;")
        .or(s.strip_suffix("!<"))
        .unwrap_or(s);
    s
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let p = RariTempl::parse(Rule::doc, r#"Foo {{jsxref("Array") }}bar {{ foo }}"#);
        println!("{:#?}", p);
    }

    #[test]
    fn custom() {
        let p = parse(r#"Foo {{jsxref("Array",,1,true) }}bar {{ foo }}"#);
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
}
