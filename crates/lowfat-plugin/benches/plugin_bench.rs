use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lowfat_plugin::manifest::PluginManifest;

const MINIMAL_MANIFEST: &str = r#"
[plugin]
name = "git-compact"
commands = ["git"]

[runtime]
type = "shell"
entry = "filter.sh"
"#;

const FULL_MANIFEST: &str = r#"
[plugin]
name = "git-compact"
version = "1.2.0"
description = "Compact git output for LLM contexts"
author = "zdk"
category = "git"
commands = ["git"]
subcommands = ["status", "diff", "log", "show"]

[runtime]
type = "shell"
entry = "filter.sh"

[runtime.requires]
bins = ["git"]
optional_bins = ["delta"]

[input]
format = "raw"

[result]
format = "raw"

[hooks]
on_install = "echo installed"

[pipeline]
pre = ["strip-ansi"]
post = ["truncate"]
"#;

fn bench_manifest_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("manifest_parse");
    group.bench_function("minimal", |b| {
        b.iter(|| PluginManifest::parse(black_box(MINIMAL_MANIFEST)).unwrap())
    });
    group.bench_function("full", |b| {
        b.iter(|| PluginManifest::parse(black_box(FULL_MANIFEST)).unwrap())
    });
    group.finish();
}

// Discovery benchmark uses tempdir to avoid filesystem coupling
fn bench_discovery(c: &mut Criterion) {
    use lowfat_plugin::discovery::{discover_plugins, resolve_plugins};
    use std::fs;

    let tmp = tempfile::tempdir().unwrap();

    // Create 10 fake plugins across 3 categories
    for cat in &["git", "docker", "npm"] {
        for i in 0..3 {
            let plugin_dir = tmp.path().join(cat).join(format!("{cat}-plugin-{i}"));
            fs::create_dir_all(&plugin_dir).unwrap();
            let manifest = format!(
                r#"
[plugin]
name = "{cat}-plugin-{i}"
commands = ["{cat}{i}"]

[runtime]
type = "shell"
entry = "filter.sh"
"#
            );
            fs::write(plugin_dir.join("lowfat.toml"), &manifest).unwrap();
            fs::write(plugin_dir.join("filter.sh"), "#!/bin/sh\ncat").unwrap();
        }
    }

    let mut group = c.benchmark_group("discovery");
    group.bench_function("discover_9_plugins", |b| {
        b.iter(|| discover_plugins(black_box(tmp.path())))
    });
    group.bench_function("resolve_9_plugins", |b| {
        let plugins = discover_plugins(tmp.path());
        b.iter(|| resolve_plugins(black_box(&plugins)))
    });
    group.finish();
}

criterion_group!(benches, bench_manifest_parse, bench_discovery);
criterion_main!(benches);
