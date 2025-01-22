use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::sync::atomic::AtomicI64;
use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use itertools::Itertools;
use schemars::JsonSchema;
use serde::Serialize;
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id};
use tracing::{Event, Subscriber};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

use crate::pages::page::{Page, PageLike};

static ISSUE_COUNTER: AtomicI64 = AtomicI64::new(0);

pub(crate) fn get_issue_counter() -> i64 {
    ISSUE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct Issue {
    pub req: u64,
    pub ic: i64,
    pub col: i64,
    pub line: i64,
    pub file: String,
    pub fields: Vec<(&'static str, String)>,
    pub spans: Vec<(&'static str, String)>,
}

#[derive(Debug, Default)]
pub struct IssueEntries {
    req: u64,
    ic: i64,
    col: i64,
    line: i64,
    file: String,
    entries: Vec<(&'static str, String)>,
}

#[derive(Clone, Default, Debug)]
pub struct InMemoryLayer {
    events: Arc<DashMap<String, Vec<Issue>>>,
}

impl InMemoryLayer {
    pub fn get_events(&self) -> Arc<DashMap<String, Vec<Issue>>> {
        Arc::clone(&self.events)
    }
}

impl Visit for IssueEntries {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.entries.push((field.name(), format!("{value:?}")));
    }
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "file" {
            self.file = value.to_string();
        } else {
            self.entries.push((field.name(), value.to_string()));
        }
    }
    fn record_u64(&mut self, field: &Field, value: u64) {
        if field.name() == "req" {
            self.req = value;
        }
    }
    fn record_i64(&mut self, field: &Field, value: i64) {
        if field.name() == "ic" {
            self.ic = value;
        } else if field.name() == "col" {
            self.col = value;
        } else if field.name() == "line" {
            self.line = value;
        }
    }
}
impl Visit for Issue {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.fields.push((field.name(), format!("{value:?}")));
    }
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "file" {
            self.file = value.to_string();
        } else {
            self.fields.push((field.name(), value.to_string()));
        }
    }
    fn record_u64(&mut self, field: &Field, value: u64) {
        if field.name() == "req" {
            self.req = value;
        }
    }
    fn record_i64(&mut self, field: &Field, value: i64) {
        if field.name() == "ic" {
            self.ic = value;
        } else if field.name() == "col" {
            self.col = value;
        } else if field.name() == "line" {
            self.line = value;
        }
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
            req: 0,
            ic: 0,
            col: 0,
            line: 0,
            file: String::default(),
            fields: vec![],
            spans: vec![],
        };
        let span = ctx.event_span(event);
        let scope = span.into_iter().flat_map(|span| span.scope());
        for span in scope {
            let ext = span.extensions();
            if let Some(entries) = ext.get::<IssueEntries>() {
                if entries.req != 0 {
                    issue.req = entries.req;
                }
                if entries.col != 0 {
                    issue.col = entries.col;
                }
                if entries.line != 0 {
                    issue.line = entries.line;
                }
                if !entries.file.is_empty() {
                    issue.file = entries.file.clone();
                }
                if entries.ic != 0 {
                    issue.ic = entries.ic;
                } else {
                    issue.ic = get_issue_counter();
                }
                issue.spans.extend(entries.entries.iter().rev().cloned());
            }
        }

        event.record(&mut issue);
        self.events
            .entry(issue.file.clone())
            .or_default()
            .push(issue);
    }
}

#[derive(Serialize, Debug, Default, Clone, JsonSchema)]
#[serde(untagged)]
pub enum Additional {
    BrokenLink {
        href: String,
    },
    MacroBrokenLink {
        href: String,
    },
    #[default]
    None,
}

#[derive(Serialize, Debug, Default, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DisplayIssue {
    pub id: i64,
    pub explanation: Option<String>,
    pub suggestion: Option<String>,
    pub fixable: Option<bool>,
    pub fixed: bool,
    pub line: Option<i64>,
    pub col: Option<i64>,
    pub source_context: Option<String>,
    pub filepath: Option<String>,
}

