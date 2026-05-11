use std::time::{Duration, Instant};

const KEYWORDS: &[&str] = &["claude", "@anthropic", "gemini", "@google/gemini", "codex", "@openai"];
const CACHE_TTL: Duration = Duration::from_secs(2);

pub struct AgentMatch {
    pub pid: u32,
    pub name: String,
    pub reason: String,
}

pub struct ForegroundReport {
    pub foreground_pid: Option<u32>,
    pub foreground_name: Option<String>,
    pub agent: Option<AgentMatch>,
}

pub struct FocusCache {
    pid: Option<u32>,
    is_agent: bool,
    checked_at: Option<Instant>,
}

impl FocusCache {
    pub fn new() -> Self {
        Self { pid: None, is_agent: false, checked_at: None }
    }
}

pub fn is_agent_foreground(cache: &mut FocusCache) -> bool {
    let current = foreground_pid();
    let now = Instant::now();

    let cache_valid = match (cache.pid, cache.checked_at) {
        (Some(p), Some(t)) => Some(p) == current && now.duration_since(t) < CACHE_TTL,
        _ => false,
    };
    if cache_valid {
        return cache.is_agent;
    }

    let agent = current.is_some() && find_agent_in_tree(current.unwrap()).is_some();
    cache.pid = current;
    cache.is_agent = agent;
    cache.checked_at = Some(now);
    agent
}

pub fn report() -> ForegroundReport {
    let pid = match foreground_pid() {
        Some(p) => p,
        None => return ForegroundReport { foreground_pid: None, foreground_name: None, agent: None },
    };

    let mut sys = sysinfo::System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let foreground_name = sys
        .process(sysinfo::Pid::from_u32(pid))
        .map(|p| p.name().to_string_lossy().into_owned());

    let agent = walk(&sys, pid);

    ForegroundReport { foreground_pid: Some(pid), foreground_name, agent }
}

fn find_agent_in_tree(root_pid: u32) -> Option<AgentMatch> {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    walk(&sys, root_pid)
}

fn walk(sys: &sysinfo::System, root_pid: u32) -> Option<AgentMatch> {
    use std::collections::HashSet;
    let mut to_visit: Vec<u32> = vec![root_pid];
    let mut visited: HashSet<u32> = HashSet::new();

    while let Some(pid_u32) = to_visit.pop() {
        if !visited.insert(pid_u32) {
            continue;
        }
        let pid = sysinfo::Pid::from_u32(pid_u32);
        if let Some(p) = sys.process(pid) {
            if let Some(reason) = match_reason(p) {
                return Some(AgentMatch {
                    pid: pid_u32,
                    name: p.name().to_string_lossy().into_owned(),
                    reason,
                });
            }
        }
        for (cpid, cproc) in sys.processes() {
            if cproc.parent() == Some(pid) {
                to_visit.push(cpid.as_u32());
            }
        }
    }
    None
}

fn match_reason(p: &sysinfo::Process) -> Option<String> {
    let raw = p.name().to_string_lossy().to_lowercase();
    let stem = raw
        .trim_end_matches(".exe")
        .trim_end_matches(".cmd")
        .trim_end_matches(".bat");

    if matches!(stem, "claude" | "gemini" | "codex") {
        return Some(format!("name = {stem}"));
    }
    if stem.starts_with("node") {
        for arg in p.cmd() {
            let a = arg.to_string_lossy().to_lowercase();
            for kw in KEYWORDS {
                if a.contains(kw) {
                    return Some(format!("node cmdline contains \"{kw}\""));
                }
            }
        }
    }
    None
}

#[cfg(windows)]
fn foreground_pid() -> Option<u32> {
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid as *mut u32));
        if pid == 0 { None } else { Some(pid) }
    }
}

#[cfg(not(windows))]
fn foreground_pid() -> Option<u32> {
    None
}
