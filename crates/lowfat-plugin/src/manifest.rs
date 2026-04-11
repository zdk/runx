use serde::Deserialize;

/// Parsed `lowfat.toml` (or `init.toml`) plugin manifest.
#[derive(Debug, Deserialize)]
pub struct PluginManifest {
    pub plugin: PluginMeta,
    #[serde(default)]
    pub runtime: RuntimeConfig,
    pub hooks: Option<HooksConfig>,
    pub pipeline: Option<PipelineConfig>,
}

#[derive(Debug, Deserialize)]
pub struct PluginMeta {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub category: Option<String>,
    /// Which commands this plugin intercepts (e.g., ["git"])
    pub commands: Vec<String>,
    /// Optional: limit to specific subcommands
    pub subcommands: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct RuntimeConfig {
    /// Entrypoint relative to plugin dir (default: "filter.sh")
    #[serde(default = "default_entry")]
    pub entry: String,
}

fn default_entry() -> String {
    "filter.sh".to_string()
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self { entry: default_entry() }
    }
}

#[derive(Debug, Deserialize)]
pub struct HooksConfig {
    pub on_install: Option<String>,
    pub on_update: Option<String>,
    pub on_remove: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PipelineConfig {
    pub pre: Option<Vec<String>>,
    pub post: Option<Vec<String>>,
}

impl PluginManifest {
    pub fn parse(content: &str) -> anyhow::Result<Self> {
        Ok(toml::from_str(content)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_manifest() {
        let toml = r#"
[plugin]
name = "git-compact"
commands = ["git"]

[runtime]
entry = "filter.sh"
"#;
        let manifest = PluginManifest::parse(toml).unwrap();
        assert_eq!(manifest.plugin.name, "git-compact");
        assert_eq!(manifest.plugin.commands, vec!["git"]);
        assert_eq!(manifest.runtime.entry, "filter.sh");
    }

    #[test]
    fn parse_minimal_manifest_no_runtime() {
        let toml = r#"
[plugin]
name = "git-compact"
commands = ["git"]
"#;
        let manifest = PluginManifest::parse(toml).unwrap();
        assert_eq!(manifest.plugin.name, "git-compact");
        assert_eq!(manifest.runtime.entry, "filter.sh");
    }

    #[test]
    fn parse_full_manifest() {
        let toml = r#"
[plugin]
name = "git-compact"
version = "1.2.0"
description = "Compact git output for LLM contexts"
author = "zdk"
category = "git"
commands = ["git"]
subcommands = ["status", "diff", "log", "show"]

[runtime]
entry = "filter.sh"

[hooks]
on_install = "chmod +x filter.sh"

[pipeline]
pre = ["strip-ansi"]
post = ["truncate"]
"#;
        let manifest = PluginManifest::parse(toml).unwrap();
        assert_eq!(manifest.plugin.name, "git-compact");
        assert!(manifest.hooks.is_some());
        assert!(manifest.pipeline.is_some());
    }
}
