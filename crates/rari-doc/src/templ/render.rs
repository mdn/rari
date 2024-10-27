use std::fmt::Write;

use rari_md::ext::{DELIM_END, DELIM_START};
use rari_types::globals::deny_warnings;
use rari_types::{AnyArg, RariEnv};
use tracing::{span, warn, Level};

use super::parser::{parse, Token};
use super::templs::invoke;
use crate::error::DocError;

pub struct Rendered {
    pub content: String,
    pub templs: Vec<String>,
    pub sidebars: Vec<String>,
}

pub(crate) fn render_for_summary(input: &str) -> Result<String, DocError> {
    let tokens = parse(input)?;
    let mut out = String::with_capacity(input.len());
    for token in tokens {
        match token {
            Token::Text(text) => {
                let slice = &input[text.start..text.end];
                push_text(&mut out, slice);
            }
            Token::Macro(mac) => {
                if let Some(s) = match mac.ident.to_ascii_lowercase().as_str() {
                    "apiref"
                    | "jsref"
                    | "compat"
                    | "page"
                    | "deprecated_header"
                    | "previous"
                    | "previousmenu"
                    | "previousnext"
                    | "previousmenunext"
                    | "quicklinkswithsubpages" => None,
                    "glossary" => mac
                        .args
                        .first()
                        .and_then(|f| f.clone())
                        .map(|arg| AnyArg::try_from(arg).unwrap().to_string()),
                    _ => mac
                        .args
                        .first()
                        .and_then(|f| f.clone())
                        .map(|arg| format!("<code>{}</code>", AnyArg::try_from(arg).unwrap())),
                } {
                    out.push_str(&s)
                }
            }
        }
    }
    Ok(out)
}

pub(crate) fn render(env: &RariEnv, input: &str, offset: usize) -> Result<Rendered, DocError> {
    let tokens = parse(input)?;
    let mut templs = vec![];
    let mut sidebars = vec![];
    let mut out = String::with_capacity(input.len());
    for token in tokens {
        match token {
            Token::Text(text) => {
                let slice = &input[text.start..text.end];
                push_text(&mut out, slice);
            }
            Token::Macro(mac) => {
                let ident = &mac.ident;
                let name = ident.to_ascii_lowercase();
                let span = span!(
                    Level::ERROR,
                    "templ",
                    templ = name,
                    line = mac.pos.0 + offset,
                    col = mac.pos.1
                );
                let _enter = span.enter();
                match invoke(env, &name, mac.args) {
                    Ok((rendered, is_sidebar)) => {
                        if is_sidebar {
                            sidebars.push(rendered)
                        } else {
                            encode_ref(templs.len(), &mut out)?;
                            templs.push(rendered)
                        }
                    }
                    Err(e) if deny_warnings() => return Err(e),
                    Err(e) => {
                        warn!("{e}",);
                        writeln!(&mut out, "{e}")?;
                    }
                };
            }
        }
    }
    Ok(Rendered {
        content: out,
        templs,
        sidebars,
    })
}

fn encode_ref(index: usize, out: &mut String) -> Result<(), DocError> {
    Ok(write!(out, "{DELIM_START}{index}{DELIM_END}",)?)
}

pub(crate) fn render_and_decode_ref(env: &RariEnv, input: &str) -> Result<String, DocError> {
    let Rendered {
        content, templs, ..
    } = render(env, input, 0)?;
    decode_ref(&content, &templs)
}

pub(crate) fn decode_ref(input: &str, templs: &[String]) -> Result<String, DocError> {
    let mut decoded = String::with_capacity(input.len());
    if !input.contains(DELIM_START) {
        return Ok(input.to_string());
    }
    let mut frags = vec![];
    for frag in input.split(DELIM_START) {
        let has_ks = frag.contains(DELIM_END);
        for (i, sub_frag) in frag.splitn(2, DELIM_END).enumerate() {
            if i == 0 && has_ks {
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
            let index = frag.parse::<usize>()?;
            if let Some(templ) = templs.get(index) {
                decoded.push_str(templ);
            } else {
                return Err(DocError::InvalidTemplIndex(index));
            };
        } else {
            decoded.push_str(frag)
        }
    }

    Ok(decoded)
}

fn push_text(out: &mut String, slice: &str) {
    let mut last = 0;
    for (i, _) in slice.match_indices("\\{") {
        out.push_str(&slice[last..i]);
        last = i + 1;
    }
    out.push_str(&slice[last..]);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic() -> Result<(), DocError> {
        let env = RariEnv {
            ..Default::default()
        };
        let Rendered {
            content, templs, ..
        } = render(&env, r#"{{ echo("doom") }}"#, 0)?;
        let out = decode_ref(&content, &templs)?;
        assert_eq!(out, r#"doom"#);
        Ok(())
    }

    #[test]
    fn test_escape() -> Result<(), DocError> {
        let env = RariEnv {
            ..Default::default()
        };
        let Rendered {
            content, templs, ..
        } = render(&env, r#"{{ echo("\"doom\"") }}"#, 0)?;
        let out = decode_ref(&content, &templs)?;
        assert_eq!(out, r#""doom""#);
        Ok(())
    }
}
