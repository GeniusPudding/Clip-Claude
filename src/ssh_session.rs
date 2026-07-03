use std::time::{Duration, Instant};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SshTarget {
    pub spec: String,
}

const CACHE_TTL: Duration = Duration::from_millis(900);

pub struct Detector {
    system: System,
    last_check: Option<Instant>,
    cached: Option<SshTarget>,
}

impl Default for Detector {
    fn default() -> Self {
        Self {
            system: System::new(),
            last_check: None,
            cached: None,
        }
    }
}

impl Detector {
    pub fn detect(&mut self) -> Option<SshTarget> {
        if let Some(t) = self.last_check {
            if t.elapsed() < CACHE_TTL {
                return self.cached.clone();
            }
        }
        self.last_check = Some(Instant::now());
        self.cached = self.detect_uncached();
        self.cached.clone()
    }

    fn detect_uncached(&mut self) -> Option<SshTarget> {
        let fg_pid = foreground_pid()?;
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::new().with_cmd(sysinfo::UpdateKind::Always),
        );

        let mut stack = vec![Pid::from_u32(fg_pid)];
        let mut seen: Vec<Pid> = Vec::new();
        while let Some(pid) = stack.pop() {
            if seen.contains(&pid) {
                continue;
            }
            seen.push(pid);

            for (child_pid, child) in self.system.processes() {
                if child.parent() != Some(pid) {
                    continue;
                }
                if let Some(target) = ssh_target_from_proc(child) {
                    return Some(target);
                }
                stack.push(*child_pid);
            }
        }
        None
    }
}

fn ssh_target_from_proc(proc: &sysinfo::Process) -> Option<SshTarget> {
    let argv = proc.cmd();
    if argv.is_empty() {
        return None;
    }
    let exe = argv[0].to_string_lossy();
    let exe_lower = exe.to_lowercase();
    let basename = exe_lower
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(&exe_lower)
        .to_string();
    let stem = basename
        .strip_suffix(".exe")
        .unwrap_or(&basename)
        .to_string();
    if stem != "ssh" && stem != "mosh-client" {
        return None;
    }
    let args: Vec<String> = argv
        .iter()
        .skip(1)
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    parse_ssh_target(&args).map(|spec| SshTarget { spec })
}

// Flags that consume the next argv slot as their value.
const VALUE_FLAGS: &[&str] = &[
    "-p", "-i", "-o", "-l", "-L", "-R", "-D", "-F", "-E", "-J", "-W", "-b", "-c", "-e", "-I", "-m",
    "-O", "-Q", "-S",
];

fn parse_ssh_target(args: &[String]) -> Option<String> {
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if VALUE_FLAGS.contains(&a.as_str()) {
            i += 2;
            continue;
        }
        if a.starts_with('-') {
            i += 1;
            continue;
        }
        return Some(a.clone());
    }
    None
}

#[cfg(windows)]
fn foreground_pid() -> Option<u32> {
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
    unsafe {
        let hwnd = GetForegroundWindow();
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            None
        } else {
            Some(pid)
        }
    }
}

#[cfg(target_os = "macos")]
fn foreground_pid() -> Option<u32> {
    use objc2_app_kit::NSWorkspace;
    unsafe {
        let ws = NSWorkspace::sharedWorkspace();
        let app = ws.frontmostApplication()?;
        let pid = app.processIdentifier();
        if pid <= 0 {
            None
        } else {
            Some(pid as u32)
        }
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
fn foreground_pid() -> Option<u32> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_target() {
        assert_eq!(
            parse_ssh_target(&["me@host".into()]),
            Some("me@host".into())
        );
    }

    #[test]
    fn skips_value_flags() {
        assert_eq!(
            parse_ssh_target(&[
                "-p".into(),
                "2222".into(),
                "-i".into(),
                "/tmp/key".into(),
                "me@host".into(),
            ]),
            Some("me@host".into())
        );
    }

    #[test]
    fn skips_boolean_flags() {
        assert_eq!(
            parse_ssh_target(&[
                "-A".into(),
                "-q".into(),
                "-4".into(),
                "host.alias".into(),
                "tail".into(),
                "-f".into(),
                "log".into(),
            ]),
            Some("host.alias".into())
        );
    }

    #[test]
    fn jump_host() {
        assert_eq!(
            parse_ssh_target(&["-J".into(), "jumper".into(), "me@dest".into()]),
            Some("me@dest".into())
        );
    }

    #[test]
    fn empty_returns_none() {
        assert_eq!(parse_ssh_target(&[]), None);
    }
}
