use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id};
use tracing::{Event, Subscriber};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

#[derive(Debug, Default)]
pub struct Issue {
    pub fields: Vec<(&'static str, String)>,
    pub spans: Vec<(&'static str, String)>,
}

#[derive(Debug, Default)]
pub struct IssueEntries {
    entries: Vec<(&'static str, String)>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Issues<'a> {
    pub templ: BTreeMap<&'a str, Vec<TemplIssue<'a>>>,
    pub other: BTreeMap<&'a str, Vec<TemplIssue<'a>>>,
    pub no_pos: BTreeMap<&'a str, Vec<TemplIssue<'a>>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TemplIssue<'a> {
    pub source: &'a str,
    pub file: &'a str,
    pub slug: &'a str,
    pub locale: &'a str,
    pub line: &'a str,
    pub col: &'a str,
    pub tail: Vec<(&'static str, &'a str)>,
}

static UNKNOWN: &str = "unknown";
static DEFAULT_TEMPL_ISSUE: TemplIssue<'static> = TemplIssue {
    source: UNKNOWN,
    file: UNKNOWN,
    slug: UNKNOWN,
    locale: UNKNOWN,
    line: UNKNOWN,
    col: UNKNOWN,
    tail: vec![],
};

impl<'a> From<&'a Issue> for TemplIssue<'a> {
    fn from(value: &'a Issue) -> Self {
        let mut tissue = DEFAULT_TEMPL_ISSUE.clone();
        for (key, value) in value.spans.iter().chain(value.fields.iter()) {
            match *key {
                "file" => {
                    tissue.file = value.as_str();
                }
                "slug" => {
                    tissue.slug = value.as_str();
                }
                "locale" => {
                    tissue.locale = value.as_str();
                }
                "line" => tissue.line = value.as_str(),
                "col" => tissue.col = value.as_str(),
                "source" => {
                    tissue.source = value.as_str();
                }
                "message" => {}
                _ => tissue.tail.push((key, value.as_str())),
            }
        }
        tissue
    }
}

pub fn issues_by(issues: &[Issue]) -> Issues {
    let mut templ: BTreeMap<&str, Vec<TemplIssue>> = BTreeMap::new();
    let mut other: BTreeMap<&str, Vec<TemplIssue>> = BTreeMap::new();
    let mut no_pos: BTreeMap<&str, Vec<TemplIssue>> = BTreeMap::new();
    for issue in issues.iter().map(TemplIssue::from) {
        if let Some(templ_name) =
            issue
                .tail
                .iter()
                .find_map(|(key, value)| if *key == "templ" { Some(value) } else { None })
        {
            templ.entry(templ_name).or_default().push(issue);
        } else if issue.line != UNKNOWN {
            other.entry(issue.source).or_default().push(issue)
        } else {
            no_pos.entry(issue.source).or_default().push(issue);
        }
    }
    Issues {
        templ,
        other,
        no_pos,
    }
}

#[derive(Clone)]
pub struct InMemoryLayer {
    events: Arc<Mutex<Vec<Issue>>>,
}

impl InMemoryLayer {
    pub fn new() -> Self {
        InMemoryLayer {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_events(&self) -> Arc<Mutex<Vec<Issue>>> {
        Arc::clone(&self.events)
    }
}

impl Visit for IssueEntries {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.entries.push((field.name(), format!("{value:?}")));
    }
    fn record_str(&mut self, field: &Field, value: &str) {
        self.entries.push((field.name(), value.to_string()));
    }
}
impl Visit for Issue {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.fields.push((field.name(), format!("{value:?}")));
    }
    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields.push((field.name(), value.to_string()));
    }
}
impl<S> Layer<S> for InMemoryLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(
        &self,
        attrs: &Attributes<'_>,
        id: &Id,
        ctx: tracing_subscriber::layer::Context<S>,
    ) {
        let span = ctx.span(id).expect("Span not found, this is a bug");
        let mut extensions = span.extensions_mut();

        if extensions.get_mut::<IssueEntries>().is_none() {
            let mut fields = IssueEntries::default();
            attrs.values().record(&mut fields);
            extensions.insert(fields);
        }
    }
    fn on_event(&self, event: &Event, ctx: tracing_subscriber::layer::Context<S>) {
        let mut issue = Issue {
            fields: vec![],
            spans: vec![],
        };
        let span = ctx.event_span(event);
        let scope = span.into_iter().flat_map(|span| span.scope());
        for span in scope {
            let ext = span.extensions();
            if let Some(entries) = ext.get::<IssueEntries>() {
                issue.spans.extend(entries.entries.iter().rev().cloned());
            }
        }

        event.record(&mut issue);
        let mut events = self.events.lock().unwrap();
        events.push(issue);
    }
}
