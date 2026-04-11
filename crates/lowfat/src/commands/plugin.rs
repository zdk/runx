use anyhow::Result;
use lowfat_core::config::RunfConfig;
use lowfat_plugin::discovery::discover_plugins;

pub fn list() -> Result<()> {
    let config = RunfConfig::resolve();
    let plugins = discover_plugins(&config.plugin_dir);

    if plugins.is_empty() {
        println!("No community plugins installed.");
        println!("  Plugin dir: {}", config.plugin_dir.display());
        return Ok(());
    }

    println!("Community plugins:");
    println!();
    for plugin in &plugins {
        let m = &plugin.manifest;
        let name = &m.plugin.name;
        let version = m.plugin.version.as_deref().unwrap_or("?");
        let cmds = m.plugin.commands.join(", ");
        let category = &plugin.category;

        println!(
            "  {category}/{name} v{version} — commands: [{cmds}]"
        );
    }

    Ok(())
}

pub fn doctor() -> Result<()> {
    let config = RunfConfig::resolve();
    let plugins = discover_plugins(&config.plugin_dir);

    if plugins.is_empty() {
        println!("No community plugins to check.");
        return Ok(());
    }

    let mut ready = 0;
    let mut total = 0;

    for plugin in &plugins {
        total += 1;
        let name = &plugin.manifest.plugin.name;

        // Check entry file exists
        let entry_path = plugin.base_dir.join(&plugin.manifest.runtime.entry);
        if !entry_path.exists() {
            println!("  {name}    x entry not found: {}", entry_path.display());
            continue;
        }

        println!("  {name}    ok ready");
        ready += 1;
    }

    println!();
    println!("  {ready}/{total} plugins ready.");
    Ok(())
}

pub fn info(name: &str) -> Result<()> {
    let config = RunfConfig::resolve();
    let plugins = discover_plugins(&config.plugin_dir);

    let plugin = plugins
        .iter()
        .find(|p| p.manifest.plugin.name == name);

    match plugin {
        Some(p) => {
            let m = &p.manifest;
            println!("Plugin: {}", m.plugin.name);
            println!("  Version:     {}", m.plugin.version.as_deref().unwrap_or("?"));
            println!("  Description: {}", m.plugin.description.as_deref().unwrap_or("-"));
            println!("  Author:      {}", m.plugin.author.as_deref().unwrap_or("-"));
            println!("  Category:    {}", p.category);
            println!("  Entry:       {}", m.runtime.entry);
            println!("  Commands:    {}", m.plugin.commands.join(", "));
            println!("  Path:        {}", p.base_dir.display());
        }
        None => {
            eprintln!("lowfat: plugin not found: {name}");
        }
    }

    Ok(())
}

pub fn trust(name: &str) -> Result<()> {
    let config = RunfConfig::resolve();
    lowfat_plugin::security::trust_plugin(name, &config.home_dir)?;
    println!("lowfat: plugin '{name}' is now trusted");
    Ok(())
}

pub fn untrust(name: &str) -> Result<()> {
    let config = RunfConfig::resolve();
    lowfat_plugin::security::untrust_plugin(name, &config.home_dir)?;
    println!("lowfat: trust revoked for plugin '{name}'");
    Ok(())
}

pub fn new_plugin(name: &str, command: &str) -> Result<()> {
    let config = RunfConfig::resolve();

    // Create plugin directory: ~/.lowfat/plugins/<command>/<name>/
    let plugin_dir = config.plugin_dir.join(command).join(name);
    if plugin_dir.exists() {
        anyhow::bail!("plugin already exists: {}", plugin_dir.display());
    }
    std::fs::create_dir_all(&plugin_dir)?;

    // Write lowfat.toml manifest
    let manifest = format!(
        r#"[plugin]
name = "{name}"
commands = ["{command}"]

[runtime]
type = "shell"
entry = "filter.sh"
"#,
        name = name,
        command = command,
    );
    std::fs::write(plugin_dir.join("lowfat.toml"), manifest)?;

    // Write filter script
    std::fs::write(plugin_dir.join("filter.sh"), scaffold_shell())?;

    // Scaffold samples/ directory
    let samples_dir = plugin_dir.join("samples");
    std::fs::create_dir_all(&samples_dir)?;
    std::fs::write(
        samples_dir.join(format!("{command}-output-full.txt")),
        "# Paste real command output here.\n# Filename convention: <command>-<subcommand>-<level>.txt\n# Run: lowfat plugin bench <name>\n",
    )?;

    // Auto-trust the plugin
    lowfat_plugin::security::trust_plugin(name, &config.home_dir)?;

    println!("lowfat: created plugin '{name}'");
    println!("  {}", plugin_dir.display());
    println!("  edit: {}", plugin_dir.join("filter.sh").display());
    println!("  bench: lowfat plugin bench {name}");
    println!("  test: lowfat {command} <args>");
    Ok(())
}

