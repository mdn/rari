use std::fmt::Write as _;
use std::fs::{self, File, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf, absolute};
use std::process::Command;

use anyhow::{Context, Error, bail};
use clap::Args;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

/// Minimum median change, in percent, for a metric to count as significant.
const THRESHOLD_PERCENT: f64 = 5.0;
/// Minimum median change of a time metric, as short phases vary by more than the threshold.
const THRESHOLD_SECONDS: f64 = 0.5;

#[derive(Args)]
pub struct PerfArgs {
    /// Base checkout, with a compiled rari binary.
    #[arg(long)]
    base_dir: PathBuf,
    /// PR checkout, with a compiled rari binary.
    #[arg(long)]
    pr_dir: PathBuf,
    /// Path of the rari binary, relative to each checkout.
    #[arg(long, default_value = "target/release/rari")]
    rari_bin: PathBuf,
    /// `DEPS_DATA_DIR` for base builds.
    #[arg(long)]
    base_deps: PathBuf,
    /// `DEPS_DATA_DIR` for PR builds.
    #[arg(long)]
    pr_deps: PathBuf,
    /// Directory for build output, timings and logs.
    #[arg(long)]
    scratch: PathBuf,
    /// Timed builds per side.
    #[arg(long, default_value_t = 3, value_parser = clap::value_parser!(u32).range(1..))]
    rounds: u32,
    /// Write the stats as JSON to this path.
    #[arg(short, long)]
    out: PathBuf,
    /// Append the comparison as Markdown to this path.
    #[arg(long)]
    summary: Option<PathBuf>,
}

