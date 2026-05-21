use anyhow::{Context, Result};
use arboard::{Clipboard, ImageData};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

const POLL_MS: u64 = 150;

pub fn run_foreground() -> Result<()> {
    eprintln!("clip-claude watching — Ctrl+C to stop");
    let stop = Arc::new(AtomicBool::new(false));
    run_loop(stop)
}

pub fn run_loop(stop: Arc<AtomicBool>) -> Result<()> {
    let mut clipboard = Clipboard::new().context("init clipboard")?;

    while !stop.load(Ordering::SeqCst) {
        if !has_text(&mut clipboard) {
            if let Ok(img) = clipboard.get_image() {
                if let Err(e) = handle_image(img) {
                    eprintln!("clip-claude: {e:#}");
                }
            }
        }
        std::thread::sleep(Duration::from_millis(POLL_MS));
    }
    Ok(())
}

fn has_text(clipboard: &mut Clipboard) -> bool {
    clipboard.get_text().is_ok()
}

fn handle_image(img: ImageData) -> Result<()> {
    let width = img.width as u32;
    let height = img.height as u32;
    let path = crate::cache::save_png(&img)?;
    let payload = crate::inject::format_payload(&path, width, height);
    crate::clipboard_io::write_image_and_text(img, &payload)?;
    eprintln!("clip-claude: captured {width}x{height} -> {}", path.display());
    let _ = crate::cache::purge_old();
    Ok(())
}
