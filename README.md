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

A tiny native Rust binary — no runtime, no Python, no Electron. Windows ships the polished install + invisible background daemon; macOS / Linux builds but auto-start is not wired (Claude Code on macOS already supports native Cmd+V paste).

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

Clip-Claude **does not destroy your image**. After capture, the clipboard carries both formats simultaneously:

- `CF_DIB` — the original image, byte-for-byte matching what Snipping Tool / .NET `Clipboard.SetImage` produce (32-bit BI_BITFIELDS bottom-up). The system synthesizes `CF_BITMAP` and `CF_DIBV5` from this.
- `CF_UNICODETEXT` — a self-describing 4-line text payload pointing at a saved PNG.

Image-paste apps (Photoshop, Telegram, Discord, Word, browser inputs) take the image. Apps that only paste text (Claude Code, Gemini CLI, Codex, Notepad, VS Code) take the text payload. No app sees the wrong thing.

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

### macOS / Linux

`./install.sh` builds the binary at `target/release/clip-claude`. **Auto-start on login is not wired** for these platforms in this release — copy the binary onto your PATH and run `clip-claude start` manually when you want it active. On macOS, Claude Code already supports native Cmd+V image paste, so this matters mainly for Gemini / Codex use cases.

Multi-format clipboard write (image + text coexist) is currently Windows-only. On macOS / Linux the fallback is text-only via `arboard` — the image is replaced by the path text on paste. Roadmap item: NSPasteboard / xclip multi-type support.

## Uninstall

```bash
.\uninstall.ps1   # Windows — stops daemon, removes Run-key entry, leaves binaries
./uninstall.sh    # macOS / Linux — kills the watcher process if running
```

Repo files stay on disk. Delete `%LOCALAPPDATA%\Clip-Claude\` (or the cloned repo) manually for a clean wipe.

## Subcommands

| Command                    | Description                                                              |
|----------------------------|--------------------------------------------------------------------------|
| `clip-claude install`      | Copy binaries to `%LOCALAPPDATA%\Clip-Claude\`, register auto-start, run.|
| `clip-claude uninstall`    | Stop the daemon, remove auto-start. Leaves binaries.                     |
| `clip-claude status`       | Report install + auto-start + running state.                             |
| `clip-claude`              | Alias for `clip-claude start`. Foreground watcher (visible console).     |
| `clip-claude start`        | Run the watcher in the foreground until Ctrl+C.                          |
| `clip-claude run -- CMD`   | Wrap `CMD`. Watcher lives for the lifetime of `CMD`.                     |
| `clip-claude doctor`       | Sanity-check clipboard access and cache dir.                             |
| `clip-claude --version`    | Print version.                                                           |
| `clip-claude --help`       | Print help.                                                              |

`clip-claude-bg.exe` is the same watcher built with `windows_subsystem = "windows"` so it shows no console window. The install command points the Run-key entry at it.

## How it works

1. Polls the system clipboard via `arboard` every 150 ms.
2. Skips iterations where the clipboard already contains text — either user copied text, or we already augmented this image.
3. When the clipboard has an image **without** text — fresh screenshot — reads the RGBA buffer.
4. Saves it as a PNG at `~/.clip-claude/cache/clip_<timestamp>.png`.
5. Builds a CF_DIB byte-for-byte matching `.NET`'s reference output (40-byte `BITMAPINFOHEADER` · `biCompression = BI_BITFIELDS` · 32-bit · positive `biHeight` for bottom-up · R/G/B color masks · BGRA pixel data in reverse row order).
6. Builds a 4-line UTF-16 text payload:
   ```
   [Clip-Claude] Pasted image (1920x1080)
   File: C:\Users\you\.clip-claude\cache\clip_20260515_143022_815.png
   Please open and analyze this file using your image-reading tool.
   (This text was auto-injected because the terminal cannot display images directly.)
   ```
7. Raw Win32: `OpenClipboard` → `EmptyClipboard` → `SetClipboardData(CF_DIB, ...)` → `SetClipboardData(CF_UNICODETEXT, ...)` → `CloseClipboard`. The system auto-synthesizes `CF_BITMAP` and `CF_DIBV5`.
8. Pasting in an agent CLI picks up the text (auto-collapsed to `[Pasted text #1, +4 lines]` in Claude Code). On submit, the agent's Read tool opens the file path; the multimodal model sees the image.
9. Pasting in Photoshop / Telegram / Discord / Word picks up the CF_DIB or synthesized CF_BITMAP — chat apps accept it because the bytes are identical to Snipping Tool's output.
10. Cache is purged of files older than 7 days on each new capture.

### Coexists with other plugins

- **[Kaikou-Claude](https://github.com/GeniusPudding/Kaikou-Claude) (voice → agent)**: the voice daemon writes transcribed text to the clipboard then sends Ctrl+V. Clip-Claude sees text-on-clipboard and stays out. They share the clipboard cleanly, in either order of operations.
- **[Listen-Claude](https://github.com/GeniusPudding/Listen-Claude) (agent → voice)**: runs in the Claude Code `Stop` hook, has no clipboard interaction — fully orthogonal.
- **General clipboard managers**: Clip-Claude's CF_DIB write is identical to a normal screenshot, so clipboard history tools record it as just another screenshot — no surprises.

### Platform notes

- **Windows**: full install path, byte-matched CF_DIB, windowless background daemon, auto-start via HKCU Run key.
- **macOS / Linux**: build works; image+text multi-type write and auto-start are not implemented. The fallback for now is text-only (image is replaced) which is acceptable when Claude Code's native Cmd+V image paste already covers the mac case.

## Architecture

```
src/
├── lib.rs            # module declarations
├── main.rs           # `clip-claude.exe` entry — dispatches subcommands, doctor
├── bg.rs             # `clip-claude-bg.exe` entry — windows_subsystem = "windows"
├── cli.rs            # clap definitions
├── watcher.rs        # 150 ms polling loop, guards on text-presence
├── clipboard_io.rs   # Win32 CF_DIB builder (byte-matches .NET) + multi-format write
├── cache.rs          # save PNG to ~/.clip-claude/cache/, purge >7d
├── inject.rs         # format the text payload
├── runner.rs         # `run -- CMD` subprocess wrapper
└── install.rs        # install / uninstall / status (HKCU Run-key based)
```

`clip-claude.exe` and `clip-claude-bg.exe` share all modules through the `clip_claude` lib.

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
- [ ] End-to-end verify Gemini CLI and Codex paste behavior; document any TUI quirks (text expansion, file-read tool naming).
- [ ] `--agent <claude|gemini|codex>` flag — tailor the text payload to each agent's preferred file-mention syntax (e.g. `@path` for Gemini).
- [ ] `clip-claude restore` — re-emit the most recent cached PNG onto the clipboard (image-only) on demand.
- [ ] macOS install: NSPasteboard multi-type write + `launchctl` LaunchAgent.
- [ ] Linux install: xclip / wl-clipboard multi-type write + systemd user service.

## License

MIT.
