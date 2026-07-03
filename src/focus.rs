// --- Windows -------------------------------------------------------------

#[cfg(windows)]
const TERMINALS_WIN: &[&str] = &[
    "windowsterminal.exe",
    "cmd.exe",
    "powershell.exe",
    "pwsh.exe",
    "code.exe",
    "code - insiders.exe",
    "cursor.exe",
    "windsurf.exe",
    "conhost.exe",
    "alacritty.exe",
    "wezterm-gui.exe",
    "mintty.exe",
    "hyper.exe",
    "tabby.exe",
    "rio.exe",
];

#[cfg(windows)]
pub fn is_terminal_foreground() -> bool {
    fg_process_name()
        .map(|name| {
            let lower = name.to_ascii_lowercase();
            TERMINALS_WIN.iter().any(|t| lower == *t)
        })
        .unwrap_or(false)
}

#[cfg(windows)]
fn fg_process_name() -> Option<String> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
    use windows::core::PWSTR;

    unsafe {
        let hwnd = GetForegroundWindow();
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return None;
        }

        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;

        let mut buf = [0u16; 260];
        let mut len = buf.len() as u32;
        let result = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buf.as_mut_ptr()),
            &mut len,
        );
        let _ = CloseHandle(handle);
        result.ok()?;

        let path = String::from_utf16_lossy(&buf[..len as usize]);
        path.rsplit('\\').next().map(|s| s.to_string())
    }
}

// --- macOS ---------------------------------------------------------------

#[cfg(target_os = "macos")]
const TERMINALS_MAC: &[&str] = &[
    "com.apple.Terminal",
    "com.googlecode.iterm2",
    "dev.warp.Warp-Stable",
    "com.microsoft.VSCode",
    "com.microsoft.VSCodeInsiders",
    "com.todesktop.230313mzl4w4u92", // Cursor
    "com.github.wez.wezterm",
    "net.kovidgoyal.kitty",
    "org.alacritty",
    "co.zeit.hyper",
    "com.tabby.tabby-app",
    "com.mitchellh.ghostty",
];

#[cfg(target_os = "macos")]
pub fn is_terminal_foreground() -> bool {
    use objc2_app_kit::NSWorkspace;
    unsafe {
        let ws = NSWorkspace::sharedWorkspace();
        let Some(app) = ws.frontmostApplication() else {
            return false;
        };
        let Some(bid) = app.bundleIdentifier() else {
            return false;
        };
        let id = bid.to_string();
        TERMINALS_MAC.iter().any(|t| id == *t)
    }
}

// --- other (Linux, BSDs) -------------------------------------------------

#[cfg(not(any(windows, target_os = "macos")))]
pub fn is_terminal_foreground() -> bool {
    true
}
