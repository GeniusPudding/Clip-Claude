use anyhow::{anyhow, Context, Result};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

pub fn run_wrapped(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!(
            "clipbridge run requires a command, e.g. `clipbridge run -- claude`"
        ));
    }

    let stop = Arc::new(AtomicBool::new(false));
    let stop_for_watcher = stop.clone();
    let watcher = thread::spawn(move || {
        if let Err(e) = crate::watcher::run_loop(stop_for_watcher, false) {
            eprintln!("clipbridge watcher: {e:#}");
        }
    });

    let status = build_command(args)
        .status()
        .with_context(|| format!("spawn child: {:?}", args))?;

    stop.store(true, Ordering::SeqCst);
    let _ = watcher.join();

    std::process::exit(status.code().unwrap_or(1));
}

#[cfg(windows)]
fn build_command(args: &[String]) -> Command {
    let mut cmd = Command::new("cmd");
    cmd.arg("/C");
    cmd.args(args);
    cmd
}

#[cfg(not(windows))]
fn build_command(args: &[String]) -> Command {
    let mut cmd = Command::new(&args[0]);
    cmd.args(&args[1..]);
    cmd
}
