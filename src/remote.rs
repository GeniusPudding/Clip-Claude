use anyhow::{anyhow, Context, Result};
use chrono::Local;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::ssh_session::SshTarget;

pub fn upload(local_png: &Path, target: &SshTarget) -> Result<PathBuf> {
    let stamp = Local::now().format("%Y%m%d_%H%M%S_%3f");
    let remote = PathBuf::from(format!("/tmp/clip-claude-{stamp}.png"));

    let local_str = local_png
        .to_str()
        .ok_or_else(|| anyhow!("local PNG path is not valid UTF-8"))?;
    let remote_str = remote.to_string_lossy();
    let dest = format!("{}:{}", target.spec, remote_str);

    let status = Command::new("scp")
        .args(["-B", "-q", "-o", "ConnectTimeout=5"])
        .arg(local_str)
        .arg(&dest)
        .status()
        .context("spawn scp (is it on PATH?)")?;

    if !status.success() {
        return Err(anyhow!(
            "scp to {dest} failed with status {status} (check SSH key auth, ConnectTimeout, host fingerprint)"
        ));
    }
    Ok(remote)
}
