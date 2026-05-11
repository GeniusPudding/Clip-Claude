# clipbridge

Paste screenshots straight into terminal AI agents — Claude Code, Gemini CLI, Codex — as if they natively supported image paste.

```
[ screenshot in clipboard ]  ──►  clipbridge  ──►  [ multi-line text path on clipboard ]
                                                            │
                                                            ▼
                                                  Ctrl+V into agent prompt
                                                  agent's Read tool opens the file
                                                  multimodal model sees the image
```

A tiny native Rust binary — no runtime, no Python, no Electron. Cross-platform (Windows · macOS · Linux).

## Why

Most terminal AI agent CLIs don't accept binary clipboard paste. macOS terminals partially work; on Windows it usually doesn't work at all. This bridges the gap **without modifying the agent CLIs** — it converts image-on-clipboard into a self-explaining text payload that any agent's file-reading tool will pick up automatically.

## Safe by default

clipbridge is **focus-aware**: it only converts the clipboard when the foreground window's process tree contains an agent CLI (`claude`, `gemini`, `codex`, or `node` running one of them). Screenshots taken while you're in Photoshop, a chat app, your browser, or anywhere else **flow through untouched** — paste behaves normally. Pass `--all-windows` to opt into legacy "always convert" behavior.

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

The compiled binary lands at `target/release/clipbridge` (or `clipbridge.exe`). Add `target/release/` to your PATH, or copy the binary somewhere on PATH.

## Usage

There are two ways to run it.

### 1. Foreground daemon (default)

```bash
clipbridge          # equivalent to `clipbridge start`
```

Runs in the foreground. The watcher only acts when an agent CLI is in the foreground process tree — your daily clipboard usage is unaffected. Ctrl+C to stop.

Run it in a side terminal, then open Claude Code / Gemini CLI / Codex elsewhere normally; clipbridge will pick up screenshots only when one of those windows is active.

### 2. Wrapper mode

```bash
clipbridge run -- claude
clipbridge run -- gemini
clipbridge run -- codex --model gpt-5
```

Starts the watcher, runs the wrapped command, stops the watcher when the wrapped command exits. The wrapped agent's stdin/stdout/stderr are passthrough — UI is identical to running the agent directly. Focus check still applies, so Alt-Tabbing out of the wrapped agent pauses conversion.

**Tip — make the wrapper invisible:** add a shell function so `claude` automatically goes through clipbridge:

```bash
# ~/.zshrc or ~/.bashrc
claude() { clipbridge run -- claude "$@" }
```

```powershell
# PowerShell profile ($PROFILE)
function claude { clipbridge run -- claude.cmd @args }
```

After this, `claude` works exactly as before — but every screenshot you take during the session is paste-able.

### Verify environment

```bash
clipbridge doctor
```

Prints clipboard-access check, cache directory, current foreground window, and whether an agent is detected in its process tree right now.

## Subcommands and flags

| Command                                   | Description                                                                              |
|-------------------------------------------|------------------------------------------------------------------------------------------|
| `clipbridge`                              | Alias for `clipbridge start`. Runs the watcher in the foreground (focus-aware).          |
| `clipbridge start`                        | Run the watcher in the foreground until Ctrl+C. Default = focus-aware.                   |
| `clipbridge start --all-windows`          | Convert every image-only clipboard regardless of foreground window. **Affects all apps.**|
| `clipbridge run -- CMD`                   | Wrap `CMD`. Watcher lives for the lifetime of `CMD`. Focus-aware.                        |
| `clipbridge doctor`                       | Print clipboard / cache / foreground / agent-detection status.                           |
| `clipbridge --version`                    | Print version.                                                                           |
| `clipbridge --help`                       | Print help.                                                                              |

## How it works

1. Watches the system clipboard via `arboard` (a pure-Rust cross-platform clipboard crate).
2. Skips iterations where the clipboard contains text (no work to do).
3. **Focus check** (Windows): `GetForegroundWindow` → PID → walks the process tree via `sysinfo`. If any descendant matches `claude` / `gemini` / `codex`, or `node` running an agent script, the watcher proceeds. Otherwise it does nothing — the clipboard image is left alone for normal use elsewhere.
4. Reads the RGBA buffer.
5. Saves it as a PNG at `~/.clipbridge/cache/clip_<timestamp>.png`.
6. Replaces the clipboard contents with this 4-line text payload:
   ```
   [clipbridge] Pasted image (1920x1080)
   File: /Users/you/.clipbridge/cache/clip_20260506_143022_815.png
   Please open and analyze this file using your image-reading tool.
   (This text was auto-injected because the terminal cannot display images directly.)
   ```
7. When you Ctrl+V in an agent CLI, multi-line text gets auto-collapsed (Claude Code shows `[Pasted text #1, +4 lines]`). Visually clean.
8. On submit, the agent reads the path with its Read/file tool — modern multimodal LLMs (Claude, Gemini, GPT-5) all see the image as image, not text.
9. Cache is purged of files older than 7 days on each new capture.

The focus check is cached (2-second TTL keyed on foreground PID) so the `sysinfo` refresh runs at most every 2 seconds, not on every poll.

### Why image-only clipboards?

Web-page copies typically include both HTML/text and an image. We only act when the clipboard has **no text**, leaving normal copy-paste of rich content untouched.

### Platform notes

- **Windows**: full focus detection via `windows` crate (`user32` + sysinfo process-tree walk).
- **macOS / Linux**: focus detection currently returns `false` (treats foreground as "not an agent"). This means without `--all-windows` clipbridge is a no-op on these platforms. macOS users running Claude Code typically don't need clipbridge anyway (Cmd+V is wired natively). For other agents on macOS, build with `--all-windows` for now or wait for a focus implementation.

## Architecture

```
src/
├── main.rs       # entry, dispatches subcommands, doctor
├── cli.rs        # clap definitions
├── watcher.rs    # polling loop (150ms), guards on text-presence + focus
├── focus.rs      # GetForegroundWindow + sysinfo process-tree walk (Windows)
├── cache.rs      # save PNG to ~/.clipbridge/cache/, purge >7d
├── inject.rs     # format the text payload that replaces the clipboard
└── runner.rs     # `run -- CMD` subprocess wrapper
```

Polling at 150ms — chosen because:
- Sub-second to feel instant (typical user delay between screenshot and Ctrl+V is >1s).
- Native event-driven clipboard hooks differ per OS; polling keeps the code one tight loop and trivially portable.

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

- [x] **Focus-aware default** — only convert when an agent CLI is in the foreground process tree.
- [ ] `clipbridge install` — auto-write a shell function/alias to `$PROFILE` / `.zshrc` / `.bashrc` so wrapped agents are invisible to set up.
- [ ] Hidden background daemon (no console window) + Task Scheduler entry → "always on" with no visible process.
- [ ] `clipbridge restore` — pop the most recent original image back onto the clipboard (for pasting into Photoshop / chat apps if you want the raw image after a conversion).
- [ ] macOS focus detection (NSWorkspace foreground + process-tree walk).
- [ ] Linux focus detection (X11 / Wayland).
- [ ] GitHub Actions release matrix → prebuilt binaries for win-x64 / mac-arm64 / mac-x64 / linux-x64.
- [ ] One-line install script (`irm .../install.ps1 | iex` / `curl ... | sh`).

## License

MIT.
