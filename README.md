# clipbridge

Paste screenshots straight into terminal AI agents — Claude Code, Gemini CLI, Codex — as if they natively supported image paste. **And** keep pasting them into Photoshop, Telegram, Discord, anywhere else, exactly as before.

```
                              ┌───────────────────────────────────────────┐
                              │  clipbridge augments the clipboard:       │
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

A tiny native Rust binary — no runtime, no Python, no Electron. Cross-platform code (Windows · macOS · Linux); Windows ships the polished install + invisible background daemon, other platforms build but require manual setup.

## Why

Terminal AI agent CLIs don't accept binary clipboard paste — on Windows it usually fails outright; on macOS Claude Code wires it natively but Gemini / Codex are inconsistent. clipbridge fills the gap **without modifying any agent CLI**.

## Safe by default

clipbridge **does not destroy your image**. After capture, the clipboard carries both formats simultaneously:

- `CF_DIB` — the original image, byte-for-byte matching what Snipping Tool / .NET `Clipboard.SetImage` produce (32-bit BI_BITFIELDS bottom-up). The system synthesizes `CF_BITMAP` and `CF_DIBV5` from this.
- `CF_UNICODETEXT` — a self-describing 4-line text payload pointing at a saved PNG.

Image-paste apps (Photoshop, Telegram, Discord, Word, browser inputs) take the image. Apps that only paste text (Claude Code, Gemini CLI, Codex, Notepad, VS Code) take the text payload. No app sees the wrong thing.

## Quick Start (Windows — one line)

```powershell
git clone https://github.com/<you>/clipbridge.git
cd clipbridge
./scripts/install.ps1
```

`install.ps1` installs `rustup` if missing, builds release binaries, copies them to `%LOCALAPPDATA%\clipbridge\`, registers an HKCU Run-key entry so the invisible background daemon launches at every login, and starts it immediately. After this, **every screenshot you take paste-works in every Windows app you care about** — agents, chat apps, image editors — for the rest of your computing life. Reboot survives.

To verify:

```powershell
clipbridge status     # if you added %LOCALAPPDATA%\clipbridge to PATH, or:
%LOCALAPPDATA%\clipbridge\clipbridge.exe status
```

Expected:
```
  ok    installed at C:\Users\you\AppData\Local\clipbridge
  ok    auto-start registry entry present
  ok    daemon running
