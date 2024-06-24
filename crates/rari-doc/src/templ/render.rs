use std::fmt::Write;

use rari_types::globals::deny_warnings;
use rari_types::RariEnv;
use tracing::{error, span, Level};

use super::macros::invoke;
use super::parser::{parse, Token};
use crate::error::DocError;

pub fn render(env: &RariEnv, input: &str) -> Result<String, DocError> {
    let tokens = parse(input)?;
    render_tokens(env, tokens, input)
}

pub fn render_tokens(env: &RariEnv, tokens: Vec<Token>, input: &str) -> Result<String, DocError> {
    let mut out = String::with_capacity(input.len());
    for token in tokens {
        match token {
            Token::Text(text) => {
                let slice = &input[text.start..text.end];
                let mut last = 0;
                for (i, _) in slice.match_indices("\\{{") {
                    out.push_str(&slice[last..i]);
                    last = i + 1;
                }
                out.push_str(&slice[last..])
            }
            Token::Macro(mac) => {
                let ident = &mac.ident;
                let span = span!(Level::ERROR, "templ", "{}", &ident);
                let _enter = span.enter();
                match invoke(env, &mac.ident, mac.args) {
                    Ok(rendered) => out.push_str(&rendered),
                    Err(e) if deny_warnings() => return Err(e),
                    Err(e) => {
                        error!("{e}");
                        writeln!(&mut out, "{e}")?;
                    }
                };
            }
        }
    }
    Ok(out)
}
