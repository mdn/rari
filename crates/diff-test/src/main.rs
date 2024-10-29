use std::borrow::Cow;
use std::cmp::max;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, LazyLock};

use anyhow::{anyhow, Error};
use base64::prelude::{Engine as _, BASE64_STANDARD_NO_PAD};
use clap::{Args, Parser, Subcommand};
use dashmap::DashMap;
use ignore::types::TypesBuilder;
use ignore::WalkBuilder;
use itertools::Itertools;
use jsonpath_lib::Compiled;
use lol_html::{element, rewrite_str, ElementContentHandlers, RewriteStrSettings, Selector};
use prettydiff::{diff_lines, diff_words};
use rayon::prelude::*;
use regex::Regex;
use serde_json::Value;
use sha2::{Digest, Sha256};
use xml::fmt_html;

mod xml;

fn html(body: &str) -> String {
    format!(
        r#"<!doctype html>
<html lang="en" prefix="og: https://ogp.me/ns#">

<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <style>
        ul {{
            display: flex;
            flex-direction: column;
            & li {{
                margin: 1rem;
                border: 1px solid gray;
                list-style: none;
                display: grid;
                grid-template-areas: "h h" "a b" "r r";
                grid-auto-columns: 1fr 1fr;
                & > span {{
                    padding: .5rem;
                    background-color: lightgray;
                    grid-area: h;
                }}
                & > div {{
                    padding: .5rem;
                    &.a {{
                        grid-area: a;
                    }}
                    &.b {{
                        grid-area: b;
                    }}
                    &.r {{
                        grid-area: r;
                    }}

                    & > pre {{
                        text-wrap: wrap;
                    }}
                }}
            }}
        }}
    </style>
</head>
<body>
{body}
</body>
</html>
"#
    )
}

pub(crate) fn walk_builder(path: &Path) -> Result<WalkBuilder, Error> {
    let mut types = TypesBuilder::new();
    types.add_def("json:index.json")?;
    types.select("json");
    let mut builder = ignore::WalkBuilder::new(path);
    builder.types(types.build()?);
    Ok(builder)
}

pub fn gather(path: &Path, selector: Option<&str>) -> Result<BTreeMap<String, Value>, Error> {
    let template = if let Some(selector) = selector {
        Some(Compiled::compile(selector).map_err(|e| anyhow!("{e}"))?)
    } else {
        None
    };
    walk_builder(path)?
        .build()
        .filter_map(Result::ok)
        .filter(|f| f.file_type().map(|ft| ft.is_file()).unwrap_or(false))
        .map(|p| {
            let json_str = fs::read_to_string(p.path())?;
            let index: Value = serde_json::from_str(&json_str)?;

            let extract = if let Some(template) = &template {
                template
                    .select(&index)
                    .unwrap_or_default()
                    .into_iter()
                    .next()
                    .cloned()
                    .unwrap_or(Value::Null)
            } else {
                index
            };
            Ok::<_, Error>((p.path().strip_prefix(path)?.display().to_string(), extract))
        })
        .collect()
}

use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Diff(BuildArgs),
}
#[derive(Args)]
struct BuildArgs {
    #[arg(short, long)]
    query: Option<String>,
    #[arg(short, long)]
    out: PathBuf,
    root_a: PathBuf,
    root_b: PathBuf,
    #[arg(long)]
    html: bool,
    #[arg(long)]
    inline: bool,
    #[arg(long)]
    ignore_html_whitespace: bool,
    #[arg(long)]
    fast: bool,
    #[arg(long)]
    value: bool,
    #[arg(short, long)]
    verbose: bool,
    #[arg(long)]
    sidebars: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PathIndex {
    Object(String),
    Array(usize),
}

fn make_key(path: &[PathIndex]) -> String {
    path.iter()
        .map(|k| match k {
            PathIndex::Object(s) => s.to_owned(),
            PathIndex::Array(i) => i.to_string(),
        })
        .join(".")
}

fn is_html(s: &str) -> bool {
    s.trim_start().starts_with('<') && s.trim_end().ends_with('>')
}

const IGNORE: &[&str] = &[
    "doc.flaws",
    "blogMeta.readTime",
    "doc.modified",
    "doc.popularity",
    "doc.source.github_url",
    "doc.source.last_commit_url",
    //"doc.sidebarHTML",
    "doc.sidebarMacro",
    "doc.hasMathML",
    "doc.other_translations",
    "doc.summary",
];

static WS_DIFF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?<x>>)[\n ]+|[\n ]+(?<y></)"#).unwrap());

