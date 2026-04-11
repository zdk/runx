use anyhow::Result;
use lowfat_core::level::Level;

/// Input to a filter plugin.
#[derive(Debug, Clone)]
pub struct FilterInput {
    /// Raw command output (stdout + stderr merged)
    pub raw: String,
    /// Command name (e.g., "git")
    pub command: String,
    /// Subcommand (e.g., "status"), empty if none
    pub subcommand: String,
    /// Full argument list
    pub args: Vec<String>,
    /// Current intensity level
    pub level: Level,
    /// Head limit for this level
    pub head_limit: usize,
    /// Command exit code
    pub exit_code: i32,
}

/// Output from a filter plugin.
#[derive(Debug, Clone)]
pub struct FilterOutput {
    /// Filtered text
    pub text: String,
    /// Whether filter passed through without changes
    pub passthrough: bool,
}

/// Info about a loaded plugin.
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub commands: Vec<String>,
    pub subcommands: Vec<String>,
}

/// Core trait for filter plugins. Implemented by both ProcessRunner and WasmRunner.
pub trait FilterPlugin: Send + Sync {
    /// Plugin metadata.
    fn info(&self) -> PluginInfo;
    /// Apply filter to command output.
    fn filter(&self, input: &FilterInput) -> Result<FilterOutput>;
}
