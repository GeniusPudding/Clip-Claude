#![windows_subsystem = "windows"]

fn main() {
    let _ = clip_claude::watcher::run_foreground();
}
