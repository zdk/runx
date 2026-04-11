use crate::filters;
use anyhow::Result;
use lowfat_core::config::RunfConfig;
use lowfat_plugin::discovery::{discover_plugins, DiscoveredPlugin};
use std::collections::BTreeMap;
use std::fmt::Write as _;

pub fn run(commands_only: bool) -> Result<()> {
    let config = RunfConfig::resolve();
    let plugins = discover_plugins(&config.plugin_dir);

    if commands_only {
        // Print one command per line — used by shell-init
        let cmds = collect_commands(&config, &plugins);
        for cmd in cmds {
            println!("{cmd}");
        }
        return Ok(());
    }

    let output = format_filters(&config, &plugins);
    print!("{output}");
    Ok(())
}

/// Collect all wrappable command names from builtins, plugins, and pipelines.
fn collect_commands(config: &RunfConfig, plugins: &[DiscoveredPlugin]) -> Vec<String> {
    use std::collections::BTreeSet;

    let mut cmds = BTreeSet::new();

    // Native built-in filters
    let all_filters = filters::builtins();
    for (cmd, plugin) in &all_filters {
        // Skip internal aliases (filter name itself, e.g. "git-compact")
        if cmd != &plugin.info().name {
            cmds.insert(cmd.clone());
        }
    }

    // External plugins
    for plugin in plugins {
        for cmd in &plugin.manifest.plugin.commands {
            cmds.insert(cmd.clone());
        }
    }

    // Pipeline declarations
    for cmd in config.pipelines.keys() {
        cmds.insert(cmd.clone());
    }

    // Only return enabled commands
    cmds.into_iter().filter(|c| config.is_enabled(c)).collect()
}

/// Format filter listing — testable without side effects.
fn format_filters(config: &RunfConfig, plugins: &[DiscoveredPlugin]) -> String {
    let mut out = String::new();

    writeln!(out, "Filters (lowfat):").unwrap();
    if let Some(cfg_path) = lowfat_core::config::find_config_display() {
        writeln!(out, "  config: {}", cfg_path.display()).unwrap();
    }
    writeln!(out, "  level: {}", config.level).unwrap();
    writeln!(out).unwrap();

    // Native built-in filters — deduplicate by grouping commands under filter name
    let all_filters = filters::builtins();
    let mut native: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (cmd, plugin) in &all_filters {
        let name = plugin.info().name.clone();
        // Skip internal alias (e.g. "git-compact") — only show real commands
        if cmd == &name {
            continue;
        }
        native.entry(name).or_default().push(cmd.clone());
    }
    if !native.is_empty() {
        writeln!(out, "  built-in:").unwrap();
        for (name, cmds) in &native {
            let enabled = cmds.iter().all(|c| config.is_enabled(c));
            format_filter(&mut out, name, &cmds.join(", "), enabled);
        }
    }

    // External plugins
    if !plugins.is_empty() {
        writeln!(out).unwrap();
        writeln!(out, "  plugins:").unwrap();
        for plugin in plugins {
            let name = &plugin.manifest.plugin.name;
            let cmds = plugin.manifest.plugin.commands.join(", ");
            let enabled = plugin
                .manifest
                .plugin
                .commands
                .iter()
                .all(|c| config.is_enabled(c));
            format_filter(&mut out, name, &cmds, enabled);
        }
    }

    if native.is_empty() && plugins.is_empty() {
        writeln!(out, "  (no filters found)").unwrap();
    }

    out
}

fn format_filter(out: &mut String, name: &str, cmds: &str, enabled: bool) {
    if enabled {
        writeln!(out, "  \x1b[92m●\x1b[0m {name}  {cmds}").unwrap();
    } else {
        writeln!(out, "  \x1b[2m○ {name}  {cmds}\x1b[0m").unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lowfat_core::config::RunfConfig;
    use lowfat_core::level::Level;
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;

    fn default_config() -> RunfConfig {
        RunfConfig {
            level: Level::Full,
            disabled: HashSet::new(),
            allowed: None,
            data_dir: PathBuf::new(),
            plugin_dir: PathBuf::new(),
            home_dir: PathBuf::new(),
            pipelines: HashMap::new(),
        }
    }

    fn config_with_disabled(disabled: &[&str]) -> RunfConfig {
        let mut config = default_config();
        for d in disabled {
            config.disabled.insert(d.to_string());
        }
        config
    }

    #[test]
    fn shows_builtin_filters() {
        let config = default_config();
        let output = format_filters(&config, &[]);
        assert!(output.contains("built-in:"));
        assert!(output.contains("git-compact"));
        assert!(output.contains("docker-compact"));
        assert!(output.contains("ls-compact"));
    }

    #[test]
    fn disabled_filter_uses_dim_marker() {
        let config = config_with_disabled(&["git"]);
        let output = format_filters(&config, &[]);
        // git-compact line should use dim marker ○
        for line in output.lines() {
            if line.contains("git-compact") {
                assert!(line.contains("○"), "disabled filter should use ○ marker");
                return;
            }
        }
        panic!("git-compact not found in output");
    }

    #[test]
    fn enabled_filter_uses_green_marker() {
        let config = default_config();
        let output = format_filters(&config, &[]);
        for line in output.lines() {
            if line.contains("git-compact") {
                assert!(line.contains("●"), "enabled filter should use ● marker");
                return;
            }
        }
        panic!("git-compact not found in output");
    }

    #[test]
    fn no_duplicate_entries() {
        let config = default_config();
        let output = format_filters(&config, &[]);
        // Each filter name should appear exactly once
        let git_count = output.matches("git-compact").count();
        assert_eq!(git_count, 1, "git-compact should appear once, got {git_count}");
    }

    #[test]
    fn hides_internal_alias() {
        let config = default_config();
        let output = format_filters(&config, &[]);
        // "docker-compact" line should show "docker" as the command, not "docker-compact"
        for line in output.lines() {
            if line.contains("docker-compact") {
                // The command list should be just "docker", not "docker-compact, docker"
                let after_name = line.split("docker-compact").nth(1).unwrap();
                assert!(
                    !after_name.contains("docker-compact"),
                    "should not show internal alias in command list"
                );
                return;
            }
        }
        panic!("docker-compact not found in output");
    }

    #[test]
    fn shows_level() {
        let config = default_config();
        let output = format_filters(&config, &[]);
        assert!(output.contains("level: full"));
    }
}
