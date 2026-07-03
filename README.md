[English](README.md) · [繁體中文](README.zh-TW.md)

# Clip-Claude (即貼即克)

Paste screenshots straight into terminal AI agents — Claude Code, Gemini CLI, Codex — as if they natively supported image paste. **And** keep pasting them into Photoshop, Telegram, Discord, anywhere else, exactly as before.

Sibling project to [Kaikou-Claude (開口即克)](https://github.com/GeniusPudding/Kaikou-Claude) (voice → agent) and [Listen-Claude (聽聲即克)](https://github.com/GeniusPudding/Listen-Claude) (agent → voice). Three plugins, one shared workflow: keep using terminal agents the way you'd use a chat app.

```
                              ┌───────────────────────────────────────────┐
                              │  Clip-Claude augments the clipboard:      │
[ screenshot in clipboard ] ─►│    keeps the original image (CF_DIB)      │
                              │    adds a text path payload (CF_UNICODE)  │
                              └───────────────────────────────────────────┘
                                              │
                  ┌───────────────────────────┴───────────────────────────┐
                  ▼                                                       ▼
       Ctrl+V in Photoshop /                                Ctrl+V in agent terminal —
       Telegram / Discord — image                           text path goes in, agent
       comes out, like it never                             reads it with its Read tool,
       got touched.                                         sees the image multimodally.
```

A tiny native Rust binary — no runtime, no Python, no Electron. Both Windows and macOS ship the full install: invisible background daemon, auto-start on login, focus-aware clipboard toggling, and an **SSH bridge** that auto-`scp`s the screenshot to the remote host when the foreground terminal is currently SSH'd somewhere.

## Why

Terminal AI agent CLIs don't accept binary clipboard paste — on Windows it usually fails outright; on macOS Claude Code wires it natively but Gemini / Codex are inconsistent (Gemini wants Alt+V, not Ctrl+V, and has known regressions). Clip-Claude fills the gap **without modifying any agent CLI**, using the most natural shortcut: **Ctrl+V**.

## Agent compatibility

| Agent | Status | Notes |
|---|---|---|
| **Claude Code** | ✅ Verified | The TUI auto-collapses the 4-line payload to `[Pasted text #1, +4 lines]`. On submit the Read tool opens the file. |
| **Gemini CLI** | ⚠️ Designed-for, not yet verified end-to-end | Gemini is multimodal and the LLM should follow the inlined instruction to read the path. The 4 lines will render expanded in the TUI rather than collapsed. |
| **Codex (OpenAI)** | ⚠️ Designed-for, not yet verified end-to-end | Same story as Gemini — multimodal model + file-read tool should pick up the path. |
| **Any other multimodal agent with a file-read tool** | Likely | Mechanism is agent-agnostic: a clipboard text payload pointing at a file. |

Reports from real-world Gemini / Codex sessions welcome via Issues.

## Safe by default

Clip-Claude **does not destroy your image** — and it doesn't pollute Slack / Mail compose boxes with the text payload either. A focus-aware state machine decides the clipboard contents per 150 ms poll:

| Foreground app           | Clipboard carries                                  |
|--------------------------|----------------------------------------------------|
| Non-terminal (Slack, Photoshop, browser) | Image only — no text                  |
| Local terminal (Windows Terminal, iTerm2, VS Code) | Image + **local** path text |
| SSH'd terminal           | Image + **remote** `/tmp/...` path (auto-scp'd)    |

Image formats (CF_DIB on Windows, `public.png` on macOS) match what Snipping Tool and macOS screenshots natively produce, so chat apps accept them. The state machine toggles in-place when you switch windows, so the wrong app never sees the wrong thing.

## Install

One command on every platform (clone + run install):

```bash
git clone https://github.com/GeniusPudding/Clip-Claude.git
cd Clip-Claude
.\install.ps1   # Windows
./install.sh    # macOS / Linux
```

Idempotent — re-run any time to upgrade or repair.

### Windows

The installer:

1. Installs `rustup` (stable, minimal profile) if missing.
2. Builds release binaries with `cargo build --release`.
3. Copies `clip-claude.exe` + `clip-claude-bg.exe` to `%LOCALAPPDATA%\Clip-Claude\`.
4. Registers `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\Clip-Claude` pointing at `clip-claude-bg.exe` so the invisible background daemon launches at every login.
5. Spawns the daemon immediately — no reboot, no shell restart needed.

Verify:

```powershell
& "$env:LOCALAPPDATA\Clip-Claude\clip-claude.exe" status
```

Expected:
```
  ok    installed at C:\Users\you\AppData\Local\Clip-Claude
  ok    auto-start registry entry present
  ok    daemon running
