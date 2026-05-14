use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;
use std::process::Command;

const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const RUN_VALUE: &str = "Clipbridge";

pub fn install_dir() -> Result<PathBuf> {
    let local_app_data = std::env::var_os("LOCALAPPDATA")
        .context("LOCALAPPDATA env var not set (Windows only)")?;
    let dir = PathBuf::from(local_app_data).join("clipbridge");
    std::fs::create_dir_all(&dir).context("create install dir")?;
    Ok(dir)
}

pub fn install() -> Result<()> {
    let current = std::env::current_exe().context("locate current exe")?;
    let src_dir = current.parent().context("locate current exe dir")?;
    let src_cli = src_dir.join("clipbridge.exe");
    let src_bg = src_dir.join("clipbridge-bg.exe");

    if !src_cli.exists() {
        return Err(anyhow!("can't find clipbridge.exe at {}", src_cli.display()));
    }
    if !src_bg.exists() {
        return Err(anyhow!(
            "can't find clipbridge-bg.exe at {} — rebuild with `cargo build --release`",
            src_bg.display()
        ));
    }

    let dest_dir = install_dir()?;
    let dest_cli = dest_dir.join("clipbridge.exe");
    let dest_bg = dest_dir.join("clipbridge-bg.exe");

    let _ = Command::new("taskkill")
        .args(["/F", "/IM", "clipbridge-bg.exe"])
        .status();

    std::fs::copy(&src_cli, &dest_cli).context("copy clipbridge.exe")?;
    std::fs::copy(&src_bg, &dest_bg).context("copy clipbridge-bg.exe")?;
    println!("  ok  copied binaries to {}", dest_dir.display());

    let dest_bg_str = dest_bg
        .to_str()
        .ok_or_else(|| anyhow!("install path is not valid UTF-8"))?;
    let status = Command::new("reg")
        .args([
            "add", RUN_KEY, "/v", RUN_VALUE, "/t", "REG_SZ", "/d", dest_bg_str, "/f",
        ])
        .status()
        .context("run reg add")?;
    if !status.success() {
        return Err(anyhow!("reg add exited with status {status}"));
    }
    println!("  ok  registered auto-start ({RUN_KEY}\\{RUN_VALUE})");

    Command::new(&dest_bg)
        .spawn()
        .context("spawn background daemon")?;
    println!("  ok  background daemon started now");

    println!();
    println!("Done. Screenshots paste correctly everywhere from this moment on,");
    println!("and clipbridge will auto-start on every login.");
    Ok(())
}

pub fn uninstall() -> Result<()> {
    let _ = Command::new("taskkill")
        .args(["/F", "/IM", "clipbridge-bg.exe"])
        .status();
    println!("  ok  stopped running daemon (if any)");

    let status = Command::new("reg")
        .args(["delete", RUN_KEY, "/v", RUN_VALUE, "/f"])
        .status()
        .context("run reg delete")?;
    if status.success() {
        println!("  ok  removed auto-start registry entry");
    } else {
        println!("  info auto-start registry entry not present (already removed?)");
    }

    let dir = install_dir()?;
    println!();
    println!("Uninstalled. Binaries left at {}", dir.display());
    println!("Remove them manually with: rmdir /S /Q \"{}\"", dir.display());
    Ok(())
}

pub fn status() -> Result<()> {
    let dir = install_dir()?;
    let cli = dir.join("clipbridge.exe");
    let bg = dir.join("clipbridge-bg.exe");

    if cli.exists() && bg.exists() {
        println!("  ok    installed at {}", dir.display());
    } else {
        println!("  info  not installed (no binaries at {})", dir.display());
        println!("        run `clipbridge install` to set up");
        return Ok(());
    }

    let q = Command::new("reg")
        .args(["query", RUN_KEY, "/v", RUN_VALUE])
        .output()
        .context("reg query")?;
    if q.status.success() {
        println!("  ok    auto-start registry entry present");
    } else {
        println!("  warn  auto-start registry entry missing");
    }

    let tasklist = Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq clipbridge-bg.exe", "/NH"])
        .output()
        .context("run tasklist")?;
    let out = String::from_utf8_lossy(&tasklist.stdout);
    if out.contains("clipbridge-bg.exe") {
        println!("  ok    daemon running");
    } else {
        println!("  warn  daemon NOT running (will start on next login)");
    }

    Ok(())
}
