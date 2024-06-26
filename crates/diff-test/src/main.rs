use std::cmp::max;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use anyhow::{anyhow, Error};
use clap::{Args, Parser, Subcommand};
use ignore::types::TypesBuilder;
use ignore::WalkBuilder;
use itertools::Itertools;
use jsonpath_lib::Compiled;
use once_cell::sync::Lazy;
use prettydiff::diff_words;
use regex::Regex;
use serde_json::Value;

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
    value: bool,
    #[arg(short, long)]
    verbose: bool,
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
    s.starts_with('<') && s.ends_with('>')
}

const IGNORE: &[&str] = &[
    "doc.flaws",
    "blogMeta.readTime",
    "doc.modified",
    "doc.popularity",
    "doc.source.github_url",
    "doc.source.last_commit_url",
    "doc.sidebarHTML",
];

static WS_DIFF: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?<x>>)[\n ]+|[\n ]+(?<y></)"#).unwrap());

fn full_diff(lhs: &Value, rhs: &Value, path: &[PathIndex], diff: &mut BTreeMap<String, String>) {
    if path.len() == 1 {
        if let PathIndex::Object(s) = &path[0] {
            if s == "url" {
                return;
            }
        }
    }
    if lhs != rhs {
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
                    );
                }
            }
            (Value::String(lhs), Value::String(rhs)) => {
                let mut lhs = lhs.to_owned();
                let mut rhs = rhs.to_owned();
                let key = make_key(path);
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
                    lhs = html_minifier::minify(WS_DIFF.replace_all(&lhs, "$x$y")).unwrap();
                    rhs = html_minifier::minify(WS_DIFF.replace_all(&rhs, "$x$y")).unwrap();
                }
                if lhs != rhs {
                    if IGNORE.contains(&key.as_str()) {
                        return;
                    }
                    if key != "doc.sidebarHTML" {
                        diff.insert(
                            key,
                            ansi_to_html::convert(&diff_words(&lhs, &rhs).to_string()).unwrap(),
                            //similar::TextDiff::from_words(&lhs, &rhs)
                            //    .unified_diff()
                            //    .to_string(),
                        );
                    } else {
                        diff.insert(key, "differs".into());
                    }
                }
            }
            (lhs, rhs) => {
                let lhs = lhs.to_string();
                let rhs = rhs.to_string();
                if lhs != rhs {
                    let key = make_key(path);
                    if IGNORE.contains(&key.as_str()) {
                        return;
                    }
                    diff.insert(
                        key,
                        //ansi_to_html::convert(
                        //    &similar::TextDiff::from_words(&lhs, &rhs)
                        //        .unified_diff()
                        //        .to_string(),
                        //)
                        //.unwrap(),
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
            let mut same = 0;
            if arg.html {
                let mut out = Vec::new();
                out.push("<ul>".to_string());
                for (k, v) in a.iter() {
                    if b.get(k) == Some(v) {
                        same += 1;
                        continue;
                    }

                    if arg.value {
                        let left = v;
                        let right = b.get(k).unwrap_or(&Value::Null);
                        let mut diff = BTreeMap::new();
                        full_diff(left, right, &[], &mut diff);
                        if !diff.is_empty() {
                            out.push(format!(
                                r#"<li><span>{k}</span><div class="r"><pre><code>{}</code></pre></div></li>"#,
                                serde_json::to_string_pretty(&diff).unwrap_or_default(),
                            ));
                        } else {
                            same += 1;
                        }
                        continue;
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
                            same += 1;
                            continue;
                        }
                        if arg.inline {
                            println!("{}", diff_words(&left, right));
                        }
                        out.push(format!(
                    r#"<li><span>{k}</span><div class="a">{}</div><div class="b">{}</div></li>"#,
                    left, right
                ))
                    }
                }
                out.push("</ul>".to_string());
                let mut file = File::create(&arg.out)?;
                file.write_all(html(&out.into_iter().collect::<String>()).as_bytes())?;
            }

            println!("Took: {:?} - {same}/{hits}", start.elapsed());
        }
    }
    Ok(())
}
