use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::Error;
use serde_json::{Map, Value, json};

/// Collects build phase durations for `rari build --timings`.
pub struct Timings {
    start: Instant,
    phases: Vec<(&'static str, Duration)>,
}

impl Timings {
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
            phases: Vec::new(),
        }
    }

    /// Records the time since `start` as `phase` and returns it.
    pub fn record(&mut self, phase: &'static str, start: Instant) -> Duration {
        let elapsed = start.elapsed();
        self.phases.push((phase, elapsed));
        elapsed
    }

    fn to_json(&self, wall: Duration, usage: Option<ResourceUsage>) -> Value {
        let phases: Map<String, Value> = self
            .phases
            .iter()
            .map(|(phase, elapsed)| (phase.to_string(), json!(elapsed.as_secs_f64())))
            .collect();
        json!({
            "wall_seconds": wall.as_secs_f64(),
            "cpu_seconds": usage.map(|u| u.cpu_seconds),
            "max_rss_bytes": usage.map(|u| u.max_rss_bytes),
            "phases": phases,
        })
    }

    pub fn write(&self, path: &Path) -> Result<(), Error> {
        let value = self.to_json(self.start.elapsed(), resource_usage());
        serde_json::to_writer_pretty(BufWriter::new(File::create(path)?), &value)?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct ResourceUsage {
    cpu_seconds: f64,
    max_rss_bytes: u64,
}

#[cfg(unix)]
fn resource_usage() -> Option<ResourceUsage> {
    let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
    // SAFETY: `getrusage` initializes the struct when it returns 0.
    let usage = unsafe {
        if libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) != 0 {
            return None;
        }
        usage.assume_init()
    };
    let secs = |tv: libc::timeval| tv.tv_sec as f64 + tv.tv_usec as f64 / 1e6;
    // `ru_maxrss` is in bytes on macOS and in kilobytes elsewhere.
    let unit = if cfg!(target_vendor = "apple") {
        1
    } else {
        1024
    };
    Some(ResourceUsage {
        cpu_seconds: secs(usage.ru_utime) + secs(usage.ru_stime),
        max_rss_bytes: usage.ru_maxrss as u64 * unit,
    })
}

#[cfg(not(unix))]
fn resource_usage() -> Option<ResourceUsage> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_phases_in_recorded_order() {
        let timings = Timings {
            start: Instant::now(),
            phases: vec![
                ("read_docs", Duration::from_millis(1500)),
                ("build_content_docs", Duration::from_secs(4)),
            ],
        };
        let usage = ResourceUsage {
            cpu_seconds: 12.5,
            max_rss_bytes: 2048,
        };
        assert_eq!(
            serde_json::to_string(&timings.to_json(Duration::from_secs(6), Some(usage))).unwrap(),
            r#"{"wall_seconds":6.0,"cpu_seconds":12.5,"max_rss_bytes":2048,"phases":{"read_docs":1.5,"build_content_docs":4.0}}"#
        );
    }

    #[test]
    fn serializes_missing_resource_usage_as_null() {
        let value = Timings::start().to_json(Duration::ZERO, None);
        assert_eq!(value["cpu_seconds"], Value::Null);
        assert_eq!(value["max_rss_bytes"], Value::Null);
    }
}
