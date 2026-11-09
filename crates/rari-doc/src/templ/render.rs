use std::fmt::Write;

use rari_md::ext::{DELIM_END, DELIM_END_LEN, DELIM_START, DELIM_START_LEN};
use rari_types::globals::deny_warnings;
use rari_types::templ::TemplType;
use rari_types::{AnyArg, ArgError, RariEnv};
use tracing::{Level, span, warn};

use super::parser::{Token, parse};
use super::templs::{TEMPL_MAPPING, invoke};
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
                let ident = mac.ident.to_ascii_lowercase();
                // Resolve named arguments like a real call would.
                let slots = match TEMPL_MAPPING.get(ident.replace('-', "_").as_str()) {
                    Some(t) => mac
                        .args
                        .bind(t.name, t.params)
                        .map(|slots| slots.into_iter().map(Option::flatten).collect())
                        .unwrap_or_default(),
                    None => mac.args.positional,
                };
                let arg = |i: usize| {
                    slots
                        .get(i)
                        .cloned()
                        .flatten()
                        .filter(|arg| !arg.is_blank())
                };
                if let Some(s) = match ident.as_str() {
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
                    "glossary" => arg(1)
                        .or_else(|| arg(0))
                        .map(|arg| AnyArg::try_from(arg).unwrap().to_string()),
                    _ => {
                        arg(0).map(|arg| format!("<code>{}</code>", AnyArg::try_from(arg).unwrap()))
                    }
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
                let line = i64::try_from(mac.pos.0 + offset + 1).unwrap_or(-1);
                // mac.pos.1 is a 0-based byte column from tree-sitter.
                let col = i64::try_from(mac.pos.1 + 1).unwrap_or(-1);
                let end_line = i64::try_from(mac.end_pos.0 + offset + 1).unwrap_or(-1);
                // A 0-based exclusive byte column already equals the 1-based
                // inclusive end column.
                let end_col = i64::try_from(mac.end_pos.1).unwrap_or(-1);
                let span = span!(
                    Level::ERROR,
                    "templ",
                    templ = name,
                    line = line,
                    col = col,
                    end_line = end_line,
                    end_col = end_col
                );
                let _enter = span.enter();
                if mac.malformed {
                    warn!(
                        source = "templ-syntax-error",
                        "Macro {name} has syntax error"
                    );
                }
                let result = if mac.positional_after_named {
                    Err(ArgError::PositionalAfterNamed {
                        templ: name.clone(),
                    }
                    .into())
                } else {
                    invoke(env, &name, mac.args)
                };
                match result {
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
                        match &e {
                            // A malformed macro already reported a syntax error and its
                            // recovered arguments are unreliable, so don't also complain
                            // about them.
                            DocError::ArgError(_) if mac.malformed => {}
                            DocError::ArgError(_) => warn!(source = "templ-arg-error", "Macro {e}"),
                            _ => warn!("{e}"),
                        }
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
    fn test_render_for_summary() {
        struct Case {
            name: &'static str,
            input: &'static str,
            expected: &'static str,
        }

        let cases = vec![
            Case {
                name: "positional",
                input: r#"a {{cssxref("color")}} b"#,
                expected: "a <code>color</code> b",
            },
            Case {
                name: "named",
                input: r#"a {{cssxref(name="color")}} b"#,
                expected: "a <code>color</code> b",
            },
            Case {
                name: "glossary label",
                input: r#"{{glossary("CSS", "style sheets")}}"#,
                expected: "style sheets",
            },
            Case {
                name: "glossary named label",
                input: r#"{{glossary("CSS", label="style sheets")}}"#,
                expected: "style sheets",
            },
            Case {
                name: "glossary without label",
                input: r#"{{glossary("CSS")}}"#,
                expected: "CSS",
            },
            Case {
                name: "glossary blank label",
                input: r#"{{glossary("CSS", "")}}"#,
                expected: "CSS",
            },
            Case {
                name: "unknown macro",
                input: r#"{{nosuchmacro("x")}}"#,
                expected: "<code>x</code>",
            },
        ];
        for case in cases {
            let actual = render_for_summary(case.input).expect("render should succeed");
            assert_eq!(actual, case.expected, "{}", case.name);
        }
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
    /// sourcepos (tree-sitter reports 0-based), and must cover the whole
    /// macro even when it spans lines. The macro sits after a text prefix so
    /// its column is non-zero, and its invalid `sandbox` argument makes
    /// `EmbedLiveSample` emit a `templ-invalid-arg` warning through a pure
    /// code path (no content/link resolution required).
    #[test]
    fn test_render_reports_macro_positions() {
        use tracing::subscriber::set_default;
        use tracing_subscriber::layer::SubscriberExt;

        use crate::issues::InMemoryLayer;

        struct Case {
            name: &'static str,
            input: String,
            line: i64,
            col: i64,
            end_line: i64,
            end_col: i64,
        }

        let prefix = "abc ";
        let mac = r#"{{EmbedLiveSample("x", 100, 100, "", "", "", "", "nope")}}"#;
        let cases = vec![
            Case {
                name: "single line",
                input: format!("{prefix}{mac}"),
                line: 1,
                col: prefix.len() as i64 + 1,
                end_line: 1,
                // Inclusive 1-based end column == start byte + macro byte length.
                end_col: (prefix.len() + mac.len()) as i64,
            },
            Case {
                name: "spans two lines",
                input: format!(
                    "{prefix}{}",
                    "{{EmbedLiveSample(\"x\", 100, 100, \"\", \"\", \"\",\n  \"\", \"nope\")}} d"
                ),
                line: 1,
                col: prefix.len() as i64 + 1,
                end_line: 2,
                // The second `}` is the 15th byte of `  "", "nope")}} d`.
                end_col: 15,
            },
        ];

        for case in cases {
            let layer = InMemoryLayer::default();
            let subscriber = tracing_subscriber::registry().with(layer.clone());
            let guard = set_default(subscriber);

            let env = RariEnv {
                ..Default::default()
            };
            render(&env, &case.input, 0).expect("render should succeed");
            drop(guard);

            let events = layer.get_events();
            let issues = events
                .get("")
                .unwrap_or_else(|| panic!("{}: expected an emitted issue", case.name));
            assert_eq!(issues.len(), 1, "{}: issue count", case.name);
            let issue = &issues[0];
            assert_eq!(issue.line, case.line, "{}: line", case.name);
            assert_eq!(issue.col, case.col, "{}: col", case.name);
            assert_eq!(issue.end_line, case.end_line, "{}: end_line", case.name);
            assert_eq!(issue.end_col, case.end_col, "{}: end_col", case.name);
        }
    }

    /// An unknown macro name is a `templ-unknown` flaw, and still renders a
    /// placeholder rather than failing.
    #[test]
    fn test_render_reports_unknown_macro() {
        use tracing::subscriber::set_default;
        use tracing_subscriber::layer::SubscriberExt;

        use crate::issues::InMemoryLayer;

        let layer = InMemoryLayer::default();
        let subscriber = tracing_subscriber::registry().with(layer.clone());
        let _guard = set_default(subscriber);

        let env = RariEnv {
            ..Default::default()
        };
        let Rendered {
            content, templs, ..
        } = render(&env, r#"{{domxreg("Element")}}"#, 0).expect("render should succeed");
        let out = decode_ref(&content, &templs, None).expect("decode should succeed");
        assert_eq!(out, "<s>unsupported templ: domxreg</s>");

        let events = layer.get_events();
        let issues = events.get("").expect("expected an emitted issue");
        assert_eq!(issues.len(), 1);
        let fields = &issues[0].fields;
        assert!(
            fields.contains(&("source", "templ-unknown".to_string())),
            "expected a templ-unknown source, got {fields:?}"
        );
        assert!(
            fields.contains(&("templ", "domxreg".to_string())),
            "expected the macro name on the event, got {fields:?}"
        );
    }

    #[test]
    fn test_render_named_args() {
        use tracing::subscriber::set_default;
        use tracing_subscriber::layer::SubscriberExt;

        use crate::issues::InMemoryLayer;

        struct Case {
            name: &'static str,
            input: &'static str,
            output: &'static str,
            arg_errors: usize,
        }

        let cases = vec![
            Case {
                name: "named argument",
                input: r#"{{echo(text="doom")}}"#,
                output: "doom",
                arg_errors: 0,
            },
            Case {
                name: "unknown name",
                input: r#"{{echo(t="doom")}}"#,
                output: "echo has no argument named t",
                arg_errors: 1,
            },
            Case {
                name: "positional after named",
                input: r#"{{echo(text="doom", "gloom")}}"#,
                output: "echo has a positional argument after a named one",
                arg_errors: 1,
            },
        ];

        let env = RariEnv {
            ..Default::default()
        };
        for case in cases {
            let layer = InMemoryLayer::default();
            let subscriber = tracing_subscriber::registry().with(layer.clone());
            let _guard = set_default(subscriber);

            let Rendered {
                content, templs, ..
            } = render(&env, case.input, 0).expect("render should succeed");
            let out = decode_ref(&content, &templs, None).expect("decode should succeed");
            assert_eq!(out, case.output, "{}", case.name);

            let events = layer.get_events();
            let arg_errors = events
                .iter()
                .flat_map(|entry| entry.value().clone())
                .filter(|issue| {
                    issue
                        .fields
                        .contains(&("source", "templ-arg-error".to_string()))
                })
                .count();
            assert_eq!(
                arg_errors, case.arg_errors,
                "{}: templ-arg-error",
                case.name
            );
        }
    }
}
