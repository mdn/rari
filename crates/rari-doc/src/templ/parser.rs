use std::fmt::Write;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
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

fn encode_macro(s: &str, out: &mut String) -> Result<(), DocError> {
    Ok(write!(
        out,
        "!::::{}::::!",
        STANDARD.encode(&s[2..(s.len() - 2)])
    )?)
}

fn decode_macro(s: &str, out: &mut String) -> Result<(), DocError> {
    Ok(write!(
        out,
        "{{{{{}}}}}",
        std::str::from_utf8(&STANDARD.decode(s)?)?
    )?)
}

pub(crate) fn encode_ks(input: &str) -> Result<(String, i32), DocError> {
    let tokens = parse(input)?;
    let mut encoded = String::with_capacity(input.len());
    let mut num_macros = 0;
    for token in tokens {
        match token {
            Token::Text(t) => {
                encoded.push_str(&input[t.start..t.end]);
            }
            Token::Macro(t) => {
                num_macros += 1;
                encode_macro(&input[t.start..t.end], &mut encoded)?
            }
        }
    }
    Ok((encoded, num_macros))
}

fn _strip_escape_residues(s: &str) -> &str {
    let s = s.strip_prefix("&gt;").or(s.strip_prefix('>')).unwrap_or(s);
    let s = s
        .strip_suffix("!&lt;")
        .or(s.strip_suffix("!<"))
        .unwrap_or(s);
    s
}

pub(crate) fn decode_ks(input: &str) -> Result<(String, i32), DocError> {
    let mut decoded = String::with_capacity(input.len());
    let mut num_macros = 0;
    // We're splitting only by `!-- ks___` because e.g. ks in a <pre> will be escaped.
    if !input.contains("!::::") {
        return Ok((input.to_string(), 0));
    }
    let mut frags = vec![];
    for frag in input.split("!::::") {
        let has_ks = frag.contains("::::!");
        for (i, sub_frag) in frag.splitn(2, "::::!").enumerate() {
            if i == 0 && has_ks {
                num_macros += 1;
                frags.push(sub_frag);
                //decode_macro(sub_frag, &mut decoded)?;
            } else {
                //decoded.push_str(strip_escape_residues(sub_frag))
                frags.push(sub_frag)
            }
        }
    }
    for i in 0..frags.len() {
        if i % 2 == 1
            && i < frags.len() + 1
            && frags[i - 1].ends_with("<p>")
            && frags[i + 1].starts_with("</p>")
        {
            frags[i - 1] = frags[i - 1].strip_suffix("<p>").unwrap();
            frags[i + 1] = frags[i + 1].strip_prefix("</p>").unwrap();
        }
    }

    for (i, frag) in frags.iter().enumerate() {
        if i % 2 == 1 {
            decode_macro(frag, &mut decoded)?;
        } else {
            decoded.push_str(frag)
        }
    }

    Ok((decoded, num_macros))
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

    #[test]
    fn ks_escape() -> Result<(), DocError> {
        let ks = r#"Foo {{jsxref("Array",,1,true) }}bar {{ foo }}"#;
        let enc = encode_ks(ks)?;
        let dec = decode_ks(&enc.0)?;
        assert_eq!(ks, dec.0);
        Ok(())
    }

    #[test]
    fn ks_escape_2() -> Result<(), DocError> {
        let ks = r#"<{{jsxref("Array",,1,true) }}>bar {{ foo }}"#;
        let enc = encode_ks(ks)?;
        let dec = decode_ks(&enc.0)?;
        assert_eq!(ks, dec.0);
        Ok(())
    }

    #[test]
    fn ks_escape_3() -> Result<(), DocError> {
        let ks = r#"{{foo}}{{foo-bar}}"#;
        let enc = encode_ks(ks)?;
        let dec = decode_ks(&enc.0)?;
        assert_eq!(ks, dec.0);
        Ok(())
    }
}
