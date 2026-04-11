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
                // Extract last field (filename) from each line
                input
                    .raw
                    .lines()
                    .filter(|l| !l.starts_with("total ") && !l.is_empty())
                    .filter_map(|l| l.split_whitespace().last())
                    .take(limit)
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            _ => {
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
}
