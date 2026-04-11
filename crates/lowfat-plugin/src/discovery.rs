use crate::manifest::PluginManifest;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// A discovered plugin with its manifest and location.
#[derive(Debug)]
pub struct DiscoveredPlugin {
    pub manifest: PluginManifest,
    pub base_dir: PathBuf,
    pub category: String,
}

/// Scan a plugin directory and discover all plugins with valid lowfat.toml or init.toml.
///
/// Directory structure:
///   plugin_dir/
///     category/
///       plugin-name/
///         lowfat.toml (or init.toml)
pub fn discover_plugins(plugin_dir: &Path) -> Vec<DiscoveredPlugin> {
    let mut plugins = Vec::new();
    scan_plugin_dir(plugin_dir, &mut plugins);
    plugins
}

fn scan_plugin_dir(dir: &Path, plugins: &mut Vec<DiscoveredPlugin>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for category_entry in entries.flatten() {
        let category_path = category_entry.path();
        if !category_path.is_dir() {
            continue;
        }
        let category = category_entry
            .file_name()
            .to_string_lossy()
            .to_string();

        let plugin_entries = match fs::read_dir(&category_path) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for plugin_entry in plugin_entries.flatten() {
            let plugin_path = plugin_entry.path();

            // Try lowfat.toml first, then init.toml for backwards compat
            let manifest_path = if plugin_path.join("lowfat.toml").is_file() {
                plugin_path.join("lowfat.toml")
            } else if plugin_path.join("init.toml").is_file() {
                plugin_path.join("init.toml")
            } else {
                continue;
            };

            let content = match fs::read_to_string(&manifest_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let manifest = match PluginManifest::parse(&content) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!(
                        "[lowfat] warning: invalid manifest at {}: {}",
                        manifest_path.display(),
                        e
                    );
                    continue;
                }
            };

            plugins.push(DiscoveredPlugin {
                manifest,
                base_dir: plugin_path,
                category,
            });
            break;
        }
    }
}

/// Build a command → plugin mapping. If multiple plugins claim the same command,
/// the last one wins.
pub fn resolve_plugins(plugins: &[DiscoveredPlugin]) -> HashMap<String, usize> {
    let mut map = HashMap::new();
    for (idx, plugin) in plugins.iter().enumerate() {
        for cmd in &plugin.manifest.plugin.commands {
            map.insert(cmd.clone(), idx);
        }
    }
    map
}
