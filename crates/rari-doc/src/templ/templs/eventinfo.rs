use std::fmt::Write;

use rari_deps::webref_events::{EventIndex, events_index};
use rari_templ_func::rari_f;
use rari_types::fm_types::PageType;
use rari_types::{ArgError, locale::Locale};

use crate::error::DocError;
use crate::helpers::api_inheritance::inheritance;
use crate::helpers::l10n::l10n_json_data;
use crate::helpers::webref_events::{Flag, parse_event_slug, select_flags};
use crate::issues::get_issue_counter;

struct Labels {
    bubbles: &'static str,
    cancelable: &'static str,
    composed: &'static str,
    yes: &'static str,
    no: &'static str,
}

impl Labels {
    fn new(locale: Locale) -> Result<Self, DocError> {
        // Falls back to English until the keys exist in `L10n-Template.json`.
        let label = |key, fallback| l10n_json_data("Template", key, locale).unwrap_or(fallback);
        Ok(Self {
            bubbles: label("bubbles", "Bubbles"),
            cancelable: label("cancelable", "Cancelable"),
            composed: label("composed", "Composed"),
            yes: l10n_json_data("Template", "yes", locale)?,
            no: l10n_json_data("Template", "no", locale)?,
        })
    }
}

fn parse_composed(arg: Option<&str>) -> Result<Option<bool>, ArgError> {
    match arg {
        None => Ok(None),
        Some("composed") => Ok(Some(true)),
        Some(_) => Err(ArgError::at(
            "eventinfo",
            1,
            "composed",
            ArgError::MustBeParsable,
        )),
    }
}

fn row(out: &mut String, label: &str, value: bool, labels: &Labels) {
    let value = if value { labels.yes } else { labels.no };
    let _ = write!(
        out,
        r#"<tr><th scope="row">{label}</th><td>{value}</td></tr>"#
    );
}

fn warn_conflict(name: &str, interface: &str, event_type: &str) {
    tracing::warn!(
        source = "templ-webref-events-conflict",
        ic = get_issue_counter(),
        "@webref/events has conflicting {name} values for {interface} \"{event_type}\""
    );
}

/// `index` is `None` if the webref data could not be loaded.
fn render_event_info(
    slug: &str,
    page_type: PageType,
    composed: Option<&str>,
    index: Option<&EventIndex>,
    is_a: &dyn Fn(&str, &str) -> bool,
    labels: &Labels,
) -> Result<String, DocError> {
    let Some((interface, event_type)) =
        parse_event_slug(slug).filter(|_| page_type == PageType::WebApiEvent)
    else {
        return Err(DocError::InvalidSlugForX(format!(
            "{slug}: eventinfo templ"
        )));
    };
    let composed = parse_composed(composed)?;

    let entries = index.map(|index| index.get(event_type).map(Vec::as_slice).unwrap_or_default());
    if entries.is_some_and(<[_]>::is_empty) {
        tracing::warn!(
            source = "templ-webref-events-missing",
            ic = get_issue_counter(),
            "@webref/events has no entry for event \"{event_type}\""
        );
    }
    let flags = select_flags(interface, entries.unwrap_or_default(), is_a);

    let mut rows = String::new();
    for (name, flag, label) in [
        ("bubbles", flags.bubbles, labels.bubbles),
        ("cancelable", flags.cancelable, labels.cancelable),
    ] {
        match flag {
            Flag::Known(value) => row(&mut rows, label, value, labels),
            Flag::Conflict => warn_conflict(name, interface, event_type),
            Flag::Unknown => {}
        }
    }
    if let Some(composed) = composed {
        row(&mut rows, labels.composed, composed, labels);
    }

    if rows.is_empty() {
        return Ok(rows);
    }
    Ok(format!(
        r#"<table class="properties"><tbody>{rows}</tbody></table>"#
    ))
}

fn real_is_a(child: &str, parent: &str) -> bool {
    inheritance(child).contains(&parent)
}

/// Renders the Bubbles and Cancelable rows of an event page from webref data.
///
/// # Arguments
/// * `composed` - Optional flag, `"composed"` adds the Composed row
#[rari_f(register = "crate::Templ")]
pub fn eventinfo(composed: Option<String>) -> Result<String, DocError> {
    render_event_info(
        env.slug,
        env.page_type,
        composed.as_deref(),
        events_index(),
        &real_is_a,
        &Labels::new(env.locale)?,
    )
}

#[cfg(test)]
mod test {
    use rari_deps::webref_events::parse_index;

    use super::*;

    const FIXTURE: &str = include_str!("../../../tests/fixtures/webref-events.json");

    const LABELS: Labels = Labels {
        bubbles: "Bubbles",
        cancelable: "Cancelable",
        composed: "Composed",
        yes: "Yes",
        no: "No",
    };

    fn is_a(child: &str, parent: &str) -> bool {
        matches!(
            (child, parent),
            ("HTMLElement" | "SVGElement" | "MathMLElement", "Element")
        )
    }

