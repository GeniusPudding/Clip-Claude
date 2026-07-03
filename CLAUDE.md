# Clip-Claude (即貼即克)

Cross-platform clipboard daemon that augments image-on-clipboard with a path-on-clipboard so terminal AI agents (Claude Code, Gemini CLI, Codex) can read screenshots as if they natively supported image paste, while leaving the image format intact for normal apps (Photoshop, chat, browsers).

Personal tool. Sibling to Kaikou-Claude (開口即克), which provides voice input for the same set of agent terminals. They coexist cleanly.

## Intent

Take a screenshot, paste into an agent's prompt, the agent sees the image. The trick: a tiny native daemon watches the clipboard, saves any image to disk, and **augments** the clipboard so it carries BOTH the original image (Windows: CF_DIB matching .NET / Snipping Tool byte-for-byte; macOS: public.png on NSPasteboard) and a 4-line text payload (CF_UNICODETEXT / public.utf8-plain-text) pointing at the saved file. A focus-aware state machine decides per poll whether the text should be present, based on which app is in front:

- non-terminal app in front (Slack, browser, Photoshop) → image only, no text
- local terminal in front → image + local file path
- SSH'd terminal in front → image + path on the *remote* `/tmp/`. The daemon `scp`s the PNG on demand, caches per target, so subsequent pastes to the same host reuse one upload.

The state machine re-evaluates each 150 ms poll and toggles instantly when the user switches windows.

## Conventions

- Toolchain: **Rust stable** via rustup.
- Build: `cargo build --release` (produces `clip-claude.exe` + `clip-claude-bg.exe`).
- End-user install / uninstall: `./install.{ps1,sh}` / `./uninstall.{ps1,sh}` at the repo root (matches Listen-Claude family layout). Both delegate to the `clip-claude install` / `clip-claude uninstall` subcommands on Windows.
- Dev helpers: `./scripts/setup.{ps1,sh}` (build only, no register) and `./scripts/dev.{ps1,sh}` (foreground watcher via `cargo run`).
- Tests: `cargo test`.
- Format + lint: `cargo fmt && cargo clippy --all-targets -- -D warnings`.

## Module layout

- `src/lib.rs` — module declarations. Both `clip-claude(.exe)` (CLI) and `clip-claude-bg(.exe)` (windowless background) consume this lib.
- `src/main.rs` — CLI entry. Dispatches subcommands, runs `doctor`.
- `src/bg.rs` — background entry. `#![windows_subsystem = "windows"]` so no console allocates on Windows; on macOS it's plain (launchd handles backgrounding).
- `src/cli.rs` — `clap` argument definitions.
- `src/focus.rs` — foreground-window terminal detection.
  - **Windows**: `GetForegroundWindow` → `QueryFullProcessImageNameW`, match exe basename against `windowsterminal.exe`, `code.exe`, etc.
  - **macOS**: `NSWorkspace.frontmostApplication.bundleIdentifier`, match against `com.apple.Terminal`, `com.googlecode.iterm2`, `com.microsoft.VSCode`, etc.
- `src/watcher.rs` — 150 ms polling loop with focus-aware state machine on Windows + macOS. Tracks clipboard sequence numbers to distinguish own writes from external changes. Each `reconcile()` call computes a "desired text path" from `decide_text_path()` (None / local / remote) and rewrites the clipboard only if it differs from what's currently injected.
- `src/clipboard_io.rs` — platform-specific multi-format clipboard write.
  - **Windows**: raw Win32 `SetClipboardData` of CF_DIB + CF_UNICODETEXT. The CF_DIB layout matches `.NET Clipboard.SetImage` exactly (40-byte BITMAPINFOHEADER, BI_BITFIELDS, 32-bit, bottom-up, R/G/B masks, BGRA pixels) so the system synthesizes CF_BITMAP and CF_DIBV5.
  - **macOS**: `NSPasteboard.clearContents()` then `setData:forType:` for `public.png` and `setString:forType:` for `public.utf8-plain-text`.
  - Provides `write_image_and_text`, `write_image_only`, `get_sequence_number` on both.
- `src/ssh_session.rs` — detect SSH subprocess inside the foreground terminal. Uses `sysinfo` to walk the process tree from the foreground PID, looks for any descendant whose argv[0] basename is `ssh` or `mosh-client`, parses argv with a minimal flag-aware parser (skips values after `-p -i -o -l -L -R -D -F -E -J -W -b -c -e -I -m -O -Q -S`), returns the first positional as `SshTarget { spec }`. Caches result for 900 ms to avoid per-poll process scans.
- `src/remote.rs` — `upload(local_png, target)` → spawns `scp -B -q -o ConnectTimeout=5` to `target.spec:/tmp/clip-claude-{ts}.png`. Batch mode means SSH key auth is required (no password prompts). Failure → caller falls back to local path.
- `src/cache.rs` — write PNG to `~/.clip-claude/cache/`, purge files older than 7 days.
- `src/inject.rs` — format the text payload (prefix `[Clip-Claude]`).
- `src/runner.rs` — `run -- <cmd>` wrapper: starts watcher, spawns child, stops watcher on exit.
- `src/install.rs` — `install` / `uninstall` / `status` subcommands, per-platform.
  - **Windows**: binaries to `%LOCALAPPDATA%\Clip-Claude\`, registers HKCU Run-key pointing at `clip-claude-bg.exe`, spawns immediately.
  - **macOS**: binary to `~/Library/Application Support/Clip-Claude/clip-claude`, writes LaunchAgent plist at `~/Library/LaunchAgents/com.clip-claude.daemon.plist`, `launchctl bootstrap gui/$UID` to register + start.

## Taboos

- No comments explaining WHAT (names do that). Comments only for WHY.
- No speculative abstractions.
- Use `anyhow::Result` everywhere — this is a binary, not a library.
- Don't read clipboard image when text is also present — only act on image-only clipboards. This is the loop guard *and* the user-rich-content guard.
- **Never replace** the clipboard image without also re-emitting it. Clip-Claude's contract is "image stays, text comes and goes." Any future feature that breaks this contract is a regression.
- CF_DIB byte layout must match `.NET Clipboard.SetImage` exactly — chat apps key off that specific encoding.
- `reconcile()` MUST only rewrite the clipboard when `desired != cap.current_text_path`. Avoid noisy re-writes on every poll — clipboard managers see each write as a history entry.
- scp must use `-B` (batch mode) — interactive password prompts would block the daemon and the user can't see them.
