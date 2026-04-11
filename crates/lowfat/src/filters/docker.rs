//! Native docker filter — compact ps, images, logs, build, pull, compose output.

use anyhow::Result;
use lowfat_core::level::Level;
use lowfat_plugin::plugin::{FilterInput, FilterOutput, FilterPlugin, PluginInfo};

pub struct DockerFilter;

impl FilterPlugin for DockerFilter {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "docker-compact".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            commands: vec!["docker".into()],
            subcommands: vec![
                "ps".into(),
                "images".into(),
                "logs".into(),
                "build".into(),
                "pull".into(),
                "compose".into(),
            ],
        }
    }

    fn filter(&self, input: &FilterInput) -> Result<FilterOutput> {
        let text = match input.subcommand.as_str() {
            "ps" => filter_ps(&input.raw, input.level),
            "images" => filter_images(&input.raw, input.level),
            "logs" => filter_logs(&input.raw, input.level),
            "build" => filter_build(&input.raw, input.level),
            "pull" => filter_pull(&input.raw, input.level),
            "compose" => filter_compose(&input.raw, input.level),
            _ => head_nonblank(&input.raw, input.level.head_limit(40)),
        };
        Ok(FilterOutput {
            passthrough: text.is_empty(),
            text,
        })
    }
}

/// Collapse multi-space columns; ultra extracts name + status only.
fn filter_ps(raw: &str, level: Level) -> String {
    let limit = level.head_limit(40);
    match level {
        Level::Ultra => {
            let mut out = vec!["NAME STATUS".to_string()];
            for line in raw.lines().skip(1).take(limit) {
                let cols: Vec<&str> = line.split_whitespace().collect();
                if cols.len() >= 5 {
                    let name = cols.last().unwrap_or(&"");
                    // Status is typically 2 cols before last
                    let status = cols.get(cols.len().saturating_sub(3)).unwrap_or(&"");
                    out.push(format!("{name} {status}"));
                }
            }
            out.join("\n")
        }
        _ => collapse_spaces(raw, limit),
    }
}

/// Collapse multi-space columns; ultra extracts repo + tag + size.
fn filter_images(raw: &str, level: Level) -> String {
    let limit = level.head_limit(40);
    match level {
        Level::Ultra => {
            let mut out = vec!["REPO TAG SIZE".to_string()];
            for line in raw.lines().skip(1).take(limit) {
                let cols: Vec<&str> = line.split_whitespace().collect();
                if cols.len() >= 4 {
                    let repo = cols[0];
                    let tag = cols[1];
                    // Size is second-to-last column
                    let size = cols.get(cols.len().saturating_sub(2)).unwrap_or(&"");
                    out.push(format!("{repo} {tag} {size}"));
                }
            }
            out.join("\n")
        }
        _ => collapse_spaces(raw, limit),
    }
}

/// Show last N lines of logs.
fn filter_logs(raw: &str, level: Level) -> String {
    let n = match level {
        Level::Lite => 60,
        Level::Full => 30,
        Level::Ultra => 10,
    };
    let lines: Vec<&str> = raw.lines().collect();
    let start = lines.len().saturating_sub(n);
    lines[start..].join("\n")
}

/// Strip build noise; ultra keeps only result lines.
fn filter_build(raw: &str, level: Level) -> String {
    match level {
        Level::Ultra => {
            let result: Vec<&str> = raw
                .lines()
                .filter(|l| {
                    l.starts_with("Successfully")
                        || l.starts_with("ERROR")
                        || l.starts_with("ERRO")
                        || l.contains("naming to")
                        || l.contains("exporting to")
                })
                .collect();
            let tail: Vec<&str> = result.iter().rev().take(3).rev().copied().collect();
            if tail.is_empty() {
                "docker build: ok".into()
            } else {
                tail.join("\n")
            }
        }
        _ => {
            let limit = level.head_limit(50);
            let filtered: Vec<&str> = raw
                .lines()
                .filter(|l| {
                    let t = l.trim_start();
                    !t.starts_with("--->")
                        && !t.starts_with("Removing intermediate")
                        && !t.starts_with("Successfully tagged")
                        && !starts_with_cached(t)
                })
                .filter(|l| !l.is_empty())
                .collect();
            let start = filtered.len().saturating_sub(limit);
            let result = filtered[start..].join("\n");
            if result.is_empty() {
                "docker build: ok".into()
            } else {
                result
            }
        }
    }
}

