use anyhow::{Context, Result};
use arboard::{Clipboard, ImageData};
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

const POLL_MS: u64 = 150;

pub fn run_foreground() -> Result<()> {
    eprintln!("clip-claude watching — Ctrl+C to stop");
    let stop = Arc::new(AtomicBool::new(false));
    run_loop(stop)
}

#[cfg(any(windows, target_os = "macos"))]
pub fn run_loop(stop: Arc<AtomicBool>) -> Result<()> {
    let mut clipboard = Clipboard::new().context("init clipboard")?;
    let mut state = FocusAwareState::default();

    while !stop.load(Ordering::SeqCst) {
        state.poll(&mut clipboard);
        std::thread::sleep(Duration::from_millis(POLL_MS));
    }
    Ok(())
}

#[cfg(not(any(windows, target_os = "macos")))]
pub fn run_loop(stop: Arc<AtomicBool>) -> Result<()> {
    let mut clipboard = Clipboard::new().context("init clipboard")?;

    while !stop.load(Ordering::SeqCst) {
        if !has_text(&mut clipboard) {
            if let Ok(img) = clipboard.get_image() {
                if let Err(e) = handle_image_simple(&img) {
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

#[cfg(not(any(windows, target_os = "macos")))]
fn handle_image_simple(img: &ImageData) -> Result<()> {
    let width = img.width as u32;
    let height = img.height as u32;
    let path = crate::cache::save_png(img)?;
    let payload = crate::inject::format_payload(&path, width, height);
    crate::clipboard_io::write_image_and_text(img, &payload)?;
    eprintln!("clip-claude: captured {width}x{height} -> {}", path.display());
    let _ = crate::cache::purge_old();
    Ok(())
}

// --- Focus-aware state machine (Windows + macOS) -------------------------
//
// Clipboard content decision per poll:
//   - non-terminal foreground   → image only (Slack-like apps see no text)
//   - local terminal foreground → image + local-path text  (agent reads file)
//   - SSH'd terminal foreground → image + remote-path text (agent on remote
//                                  Read()'s /tmp/clip-claude-*.png that we
//                                  uploaded via scp on demand)
//
// We track the path currently injected in `cap.current_text_path` and only
// re-write the clipboard when it differs from `decide_text_path()`'s output.

#[cfg(any(windows, target_os = "macos"))]
use std::collections::{HashMap, HashSet};

#[cfg(any(windows, target_os = "macos"))]
struct PendingCapture {
    rgba: Vec<u8>,
    img_width: usize,
    img_height: usize,
    local_path: PathBuf,
    px_width: u32,
    px_height: u32,
    current_text_path: Option<PathBuf>,
    remote_cache: HashMap<String, PathBuf>,
    failed_targets: HashSet<String>,
}

#[cfg(any(windows, target_os = "macos"))]
impl PendingCapture {
    fn image_data(&self) -> ImageData<'_> {
        ImageData {
            width: self.img_width,
            height: self.img_height,
            bytes: Cow::Borrowed(&self.rgba),
        }
    }
}

#[cfg(any(windows, target_os = "macos"))]
#[derive(Default)]
struct FocusAwareState {
    our_seq: Option<u32>,
    last_external_seq: Option<u32>,
    pending: Option<PendingCapture>,
    ssh: crate::ssh_session::Detector,
}

#[cfg(any(windows, target_os = "macos"))]
impl FocusAwareState {
    fn poll(&mut self, clipboard: &mut Clipboard) {
        let seq = crate::clipboard_io::get_sequence_number();

        let ours = self.our_seq == Some(seq);
        let same_external =
            self.last_external_seq == Some(seq) && self.pending.is_some();
        if ours || same_external {
            self.reconcile();
        } else {
            self.handle_new_clipboard(clipboard, seq);
        }
    }

    /// Compare the path we want injected (based on current focus + ssh) against
    /// the path currently on the clipboard.  Rewrite only if they differ.
    fn reconcile(&mut self) {
        let desired = self.decide_text_path();

        let cap = match self.pending.as_mut() {
            Some(c) => c,
            None => return,
        };

        if desired == cap.current_text_path {
            return;
        }

        let result = match desired.as_ref() {
            Some(path) => {
                let payload =
                    crate::inject::format_payload(path, cap.px_width, cap.px_height);
                crate::clipboard_io::write_image_and_text(&cap.image_data(), &payload)
            }
            None => crate::clipboard_io::write_image_only(&cap.image_data()),
        };

        match result {
            Ok(()) => {
                self.our_seq = Some(crate::clipboard_io::get_sequence_number());
                cap.current_text_path = desired;
            }
            Err(e) => eprintln!("clip-claude: {e:#}"),
        }
    }

    fn decide_text_path(&mut self) -> Option<PathBuf> {
        if !crate::focus::is_terminal_foreground() {
            return None;
        }
        let cap = self.pending.as_mut()?;

        match self.ssh.detect() {
            None => Some(cap.local_path.clone()),
            Some(target) => {
                if let Some(p) = cap.remote_cache.get(&target.spec) {
                    return Some(p.clone());
                }
                if cap.failed_targets.contains(&target.spec) {
                    return Some(cap.local_path.clone());
                }
                match crate::remote::upload(&cap.local_path, &target) {
                    Ok(remote) => {
                        eprintln!(
                            "clip-claude: scp {} -> {}:{}",
                            cap.local_path.display(),
                            target.spec,
                            remote.display()
                        );
                        cap.remote_cache.insert(target.spec.clone(), remote.clone());
                        Some(remote)
                    }
                    Err(e) => {
                        eprintln!(
                            "clip-claude: scp to {} failed: {e:#} — using local path for this capture",
                            target.spec
                        );
                        cap.failed_targets.insert(target.spec.clone());
                        Some(cap.local_path.clone())
                    }
                }
            }
        }
    }

    fn handle_new_clipboard(&mut self, clipboard: &mut Clipboard, seq: u32) {
        self.last_external_seq = Some(seq);
        self.pending = None;
        self.our_seq = None;

        if has_text(clipboard) {
            return;
        }

        let img = match clipboard.get_image() {
            Ok(img) => img,
            Err(_) => return,
        };

        let width = img.width as u32;
        let height = img.height as u32;

        let path = match crate::cache::save_png(&img) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("clip-claude: {e:#}");
                return;
            }
        };
        let _ = crate::cache::purge_old();
        eprintln!(
            "clip-claude: captured {width}x{height} -> {}",
            path.display()
        );

        // Default-augment with local path so a fast paste into a local terminal
        // doesn't race the next poll.  reconcile() in subsequent polls handles
        // the de-augment (Slack case) and remote-path upgrade (SSH case).
        let payload = crate::inject::format_payload(&path, width, height);
        let img_ref = ImageData {
            width: img.width,
            height: img.height,
            bytes: Cow::Borrowed(&img.bytes),
        };
        let mut current_text_path = None;
        match crate::clipboard_io::write_image_and_text(&img_ref, &payload) {
            Ok(()) => {
                self.our_seq = Some(crate::clipboard_io::get_sequence_number());
                current_text_path = Some(path.clone());
            }
            Err(e) => eprintln!("clip-claude: {e:#}"),
        }

        self.pending = Some(PendingCapture {
            rgba: img.bytes.into_owned(),
            img_width: img.width,
            img_height: img.height,
            local_path: path,
            px_width: width,
            px_height: height,
            current_text_path,
            remote_cache: HashMap::new(),
            failed_targets: HashSet::new(),
        });
    }
}
