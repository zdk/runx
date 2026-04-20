use anyhow::Result;
use lowfat_core::config::RunfConfig;
use lowfat_core::db::Db;

const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";
const CYAN: &str = "\x1b[36m";
const YELLOW: &str = "\x1b[33m";
const MAGENTA: &str = "\x1b[35m";
const WHITE: &str = "\x1b[97m";

fn fmt_tokens(n: f64) -> String {
    if n >= 1_000_000.0 {
        format!("{:.1}M", n / 1_000_000.0)
    } else if n >= 1_000.0 {
        format!("{:.1}K", n / 1_000.0)
    } else {
        // One decimal so avg raw / avg saved reconcile with the % column
        // (e.g. 77.3 × 73.7% ≈ 57.0, which reads clean).
        format!("{n:.1}")
    }
}

/// Plugin-candidate ranking. High score = called often, big output,
/// and lowfat isn't trimming it much yet.
pub fn run(limit: usize) -> Result<()> {
    let config = RunfConfig::resolve();
    let db = Db::open(&config.data_dir)?;
    let rows = db.history_ranking(limit)?;

    println!();
    println!("  {BOLD}{WHITE}lowfat{RESET} {DIM}plugin candidates{RESET}");
    println!("  {DIM}─────────────────────────────────────────────────────────{RESET}");
    println!();

    if rows.is_empty() {
        println!("  {DIM}No data yet. Run some commands through lowfat!{RESET}");
        println!();
        return Ok(());
    }

    println!(
        "  {DIM}{:>3}  {:<25} {:>5}  {:>9}  {:>9}  {:>8}  {:>10}{RESET}",
        "#", "command", "runs", "avg raw", "avg saved", "savings", "has_plugin"
    );

    for (i, r) in rows.iter().enumerate() {
        let rank = i + 1;
        let label = if r.subcommand.is_empty() {
            r.command.clone()
        } else {
            format!("{} {}", r.command, r.subcommand)
        };
        // Under-performing filter: has a plugin but isn't saving much,
        // on output big enough that it should be. Same magenta as
        // "no plugin" — both signal "act on this row".
        let underperforming =
            r.plugin_ratio >= 0.5 && r.savings_pct < 20.0 && r.avg_raw_tokens > 50.0;
        let save_color = if r.savings_pct >= 50.0 {
            DIM
        } else if r.plugin_ratio < 0.5 || underperforming {
            MAGENTA
        } else {
            YELLOW
        };
        let (plugin_text, plugin_color) = if r.plugin_ratio >= 0.99 {
            ("yes".to_string(), DIM)
        } else if r.plugin_ratio <= 0.01 {
            ("no".to_string(), MAGENTA)
        } else {
            (format!("{:.0}%", r.plugin_ratio * 100.0), YELLOW)
        };
        // Pad the visible text first so color codes don't break alignment.
        let plugin_mark = format!("{plugin_color}{plugin_text:>10}{RESET}");
        println!(
            "  {BOLD}{:>3}{RESET}  {CYAN}{:<25}{RESET} {:>4}x  {:>9}  {:>9}  {save_color}{:>7.1}%{RESET}  {}",
            rank,
            label,
            r.runs,
            fmt_tokens(r.avg_raw_tokens),
            fmt_tokens(r.avg_saved_tokens),
            r.savings_pct,
            plugin_mark,
        );
    }

    println!();
    println!(
        "  {DIM}Tip: rows marked {MAGENTA}\"no\"{RESET}{DIM} have no filter yet — good plugin candidates.{RESET}"
    );
    println!(
        "       {DIM}Scaffold one with:{RESET} {BOLD}lowfat plugin new <command>{RESET}"
    );
    println!(
        "       {DIM}rows marked {MAGENTA}\"yes\"{RESET}{DIM} with {MAGENTA}low savings{RESET}{DIM} — the filter may need tuning.{RESET}"
    );
    println!();

    Ok(())
}
