use crate::filters;
use anyhow::Result;
use lowfat_core::config::RunfConfig;
use lowfat_core::pipeline::Pipeline;
use lowfat_plugin::discovery::{discover_plugins, resolve_plugins};

/// Show active pipeline for a command.
pub fn run(cmd: &str) -> Result<()> {
    let config = RunfConfig::resolve();

    println!("Pipeline for '{cmd}' (level: {}):", config.level);
    println!();

    // Check .lowfat conditional pipelines
    if let Some(conditional) = config.pipeline_for(cmd) {
        if let Some(ref p) = conditional.default {
            println!("  default:  {}", p.display());
        }
        if let Some(ref p) = conditional.on_error {
            println!("  on error: {}", p.display());
        }
        if let Some(ref p) = conditional.on_empty {
            println!("  on empty: {}", p.display());
        }
        if let Some(ref p) = conditional.on_large {
            println!("  on large: {}", p.display());
        }
        return Ok(());
    }

    // Check native builtins
    let all_filters = filters::builtins();
    if let Some(plugin) = all_filters.get(cmd) {
        let name = plugin.info().name;
        println!("  default:  {name} (native)");
        println!();
        println!("Configure a pipeline in .lowfat:");
        println!("  pipeline.{cmd} = strip-ansi, {name}, truncate");
        println!("  pipeline.{cmd}.error = strip-ansi, head:20");
        println!("  pipeline.{cmd}.large = {name}, token-budget:1500");
        println!();
        println!("Parameterized stages: truncate:N, head:N, token-budget:N");
        return Ok(());
    }

    // Check community plugins
    let plugins = discover_plugins(&config.plugin_dir);
    let cmd_map = resolve_plugins(&plugins);

    if let Some(&idx) = cmd_map.get(cmd) {
        let plugin = &plugins[idx];
        let name = &plugin.manifest.plugin.name;

        if let Some(ref pipe_config) = plugin.manifest.pipeline {
            let pre = pipe_config.pre.as_deref().unwrap_or(&[]);
            let post = pipe_config.post.as_deref().unwrap_or(&[]);
            if !pre.is_empty() || !post.is_empty() {
                let p = Pipeline::from_parts(pre, name, post);
                println!("  default:  {} (from lowfat.toml)", p.display());
                return Ok(());
            }
        }

        println!("  default:  {name} (community plugin)");
        return Ok(());
    }

    println!("  (no filter for '{cmd}')");
    Ok(())
}
