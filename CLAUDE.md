# Clip-Claude (即貼即克)

Cross-platform clipboard daemon that augments image-on-clipboard with a path-on-clipboard so terminal AI agents (Claude Code, Gemini CLI, Codex) can read screenshots as if they natively supported image paste, while leaving the image format intact for normal apps (Photoshop, chat, browsers).

Personal tool. Sibling to Kaikou-Claude (開口即克), which provides voice input for the same set of agent terminals. They coexist cleanly.

## Intent

Take a screenshot (Win+Shift+S), Ctrl+V into an agent's prompt, the agent sees the image. The trick: a tiny native daemon watches the clipboard, saves any image to disk, and **augments** the clipboard so it carries BOTH the original image (CF_DIB matching .NET / Snipping Tool byte-for-byte) and a 4-line text payload pointing at the saved file (CF_UNICODETEXT). Image-paste apps take the image; text-paste agent terminals take the path. No focus detection, no app sees the wrong thing.

## Conventions

- Toolchain: **Rust stable** via rustup.
- Build: `cargo build --release` (produces `clip-claude.exe` + `clip-claude-bg.exe`).
- End-user install / uninstall: `./install.{ps1,sh}` / `./uninstall.{ps1,sh}` at the repo root (matches Listen-Claude family layout). Both delegate to the `clip-claude install` / `clip-claude uninstall` subcommands on Windows.
- Dev helpers: `./scripts/setup.{ps1,sh}` (build only, no register) and `./scripts/dev.{ps1,sh}` (foreground watcher via `cargo run`).
- Tests: `cargo test`.
- Format + lint: `cargo fmt && cargo clippy --all-targets -- -D warnings`.

## Module layout

- `src/lib.rs` — module declarations. Both `clip-claude.exe` (CLI) and `clip-claude-bg.exe` (windowless background) consume this lib.
- `src/main.rs` — `clip-claude.exe` entry. Dispatches subcommands, runs `doctor`.
- `src/bg.rs` — `clip-claude-bg.exe` entry. `#![windows_subsystem = "windows"]` so no console allocates. Just calls `watcher::run_foreground()`.
- `src/cli.rs` — `clap` argument definitions.
- `src/watcher.rs` — 150 ms polling loop. Skips when clipboard already has text; otherwise reads image, saves PNG, calls `clipboard_io::write_image_and_text`.
- `src/clipboard_io.rs` — Win32 raw clipboard multi-format write. Builds a CF_DIB whose bytes match `.NET Clipboard.SetImage` exactly (40-byte BITMAPINFOHEADER, BI_BITFIELDS, 32-bit, bottom-up, R/G/B masks, BGRA pixels). The system synthesizes CF_BITMAP and CF_DIBV5 from this. Pairs with a CF_UNICODETEXT payload. Non-Windows fallback is text-only via arboard.
- `src/cache.rs` — write PNG to `~/.clip-claude/cache/`, purge files older than 7 days.
- `src/inject.rs` — format the text payload (prefix `[Clip-Claude]`).
- `src/runner.rs` — `run -- <cmd>` wrapper: starts watcher, spawns child, stops watcher on exit.
- `src/install.rs` — `install` / `uninstall` / `status` subcommands. Install copies binaries to `%LOCALAPPDATA%\Clip-Claude\`, registers `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\Clip-Claude` pointing at `clip-claude-bg.exe`, and spawns the daemon immediately. Uses `reg` rather than `schtasks` so no admin / sandboxed contexts work.

## Taboos

- No comments explaining WHAT (names do that). Comments only for WHY.
- No speculative abstractions.
- Use `anyhow::Result` everywhere — this is a binary, not a library.
- Don't read clipboard image when text is also present — only act on image-only clipboards. This is the loop guard *and* the user-rich-content guard.
- **Never replace** the clipboard image without also re-emitting it. Clip-Claude's contract is "image stays, text is added." Any future feature that breaks this contract is a regression.
- CF_DIB byte layout must match `.NET Clipboard.SetImage` exactly — chat apps key off that specific encoding.
