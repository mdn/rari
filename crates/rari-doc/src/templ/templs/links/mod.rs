use rari_types::ArgError;

use crate::issues::get_issue_counter;

pub mod csp;
pub mod cssxref;
pub mod domxref;
pub mod htmlxref;
pub mod http;
pub mod jsxref;
pub mod mathmlxref;
pub mod rfc;
pub mod svgattr;
pub mod svgxref;
pub mod webextapixref;

/// Returns the fragment of a link macro's `anchor` argument (3rd position),
/// without its leading `#`, or `None` if there is none.
///
/// An anchor not starting with exactly one `#` is reported as a
/// `templ-arg-error`, but its fragment is still used so the link works.
pub(crate) fn anchor_fragment<'a>(templ: &'static str, anchor: Option<&'a str>) -> Option<&'a str> {
    let anchor = anchor.filter(|a| !a.is_empty())?;
    let fragment = anchor.trim_start_matches('#');
    if anchor.len() - fragment.len() != 1 {
        let ic = get_issue_counter();
        let e = ArgError::at(templ, 3, "anchor", ArgError::MustStartWithSingleHash);
        tracing::warn!(source = "templ-arg-error", ic = ic, "Macro {e}");
    }
    Some(fragment).filter(|f| !f.is_empty())
}

#[cfg(test)]
mod tests {
    use rari_types::RariEnv;
    use tracing::subscriber::set_default;
    use tracing_subscriber::layer::SubscriberExt;

    use super::*;
    use crate::issues::InMemoryLayer;
    use crate::templ::render::{decode_ref, render};

    /// Runs `f`, returning its result and the messages of the `templ-arg-error`s it emitted.
    fn with_arg_errors<T>(f: impl FnOnce() -> T) -> (T, Vec<String>) {
        let layer = InMemoryLayer::default();
        let subscriber = tracing_subscriber::registry().with(layer.clone());
        let guard = set_default(subscriber);
        let out = f();
        drop(guard);
        let messages = layer
            .get_events()
            .iter()
            .flat_map(|e| e.value().clone())
            .filter(|i| {
                i.fields
                    .contains(&("source", "templ-arg-error".to_string()))
            })
            .filter_map(|i| {
                i.fields
                    .iter()
                    .find(|(k, _)| *k == "message")
                    .map(|(_, v)| v.clone())
            })
            .collect();
        (out, messages)
    }

    #[test]
    fn anchor_fragment_cases() {
        struct Case {
            name: &'static str,
            anchor: Option<&'static str>,
            fragment: Option<&'static str>,
            error: bool,
        }

        let cases = vec![
            Case {
                name: "single hash",
                anchor: Some("#x"),
                fragment: Some("x"),
                error: false,
            },
            Case {
                name: "bare",
                anchor: Some("x"),
                fragment: Some("x"),
                error: true,
            },
            Case {
                name: "double hash",
                anchor: Some("##x"),
                fragment: Some("x"),
                error: true,
            },
            Case {
                name: "empty",
                anchor: Some(""),
                fragment: None,
                error: false,
            },
            Case {
                name: "none",
                anchor: None,
                fragment: None,
                error: false,
            },
        ];

        for c in cases {
            let (fragment, errors) = with_arg_errors(|| anchor_fragment("cssxref", c.anchor));
            assert_eq!(fragment, c.fragment, "fragment [{}]", c.name);
            let expected: Vec<String> = if c.error {
                vec!["Macro cssxref argument 3 (anchor) must start with a single `#`".to_string()]
            } else {
                vec![]
            };
            assert_eq!(errors, expected, "errors [{}]", c.name);
        }
    }

    /// Only covers macros whose URL doesn't depend on a content index.
    #[test]
    fn link_macro_anchor_cases() {
        struct Case {
            name: &'static str,
            input: &'static str,
            output: &'static str,
            error: Option<&'static str>,
        }

        let cases = vec![
            Case {
                name: "HTTPStatus single hash",
                input: r###"{{HTTPStatus("500", "", "#syntax")}}"###,
                output: r#"<a data-templ-link href="/en-US/docs/Web/HTTP/Reference/Status/500#syntax"><code>500.syntax</code></a>"#,
                error: None,
            },
            Case {
                name: "HTTPHeader bare",
                input: r#"{{HTTPHeader("Cache-Control", "", "syntax")}}"#,
                output: r#"<a data-templ-link href="/en-US/docs/Web/HTTP/Reference/Headers/Cache-Control#syntax"><code>Cache-Control.syntax</code></a>"#,
                error: Some("httpheader"),
            },
            Case {
                name: "HTTPMethod double hash",
                input: r###"{{HTTPMethod("PUT", "", "##syntax")}}"###,
                output: r#"<a data-templ-link href="/en-US/docs/Web/HTTP/Reference/Methods/PUT#syntax"><code>PUT.syntax</code></a>"#,
                error: Some("httpmethod"),
            },
            Case {
                name: "HTMLElement bare",
                input: r#"{{HTMLElement("form", "", "attributes")}}"#,
                output: r#"<a data-templ-link href="/en-US/docs/Web/HTML/Reference/Elements/form#attributes"><code>&lt;form&gt;</code></a>"#,
                error: Some("htmlelement"),
            },
            Case {
                name: "WebExtAPIRef single hash",
                input: r###"{{WebExtAPIRef("storage.local", "", "#get")}}"###,
                output: r#"<a data-templ-link href="/en-US/docs/Mozilla/Add-ons/WebExtensions/API/storage/local#get"><code>storage.local#get</code></a>"#,
                error: None,
            },
        ];

        for c in cases {
            let (out, errors) = with_arg_errors(|| {
                let env = RariEnv {
                    ..Default::default()
                };
                let rendered = render(&env, c.input, 0).expect("render should succeed");
                decode_ref(&rendered.content, &rendered.templs, None)
                    .expect("decode should succeed")
            });
            assert_eq!(out, c.output, "output [{}]", c.name);
            let expected: Vec<String> = c
                .error
                .map(|templ| {
                    format!("Macro {templ} argument 3 (anchor) must start with a single `#`")
                })
                .into_iter()
                .collect();
            assert_eq!(errors, expected, "errors [{}]", c.name);
        }
    }
}