```

### macOS

The installer:

1. Installs `rustup` (stable, minimal profile) if missing.
2. Builds release binaries with `cargo build --release`.
3. Copies `clip-claude` to `~/Library/Application Support/Clip-Claude/`.
4. Writes a LaunchAgent plist at `~/Library/LaunchAgents/com.clip-claude.daemon.plist` and `launchctl bootstrap gui/$UID` it so the daemon launches at every login.
5. Spawns the daemon immediately.

Multi-format clipboard write (image + text coexist) is via raw `NSPasteboard` — `public.png` for the image, `public.utf8-plain-text` for the path. Focus detection uses `NSWorkspace.frontmostApplication.bundleIdentifier`.

Verify:

```bash
"$HOME/Library/Application Support/Clip-Claude/clip-claude" status
```

### Linux

`./install.sh` builds the binary at `target/release/clip-claude`. Auto-start is not wired for Linux in this release — run `target/release/clip-claude start` manually.

## SSH bridge

When the foreground terminal has an `ssh` subprocess (the daemon walks the process tree of the foreground window's PID), Clip-Claude `scp`s the screenshot to the remote host's `/tmp/` and injects the **remote** path into the clipboard. From the agent's point of view on the remote machine, the file path in the pasted text just exists locally and the Read tool opens it.

Mechanics:

1. New screenshot detected → save PNG to `~/.clip-claude/cache/` and augment the local clipboard with the local path (so a fast paste into a local terminal still works).
2. Each 150 ms poll, `reconcile()` re-evaluates focus + SSH state.
3. If the foreground terminal is SSH'd to `user@host`:
   - First time: `scp -B -q -o ConnectTimeout=5 <png> user@host:/tmp/clip-claude-<ts>.png`
   - Subsequent: cached per `user@host`, no re-upload until a new screenshot lands
4. Clipboard text gets rewritten with the remote `/tmp/...` path. The agent on the remote machine reads that path.
5. Switching to a local terminal switches the text back to the local path; switching to Slack removes the text entirely.

Requirements:

- `scp` on PATH (comes with OpenSSH; standard on macOS / Windows 10+).
- SSH key auth — `scp -B` (batch mode) fails on password prompts. Verify with `ssh -o BatchMode=yes user@host true` first.
- Host fingerprint already in `~/.ssh/known_hosts` (do one interactive `ssh user@host` to accept it).
- The `ssh` argv parser handles common flags (`-p -i -o -l -L -R -D -F -E -J -W` etc.) and `~/.ssh/config` host aliases (passed verbatim to scp, which resolves them the same way).

If scp fails (no network, auth issue, no scp on PATH), the daemon logs the error and falls back to the local path — so local-terminal pastes still work, only the remote case loses.

## Uninstall

```bash
.\uninstall.ps1   # Windows — stops daemon, removes Run-key entry, leaves binaries
./uninstall.sh    # macOS / Linux — kills the watcher process if running
```

Repo files stay on disk. Delete `%LOCALAPPDATA%\Clip-Claude\` (or the cloned repo) manually for a clean wipe.

## Subcommands

| Command                    | Description                                                              |
|----------------------------|--------------------------------------------------------------------------|
| `clip-claude install`      | Win: copy binaries to `%LOCALAPPDATA%\Clip-Claude\`, register HKCU Run-key. macOS: copy to `~/Library/Application Support/Clip-Claude/`, write LaunchAgent plist, `launchctl bootstrap`. Both start the daemon immediately. |
| `clip-claude uninstall`    | Win: stop daemon, remove Run-key. macOS: `launchctl bootout`, remove plist. Leaves binaries. |
| `clip-claude status`       | Report install + auto-start + running state.                             |
| `clip-claude`              | Alias for `clip-claude start`. Foreground watcher.                       |
| `clip-claude start`        | Run the watcher in the foreground until Ctrl+C.                          |
| `clip-claude run -- CMD`   | Wrap `CMD`. Watcher lives for the lifetime of `CMD`.                     |
| `clip-claude doctor`       | Sanity-check clipboard access and cache dir.                             |
| `clip-claude --version`    | Print version.                                                           |
| `clip-claude --help`       | Print help.                                                              |

`clip-claude-bg.exe` (Windows) is the same watcher built with `windows_subsystem = "windows"` so it shows no console window. On macOS the LaunchAgent points at `clip-claude start` directly; launchd handles backgrounding.

## How it works

1. Polls the system clipboard via `arboard` every 150 ms.
2. Tracks the OS clipboard sequence number (Windows `GetClipboardSequenceNumber`, macOS `NSPasteboard.changeCount`) to distinguish own writes from external changes.
3. When a new image-only clipboard appears — fresh screenshot — reads the RGBA buffer, saves a PNG at `~/.clip-claude/cache/clip_<timestamp>.png`, and immediately writes image + local-path text to the clipboard (so a fast paste into a local terminal always works).
4. Each subsequent poll calls `reconcile()`, which calls `decide_text_path()`:
   - foreground app not a terminal → `None` (image only, text stripped)
   - foreground terminal, not SSH'd → current local path
   - foreground terminal, SSH'd → remote `/tmp/...` path (scp once per host, cached)
5. `reconcile()` only rewrites the clipboard when the desired path differs from `cap.current_text_path`. No-op polls = no clipboard noise.
6. The injected text is a 4-line payload:
   ```
   [Clip-Claude] Pasted image (1920x1080)
   File: <local-or-remote-path-to-png>
   Please open and analyze this file using your image-reading tool.
   (This text was auto-injected because the terminal cannot display images directly.)
   ```
7. **Windows raw Win32**: `OpenClipboard` → `EmptyClipboard` → `SetClipboardData(CF_DIB, ...)` → `SetClipboardData(CF_UNICODETEXT, ...)` → `CloseClipboard`. CF_DIB is byte-for-byte `.NET Clipboard.SetImage` output; system auto-synthesizes CF_BITMAP and CF_DIBV5.
8. **macOS NSPasteboard**: `clearContents` → `setData:forType:public.png` → `setString:forType:public.utf8-plain-text`.
9. Pasting in an agent CLI picks up the text (auto-collapsed to `[Pasted text #1, +4 lines]` in Claude Code). On submit, the agent's Read tool opens the file path; the multimodal model sees the image.
10. Pasting in Photoshop / Telegram / Discord / Word picks up the image format — chat apps accept it because the bytes are identical to native screenshot output.
11. Cache is purged of files older than 7 days on each new capture.

### Coexists with other plugins

- **[Kaikou-Claude](https://github.com/GeniusPudding/Kaikou-Claude) (voice → agent)**: the voice daemon writes transcribed text to the clipboard then sends Ctrl+V. Clip-Claude sees text-on-clipboard and stays out. They share the clipboard cleanly, in either order of operations.
- **[Listen-Claude](https://github.com/GeniusPudding/Listen-Claude) (agent → voice)**: runs in the Claude Code `Stop` hook, has no clipboard interaction — fully orthogonal.
- **General clipboard managers**: Clip-Claude's CF_DIB write is identical to a normal screenshot, so clipboard history tools record it as just another screenshot — no surprises.

### Platform notes

- **Windows**: byte-matched CF_DIB (chat-app compatible), windowless `clip-claude-bg.exe` daemon, auto-start via HKCU Run key.
- **macOS**: `NSPasteboard` public.png + public.utf8-plain-text, LaunchAgent at `~/Library/LaunchAgents/com.clip-claude.daemon.plist`, focus detection via `NSWorkspace`.
- **Linux**: builds, but the focus-aware state machine and multi-format write are not wired. Falls back to text-only via `arboard`.
- **SSH bridge**: works from any host running the daemon. Detection is `sysinfo`-based process-tree walking; upload uses your existing `~/.ssh/config` + SSH keys via the standard `scp` binary.

## Architecture

```
src/
├── lib.rs            # module declarations
├── main.rs           # CLI entry — dispatches subcommands, doctor
├── bg.rs             # background entry — windows_subsystem = "windows" on Win, plain on macOS
├── cli.rs            # clap definitions
├── watcher.rs        # 150 ms polling loop + focus-aware state machine (reconcile)
├── focus.rs          # foreground-window terminal detection (Win + macOS)
├── clipboard_io.rs   # multi-format write — CF_DIB on Win, NSPasteboard public.png on macOS
├── ssh_session.rs    # detect ssh subprocess under foreground PID, parse argv → target
├── remote.rs         # scp -B upload of cached PNG to remote /tmp/
├── cache.rs          # save PNG to ~/.clip-claude/cache/, purge >7d
├── inject.rs         # format the text payload
├── runner.rs         # `run -- CMD` subprocess wrapper
└── install.rs        # install / uninstall / status — HKCU Run-key on Win, LaunchAgent on macOS
```

Both binaries share all modules through the `clip_claude` lib.

## Toolchain

See [docs/toolchain.md](docs/toolchain.md).

## Development

```bash
./scripts/dev.sh -- start         # cargo run --bin clip-claude -- start
./scripts/dev.sh -- doctor
cargo test
cargo fmt && cargo clippy --all-targets -- -D warnings
```

## Roadmap

- [x] Multi-format clipboard — image + text coexist, every app paste does the right thing.
- [x] Byte-match .NET CF_DIB — confirmed compatible with LINE / Telegram / Discord / Photoshop / browser paste targets.
- [x] Windows install command — one-line install, invisible background daemon, auto-start at login.
- [x] Focus-aware state machine — Slack and other rich-paste apps get image-only; terminals get image + path.
- [x] macOS install — NSPasteboard multi-type write + `launchctl` LaunchAgent.
- [x] SSH bridge — auto-scp + remote-path injection when foreground terminal is SSH'd.
- [ ] End-to-end verify Gemini CLI and Codex paste behavior; document any TUI quirks (text expansion, file-read tool naming).
- [ ] `--agent <claude|gemini|codex>` flag — tailor the text payload to each agent's preferred file-mention syntax (e.g. `@path` for Gemini).
- [ ] `clip-claude restore` — re-emit the most recent cached PNG onto the clipboard (image-only) on demand.
- [ ] Background scp + two-stage clipboard update — don't block the poll loop on slow uploads.
- [ ] Linux install: xclip / wl-clipboard multi-type write + systemd user service.

## License

MIT.