fn scaffold_shell() -> String {
    r#"#!/bin/sh
# lowfat plugin — reads raw output from stdin, writes filtered output to stdout
# env: $LOWFAT_LEVEL (lite|full|ultra), $LOWFAT_COMMAND, $LOWFAT_SUBCOMMAND, $LOWFAT_ARGS, $LOWFAT_EXIT_CODE
#
# Level convention:
#   lite  — gentle trim, keep most output (~60 lines)
#   full  — balanced, strip noise (~30 lines)
#   ultra — summary only, minimal output (~10 lines)

LEVEL="${LOWFAT_LEVEL:-full}"
SUB="$LOWFAT_SUBCOMMAND"

case "$LEVEL" in
  lite)  head -n 60 ;;
  ultra) head -n 10 ;;
  *)     head -n 30 ;;
esac
"#
    .to_string()
}

pub fn bench(name: &str) -> Result<()> {
    let config = RunfConfig::resolve();
    let plugins = discover_plugins(&config.plugin_dir);

    let plugin = plugins
        .iter()
        .find(|p| p.manifest.plugin.name == name);

    let plugin = match plugin {
        Some(p) => p,
        None => {
            // Also check repo plugins/ directory
            anyhow::bail!("plugin not found: {name} (install it to ~/.lowfat/plugins/ first)");
        }
    };

    let samples_dir = plugin.base_dir.join("samples");
    if !samples_dir.is_dir() {
        anyhow::bail!("no samples/ directory in plugin '{name}' — add .txt files with sample command output");
    }

    let mut entries: Vec<_> = std::fs::read_dir(&samples_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "txt"))
        .collect();
    entries.sort_by_key(|e| e.path());

    if entries.is_empty() {
        anyhow::bail!("no .txt sample files in {}", samples_dir.display());
    }

    // Build the process filter
    let process_filter = lowfat_runner::process::ProcessFilter {
        info: lowfat_plugin::plugin::PluginInfo {
            name: plugin.manifest.plugin.name.clone(),
            version: plugin.manifest.plugin.version.clone().unwrap_or_default(),
            commands: plugin.manifest.plugin.commands.clone(),
            subcommands: plugin.manifest.plugin.subcommands.clone().unwrap_or_default(),
        },
        entry: plugin.base_dir.join(&plugin.manifest.runtime.entry),
        base_dir: plugin.base_dir.clone(),
    };

    println!("Benchmark: {name}");
    println!();

    let mut total_raw = 0usize;
    let mut total_filtered = 0usize;

    for entry in &entries {
        let path = entry.path();
        let sample_name = path.file_stem().unwrap_or_default().to_string_lossy();

        // Parse sample name: "git-status-full.txt" → command=git, subcommand=status, level=full
        let parts: Vec<&str> = sample_name.split('-').collect();
        let (command, subcommand, level_str) = match parts.len() {
            1 => (parts[0], "", "full"),
            2 => (parts[0], parts[1], "full"),
            _ => (parts[0], parts[1], parts[parts.len() - 1]),
        };

        let level = match level_str {
            "lite" => lowfat_core::level::Level::Lite,
            "ultra" => lowfat_core::level::Level::Ultra,
            _ => lowfat_core::level::Level::Full,
        };

        let raw = std::fs::read_to_string(&path)?;
        let raw_tokens = lowfat_core::tokens::estimate_tokens(&raw);

        let input = lowfat_plugin::plugin::FilterInput {
            raw: raw.clone(),
            command: command.to_string(),
            subcommand: subcommand.to_string(),
            args: vec![],
            level,
            head_limit: level.head_limit(40),
            exit_code: 0,
        };

        use lowfat_plugin::plugin::FilterPlugin;
        let result = process_filter.filter(&input)?;
        let filtered_tokens = lowfat_core::tokens::estimate_tokens(&result.text);
        let pct = if raw_tokens > 0 {
            (1.0 - filtered_tokens as f64 / raw_tokens as f64) * 100.0
        } else {
            0.0
        };

        total_raw += raw_tokens;
        total_filtered += filtered_tokens;

        println!(
            "  {:<30} {:>6} → {:>6} tokens  ({:>-3.0}%)",
            format!("{sample_name} ({level})"), raw_tokens, filtered_tokens, -pct
        );
    }

    if total_raw > 0 {
        let total_pct = (1.0 - total_filtered as f64 / total_raw as f64) * 100.0;
        println!();
        println!(
            "  {:<30} {:>6} → {:>6} tokens  ({:>-3.0}%)",
            "TOTAL", total_raw, total_filtered, -total_pct
        );
    }

    Ok(())
}