    fn row_html(label: &str, value: &str) -> String {
        format!(r#"<tr><th scope="row">{label}</th><td>{value}</td></tr>"#)
    }

    fn table(rows: &[String]) -> String {
        format!(
            r#"<table class="properties"><tbody>{}</tbody></table>"#,
            rows.concat()
        )
    }

    #[test]
    fn test_render_event_info() {
        struct Case {
            name: &'static str,
            slug: &'static str,
            page_type: PageType,
            composed: Option<&'static str>,
            with_index: bool,
            expected: Result<String, &'static str>,
        }
        let ev = PageType::WebApiEvent;
        let cases = vec![
            Case {
                name: "bubbles and cancelable",
                slug: "Web/API/Element/click_event",
                page_type: ev,
                composed: None,
                with_index: true,
                expected: Ok(table(&[
                    row_html("Bubbles", "Yes"),
                    row_html("Cancelable", "Yes"),
                ])),
            },
            Case {
                name: "composed flag adds last row",
                slug: "Web/API/Element/click_event",
                page_type: ev,
                composed: Some("composed"),
                with_index: true,
                expected: Ok(table(&[
                    row_html("Bubbles", "Yes"),
                    row_html("Cancelable", "Yes"),
                    row_html("Composed", "Yes"),
                ])),
            },
            Case {
                name: "only composed when webref has no flags",
                slug: "Web/API/SpeechRecognition/audioend_event",
                page_type: ev,
                composed: Some("composed"),
                with_index: true,
                expected: Ok(table(&[row_html("Composed", "Yes")])),
            },
            Case {
                name: "nothing known renders nothing",
                slug: "Web/API/SpeechRecognition/audioend_event",
                page_type: ev,
                composed: None,
                with_index: true,
                expected: Ok(String::new()),
            },
            Case {
                name: "unknown event type renders nothing",
                slug: "Web/API/Element/nonsense_event",
                page_type: ev,
                composed: None,
                with_index: true,
                expected: Ok(String::new()),
            },
            Case {
                name: "index failed to load renders nothing",
                slug: "Web/API/Element/click_event",
                page_type: ev,
                composed: None,
                with_index: false,
                expected: Ok(String::new()),
            },
            Case {
                name: "conflicting values are hidden",
                slug: "Web/API/Window/error_event",
                page_type: ev,
                composed: None,
                with_index: true,
                expected: Ok(String::new()),
            },
            Case {
                name: "invalid composed flag",
                slug: "Web/API/Element/click_event",
                page_type: ev,
                composed: Some("true"),
                with_index: true,
                expected: Err("eventinfo argument 1 (composed)"),
            },
            Case {
                name: "wrong page type",
                slug: "Web/API/Element/click_event",
                page_type: PageType::WebApiInterface,
                composed: None,
                with_index: true,
                expected: Err("eventinfo templ"),
            },
            Case {
                name: "wrong slug",
                slug: "Web/API/Element",
                page_type: ev,
                composed: None,
                with_index: true,
                expected: Err("eventinfo templ"),
            },
        ];
        let index = parse_index(FIXTURE).unwrap();
        for case in cases {
            let result = render_event_info(
                case.slug,
                case.page_type,
                case.composed,
                case.with_index.then_some(&index),
                &is_a,
                &LABELS,
            );
            match (result, case.expected) {
                (Ok(actual), Ok(expected)) => assert_eq!(actual, expected, "{}", case.name),
                (Err(err), Err(fragment)) => {
                    assert!(err.to_string().contains(fragment), "{}: {err}", case.name)
                }
                (actual, expected) => panic!("{}: {actual:?} vs {expected:?}", case.name),
            }
        }
    }

    #[test]
    fn test_real_inheritance_data() {
        // Needs the content repo's `jsondata/InterfaceData.json`.
        if !rari_types::globals::content_root()
            .join("jsondata/InterfaceData.json")
            .exists()
        {
            return;
        }
        let index = parse_index(FIXTURE).unwrap();
        struct Case {
            name: &'static str,
            slug: &'static str,
            expected: String,
        }
        let cases = vec![
            Case {
                name: "Element inherited by HTMLElement and friends",
                slug: "Web/API/Element/click_event",
                expected: table(&[row_html("Bubbles", "Yes"), row_html("Cancelable", "Yes")]),
            },
            Case {
                name: "page interface inherits from target",
                slug: "Web/API/HTMLInputElement/selectionchange_event",
                expected: table(&[row_html("Bubbles", "Yes"), row_html("Cancelable", "No")]),
            },
            Case {
                name: "HTMLVideoElement inherits HTMLMediaElement",
                slug: "Web/API/HTMLVideoElement/resize_event",
                expected: table(&[row_html("Bubbles", "No")]),
            },
        ];
        for case in cases {
            let out = render_event_info(
                case.slug,
                PageType::WebApiEvent,
                None,
                Some(&index),
                &real_is_a,
                &LABELS,
            )
            .unwrap();
            assert_eq!(out, case.expected, "{}", case.name);
        }
    }
}
