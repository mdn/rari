use std::fmt::Write;

use rari_md::ext::{DELIM_END, DELIM_END_LEN, DELIM_START, DELIM_START_LEN};
use rari_types::globals::deny_warnings;
use rari_types::templ::TemplType;
use rari_types::{AnyArg, RariEnv};
use tracing::{Level, span, warn};

use super::parser::{Token, parse};
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
                        .get(1)
                        .or(mac.args.first())
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
                // tree-sitter positions are 0-based; add 1 to report 1-based
                // positions, consistent with comrak's sourcepos (see `fix_link`).
                let line = i64::try_from(mac.pos.0 + offset)
                    .map(|l| l + 1)
                    .unwrap_or(-1);
                // mac.pos.1 is a 0-based byte column from tree-sitter.
                let col = i64::try_from(mac.pos.1).map(|c| c + 1).unwrap_or(-1);
                // end_col in bytes: start byte + macro byte length. As a 0-based
                // exclusive end this already equals the 1-based inclusive end column.
                let macro_byte_len = mac.end - mac.start;
                let end_col = i64::try_from(mac.pos.1 + macro_byte_len).unwrap_or(-1);
                let span = span!(
                    Level::ERROR,
                    "templ",
                    templ = name,
                    line = line,
                    col = col,
                    end_line = line,
                    end_col = end_col
                );
                let _enter = span.enter();
                match invoke(env, &name, mac.args) {
                    Ok((rendered, TemplType::Sidebar)) => {
                        encode_ref(templs.len(), &mut out, mac.end - mac.start)?;
                        templs.push(String::default());
                        sidebars.push(rendered);
                    }
                    Ok((rendered, _)) => {
                        encode_ref(templs.len(), &mut out, mac.end - mac.start)?;
                        templs.push(rendered);
                    }
                    Err(e) if deny_warnings() => return Err(e),
                    Err(e) => {
                        warn!("{e}",);
                        encode_ref(templs.len(), &mut out, mac.end - mac.start)?;
                        //templs.push(format!("___ERROR in ({ident}): {e}___"));
                        templs.push(e.to_string())
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

fn encode_ref(index: usize, out: &mut String, len: usize) -> Result<(), DocError> {
    let padding = len - DELIM_START_LEN - DELIM_END_LEN;
    Ok(write!(out, "{DELIM_START}{index:x>padding$}{DELIM_END}",)?)
}

pub(crate) fn render_and_decode_ref(env: &RariEnv, input: &str) -> Result<String, DocError> {
    let Rendered {
        content, templs, ..
    } = render(env, input, 0)?;
    decode_ref(&content, &templs, None)
}

pub(crate) fn decode_ref(
    input: &str,
    templs: &[String],
    prepred: Option<&[String]>,
) -> Result<String, DocError> {
    let mut decoded = String::with_capacity(input.len());
    if let Some(prepend) = prepred {
        decoded.extend(prepend.iter().map(String::as_str));
    }
    if !input.contains(DELIM_START) {
        return Ok(input.to_string());
    }
    let mut frags = vec![];
    for frag in input.split(DELIM_START) {
        let has_ks = frag.contains(DELIM_END);
        for (i, sub_frag) in frag.splitn(2, DELIM_END).enumerate() {
            if i == 0 && has_ks {
                frags.push(sub_frag);
            } else {
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
            let index = frag.trim_start_matches("x").parse::<usize>()?;
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
        push_text_inner(out, &slice[last..i]);
        last = i + 1;
    }
    push_text_inner(out, &slice[last..]);
}

fn push_text_inner(out: &mut String, slice: &str) {
    let mut last = 0;
    for (i, _) in slice.match_indices("\\}") {
        out.push_str(&slice[last..i]);
        last = i + 1;
    }
    out.push_str(&slice[last..]);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_push_text() {
        let mut out = String::new();
        push_text(&mut out, "foo \\\\{ bar \\\\} \\} 2000");
        assert_eq!(out, "foo \\{ bar \\} } 2000")
    }

    #[test]
    fn test_basic() -> Result<(), DocError> {
        let env = RariEnv {
            ..Default::default()
        };
        let Rendered {
            content, templs, ..
        } = render(&env, r#"{{ echo("doom") }}"#, 0)?;
        let out = decode_ref(&content, &templs, None)?;
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
        let out = decode_ref(&content, &templs, None)?;
        assert_eq!(out, r#""doom""#);
        Ok(())
    }

    /// Positions emitted by `render` must be 1-based, matching comrak's
    /// sourcepos (tree-sitter reports 0-based). The macro sits after a text
    /// prefix so its column is non-zero, and its invalid `sandbox` argument
    /// makes `EmbedLiveSample` emit a `templ-invalid-arg` warning through a
    /// pure code path (no content/link resolution required).
    #[test]
    fn test_render_reports_1_based_positions() {
        use tracing::subscriber::set_default;
        use tracing_subscriber::layer::SubscriberExt;

        use crate::issues::InMemoryLayer;

        let layer = InMemoryLayer::default();
        let subscriber = tracing_subscriber::registry().with(layer.clone());
        let _guard = set_default(subscriber);

        let env = RariEnv {
            ..Default::default()
        };
        let prefix = "abc ";
        let mac = r#"{{EmbedLiveSample("x", 100, 100, "", "", "", "", "nope")}}"#;
        render(&env, &format!("{prefix}{mac}"), 0).expect("render should succeed");

        let events = layer.get_events();
        let issues = events.get("").expect("expected an emitted issue");
        assert_eq!(issues.len(), 1);
        let issue = &issues[0];
        // tree-sitter row 0 (+ offset 0), reported 1-based.
        assert_eq!(issue.line, 1);
        assert_eq!(issue.end_line, 1);
        // 0-based byte column `prefix.len()` -> 1-based start column.
        assert_eq!(issue.col, prefix.len() as i64 + 1);
        // Inclusive 1-based end column == start byte + macro byte length.
        assert_eq!(issue.end_col, (prefix.len() + mac.len()) as i64);
    }
}
