use anyhow::{Context, Result};
use arboard::{Clipboard, ImageData};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

const POLL_MS: u64 = 150;

pub fn run_foreground() -> Result<()> {
    eprintln!("clipbridge watching (Ctrl+C to stop)");
    let never_stop = Arc::new(AtomicBool::new(false));
    run_loop(never_stop)
}

pub fn run_loop(stop: Arc<AtomicBool>) -> Result<()> {
    let mut clipboard = Clipboard::new().context("init clipboard")?;
    while !stop.load(Ordering::SeqCst) {
        if !has_text(&mut clipboard) {
            if let Ok(img) = clipboard.get_image() {
                if let Err(e) = handle_image(&mut clipboard, img) {
                    eprintln!("clipbridge: {e:#}");
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

fn handle_image(clipboard: &mut Clipboard, img: ImageData) -> Result<()> {
    let width = img.width as u32;
    let height = img.height as u32;
    let path = crate::cache::save_png(img)?;
    let payload = crate::inject::format_payload(&path, width, height);
    clipboard.set_text(&payload).context("write text payload")?;
    eprintln!("clipbridge: captured {width}x{height} -> {}", path.display());
    let _ = crate::cache::purge_old();
    Ok(())
}