/// The output of `rari build --timings`.
#[derive(Deserialize)]
struct Sample {
    wall_seconds: f64,
    cpu_seconds: f64,
    max_rss_bytes: f64,
    phases: Map<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
struct Stat {
    median: f64,
    min: f64,
    max: f64,
}

impl Stat {
    fn new(values: impl IntoIterator<Item = f64>) -> Self {
        let mut values: Vec<f64> = values.into_iter().collect();
        values.sort_by(f64::total_cmp);
        let mid = values.len() / 2;
        let median = if values.len() % 2 == 1 {
            values[mid]
        } else {
            (values[mid - 1] + values[mid]) / 2.0
        };
        Self {
            median,
            min: values[0],
            max: values[values.len() - 1],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Verdict {
    Regression,
    Improvement,
}

impl Verdict {
    fn as_str(self) -> &'static str {
        match self {
            Verdict::Regression => "regression",
            Verdict::Improvement => "improvement",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
struct Metric {
    base: Stat,
    pr: Stat,
}

impl Metric {
    fn new(base: &[Sample], pr: &[Sample], value: impl Fn(&Sample) -> f64) -> Self {
        Self {
            base: Stat::new(base.iter().map(&value)),
            pr: Stat::new(pr.iter().map(&value)),
        }
    }

    fn delta_percent(&self) -> f64 {
        if self.base.median > 0.0 {
            (self.pr.median - self.base.median) / self.base.median * 100.0
        } else {
            0.0
        }
    }

    /// Significant: the median changes by the thresholds and the ranges do not overlap.
    fn verdict(&self, unit: Unit) -> Option<Verdict> {
        if (self.pr.median - self.base.median).abs() < unit.threshold() {
            return None;
        }
        let delta = self.delta_percent();
        if delta >= THRESHOLD_PERCENT && self.pr.min > self.base.max {
            Some(Verdict::Regression)
        } else if delta <= -THRESHOLD_PERCENT && self.pr.max < self.base.min {
            Some(Verdict::Improvement)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy)]
enum Unit {
    Seconds,
    Bytes,
}

impl Unit {
    fn threshold(self) -> f64 {
        match self {
            Unit::Seconds => THRESHOLD_SECONDS,
            Unit::Bytes => 0.0,
        }
    }

    fn fmt(self, value: f64) -> String {
        match self {
            Unit::Seconds => format!("{value:.2}s"),
            Unit::Bytes => format!("{:.2} MiB", value / 1024.0 / 1024.0),
        }
    }
}

struct PerfStats {
    rounds: u32,
    wall_seconds: Metric,
    cpu_seconds: Metric,
    max_rss_bytes: Metric,
    phases: Vec<(String, Metric)>,
}

impl PerfStats {
    fn new(base: &[Sample], pr: &[Sample]) -> Self {
        let phase = |sample: &Sample, key: &str| sample.phases.get(key).and_then(Value::as_f64);
        // Only phases that every run reported, in build order.
        let phases = base[0]
            .phases
            .keys()
            .filter(|key| base.iter().chain(pr).all(|s| phase(s, key).is_some()))
            .map(|key| {
                let metric = Metric::new(base, pr, |s| phase(s, key).unwrap_or_default());
                (key.clone(), metric)
            })
            .collect();
        Self {
            rounds: base.len() as u32,
            wall_seconds: Metric::new(base, pr, |s| s.wall_seconds),
            cpu_seconds: Metric::new(base, pr, |s| s.cpu_seconds),
            max_rss_bytes: Metric::new(base, pr, |s| s.max_rss_bytes),
            phases,
        }
    }

    fn to_json(&self) -> Value {
        let phases: Map<String, Value> = self
            .phases
            .iter()
            .map(|(key, metric)| (key.clone(), json!(metric)))
            .collect();
        json!({
            "rounds": self.rounds,
            "wall_seconds": self.wall_seconds,
            "cpu_seconds": self.cpu_seconds,
            "max_rss_bytes": self.max_rss_bytes,
            "phases": phases,
        })
    }

    fn rows(&self) -> Vec<(String, Unit, Metric)> {
        let mut rows = vec![
            ("Wall time".to_string(), Unit::Seconds, self.wall_seconds),
            (
                "CPU time (user + sys)".to_string(),
                Unit::Seconds,
                self.cpu_seconds,
            ),
            ("Peak RSS".to_string(), Unit::Bytes, self.max_rss_bytes),
        ];
        rows.extend(
            self.phases
                .iter()
                .map(|(key, metric)| (format!("`{key}`"), Unit::Seconds, *metric)),
        );
        rows
    }

    fn to_markdown(&self) -> String {
        let rows = self.rows();
        let mut out = format!(
            "## Build performance\n\n\
             Median of {} interleaved rounds per side (`rari --skip-updates build`, warm caches, shared runner).\n\n\
             | Metric | Base | PR | Delta | Base range | PR range | Significant |\n\
             | --- | ---: | ---: | ---: | ---: | ---: | --- |\n",
            self.rounds
        );
        for (name, unit, metric) in &rows {
            let range = |stat: Stat| format!("{} - {}", unit.fmt(stat.min), unit.fmt(stat.max));
            writeln!(
                out,
                "| {name} | {} | {} | {:+.2}% | {} | {} | {} |",
                unit.fmt(metric.base.median),
                unit.fmt(metric.pr.median),
                metric.delta_percent(),
                range(metric.base),
                range(metric.pr),
                metric
                    .verdict(*unit)
                    .map(Verdict::as_str)
                    .unwrap_or_default(),
            )
            .unwrap();
        }
        let names = |verdict| {
            rows.iter()
                .filter(|(_, unit, metric)| metric.verdict(*unit) == Some(verdict))
                .map(|(name, _, _)| name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        };
        let changes: Vec<String> = [
            ("Regressed", names(Verdict::Regression)),
            ("Improved", names(Verdict::Improvement)),
        ]
        .into_iter()
        .filter(|(_, names)| !names.is_empty())
        .map(|(label, names)| format!("{label}: {names}."))
        .collect();
        let conclusion = if changes.is_empty() {
            "No significant change.".to_string()
        } else {
            changes.join(" ")
        };
        write!(
            out,
            "\n{conclusion}\n\n\
             Significant means a median change of at least {THRESHOLD_PERCENT}% (and {THRESHOLD_SECONDS}s for times) with non-overlapping base and PR ranges.\n"
        )
        .unwrap();
        out
    }
}

struct Side<'a> {
    name: &'static str,
    dir: &'a Path,
    bin: &'a Path,
    deps: &'a Path,
}

fn timed_build(side: &Side, round: u32, scratch: &Path) -> Result<Sample, Error> {
    let out_root = scratch.join("out");
    let timings = scratch.join(format!("{}-{round}.json", side.name));
    let log_path = scratch.join(format!("{}-{round}.log", side.name));
    println!("Round {round}: building {}", side.name);

    let log = File::create(&log_path)?;
    let status = Command::new(side.dir.join(side.bin))
        .args(["--skip-updates", "build", "--timings"])
        .arg(&timings)
        .current_dir(side.dir)
        .env("BUILD_OUT_ROOT", &out_root)
        .env("DEPS_DATA_DIR", side.deps)
        .stdout(log.try_clone()?)
        .stderr(log)
        .status()
        .with_context(|| format!("failed to run rari in {}", side.dir.display()))?;
    // Every run starts without output, and the output must not fill the disk.
    let _ = fs::remove_dir_all(&out_root);
    if !status.success() {
        let log = fs::read_to_string(&log_path).unwrap_or_default();
        let tail: Vec<&str> = log.lines().rev().take(20).collect();
        bail!(
            "timed build failed ({}, round {round}, {status}):\n{}",
            side.name,
            tail.into_iter().rev().collect::<Vec<_>>().join("\n")
        );
    }
    serde_json::from_slice(&fs::read(&timings)?)
        .with_context(|| format!("invalid timings in {}", timings.display()))
}

pub fn run(args: &PerfArgs) -> Result<(), Error> {
    fs::create_dir_all(&args.scratch)?;
    let scratch = absolute(&args.scratch)?;
    let (base_dir, base_deps) = (absolute(&args.base_dir)?, absolute(&args.base_deps)?);
    let (pr_dir, pr_deps) = (absolute(&args.pr_dir)?, absolute(&args.pr_deps)?);
    let base_side = Side {
        name: "base",
        dir: &base_dir,
        bin: &args.rari_bin,
        deps: &base_deps,
    };
    let pr_side = Side {
        name: "pr",
        dir: &pr_dir,
        bin: &args.rari_bin,
        deps: &pr_deps,
    };

    let (mut base, mut pr) = (Vec::new(), Vec::new());
    for round in 1..=args.rounds {
        // Alternate which side goes first, so drift on the runner affects both alike.
        if round % 2 == 1 {
            base.push(timed_build(&base_side, round, &scratch)?);
            pr.push(timed_build(&pr_side, round, &scratch)?);
        } else {
            pr.push(timed_build(&pr_side, round, &scratch)?);
            base.push(timed_build(&base_side, round, &scratch)?);
        }
    }

    let stats = PerfStats::new(&base, &pr);
    fs::write(&args.out, serde_json::to_vec_pretty(&stats.to_json())?)?;
    println!("Wrote {}", args.out.display());
    if let Some(summary) = &args.summary {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(summary)?
            .write_all(stats.to_markdown().as_bytes())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stat(min: f64, median: f64, max: f64) -> Stat {
        Stat { median, min, max }
    }

    fn sample(wall: f64, phases: Value) -> Sample {
        Sample {
            wall_seconds: wall,
            cpu_seconds: wall * 4.0,
            max_rss_bytes: 1024.0 * 1024.0 * 512.0,
            phases: phases.as_object().unwrap().clone(),
        }
    }

    #[test]
    fn stat_takes_median_min_max() {
        struct Case {
            name: &'static str,
            values: &'static [f64],
            expected: Stat,
        }

        let cases = [
            Case {
                name: "single value",
                values: &[2.0],
                expected: stat(2.0, 2.0, 2.0),
            },
            Case {
                name: "odd count, unsorted",
                values: &[3.0, 1.0, 2.0],
                expected: stat(1.0, 2.0, 3.0),
            },
            Case {
                name: "even count averages the middle",
                values: &[4.0, 1.0, 2.0, 3.0],
                expected: stat(1.0, 2.5, 4.0),
            },
        ];

        for case in cases {
            assert_eq!(
                Stat::new(case.values.iter().copied()),
                case.expected,
                "{}",
                case.name
            );
        }
    }

    #[test]
    fn verdict_requires_threshold_and_disjoint_ranges() {
        struct Case {
            name: &'static str,
            unit: Unit,
            base: Stat,
            pr: Stat,
            expected: Option<Verdict>,
        }

        let cases = [
            Case {
                name: "slower with disjoint ranges",
                unit: Unit::Seconds,
                base: stat(9.0, 10.0, 11.0),
                pr: stat(11.5, 12.0, 13.0),
                expected: Some(Verdict::Regression),
            },
            Case {
                name: "faster with disjoint ranges",
                unit: Unit::Seconds,
                base: stat(9.0, 10.0, 11.0),
                pr: stat(7.0, 8.0, 8.5),
                expected: Some(Verdict::Improvement),
            },
            Case {
                name: "slower with overlapping ranges",
                unit: Unit::Seconds,
                base: stat(9.0, 10.0, 11.0),
                pr: stat(10.5, 12.0, 13.0),
                expected: None,
            },
            Case {
                name: "faster with overlapping ranges",
                unit: Unit::Seconds,
                base: stat(9.0, 10.0, 11.0),
                pr: stat(7.0, 8.0, 9.5),
                expected: None,
            },
            Case {
                name: "disjoint ranges below the threshold",
                unit: Unit::Seconds,
                base: stat(10.0, 10.0, 10.0),
                pr: stat(10.4, 10.4, 10.4),
                expected: None,
            },
            Case {
                name: "short time below the absolute threshold",
                unit: Unit::Seconds,
                base: stat(0.04, 0.04, 0.04),
                pr: stat(0.03, 0.03, 0.03),
                expected: None,
            },
            Case {
                name: "bytes have no absolute threshold",
                unit: Unit::Bytes,
                base: stat(0.04, 0.04, 0.04),
                pr: stat(0.03, 0.03, 0.03),
                expected: Some(Verdict::Improvement),
            },
            Case {
                name: "zero base median",
                unit: Unit::Seconds,
                base: stat(0.0, 0.0, 0.0),
                pr: stat(1.0, 1.0, 1.0),
                expected: None,
            },
        ];

        for case in cases {
            let metric = Metric {
                base: case.base,
                pr: case.pr,
            };
            assert_eq!(metric.verdict(case.unit), case.expected, "{}", case.name);
        }
    }

    #[test]
    fn stats_keep_only_phases_of_all_runs() {
        let base = [
            sample(10.0, json!({"read_docs": 1.0, "build_spas": 2.0})),
            sample(10.0, json!({"read_docs": 1.0, "build_spas": 2.0})),
        ];
        let pr = [
            sample(10.0, json!({"read_docs": 1.0, "build_blog": 3.0})),
            sample(10.0, json!({"read_docs": 1.0, "build_blog": 3.0})),
        ];
        let stats = PerfStats::new(&base, &pr);
        let keys: Vec<&str> = stats.phases.iter().map(|(key, _)| key.as_str()).collect();
        assert_eq!(keys, ["read_docs"]);
    }

    #[test]
    fn markdown_reports_significant_changes() {
        let base = [10.0, 10.2, 9.8].map(|wall| sample(wall, json!({"read_docs": 1.0})));
        let pr = [12.0, 12.2, 11.8].map(|wall| sample(wall, json!({"read_docs": 1.0})));
        let markdown = PerfStats::new(&base, &pr).to_markdown();
        for expected in [
            "Median of 3 interleaved rounds per side",
            "| Wall time | 10.00s | 12.00s | +20.00% | 9.80s - 10.20s | 11.80s - 12.20s | regression |",
            "| Peak RSS | 512.00 MiB | 512.00 MiB | +0.00% | 512.00 MiB - 512.00 MiB | 512.00 MiB - 512.00 MiB |  |",
            "| `read_docs` | 1.00s | 1.00s | +0.00% | 1.00s - 1.00s | 1.00s - 1.00s |  |",
            "Regressed: Wall time, CPU time (user + sys).",
        ] {
            assert!(
                markdown.contains(expected),
                "missing {expected:?} in:\n{markdown}"
            );
        }
    }
}
