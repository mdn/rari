//! Event metadata from the `@webref/events` package, used by the `EventInfo` macro.
use rari_deps::webref_events::EventEntry;

/// Extracts `(interface, event type)` from a `Web/API/<Interface>/<type>_event` slug.
pub(crate) fn parse_event_slug(slug: &str) -> Option<(&str, &str)> {
    let (interface, page) = slug.strip_prefix("Web/API/")?.split_once('/')?;
    let event_type = page.strip_suffix("_event")?;
    (!interface.is_empty() && !event_type.is_empty() && !event_type.contains('/'))
        .then_some((interface, event_type))
}

/// How a webref target relates to the interface of the page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Relation {
    /// The target inherits from the page interface.
    Descendant,
    Exact,
    /// The page interface inherits from the target.
    Ancestor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Flag {
    Known(bool),
    Unknown,
    Conflict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EventFlags {
    pub(crate) bubbles: Flag,
    pub(crate) cancelable: Flag,
}

/// Picks the value from the closest tier of candidates.
///
/// Descendants win over the exact match, because webref often lists a base
/// interface (e.g. `Element` for `click`) with the value for direct dispatch while the
/// concrete subclasses carry the value users observe. Descendants only win if they
/// agree; disagreeing descendants fall back to the exact tier. Within the exact and
/// ancestor tiers, disagreement is a conflict, and so are disagreeing descendants
/// with no exact value to decide.
///
/// `cancelable` is per entry in webref, so an entry's value applies to all its matching
/// targets (known limitation).
fn resolve(candidates: &[(Relation, bool)]) -> Flag {
    let mut descendants_conflict = false;
    for tier in [Relation::Descendant, Relation::Exact, Relation::Ancestor] {
        let mut values = candidates
            .iter()
            .filter(|(relation, _)| *relation == tier)
            .map(|(_, value)| *value);
        let Some(first) = values.next() else {
            continue;
        };
        if tier == Relation::Ancestor && descendants_conflict {
            return Flag::Conflict;
        }
        if values.all(|value| value == first) {
            return Flag::Known(first);
        }
        if tier != Relation::Descendant {
            return Flag::Conflict;
        }
        descendants_conflict = true;
    }
    if descendants_conflict {
        Flag::Conflict
    } else {
        Flag::Unknown
    }
}

/// Resolves `bubbles` and `cancelable` for an event page.
///
/// `is_a(child, parent)` tells whether `child` inherits from `parent`.
pub(crate) fn select_flags(
    page_interface: &str,
    entries: &[EventEntry],
    is_a: &dyn Fn(&str, &str) -> bool,
) -> EventFlags {
    let relation = |target: &str| {
        if target == page_interface {
            Some(Relation::Exact)
        } else if is_a(target, page_interface) {
            Some(Relation::Descendant)
        } else if is_a(page_interface, target) {
            Some(Relation::Ancestor)
        } else {
            None
        }
    };

    let mut bubbles = vec![];
    let mut cancelable = vec![];
    for entry in entries {
        let mut entry_relations = vec![];
        for target in &entry.targets {
            let Some(relation) = relation(&target.target) else {
                continue;
            };
            if !entry_relations.contains(&relation) {
                entry_relations.push(relation);
            }
            if let Some(value) = target.bubbles {
                bubbles.push((relation, value));
            }
        }
        if let Some(value) = entry.cancelable {
            cancelable.extend(
                entry_relations
                    .into_iter()
                    .map(|relation| (relation, value)),
            );
        }
    }
    EventFlags {
        bubbles: resolve(&bubbles),
        cancelable: resolve(&cancelable),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../../tests/fixtures/webref-events.json");

    fn is_a(child: &str, parent: &str) -> bool {
        let chain: &[&str] = match child {
            "HTMLElement" | "SVGElement" | "MathMLElement" => &["Element", "Node", "EventTarget"],
            "HTMLInputElement" => &["HTMLElement", "Element", "Node", "EventTarget"],
            "HTMLVideoElement" => &[
                "HTMLMediaElement",
                "HTMLElement",
                "Element",
                "Node",
                "EventTarget",
            ],
            "Element" => &["Node", "EventTarget"],
            _ => &[],
        };
        chain.contains(&parent)
    }

    #[test]
    fn test_parse_event_slug() {
        struct Case {
            name: &'static str,
            slug: &'static str,
            expected: Option<(&'static str, &'static str)>,
        }
        let cases = vec![
            Case {
                name: "event page",
                slug: "Web/API/Element/click_event",
                expected: Some(("Element", "click")),
            },
            Case {
                name: "dashed type",
                slug: "Web/API/Window/beforeunload_event",
                expected: Some(("Window", "beforeunload")),
            },
            Case {
                name: "not an event",
                slug: "Web/API/Element/click",
                expected: None,
            },
            Case {
                name: "not web api",
                slug: "Web/CSS/color_event",
                expected: None,
            },
            Case {
                name: "interface page",
                slug: "Web/API/Element",
                expected: None,
            },
            Case {
                name: "nested",
                slug: "Web/API/Foo/Bar/click_event",
                expected: None,
            },
            Case {
                name: "empty type",
                slug: "Web/API/Element/_event",
                expected: None,
            },
        ];
        for case in cases {
            assert_eq!(parse_event_slug(case.slug), case.expected, "{}", case.name);
        }
    }

    #[test]
    fn test_select_flags() {
        use Flag::{Conflict, Known, Unknown};
        struct Case {
            name: &'static str,
            interface: &'static str,
            event_type: &'static str,
            bubbles: Flag,
            cancelable: Flag,
        }
        let cases = vec![
            Case {
                name: "click on Element prefers agreeing descendants",
                interface: "Element",
                event_type: "click",
                bubbles: Known(true),
                cancelable: Known(true),
            },
            Case {
                name: "click on HTMLElement is exact",
                interface: "HTMLElement",
                event_type: "click",
                bubbles: Known(true),
                cancelable: Known(true),
            },
            Case {
                name: "change with multiple entries",
                interface: "HTMLInputElement",
                event_type: "change",
                bubbles: Known(true),
                cancelable: Unknown,
            },
            Case {
                name: "change on unrelated interface",
                interface: "Screen",
                event_type: "change",
                bubbles: Unknown,
                cancelable: Unknown,
            },
            Case {
                name: "no flags",
                interface: "SpeechRecognition",
                event_type: "audioend",
                bubbles: Unknown,
                cancelable: Unknown,
            },
            Case {
                name: "unknown type",
                interface: "Element",
                event_type: "nonsense",
                bubbles: Unknown,
                cancelable: Unknown,
            },
            Case {
                name: "duplicate bubbles conflict",
                interface: "HTMLElement",
                event_type: "wheel",
                bubbles: Conflict,
                cancelable: Known(false),
            },
            Case {
                name: "cancelable conflict",
                interface: "Window",
                event_type: "error",
                bubbles: Unknown,
                cancelable: Conflict,
            },
            Case {
                name: "page inherits from target",
                interface: "HTMLInputElement",
                event_type: "selectionchange",
                bubbles: Known(true),
                cancelable: Known(false),
            },
            Case {
                name: "page inherits from target, other entry",
                interface: "HTMLVideoElement",
                event_type: "resize",
                bubbles: Known(false),
                cancelable: Unknown,
            },
        ];
        let index = rari_deps::webref_events::parse_index(FIXTURE).unwrap();
        for case in cases {
            let entries = index
                .get(case.event_type)
                .map(Vec::as_slice)
                .unwrap_or_default();
            let flags = select_flags(case.interface, entries, &is_a);
            assert_eq!(flags.bubbles, case.bubbles, "{} (bubbles)", case.name);
            assert_eq!(
                flags.cancelable, case.cancelable,
                "{} (cancelable)",
                case.name
            );
        }
    }
}
