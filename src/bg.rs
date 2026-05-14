#![windows_subsystem = "windows"]

fn main() {
    let _ = clipbridge::watcher::run_foreground();
}