```

To remove:

```powershell
%LOCALAPPDATA%\clipbridge\clipbridge.exe uninstall
```

Stops the daemon, removes the Run-key entry, leaves the binaries (delete the folder manually if you want them gone).

### macOS / Linux

```bash
git clone https://github.com/<you>/clipbridge.git
cd clipbridge
./scripts/install.sh
```

Builds the binary at `target/release/clipbridge`. Auto-start / windowless daemon are not yet wired for macOS / Linux — copy the binary onto your PATH and run `clipbridge` manually when you want it active. (See Roadmap.)

## Subcommands

| Command                  | Description                                                              |
|--------------------------|--------------------------------------------------------------------------|
| `clipbridge install`     | Copy binaries to `%LOCALAPPDATA%\clipbridge\`, register auto-start, run. |
| `clipbridge uninstall`   | Stop the daemon, remove auto-start. Leaves binaries.                     |
| `clipbridge status`      | Report install + auto-start + running state.                             |
| `clipbridge`             | Alias for `clipbridge start`. Foreground watcher (visible console).      |
| `clipbridge start`       | Run the watcher in the foreground until Ctrl+C.                          |
| `clipbridge run -- CMD`  | Wrap `CMD`. Watcher lives for the lifetime of `CMD`.                     |
| `clipbridge doctor`      | Sanity-check clipboard access and cache dir.                             |
| `clipbridge --version`   | Print version.                                                           |
| `clipbridge --help`      | Print help.                                                              |

`clipbridge-bg.exe` is the same watcher built with `windows_subsystem = "windows"` so it shows no console window. The install command points the Run-key entry at it.

## How it works

1. Polls the system clipboard via `arboard` every 150 ms.
2. Skips iterations where the clipboard already contains text — either user copied text, or we already augmented this image.
3. When the clipboard has an image **without** text — fresh screenshot — reads the RGBA buffer.
4. Saves it as a PNG at `~/.clipbridge/cache/clip_<timestamp>.png`.
5. Builds a CF_DIB byte-for-byte matching `.NET`'s reference output (40-byte `BITMAPINFOHEADER` · `biCompression = BI_BITFIELDS` · 32-bit · positive `biHeight` for bottom-up · R/G/B color masks · BGRA pixel data in reverse row order).
6. Builds a 4-line UTF-16 text payload:
   ```
   [clipbridge] Pasted image (1920x1080)
   File: C:\Users\you\.clipbridge\cache\clip_20260514_143022_815.png
   Please open and analyze this file using your image-reading tool.
   (This text was auto-injected because the terminal cannot display images directly.)
   ```
7. Raw Win32: `OpenClipboard` → `EmptyClipboard` → `SetClipboardData(CF_DIB, ...)` → `SetClipboardData(CF_UNICODETEXT, ...)` → `CloseClipboard`. The system auto-synthesizes `CF_BITMAP` and `CF_DIBV5`.
8. Pasting in an agent CLI picks up the text (auto-collapsed to `[Pasted text #1, +4 lines]` in Claude Code). On submit, the agent's Read tool opens the file path; the multimodal model sees the image.
9. Pasting in Photoshop / Telegram / Discord / Word picks up the CF_DIB or synthesized CF_BITMAP, exactly as if Snipping Tool had been the source. Tested against the byte layout that `System.Windows.Forms.Clipboard.SetImage` emits — chat apps accept it.
10. Cache is purged of files older than 7 days on each new capture.

### Coexists with other plugins

- **Voice input plugins (e.g. Kaikou / claude-voice-zh)**: the voice daemon writes transcribed text to the clipboard then sends Ctrl+V. clipbridge sees text-on-clipboard and stays out. They share the clipboard cleanly, in either order of operations.
- **General clipboard managers**: clipbridge's CF_DIB write is identical to a normal screenshot, so clipboard history tools record it as just another screenshot — no surprises.

### Platform notes

- **Windows**: full install path, byte-matched CF_DIB, windowless background daemon, auto-start via HKCU Run key.
- **macOS / Linux**: build works; image+text multi-type write and auto-start are TODO. The fallback for now is text-only (image is replaced), which is acceptable when Claude Code's native Cmd+V image paste already covers the mac case.

## Architecture

```
src/
├── lib.rs            # module declarations
├── main.rs           # entry, dispatches subcommands, doctor
├── bg.rs             # `windows_subsystem = "windows"` entry — runs the watcher
├── cli.rs            # clap definitions
├── watcher.rs        # 150 ms polling loop, guards on text-presence
├── clipboard_io.rs   # Win32 CF_DIB builder (byte-matches .NET) + multi-format write
├── cache.rs          # save PNG to ~/.clipbridge/cache/, purge >7d
├── inject.rs         # format the text payload
├── runner.rs         # `run -- CMD` subprocess wrapper
└── install.rs        # install / uninstall / status (HKCU Run-key based)
```

`clipbridge.exe` and `clipbridge-bg.exe` share all modules through the `clipbridge` lib.

Polling at 150 ms: sub-second so the watcher feels instant; polling keeps clipboard hook code one tight loop and trivially portable.

## Toolchain

See [docs/toolchain.md](docs/toolchain.md).

## Development

```bash
./scripts/dev.sh -- start         # cargo run --bin clipbridge -- start (debug build)
./scripts/dev.sh -- doctor
cargo test
cargo fmt && cargo clippy --all-targets -- -D warnings
```

## Roadmap

- [x] **Multi-format clipboard** — image + text coexist, every app paste does the right thing.
- [x] **Byte-match .NET CF_DIB** — confirmed compatible with LINE / Telegram / Discord / Photoshop / browser paste targets.
- [x] **Windows install command** — one-line install, invisible background daemon, auto-start at login.
- [ ] Prebuilt binaries on GitHub Releases + `irm https://.../install.ps1 | iex` for users without Rust.
- [ ] `clipbridge restore` — re-emit the most recent cached PNG onto the clipboard (image-only) on demand.
- [ ] macOS install: NSPasteboard multi-type write + `launchctl` LaunchAgent.
- [ ] Linux install: X11 / Wayland multi-target clipboard write + systemd user unit.

## License

MIT.
