pub fn run() {
    println!("Usage: lowfat <command> [args...]");
    println!();
    println!("Token-aware command filter for LLM environments.");
    println!("Intercepts commands, applies filters, reduces token usage.");
    println!();
    println!("Levels: lite (gentle), full (default), ultra (max compression)");
    println!("  Set: export LOWFAT_LEVEL=ultra  or  level=ultra in .lowfat");
    println!();
    println!("Commands:");
    println!("  lowfat <cmd> [args]   Filter a command (e.g., lowfat git diff)");
    println!("  lowfat filters        List enabled/disabled filters");
    println!("  lowfat gain           Token savings report");
    println!("  lowfat bench          Benchmark savings in current project");
    println!("  lowfat compress <f>   Compress a markdown file");
    println!("  lowfat level [val]    Get or set intensity level");
    println!("  lowfat status         Show status badge");
    println!("  lowfat shell-init     Print shell init script for eval");
    println!("  lowfat plugin <act>   Manage plugins (list, doctor, info, install)");
    println!("  lowfat help           This help message");
}
