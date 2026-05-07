mod cache;
mod cli;
mod inject;
mod runner;
mod watcher;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let args = cli::Cli::parse();
    match args.command {
        None | Some(cli::Command::Start) => watcher::run_foreground(),
        Some(cli::Command::Run { args }) => runner::run_wrapped(&args),
        Some(cli::Command::Doctor) => doctor(),
    }
}

fn doctor() -> Result<()> {
    println!("clipbridge {}", env!("CARGO_PKG_VERSION"));
    let _ = arboard::Clipboard::new()?;
    println!("  ok  clipboard accessible");
    let dir = cache::cache_dir()?;
    println!("  ok  cache dir = {}", dir.display());
    println!("  ok  ready");
    Ok(())
}
