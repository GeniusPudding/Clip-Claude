use anyhow::Result;

// --- Windows -------------------------------------------------------------

#[cfg(windows)]
use anyhow::{anyhow, Context};
#[cfg(windows)]
use std::path::PathBuf;
#[cfg(windows)]
use std::process::Command;

#[cfg(windows)]
const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
#[cfg(windows)]
const RUN_VALUE: &str = "Clip-Claude";

#[cfg(windows)]
pub fn install_dir() -> Result<PathBuf> {
    let local_app_data = std::env::var_os("LOCALAPPDATA")
        .context("LOCALAPPDATA env var not set (Windows only)")?;
    let dir = PathBuf::from(local_app_data).join("Clip-Claude");
    std::fs::create_dir_all(&dir).context("create install dir")?;
    Ok(dir)
}

#[cfg(windows)]
pub fn install() -> Result<()> {
    let current = std::env::current_exe().context("locate current exe")?;
    let src_dir = current.parent().context("locate current exe dir")?;
    let src_cli = src_dir.join("clip-claude.exe");
    let src_bg = src_dir.join("clip-claude-bg.exe");

    if !src_cli.exists() {
        return Err(anyhow!("can't find clip-claude.exe at {}", src_cli.display()));
    }
    if !src_bg.exists() {
        return Err(anyhow!(
            "can't find clip-claude-bg.exe at {} — rebuild with `cargo build --release`",
            src_bg.display()
        ));
    }

    let dest_dir = install_dir()?;
    let dest_cli = dest_dir.join("clip-claude.exe");
    let dest_bg = dest_dir.join("clip-claude-bg.exe");

    let _ = Command::new("taskkill")
        .args(["/F", "/IM", "clip-claude-bg.exe"])
        .status();

    std::fs::copy(&src_cli, &dest_cli).context("copy clip-claude.exe")?;
    std::fs::copy(&src_bg, &dest_bg).context("copy clip-claude-bg.exe")?;
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
    println!("and clip-claude will auto-start on every login.");
    Ok(())
}

#[cfg(windows)]
pub fn uninstall() -> Result<()> {
    let _ = Command::new("taskkill")
        .args(["/F", "/IM", "clip-claude-bg.exe"])
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

#[cfg(windows)]
pub fn status() -> Result<()> {
    let dir = install_dir()?;
    let cli = dir.join("clip-claude.exe");
    let bg = dir.join("clip-claude-bg.exe");

    if cli.exists() && bg.exists() {
        println!("  ok    installed at {}", dir.display());
    } else {
        println!("  info  not installed (no binaries at {})", dir.display());
        println!("        run `clip-claude install` to set up");
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
        .args(["/FI", "IMAGENAME eq clip-claude-bg.exe", "/NH"])
        .output()
        .context("run tasklist")?;
    let out = String::from_utf8_lossy(&tasklist.stdout);
    if out.contains("clip-claude-bg.exe") {
        println!("  ok    daemon running");
    } else {
        println!("  warn  daemon NOT running (will start on next login)");
    }

    Ok(())
}

// --- macOS ---------------------------------------------------------------

#[cfg(target_os = "macos")]
use anyhow::{anyhow, Context};
#[cfg(target_os = "macos")]
use std::path::PathBuf;
#[cfg(target_os = "macos")]
use std::process::Command;

#[cfg(target_os = "macos")]
const LAUNCH_LABEL: &str = "com.clip-claude.daemon";

#[cfg(target_os = "macos")]
pub fn install_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("locate home directory")?;
    let dir = home.join("Library/Application Support/Clip-Claude");
    std::fs::create_dir_all(&dir).context("create install dir")?;
    Ok(dir)
}

#[cfg(target_os = "macos")]
fn plist_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("locate home directory")?;
    let dir = home.join("Library/LaunchAgents");
    std::fs::create_dir_all(&dir).context("create LaunchAgents dir")?;
    Ok(dir.join("com.clip-claude.daemon.plist"))
}

#[cfg(target_os = "macos")]
fn uid() -> Result<u32> {
    let out = Command::new("id").arg("-u").output().context("run id -u")?;
    String::from_utf8_lossy(&out.stdout)
        .trim()
        .parse::<u32>()
        .context("parse UID")
}

