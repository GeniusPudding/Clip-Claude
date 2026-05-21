use anyhow::{anyhow, Context, Result};
use arboard::ImageData;
use chrono::Local;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

const PURGE_AFTER_DAYS: u64 = 7;

pub fn cache_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("locate home directory")?;
    let dir = home.join(".clip-claude").join("cache");
    std::fs::create_dir_all(&dir).with_context(|| format!("create cache dir {}", dir.display()))?;
    Ok(dir)
}

pub fn save_png(img: &ImageData) -> Result<PathBuf> {
    let dir = cache_dir()?;
    let stamp = Local::now().format("%Y%m%d_%H%M%S_%3f");
    let path = dir.join(format!("clip_{stamp}.png"));

    let buffer = image::RgbaImage::from_raw(
        img.width as u32,
        img.height as u32,
        img.bytes.to_vec(),
    )
    .ok_or_else(|| anyhow!("clipboard image bytes don't match {}x{} RGBA", img.width, img.height))?;

    buffer
        .save(&path)
        .with_context(|| format!("write PNG {}", path.display()))?;
    Ok(path)
}

pub fn purge_old() -> Result<()> {
    let dir = cache_dir()?;
    let cutoff = SystemTime::now()
        .checked_sub(Duration::from_secs(PURGE_AFTER_DAYS * 86_400))
        .unwrap_or(SystemTime::UNIX_EPOCH);
    for entry in std::fs::read_dir(&dir)? {
        let Ok(entry) = entry else { continue };
        let Ok(metadata) = entry.metadata() else { continue };
        let Ok(mtime) = metadata.modified() else { continue };
        if mtime < cutoff {
            let _ = std::fs::remove_file(entry.path());
        }
    }
    Ok(())
}
