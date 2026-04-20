//! Native ls filter — compact directory listing.

use anyhow::Result;
use lowfat_core::level::Level;
use lowfat_plugin::plugin::{FilterInput, FilterOutput, FilterPlugin, PluginInfo};

pub struct LsFilter;

impl FilterPlugin for LsFilter {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "ls-compact".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            commands: vec!["ls".into()],
            subcommands: vec![],
        }
    }

    fn filter(&self, input: &FilterInput) -> Result<FilterOutput> {
        let limit = input.level.head_limit(40);
        let text = match input.level {
            Level::Ultra => {
                // Filenames only
                input
                    .raw
                    .lines()
                    .filter(|l| !l.starts_with("total ") && !l.is_empty())
                    .filter_map(|l| l.split_whitespace().last())
                    .take(limit)
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            Level::Full => {
                // Long-form (`ls -l`) lines get compacted to `<type> <size> <name>`;
                // short-form output (plain `ls`) passes through unchanged.
                input
                    .raw
                    .lines()
                    .filter(|l| !l.starts_with("total ") && !l.is_empty())
                    .map(|l| {
                        if is_long_form(l) {
                            compact_long_form(l)
                        } else {
                            l.to_string()
                        }
                    })
                    .take(limit)
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            Level::Lite => {
                // Minimal — drop only the `total N` header and blanks.
                input
                    .raw
                    .lines()
                    .filter(|l| !l.starts_with("total ") && !l.is_empty())
                    .take(limit)
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        };
        Ok(FilterOutput {
            passthrough: text.is_empty(),
            text,
        })
    }
}

/// Heuristic: a long-form `ls -l` line starts with a file-type char followed
/// by a permission bit.
fn is_long_form(line: &str) -> bool {
    let bytes = line.as_bytes();
    if bytes.len() < 10 {
        return false;
    }
    matches!(bytes[0], b'-' | b'd' | b'l' | b'b' | b'c' | b'p' | b's')
        && matches!(bytes[1], b'-' | b'r')
}

/// Collapse a long-form line to `<type> <size> <name>`. Falls back to the
/// original line if the field count looks off.
fn compact_long_form(line: &str) -> String {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 9 {
        return line.to_string();
    }
    let type_char = parts[0].chars().next().unwrap_or(' ');
    let size = parts[4];
    // Names can contain spaces — rejoin everything after the timestamp field.
    let name = parts[8..].join(" ");
    format!("{type_char} {size} {name}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use lowfat_core::level::Level;

    fn make_input(raw: &str, level: Level) -> FilterInput {
        FilterInput {
            raw: raw.into(),
            command: "ls".into(),
            subcommand: String::new(),
            args: vec!["-la".into()],
            level,
            head_limit: level.head_limit(40),
            exit_code: 0,
        }
    }

    #[test]
    fn strips_total_line() {
        let raw = "total 48\n-rw-r--r-- 1 user staff 100 Jan 1 main.rs\n";
        let out = LsFilter.filter(&make_input(raw, Level::Full)).unwrap();
        assert!(!out.text.contains("total 48"));
        assert!(out.text.contains("main.rs"));
    }

    #[test]
    fn ultra_filenames_only() {
        let raw = "total 16\n-rw-r--r-- 1 user staff 100 Jan 1 main.rs\ndrwxr-xr-x 3 user staff 96 Jan 1 src\n";
        let out = LsFilter.filter(&make_input(raw, Level::Ultra)).unwrap();
        assert_eq!(out.text, "main.rs\nsrc");
    }

    #[test]
    fn full_compacts_long_form() {
        let raw = "total 16\n\
            -rw-r--r--  1 user  staff   100 Jan  1 12:34 main.rs\n\
            drwxr-xr-x  3 user  staff    96 Jan  1 12:34 src\n";
        let out = LsFilter.filter(&make_input(raw, Level::Full)).unwrap();
        assert_eq!(out.text, "- 100 main.rs\nd 96 src");
    }

    #[test]
    fn full_passes_short_form_unchanged() {
        // Plain `ls` (no -l) — filter should not corrupt filenames.
        let raw = "main.rs\nCargo.toml\nsrc\n";
        let out = LsFilter.filter(&make_input(raw, Level::Full)).unwrap();
        assert_eq!(out.text, "main.rs\nCargo.toml\nsrc");
    }

    #[test]
    fn lite_keeps_full_metadata() {
        let raw = "total 16\n-rw-r--r--  1 user  staff   100 Jan  1 12:34 main.rs\n";
        let out = LsFilter.filter(&make_input(raw, Level::Lite)).unwrap();
        assert!(out.text.contains("-rw-r--r--"));
        assert!(out.text.contains("main.rs"));
        assert!(!out.text.contains("total 16"));
    }
}
