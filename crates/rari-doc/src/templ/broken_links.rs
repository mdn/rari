use std::cell::RefCell;

use crate::issues::get_issue_counter;

#[derive(Default)]
struct State {
    allow: bool,
    broken: bool,
}

thread_local! {
    static STATES: RefCell<Vec<State>> = const { RefCell::new(Vec::new()) };
}

// Scope suppression to an invocation, including nested renders and parallel builds.
pub(super) struct BrokenLinks;

impl BrokenLinks {
    pub fn new(allow: bool) -> Self {
        STATES.with_borrow_mut(|states| {
            states.push(State {
                allow,
                broken: false,
            })
        });
        Self
    }

    pub fn unnecessary(&self) -> bool {
        STATES.with_borrow(|states| states.last().is_some_and(|s| s.allow && !s.broken))
    }
}

impl Drop for BrokenLinks {
    fn drop(&mut self) {
        STATES.with_borrow_mut(|states| {
            states.pop();
        });
    }
}

pub(crate) fn report_broken_link(url: &str) {
    let suppressed = STATES.with_borrow_mut(|states| {
        for state in states.iter_mut() {
            state.broken = true;
        }
        states.iter().any(|state| state.allow)
    });
    if !suppressed {
        let ic = get_issue_counter();
        tracing::warn!(source = "templ-broken-link", ic = ic, url = url);
    }
}

#[cfg(test)]
pub(crate) fn create_test_target() {
    static TARGET: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    TARGET.get_or_init(|| {
        let folder = rari_types::globals::content_root().join("en-us/glossary/marker_test_target");
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(
            folder.join("index.md"),
            "---\ntitle: Marker test target\nslug: Glossary/Marker_test_target\n---\n",
        )
        .unwrap();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_invocations_inherit_suppression_and_restore_scope() {
        let outer = BrokenLinks::new(true);
        {
            let inner = BrokenLinks::new(false);
            report_broken_link("/missing");
            assert!(!inner.unnecessary());
        }
        assert!(!outer.unnecessary());
        report_broken_link("/missing");
        assert!(!outer.unnecessary());
        drop(outer);
        STATES.with_borrow(|states| assert!(states.is_empty()));
    }

    #[test]
    fn marked_cssinfo_scope_suppresses_generated_cssxref() {
        use crate::helpers::css_info::write_computed_output;
        use crate::issues::InMemoryLayer;
        use rari_types::{RariEnv, locale::Locale};
        use tracing_subscriber::layer::SubscriberExt;

        let folder = rari_types::globals::content_root().join("en-us/web/css/reference");
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(
            folder.join("index.md"),
            "---\ntitle: CSS reference\nslug: Web/CSS/Reference\n---\n",
        )
        .unwrap();
        let layer = InMemoryLayer::default();
        let subscriber = tracing_subscriber::registry().with(layer.clone());
        let _guard = tracing::subscriber::set_default(subscriber);
        let parent = BrokenLinks::new(true);
        let data =
            serde_json::json!({ "computed": { "en-US": "{{cssxref(\"BrandNewProperty\")}}" } });
        let mut out = String::new();
        write_computed_output(
            &RariEnv::default(),
            &mut out,
            Locale::EnUs,
            &data,
            "computed",
            None,
        )
        .unwrap();
        assert!(out.contains("BrandNewProperty"));
        let events = layer.get_events();
        let sources: Vec<_> = events
            .iter()
            .flat_map(|entry| entry.value().clone())
            .flat_map(|issue| issue.fields)
            .filter(|(key, _)| *key == "source")
            .map(|(_, value)| value)
            .collect();
        assert!(sources.is_empty(), "unexpected flaws: {sources:?}");
        assert!(!parent.unnecessary());
    }

    #[test]
    fn isolates_threads() {
        let outer = BrokenLinks::new(true);
        std::thread::spawn(|| {
            let inner = BrokenLinks::new(true);
            report_broken_link("/missing");
            assert!(!inner.unnecessary());
        })
        .join()
        .unwrap();
        assert!(outer.unnecessary());
    }
}
