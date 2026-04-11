use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lowfat_core::level::Level;
use lowfat_core::pipeline::Pipeline;
use lowfat_plugin::plugin::{FilterInput, FilterOutput, FilterPlugin, PluginInfo};
use lowfat_runner::runner::execute_pipeline;
use std::collections::HashMap;

/// Fake plugin that just returns input unchanged (measures pipeline overhead)
struct NoopPlugin;

impl FilterPlugin for NoopPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "noop".to_string(),
            version: "0.0.0".to_string(),
            commands: vec![],
            subcommands: vec![],
        }
    }

    fn filter(&self, input: &FilterInput) -> anyhow::Result<FilterOutput> {
        Ok(FilterOutput {
            text: input.raw.clone(),
            passthrough: false,
        })
    }
}

fn make_input(raw: &str) -> FilterInput {
    FilterInput {
        raw: raw.to_string(),
        command: "test".to_string(),
        subcommand: String::new(),
        args: vec![],
        level: Level::Full,
        head_limit: 40,
        exit_code: 0,
    }
}

fn make_ansi_text(lines: usize) -> String {
    (0..lines)
        .map(|i| format!("\x1b[32m+\x1b[0m line {i}: some code here"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn bench_execute_pipeline(c: &mut Criterion) {
    let small = make_ansi_text(20);
    let large = make_ansi_text(500);

    let mut plugins: HashMap<String, Box<dyn FilterPlugin>> = HashMap::new();
    plugins.insert("noop".to_string(), Box::new(NoopPlugin));

    let mut group = c.benchmark_group("execute_pipeline");

    // Builtin-only pipeline
    let builtin_pipeline = Pipeline::parse("strip-ansi | dedup-blank | truncate");
    group.bench_function("builtin_3stage_20lines", |b| {
        let input = make_input(&small);
        b.iter(|| {
            execute_pipeline(
                black_box(&builtin_pipeline),
                black_box(&small),
                black_box(&input),
                black_box(&plugins),
            )
        })
    });
    group.bench_function("builtin_3stage_500lines", |b| {
        let input = make_input(&large);
        b.iter(|| {
            execute_pipeline(
                black_box(&builtin_pipeline),
                black_box(&large),
                black_box(&input),
                black_box(&plugins),
            )
        })
    });

    // Pipeline with noop plugin (measures dispatch overhead)
    let plugin_pipeline = Pipeline::parse("strip-ansi | noop | truncate");
    group.bench_function("with_noop_plugin_20lines", |b| {
        let input = make_input(&small);
        b.iter(|| {
            execute_pipeline(
                black_box(&plugin_pipeline),
                black_box(&small),
                black_box(&input),
                black_box(&plugins),
            )
        })
    });
    group.bench_function("with_noop_plugin_500lines", |b| {
        let input = make_input(&large);
        b.iter(|| {
            execute_pipeline(
                black_box(&plugin_pipeline),
                black_box(&large),
                black_box(&input),
                black_box(&plugins),
            )
        })
    });

    // Single passthrough (baseline)
    let passthrough = Pipeline::parse("passthrough");
    group.bench_function("passthrough_500lines", |b| {
        let input = make_input(&large);
        b.iter(|| {
            execute_pipeline(
                black_box(&passthrough),
                black_box(&large),
                black_box(&input),
                black_box(&plugins),
            )
        })
    });

    // Full realistic chain: strip-ansi → noop → dedup-blank → token-budget
    let full_pipeline = Pipeline::parse("strip-ansi | noop | dedup-blank | token-budget");
    group.bench_function("full_4stage_500lines", |b| {
        let input = make_input(&large);
        b.iter(|| {
            execute_pipeline(
                black_box(&full_pipeline),
                black_box(&large),
                black_box(&input),
                black_box(&plugins),
            )
        })
    });

    group.finish();
}

criterion_group!(benches, bench_execute_pipeline);
criterion_main!(benches);
