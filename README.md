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

Runs in the foreground. Every screenshot you take while this is running gets converted. Ctrl+C to stop.

Run it in a side terminal, then open Claude Code / Gemini CLI / Codex elsewhere normally.

### 2. Wrapper mode (recommended)

```bash
clipbridge run -- claude
clipbridge run -- gemini
clipbridge run -- codex --model gpt-5
```

Starts the watcher, runs the wrapped command, stops the watcher when the wrapped command exits. The wrapped agent's stdin/stdout/stderr are passthrough — UI is identical to running the agent directly.

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

Prints clipboard-access check + cache directory location.

## Subcommands

| Command                  | Description                                                              |
|--------------------------|--------------------------------------------------------------------------|
| `clipbridge`             | Alias for `clipbridge start`. Runs the watcher in the foreground.        |
| `clipbridge start`       | Run the watcher in the foreground until Ctrl+C.                          |
| `clipbridge run -- CMD`  | Wrap `CMD`. Watcher lives for the lifetime of `CMD`.                     |
| `clipbridge doctor`      | Sanity-check clipboard access and cache dir.                             |
| `clipbridge --version`   | Print version.                                                           |
| `clipbridge --help`      | Print help.                                                              |

## How it works

1. Watches the system clipboard via `arboard` (a pure-Rust cross-platform clipboard crate).
2. When the clipboard contains **only an image** (no text — i.e. a screenshot, not a web copy), reads the RGBA buffer.
3. Saves it as a PNG at `~/.clipbridge/cache/clip_<timestamp>.png`.
4. Replaces the clipboard contents with this 4-line text payload:
   ```
   [clipbridge] Pasted image (1920x1080)
   File: /Users/you/.clipbridge/cache/clip_20260506_143022_815.png
   Please open and analyze this file using your image-reading tool.
   (This text was auto-injected because the terminal cannot display images directly.)
   ```
5. When you Ctrl+V in an agent CLI, multi-line text gets auto-collapsed (Claude Code shows `[Pasted text #1, +4 lines]`). Visually clean.
6. On submit, the agent reads the path with its Read/file tool — modern multimodal LLMs (Claude, Gemini, GPT-5) all see the image as image, not text.
7. Cache is purged of files older than 7 days on each new capture.

### Why image-only clipboards?

Web-page copies typically include both HTML/text and an image. We only act when the clipboard has **no text**, leaving normal copy-paste of rich content untouched.

## Architecture

```
src/
├── main.rs       # entry, dispatches subcommands, doctor
├── cli.rs        # clap definitions
├── watcher.rs    # polling loop (150ms), guards on text-presence
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

- [ ] `clipbridge restore` — pop the most recent original image back onto the clipboard (for pasting into Photoshop / chat apps after capture).
- [ ] Background daemon mode with proper start/stop/status commands and PID file.
- [ ] GitHub Actions release matrix → prebuilt binaries for win-x64 / mac-arm64 / mac-x64 / linux-x64.
- [ ] One-line install script (`irm .../install.ps1 | iex` / `curl ... | sh`).
- [ ] Wayland clipboard support (Linux).

## License

MIT.