#[cfg(target_os = "macos")]
fn bootout(plist: &PathBuf) {
    if let Ok(uid) = uid() {
        let _ = Command::new("launchctl")
            .args(["bootout", &format!("gui/{}", uid)])
            .arg(plist)
            .status();
    }
}

#[cfg(target_os = "macos")]
pub fn install() -> Result<()> {
    let current = std::env::current_exe().context("locate current exe")?;
    if !current.exists() {
        return Err(anyhow!("can't find clip-claude at {}", current.display()));
    }

    let dest_dir = install_dir()?;
    let dest = dest_dir.join("clip-claude");
    let plist = plist_path()?;

    bootout(&plist);

    std::fs::copy(&current, &dest).context("copy clip-claude binary")?;
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&dest)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&dest, perms)?;
    }
    println!("  ok  copied binary to {}", dest.display());

    let dest_str = dest
        .to_str()
        .ok_or_else(|| anyhow!("install path is not valid UTF-8"))?;
    let plist_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{LAUNCH_LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{dest_str}</string>
        <string>start</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardErrorPath</key>
    <string>/tmp/clip-claude.err</string>
</dict>
</plist>
"#
    );
    std::fs::write(&plist, plist_xml).context("write LaunchAgent plist")?;
    println!("  ok  wrote LaunchAgent plist to {}", plist.display());

    let uid = uid()?;
    let status = Command::new("launchctl")
        .args(["bootstrap", &format!("gui/{}", uid)])
        .arg(&plist)
        .status()
        .context("run launchctl bootstrap")?;
    if !status.success() {
        return Err(anyhow!("launchctl bootstrap exited with status {status}"));
    }
    println!("  ok  registered LaunchAgent ({LAUNCH_LABEL})");
    println!("  ok  background daemon started now");

    println!();
    println!("Done. Screenshots paste correctly everywhere from this moment on,");
    println!("and clip-claude will auto-start on every login.");
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn uninstall() -> Result<()> {
    let plist = plist_path()?;
    bootout(&plist);
    println!("  ok  unregistered LaunchAgent (if loaded)");

    if plist.exists() {
        std::fs::remove_file(&plist).context("remove plist")?;
        println!("  ok  removed plist {}", plist.display());
    } else {
        println!("  info plist not present (already removed?)");
    }

    let dir = install_dir()?;
    println!();
    println!("Uninstalled. Binary left at {}", dir.display());
    println!("Remove it manually with: rm -rf \"{}\"", dir.display());
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn status() -> Result<()> {
    let dir = install_dir()?;
    let cli = dir.join("clip-claude");

    if cli.exists() {
        println!("  ok    installed at {}", dir.display());
    } else {
        println!("  info  not installed (no binary at {})", cli.display());
        println!("        run `clip-claude install` to set up");
        return Ok(());
    }

    let plist = plist_path()?;
    if plist.exists() {
        println!("  ok    LaunchAgent plist present at {}", plist.display());
    } else {
        println!("  warn  LaunchAgent plist missing");
    }

    let uid = uid()?;
    let q = Command::new("launchctl")
        .args(["print", &format!("gui/{}/{}", uid, LAUNCH_LABEL)])
        .output()
        .context("run launchctl print")?;
    if q.status.success() {
        println!("  ok    daemon running");
    } else {
        println!("  warn  daemon NOT running (will start on next login)");
    }

    Ok(())
}

// --- other (Linux, BSDs) -------------------------------------------------

#[cfg(not(any(windows, target_os = "macos")))]
pub fn install_dir() -> Result<std::path::PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("locate home directory"))?;
    Ok(home.join(".local/bin"))
}

#[cfg(not(any(windows, target_os = "macos")))]
pub fn install() -> Result<()> {
    println!("install: auto-start not wired on this platform yet.");
    println!("Build with `cargo build --release` and run `target/release/clip-claude start` manually.");
    Ok(())
}

#[cfg(not(any(windows, target_os = "macos")))]
pub fn uninstall() -> Result<()> {
    println!("uninstall: nothing to undo on this platform.");
    Ok(())
}

#[cfg(not(any(windows, target_os = "macos")))]
pub fn status() -> Result<()> {
    println!("status: not implemented on this platform.");
    Ok(())
}