static DIFF_MAP: LazyLock<Arc<DashMap<String, String>>> =
    LazyLock::new(|| Arc::new(DashMap::new()));

/// Run html content through these handlers to clean up the hatml before minifying and diffing.
fn pre_diff_element_massaging_handlers<'a>() -> Vec<(Cow<'a, Selector>, ElementContentHandlers<'a>)>
{
    vec![
        // remove data-flaw-src attributes
        element!("*[data-flaw-src]", |el| {
            el.remove_attribute("data-flaw-src");
            Ok(())
        }),
        // remove ids from notecards, example-headers, code-examples
        element!("div.notecard, div.example-header, div.code-example", |el| {
            el.remove_attribute("id");
            Ok(())
        }),
    ]
}

fn full_diff(
    lhs: &Value,
    rhs: &Value,
    path: &[PathIndex],
    diff: &mut BTreeMap<String, String>,
    fast: bool,
    sidebars: bool,
) {
    if path.len() == 1 {
        if let PathIndex::Object(s) = &path[0] {
            if s == "url" {
                return;
            }
        }
    }
    if lhs != rhs {
        let key = make_key(path);
        if IGNORE.iter().any(|i| key.starts_with(i)) || key == "doc.sidebarHTML" && !sidebars {
            return;
        }
        match (lhs, rhs) {
            (Value::Array(lhs), Value::Array(rhs)) => {
                let len = max(lhs.len(), rhs.len());
                for i in 0..len {
                    let mut path = path.to_vec();
                    path.push(PathIndex::Array(i));
                    full_diff(
                        lhs.get(i).unwrap_or(&Value::Null),
                        rhs.get(i).unwrap_or(&Value::Null),
                        &path,
                        diff,
                        fast,
                        sidebars,
                    );
                }
            }
            (Value::Object(lhs), Value::Object(rhs)) => {
                let mut keys: HashSet<&String> = HashSet::from_iter(lhs.keys());
                keys.extend(rhs.keys());
                for key in keys {
                    let mut path = path.to_vec();
                    path.push(PathIndex::Object(key.to_string()));
                    full_diff(
                        lhs.get(key).unwrap_or(&Value::Null),
                        rhs.get(key).unwrap_or(&Value::Null),
                        &path,
                        diff,
                        fast,
                        sidebars,
                    );
                }
            }
            (Value::String(lhs), Value::String(rhs)) => {
                let mut lhs = lhs.to_owned();
                let mut rhs = rhs.to_owned();
                match key.as_str() {
                    "doc.sidebarMacro" => {
                        lhs = lhs.to_lowercase();
                        rhs = rhs.to_lowercase();
                    }
                    "doc.summary" => {
                        lhs = lhs.replace("\n  ", "\n");
                        rhs = rhs.replace("\n  ", "\n");
                    }
                    _ => {}
                };
                if is_html(&lhs) && is_html(&rhs) {
                    let lhs_t = WS_DIFF.replace_all(&lhs, "$x$y");
                    let rhs_t = WS_DIFF.replace_all(&rhs, "$x$y");
                    let lhs_t = rewrite_str(
                        &lhs_t,
                        RewriteStrSettings {
                            element_content_handlers: pre_diff_element_massaging_handlers(),
                            ..RewriteStrSettings::new()
                        },
                    )
                    .expect("lolhtml processing failed");
                    let rhs_t = rewrite_str(
                        &rhs_t,
                        RewriteStrSettings {
                            element_content_handlers: pre_diff_element_massaging_handlers(),
                            ..RewriteStrSettings::new()
                        },
                    )
                    .expect("lolhtml processing failed");
                    lhs = fmt_html(&html_minifier::minify(lhs_t).unwrap());
                    rhs = fmt_html(&html_minifier::minify(rhs_t).unwrap());
                }
                if lhs != rhs {
                    let mut diff_hash = Sha256::new();
                    diff_hash.write_all(lhs.as_bytes()).unwrap();
                    diff_hash.write_all(rhs.as_bytes()).unwrap();
                    let diff_hash = BASE64_STANDARD_NO_PAD.encode(&diff_hash.finalize()[..]);
                    if let Some(hash) = DIFF_MAP.get(&diff_hash) {
                        diff.insert(key, format!("See {}", hash.as_str()));
                        return;
                    }
                    DIFF_MAP.insert(diff_hash, "somewhere else".into());
                    diff.insert(
                        key,
                        ansi_to_html::convert(&if fast {
                            diff_lines(&lhs, &rhs).to_string()
                        } else {
                            diff_words(&lhs, &rhs).to_string()
                        })
                        .unwrap(),
                    );
                }
            }
            (lhs, rhs) => {
                let lhs = lhs.to_string();
                let rhs = rhs.to_string();
                if lhs != rhs {
                    diff.insert(
                        key,
                        ansi_to_html::convert(&diff_words(&lhs, &rhs).to_string()).unwrap(),
                    );
                }
            }
        }
    }
}

fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Diff(arg) => {
            println!("Gathering everything 🧺");
            let start = std::time::Instant::now();
            let a = gather(&arg.root_a, arg.query.as_deref())?;
            let b = gather(&arg.root_b, arg.query.as_deref())?;

            let hits = max(a.len(), b.len());
            let same = AtomicUsize::new(0);
            if arg.html {
                let mut out = Vec::new();
                out.push("<ul>".to_string());
                out.extend(a.par_iter().filter_map(|(k, v)| {
                    if b.get(k) == Some(v) {
                        same.fetch_add(1, Relaxed);
                        return None;
                    }

                    if arg.value {
                        let left = v;
                        let right = b.get(k).unwrap_or(&Value::Null);
                        let mut diff = BTreeMap::new();
                        full_diff(left, right, &[], &mut diff, arg.fast, arg.sidebars);
                        if !diff.is_empty() {
                            return Some(format!(
                                r#"<li><span>{k}</span><div class="r"><pre><code>{}</code></pre></div></li>"#,
                                serde_json::to_string_pretty(&diff).unwrap_or_default(),
                            ));
                        } else {
                            same.fetch_add(1, Relaxed);
                        }
                        None
                    } else {
                        let left = &v.as_str().unwrap_or_default();
                        let right = b
                            .get(k)
                            .unwrap_or(&Value::Null)
                            .as_str()
                            .unwrap_or_default();
                        let htmls = if arg.ignore_html_whitespace {
                            let left_html =
                                html_minifier::minify(WS_DIFF.replace_all(left, "$x$y")).unwrap();
                            let right_html =
                                html_minifier::minify(WS_DIFF.replace_all(right, "$x$y")).unwrap();
                            Some((left_html, right_html))
                        } else {
                            None
                        };

                        let (left, right) = htmls
                            .as_ref()
                            .map(|(l, r)| (l.as_str(), r.as_str()))
                            .unwrap_or((left, right));
                        let broken_link = r#" class="page-not-created" title="This is a link to an unwritten page""#;
                        let left = left.replace(broken_link, "");
                        if left == right {
                            println!("only broken links differ");
                            same.fetch_add(1, Relaxed);
                            return None;
                        }
                        if arg.inline {
                            println!("{}", diff_words(&left, right));
                        }
                        Some(format!(
                    r#"<li><span>{k}</span><div class="a">{}</div><div class="b">{}</div></li>"#,
                    left, right
                ))
                    }
                }
                ).collect::<Vec<_>>());
                out.push("</ul>".to_string());
                let mut file = File::create(&arg.out)?;
                file.write_all(html(&out.into_iter().collect::<String>()).as_bytes())?;
            }

            println!(
                "Took: {:?} - {}/{hits}",
                start.elapsed(),
                same.load(Relaxed)
            );
        }
    }
    Ok(())
}
