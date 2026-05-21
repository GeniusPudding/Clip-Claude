use anyhow::Result;
use clap::Parser;
use clip_claude::{cache, cli, install, runner, watcher};

fn main() -> Result<()> {
    let args = cli::Cli::parse();
    match args.command {
        None | Some(cli::Command::Start) => watcher::run_foreground(),
        Some(cli::Command::Run { args }) => runner::run_wrapped(&args),
        Some(cli::Command::Doctor) => doctor(),
        Some(cli::Command::Install) => install::install(),
        Some(cli::Command::Uninstall) => install::uninstall(),
        Some(cli::Command::Status) => install::status(),
    }
}

fn doctor() -> Result<()> {
    println!("Clip-Claude {}", env!("CARGO_PKG_VERSION"));
    let _ = arboard::Clipboard::new()?;
    println!("  ok  clipboard accessible");
    let dir = cache::cache_dir()?;
    println!("  ok  cache dir = {}", dir.display());
    println!("  ok  ready");
    Ok(())
}
