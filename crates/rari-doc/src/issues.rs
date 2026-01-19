use std::collections::{BTreeMap, HashMap};
use std::convert::Infallible;
use std::str::FromStr;
use std::sync::atomic::AtomicI64;
use std::sync::{Arc, LazyLock, OnceLock};
use std::{fmt, iter};

use dashmap::DashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id};
use tracing::{Event, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::registry::LookupSpan;

use crate::pages::page::{Page, PageLike};
use crate::position_utils::byte_to_char_column;

pub static ISSUE_COUNTER_F: OnceLock<fn() -> i64> = OnceLock::new();
static ISSUE_COUNTER: AtomicI64 = AtomicI64::new(0);

pub(crate) fn get_issue_counter() -> i64 {
    ISSUE_COUNTER_F.get_or_init(|| get_issue_counter_f)()
}

pub(crate) fn get_issue_counter_f() -> i64 {
    ISSUE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// Internal representation of an issue detected during build.
///
/// This struct stores position information in **byte offsets** (from tree-sitter and comrak),
/// which are later converted to character positions in `DisplayIssue` for user-facing output.
#[derive(Debug, Clone, Serialize)]
pub struct Issue {
    pub req: u64,
    pub ic: i64,
    /// Column in BYTES from start of line (from tree-sitter or comrak sourcepos)
    pub col: i64,
    /// Line number (1-based)
    pub line: i64,
    /// End column in BYTES from start of line
    pub end_col: i64,
    /// End line number (1-based)
    pub end_line: i64,
    pub file: String,
    pub ignore: bool,
    pub fields: Vec<(&'static str, String)>,
    pub spans: Vec<(&'static str, String)>,
}

#[derive(Debug, Default)]
pub struct IssueEntries {
    req: u64,
    ic: i64,
    col: i64,
    line: i64,
    end_col: i64,
    end_line: i64,
    file: String,
    ignore: bool,
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
    fn record_bool(&mut self, field: &Field, value: bool) {
        if field.name() == "ignore" {
            self.ignore = value;
            eprintln!("Setting IssueEntries.ignore = {}", value);
        }
    }
    fn record_i64(&mut self, field: &Field, value: i64) {
        if field.name() == "ic" {
            self.ic = value;
        } else if field.name() == "col" {
            self.col = value;
        } else if field.name() == "line" {
            self.line = value;
        } else if field.name() == "end_col" {
            self.end_col = value;
        } else if field.name() == "end_line" {
            self.end_line = value;
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
    fn record_bool(&mut self, field: &Field, value: bool) {
        if field.name() == "ignore" {
            self.ignore = value;
            eprintln!("Setting Issue.ignore = {} (issue: {:?})", value, self);
        }
    }
    fn record_i64(&mut self, field: &Field, value: i64) {
        if field.name() == "ic" {
            self.ic = value;
        } else if field.name() == "col" {
            self.col = value;
        } else if field.name() == "line" {
            self.line = value;
        } else if field.name() == "end_col" {
            self.end_col = value;
        } else if field.name() == "end_line" {
            self.end_line = value;
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
            ic: -1,
            col: 0,
            line: 0,
            end_col: 0,
            end_line: 0,
            file: String::default(),
            ignore: false,
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
                if entries.end_col != 0 {
                    issue.end_col = entries.end_col;
                }
                if entries.end_line != 0 {
                    issue.end_line = entries.end_line;
                }
                if !entries.file.is_empty() {
                    issue.file = entries.file.clone();
                }
                if entries.ic != -1 {
                    issue.ic = entries.ic;
                }
                if entries.ignore {
                    issue.ignore = entries.ignore;
                    eprintln!("Setting issue.ignore = {} from entries", entries.ignore);
                }
                issue.spans.extend(entries.entries.iter().rev().cloned());
            }
        }

        if issue.ic == -1 && !issue.ignore {
            issue.ic = get_issue_counter();
        }

        if !issue.ignore {
            // Record first.
            event.record(&mut issue);
        }

        if !issue.ignore {
            // Check again after recording.
            self.events
                .entry(issue.file.clone())
                .or_default()
                .push(issue);
        }
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

/// User-facing representation of an issue for display and JSON output.
///
/// This struct stores position information in **character positions** for proper display
/// in editors and user interfaces. The positions are converted from byte offsets in `Issue`.
#[derive(Serialize, Deserialize, Debug, Default, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DisplayIssue {
    pub id: i64,
    pub explanation: Option<String>,
    pub suggestion: Option<String>,
    pub fixable: Option<bool>,
    pub fixed: bool,
    /// Line number (1-based)
    pub line: Option<i64>,
    /// Column in CHARACTERS from start of line (1-based, user-facing)
    pub column: Option<i64>,
    /// End line number (1-based)
    pub end_line: Option<i64>,
    /// End column in CHARACTERS from start of line (1-based, user-facing)
    pub end_column: Option<i64>,
    pub source_context: Option<String>,
    pub filepath: Option<String>,
    pub name: IssueType,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum IssueType {
    TemplRedirectedLink,
    TemplBrokenLink,
    TemplIllCasedLink,
    TemplInvalidArg,
    RedirectedLink,
    BrokenLink,
    IllCasedLink,
    #[default]
    Unknown,
}

impl FromStr for IssueType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "templ-redirected-link" => Self::TemplRedirectedLink,
            "templ-broken-link" => Self::TemplBrokenLink,
            "templ-ill-cased-link" => Self::TemplIllCasedLink,
            "templ-invalid-arg" => Self::TemplInvalidArg,
            "redirected-link" => Self::RedirectedLink,
            "broken-link" => Self::BrokenLink,
            "ill-cased-link" => Self::IllCasedLink,
            _ => Self::Unknown,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", tag = "type")]
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

impl DIssue {
    pub fn display_issue(&self) -> &DisplayIssue {
        match self {
            DIssue::BrokenLink { display_issue, .. }
            | DIssue::Macros { display_issue, .. }
            | DIssue::Unknown { display_issue } => display_issue,
        }
    }
    pub fn content(&self) -> Option<&str> {
        match self {
            DIssue::BrokenLink { href, .. } | DIssue::Macros { href, .. } => href.as_deref(),
            DIssue::Unknown { .. } => None,
        }
    }
}

pub type DisplayIssues = BTreeMap<&'static str, Vec<DIssue>>;

impl DIssue {
    pub fn from_issue(issue: Issue, page: &Page) -> Option<Self> {
        if let Ok(id) = usize::try_from(issue.ic) {
            // Convert byte columns to character columns for user-facing display
            let (char_col, char_end_col) = if issue.line != 0 && issue.col != 0 {
                // Get the line content (adjust for frontmatter offset)
                let line_idx =
                    (issue.line.saturating_sub(1) as usize).saturating_sub(page.fm_offset());
                if let Some(line_content) = page.content().lines().nth(line_idx) {
                    let char_col = byte_to_char_column(line_content, issue.col as usize) as i64 + 1; // +1 for 1-based
                    let char_end_col = if issue.end_col != 0 {
                        byte_to_char_column(line_content, issue.end_col as usize) as i64 + 1
                    } else {
                        0
                    };
                    (char_col, char_end_col)
                } else {
                    // Fallback: if we can't get the line, use byte positions (legacy behavior)
                    (issue.col, issue.end_col)
                }
            } else {
                (issue.col, issue.end_col)
            };

            let mut di = DisplayIssue {
                id: id as i64,
                column: if char_col == 0 { None } else { Some(char_col) },
                line: if issue.line == 0 {
                    None
                } else {
                    Some(issue.line)
                },
                end_column: if char_end_col == 0 {
                    None
                } else {
                    Some(char_end_col)
                },
                end_line: if issue.end_line == 0 {
                    None
                } else {
                    Some(issue.end_line)
                },
                ..Default::default()
            };
            if let (Some(col), Some(line)) = (di.column, di.line) {
                let line = line - page.fm_offset() as i64;
                // take surrounding +- 3 lines (7 in total)
                let (skip, take, highlight) = if line < 4 {
                    (0, 7 - (4 - line), (line - 1) as usize)
                } else {
                    (line - 4, 7, 3)
                };
                let context = page
                    .content()
                    .lines()
                    .skip(skip as usize)
                    .take(take as usize)
                    .collect::<Vec<_>>();

                let source_context =
                    context
                        .iter()
                        .enumerate()
                        .fold(String::new(), |mut acc, (i, line)| {
                            acc.push_str(line);
                            acc.push('\n');
                            if i == highlight {
                                acc.extend(iter::repeat_n('-', (col - 1) as usize));
                                acc.push_str("^\n");
                            }
                            acc
                        });

                di.source_context = Some(source_context);
            }

            di.filepath = Some(page.full_path().to_string_lossy().into_owned());

            let mut additional = HashMap::new();
            for (key, value) in issue.spans.into_iter().chain(issue.fields.into_iter()) {
                match key {
                    "source" => {
                        di.name = IssueType::from_str(&value).unwrap();
                    }
                    "redirect" => di.suggestion = Some(value),

                    _ => {
                        additional.insert(key, value);
                    }
                }
            }
            let dissue = match di.name {
                IssueType::IllCasedLink => {
                    di.fixed = false;
                    di.fixable = Some(true);
                    di.explanation = Some(format!(
                        "Link {} is ill cased",
                        additional.get("url").map(|s| s.as_str()).unwrap_or("?")
                    ));
                    DIssue::BrokenLink {
                        display_issue: di,
                        href: additional.remove("url"),
                    }
                }
                IssueType::RedirectedLink => {
                    di.fixed = false;
                    di.fixable = Some(true);
                    di.explanation = Some(format!(
                        "Link {} is a redirect",
                        additional.get("url").map(|s| s.as_str()).unwrap_or("?")
                    ));
                    DIssue::BrokenLink {
                        display_issue: di,
                        href: additional.remove("url"),
                    }
                }
                IssueType::BrokenLink => {
                    di.fixed = false;
                    di.fixable = Some(false);
                    di.explanation = Some(format!(
                        "Link {} doesn't resolve",
                        additional.get("url").map(|s| s.as_str()).unwrap_or("?")
                    ));
                    DIssue::BrokenLink {
                        display_issue: di,
                        href: additional.remove("url"),
                    }
                }
                IssueType::TemplBrokenLink => {
                    let macro_name = additional.remove("templ");
                    di.fixed = false;
                    di.fixable = Some(false);
                    di.explanation = Some(format!(
                        "Macro {} produces link {} which doesn't resolve",
                        macro_name.as_deref().unwrap_or("?"),
                        additional.get("url").map(|s| s.as_str()).unwrap_or("?")
                    ));
                    DIssue::Macros {
                        display_issue: di,
                        macro_name,
                        href: additional.remove("url"),
                    }
                }
                IssueType::TemplRedirectedLink => {
                    let macro_name = additional.remove("templ");
                    di.fixed = false;
                    di.fixable = Some(is_fixable_template(macro_name.as_deref()));
                    di.explanation = Some(format!(
                        "Macro {} produces link {} which is a redirect",
                        macro_name.as_deref().unwrap_or("?"),
                        additional.get("url").map(|s| s.as_str()).unwrap_or("?")
                    ));
                    DIssue::Macros {
                        display_issue: di,
                        macro_name,
                        href: additional.remove("url"),
                    }
                }
                IssueType::TemplIllCasedLink => {
                    let macro_name = additional.remove("templ");
                    di.fixed = false;
                    di.fixable = Some(is_fixable_template(macro_name.as_deref()));
                    di.explanation = Some(format!(
                        "Macro {} produces link {} which is ill cased",
                        macro_name.as_deref().unwrap_or("?"),
                        additional.get("url").map(|s| s.as_str()).unwrap_or("?")
                    ));
                    DIssue::Macros {
                        display_issue: di,
                        macro_name,
                        href: additional.remove("url"),
                    }
                }
                IssueType::TemplInvalidArg => {
                    let macro_name = additional.remove("templ");
                    di.fixed = false;
                    di.explanation = Some(format!(
                        "Macro {} received argument ({}) which is not valid.",
                        macro_name.as_deref().unwrap_or("?"),
                        additional.get("arg").map(|s| s.as_str()).unwrap_or("?")
                    ));
                    DIssue::Macros {
                        display_issue: di,
                        macro_name,
                        href: None,
                    }
                }
                _ => {
                    di.explanation = additional.remove("message");
                    DIssue::Unknown { display_issue: di }
                }
            };
            Some(dissue)
        } else {
            None
        }
    }
}

pub fn to_display_issues(issues: Vec<Issue>, page: &Page) -> DisplayIssues {
    let mut map = BTreeMap::new();
    for issue in issues.into_iter() {
        if let Some(di) = DIssue::from_issue(issue, page) {
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
    }
    map
}

/// Check if a template macro issue can be automatically fixed.
/// Only navigation templates have fixable slug parameters in the markdown source.
fn is_fixable_template(macro_name: Option<&str>) -> bool {
    matches!(
        macro_name,
        Some("previous" | "previousmenu" | "previousnext" | "previousmenunext")
    )
}

pub static IN_MEMORY: LazyLock<InMemoryLayer> = LazyLock::new(InMemoryLayer::default);
