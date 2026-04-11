//! Native built-in filters — compiled into the binary, zero subprocess overhead.

mod docker;
mod git;
mod ls;

use lowfat_plugin::plugin::FilterPlugin;
use std::collections::HashMap;

/// Return all native built-in filters.
/// Keyed by both command name ("git") and plugin name ("git-compact")
/// so pipeline stages can reference either.
pub fn builtins() -> HashMap<String, Box<dyn FilterPlugin>> {
    let filters: Vec<(String, Box<dyn FilterPlugin>)> = vec![
        ("git".into(), Box::new(git::GitFilter)),
        ("docker".into(), Box::new(docker::DockerFilter)),
        ("ls".into(), Box::new(ls::LsFilter)),
    ];

    let mut map: HashMap<String, Box<dyn FilterPlugin>> = HashMap::new();
    for (cmd, filter) in filters {
        let plugin_name = filter.info().name.clone();
        // Register under plugin name (used by pipeline stages)
        map.insert(plugin_name, filter);
        // Register a second instance under command name (used by run.rs lookup)
        let filter2: Box<dyn FilterPlugin> = match cmd.as_str() {
            "git" => Box::new(git::GitFilter),
            "docker" => Box::new(docker::DockerFilter),
            "ls" => Box::new(ls::LsFilter),
            _ => unreachable!(),
        };
        map.insert(cmd, filter2);
    }
    map
}
