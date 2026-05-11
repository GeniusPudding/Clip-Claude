mod cache;
mod cli;
mod focus;
mod inject;
mod runner;
mod watcher;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let args = cli::Cli::parse();
    match args.command {
        None => watcher::run_foreground(false),
        Some(cli::Command::Start { all_windows }) => watcher::run_foreground(all_windows),
        Some(cli::Command::Run { args }) => runner::run_wrapped(&args),
        Some(cli::Command::Doctor) => doctor(),
    }
}

fn doctor() -> Result<()> {
    println!("clipbridge {}", env!("CARGO_PKG_VERSION"));

    let _ = arboard::Clipboard::new()?;
    println!("  ok    clipboard accessible");

    let dir = cache::cache_dir()?;
    println!("  ok    cache dir = {}", dir.display());

    let report = focus::report();
    match report.foreground_pid {
        Some(pid) => {
            let name = report.foreground_name.as_deref().unwrap_or("?");
            println!("  info  foreground = {name} (pid {pid})");
        }
        None => println!("  info  no foreground window detected"),
    }
    match report.agent {
        Some(m) => println!(
            "  info  agent in tree: yes — {} (pid {}) [{}]",
            m.name, m.pid, m.reason
        ),
        None => println!("  info  agent in tree: no — clipbridge would NOT convert images now"),
    }

    println!("  ok    ready");
    Ok(())
}