#[derive(Serialize, Debug, Clone, JsonSchema)]
#[serde(tag = "name")]
#[serde(rename_all = "snake_case")]
pub enum DIssue {
    BrokenLink {
        #[serde(flatten)]
        display_issue: DisplayIssue,
        href: Option<String>,
    },
    Macros {
        #[serde(flatten)]
        display_issue: DisplayIssue,
        #[serde(rename = "macroName")]
        macro_name: Option<String>,
        href: Option<String>,
    },
    Unknown {
        #[serde(flatten)]
        display_issue: DisplayIssue,
    },
}

pub type DisplayIssues = BTreeMap<&'static str, Vec<DIssue>>;

impl DIssue {
    fn from_issue_with_id(issue: Issue, page: &Page, id: usize) -> Self {
        let mut di = DisplayIssue {
            id: id as i64,
            col: if issue.col == 0 {
                None
            } else {
                Some(issue.col)
            },
            line: if issue.line == 0 {
                None
            } else {
                Some(issue.line)
            },
            ..Default::default()
        };
        if let (Some(_col), Some(line)) = (di.col, di.line) {
            let line = line - page.fm_offset() as i64;
            // take surrounding +- 3 lines (7 in total)
            let (skip, take) = if line < 4 {
                (0, 7 - (4 - line))
            } else {
                (line - 4, 7)
            };
            let context = page
                .content()
                .lines()
                .skip(skip as usize)
                .take(take as usize)
                .join("\n");

            di.source_context = Some(context);
        }

        di.filepath = Some(page.full_path().to_string_lossy().into_owned());

        let mut name = "Unknown".to_string();
        let mut additional = HashMap::new();
        for (key, value) in issue.spans.into_iter().chain(issue.fields.into_iter()) {
            match key {
                "source" => {
                    name = value;
                }
                "redirect" => di.suggestion = Some(value),

                _ => {
                    additional.insert(key, value);
                }
            }
        }
        match name.as_str() {
            "redirected-link" => {
                di.fixed = false;
                di.fixable = Some(true);
                di.explanation = Some(format!(
                    "{} is a redirect",
                    additional.get("url").map(|s| s.as_str()).unwrap_or("?")
                ));
                DIssue::BrokenLink {
                    display_issue: di,
                    href: additional.remove("url"),
                }
            }
            "broken-link" => {
                di.fixed = false;
                di.fixable = Some(false);
                di.explanation = Some(format!(
                    "Can't resolve {}",
                    additional.get("url").map(|s| s.as_str()).unwrap_or("?")
                ));
                DIssue::BrokenLink {
                    display_issue: di,
                    href: None,
                }
            }
            "templ-broken-link" => {
                di.fixed = false;
                di.fixable = Some(false);
                di.explanation = Some(format!(
                    "Can't resolve {}",
                    additional.get("url").map(|s| s.as_str()).unwrap_or("?")
                ));
                DIssue::Macros {
                    display_issue: di,
                    macro_name: additional.remove("templ"),
                    href: None,
                }
            }
            "templ-redirected-link" => {
                di.fixed = false;
                di.fixable = Some(true);
                di.explanation = Some(format!(
                    "Macro produces link {} which is a redirect",
                    additional.get("url").map(|s| s.as_str()).unwrap_or("?")
                ));
                DIssue::Macros {
                    display_issue: di,
                    macro_name: additional.remove("templ"),
                    href: additional.remove("url"),
                }
            }
            _ => {
                di.explanation = additional.remove("message");
                DIssue::Unknown { display_issue: di }
            }
        }
    }
}

pub fn to_display_issues(issues: Vec<Issue>, page: &Page) -> DisplayIssues {
    let mut map = BTreeMap::new();
    for (id, issue) in issues.into_iter().enumerate() {
        let di = DIssue::from_issue_with_id(issue, page, id);
        match di {
            DIssue::BrokenLink { .. } => {
                let entry: &mut Vec<_> = map.entry("broken_links").or_default();
                entry.push(di);
            }
            DIssue::Macros { .. } => {
                let entry: &mut Vec<_> = map.entry("macros").or_default();
                entry.push(di);
            }
            DIssue::Unknown { .. } => {
                let entry: &mut Vec<_> = map.entry("unknown").or_default();
                entry.push(di);
            }
        }
    }
    map
}

pub static IN_MEMORY: LazyLock<InMemoryLayer> = LazyLock::new(InMemoryLayer::default);