/// Strip layer progress; ultra keeps Status/Digest only.
fn filter_pull(raw: &str, level: Level) -> String {
    match level {
        Level::Ultra => {
            let result: Vec<&str> = raw
                .lines()
                .filter(|l| l.starts_with("Status:") || l.starts_with("Digest:"))
                .collect();
            let tail: Vec<&str> = result.iter().rev().take(2).rev().copied().collect();
            if tail.is_empty() {
                "docker pull: ok".into()
            } else {
                tail.join("\n")
            }
        }
        _ => {
            let filtered: Vec<&str> = raw
                .lines()
                .filter(|l| !is_pull_progress(l))
                .filter(|l| !l.is_empty())
                .take(10)
                .collect();
            if filtered.is_empty() {
                "docker pull: ok".into()
            } else {
                filtered.join("\n")
            }
        }
    }
}

/// Filter compose noise.
fn filter_compose(raw: &str, level: Level) -> String {
    let limit = level.head_limit(30);
    let filtered: Vec<&str> = raw
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            !t.starts_with("Pulling")
                && !t.starts_with("Creating")
                && !t.starts_with("Starting")
                && !t.starts_with("Waiting")
        })
        .filter(|l| !l.is_empty())
        .take(limit)
        .collect();
    if filtered.is_empty() {
        "docker compose: ok".into()
    } else {
        filtered.join("\n")
    }
}

// --- helpers ---

fn collapse_spaces(raw: &str, limit: usize) -> String {
    raw.lines()
        .take(limit)
        .map(|l| {
            // Replace runs of 2+ spaces with single space
            let mut result = String::with_capacity(l.len());
            let mut prev_space = false;
            for c in l.chars() {
                if c == ' ' {
                    if !prev_space {
                        result.push(' ');
                    }
                    prev_space = true;
                } else {
                    prev_space = false;
                    result.push(c);
                }
            }
            result
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn starts_with_cached(s: &str) -> bool {
    // Lines like "#12 CACHED" or "#5 sha256:..."
    s.starts_with('#')
        && s.get(1..2).map_or(false, |c| c.chars().next().map_or(false, |ch| ch.is_ascii_digit()))
        && (s.contains("CACHED") || s.contains("sha256:"))
}

fn is_pull_progress(l: &str) -> bool {
    // Layer progress: "abc123: Pulling fs layer", "abc123: Downloading", etc.
    let Some((prefix, rest)) = l.split_once(": ") else {
        return false;
    };
    prefix.chars().all(|c| c.is_ascii_hexdigit())
        && (rest.starts_with("Pulling")
            || rest.starts_with("Waiting")
            || rest.starts_with("Downloading")
            || rest.starts_with("Extracting")
            || rest.starts_with("Verifying")
            || rest.starts_with("Pull complete"))
}

fn head_nonblank(raw: &str, limit: usize) -> String {
    raw.lines()
        .filter(|l| !l.is_empty())
        .take(limit)
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ps_ultra_extracts_name_status() {
        let raw = "CONTAINER ID  IMAGE  COMMAND  CREATED  STATUS  PORTS  NAMES\nabc123  nginx  \"/docker\"  2h  Up 2h  80/tcp  web\n";
        let out = filter_ps(raw, Level::Ultra);
        assert!(out.contains("NAME STATUS"));
        assert!(out.contains("web"));
    }

    #[test]
    fn build_ultra_ok_fallback() {
        let raw = "#1 [internal] load build definition\n#2 CACHED\n";
        let out = filter_build(raw, Level::Ultra);
        assert_eq!(out, "docker build: ok");
    }

    #[test]
    fn pull_ultra_status() {
        let raw = "abc: Pulling fs layer\nDigest: sha256:abc\nStatus: Downloaded newer image\n";
        let out = filter_pull(raw, Level::Ultra);
        assert!(out.contains("Digest:"));
        assert!(out.contains("Status:"));
    }

    #[test]
    fn logs_respects_level() {
        let raw = (0..100).map(|i| format!("line{i}")).collect::<Vec<_>>().join("\n");
        let out = filter_logs(&raw, Level::Ultra);
        assert_eq!(out.lines().count(), 10);
    }
}
