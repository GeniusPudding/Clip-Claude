# clipbridge

Paste screenshots straight into terminal AI agents — Claude Code, Gemini CLI, Codex — as if they natively supported image paste. **And** keep pasting them into Photoshop, Telegram, Discord, anywhere else, exactly as before.

```
                              ┌───────────────────────────────────────────┐
                              │  clipbridge augments the clipboard:       │
[ screenshot in clipboard ] ─►│    keeps the image (CF_DIBV5)             │
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

A tiny native Rust binary — no runtime, no Python, no Electron. Cross-platform code (Windows · macOS · Linux); Windows ships polished multi-format writing, other platforms fall back to text-only.

## Why

Terminal AI agent CLIs don't accept binary clipboard paste — on Windows it usually fails outright; on macOS Claude Code wires it natively but Gemini / Codex are inconsistent. clipbridge fills the gap **without modifying any agent CLI**.

## Safe by default

clipbridge **does not destroy your image**. After capture, the clipboard carries both formats simultaneously:

- `CF_DIBV5` — the original image (system also synthesizes `CF_DIB` and `CF_BITMAP`)
- `CF_UNICODETEXT` — a self-describing 4-line text payload pointing at a saved PNG

Apps that paste images (Photoshop, Telegram, Discord, Word, browser inputs) take the image. Apps that only paste text (Claude Code, Gemini CLI, Codex, Notepad, VS Code) take the text payload. No app sees both; no app sees the wrong thing.

## Quick Start

### Windows

```powershell
git clone https://github.com/<you>/clipbridge.git
cd clipbridge
./scripts/setup.ps1     # installs rustup if missing, builds release binary
```

### macOS / Linux

```bash
git clone https://github.com/<you>/clipbridge.git
cd clipbridge
./scripts/setup.sh
```

The compiled binary lands at `target/release/clipbridge` (or `clipbridge.exe`). Add `target/release/` to your PATH or copy the binary somewhere on PATH.

## Usage

### 1. Foreground daemon (default)

```bash
clipbridge          # equivalent to `clipbridge start`
```

Runs in the foreground until Ctrl+C. While running, every screenshot you take gets both the original image and a text path payload on the clipboard, simultaneously. Pasting in any app does the right thing for that app.

Run it in a side terminal, open your agents elsewhere normally.

### 2. Wrapper mode

```bash
clipbridge run -- claude
clipbridge run -- gemini
clipbridge run -- codex --model gpt-5
```

Starts the watcher, runs the wrapped command, stops the watcher when the wrapped command exits. The wrapped agent's stdio is passthrough — its UI is identical to running directly.

**Tip — make the wrapper invisible:** add a shell function so `claude` automatically goes through clipbridge:

```bash
# ~/.zshrc or ~/.bashrc
claude() { clipbridge run -- claude "$@" }
```

```powershell
# PowerShell profile ($PROFILE)
function claude { clipbridge run -- claude.cmd @args }
```

After this, `claude` works exactly as before — but every screenshot during the session is paste-able.

### Verify environment

```bash
clipbridge doctor
```

Prints clipboard-access check + cache directory location.

## Subcommands

| Command                  | Description                                                              |
|--------------------------|--------------------------------------------------------------------------|
| `clipbridge`             | Alias for `clipbridge start`. Foreground watcher.                        |
| `clipbridge start`       | Run the watcher in the foreground until Ctrl+C.                          |
| `clipbridge run -- CMD`  | Wrap `CMD`. Watcher lives for the lifetime of `CMD`.                     |
| `clipbridge doctor`      | Sanity-check clipboard access and cache dir.                             |
| `clipbridge --version`   | Print version.                                                           |
| `clipbridge --help`      | Print help.                                                              |

## How it works

1. Polls the system clipboard via `arboard` every 150 ms.
2. Skips iterations where the clipboard contains text (nothing to do; either user copied text, or we already augmented this image).
3. When the clipboard has an image **without** text — fresh screenshot — reads the RGBA buffer.
4. Saves it as a PNG at `~/.clipbridge/cache/clip_<timestamp>.png`.
5. Builds a `BITMAPV5HEADER` DIB from the RGBA bytes (with proper alpha mask + sRGB color space) and a 4-line UTF-16 text payload:
   ```
   [clipbridge] Pasted image (1920x1080)
   File: C:\Users\you\.clipbridge\cache\clip_20260512_143022_815.png
   Please open and analyze this file using your image-reading tool.
   (This text was auto-injected because the terminal cannot display images directly.)
   ```
6. Raw Win32: `OpenClipboard` → `EmptyClipboard` → `SetClipboardData(CF_DIBV5, ...)` → `SetClipboardData(CF_UNICODETEXT, ...)` → `CloseClipboard`. The system auto-synthesizes `CF_DIB` and `CF_BITMAP` from V5.
7. When you Ctrl+V in an agent CLI, multi-line text auto-collapses (Claude Code shows `[Pasted text #1, +4 lines]`). On submit, the agent's Read tool opens the file path — modern multimodal LLMs see the image as image.
8. When you Ctrl+V in Photoshop / Telegram / Discord / Word, those apps request `CF_DIB` or `CF_BITMAP`, the system serves the synthesized format, you get your image.
9. Cache is purged of files older than 7 days on each new capture.

### Why image-only clipboards (no text)?

If the clipboard already has text — web-page copy with HTML+text+image, manually copied text alongside an image, or our own previously-written augment — we skip. This prevents loops and avoids stomping on rich content.

### Platform notes

- **Windows**: full multi-format support via raw Win32 (`Win32_System_DataExchange` + `Win32_System_Memory`).
- **macOS / Linux**: image+text multi-type write is not yet implemented; the fallback is text-only (image is replaced). macOS users running Claude Code typically don't need clipbridge anyway (Cmd+V is wired natively). For Gemini / Codex on those platforms, the text-only fallback still gets the path to the agent; you just lose the ability to also paste the image into an image app. Multi-type writes (NSPasteboard / wl-copy / xclip targets) are on the roadmap.

## Architecture

```
src/
├── main.rs           # entry, dispatches subcommands, doctor
├── cli.rs            # clap definitions
├── watcher.rs        # 150 ms polling loop, guards on text-presence
├── clipboard_io.rs   # Win32 BITMAPV5HEADER builder + multi-format clipboard write
├── cache.rs          # save PNG to ~/.clipbridge/cache/, purge >7d
├── inject.rs         # format the text payload
└── runner.rs         # `run -- CMD` subprocess wrapper
```

Polling at 150 ms: sub-second so the watcher feels instant; polling keeps clipboard hook code one tight loop and trivially portable to other platforms.

## Toolchain

See [docs/toolchain.md](docs/toolchain.md).

## Development

```bash
./scripts/dev.sh -- start         # cargo run -- start (debug build)
./scripts/dev.sh -- doctor
cargo test
cargo fmt && cargo clippy --all-targets -- -D warnings
```

## Roadmap

- [x] **Multi-format clipboard** — image + text coexist, every app paste does the right thing without focus detection.
- [ ] `clipbridge install` — auto-write a shell function/alias to `$PROFILE` / `.zshrc` / `.bashrc` so wrapped agents are invisible to set up.
- [ ] Hidden background daemon (no console window) + Task Scheduler entry → "always on" with no visible process.
- [ ] `clipbridge restore` — re-emit the most recent cached PNG onto the clipboard (image-only) on demand.
- [ ] macOS NSPasteboard multi-type write (image + text simultaneously).
- [ ] Linux X11 / Wayland multi-target clipboard write.
- [ ] GitHub Actions release matrix → prebuilt binaries for win-x64 / mac-arm64 / mac-x64 / linux-x64.
- [ ] One-line install script (`irm .../install.ps1 | iex` / `curl ... | sh`).

## License

MIT.
