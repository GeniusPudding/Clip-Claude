# clipbridge

Cross-platform clipboard daemon that turns image-on-clipboard into a path-on-clipboard so terminal AI agents (Claude Code, Gemini CLI, Codex) can read screenshots as if they natively supported image paste.

## Intent

Take a screenshot (Win+Shift+S / Cmd+Shift+4 to clipboard), Ctrl+V into an agent's prompt, the agent sees the image. The trick: a tiny native daemon watches the clipboard, saves any image to disk, and **augments** the clipboard so it carries BOTH the original image (CF_DIBV5, system-synthesized CF_DIB/CF_BITMAP) and a 4-line text payload pointing at the saved file (CF_UNICODETEXT). Image-paste apps take the image; text-paste agent terminals take the path. No focus detection, no app sees the wrong thing.

## Conventions

- Toolchain: **Rust stable** via rustup.
- Build: `cargo build` (debug) or `cargo build --release` (production binary).
- Run: `./scripts/dev.sh` or `./scripts/dev.ps1`.
- Setup: `./scripts/setup.sh` or `./scripts/setup.ps1` (idempotent).
- Tests: `cargo test`.
- Format + lint: `cargo fmt && cargo clippy --all-targets -- -D warnings`.

## Module layout

- `src/main.rs` — entry, dispatches subcommands, `doctor`.
- `src/cli.rs` — `clap` argument definitions.
- `src/watcher.rs` — 150 ms polling loop. Skips when clipboard already has text; otherwise reads image, saves PNG, calls `clipboard_io::write_image_and_text`.
- `src/clipboard_io.rs` — Win32 raw clipboard multi-format write. Builds a `BITMAPV5HEADER` DIB from RGBA bytes (with alpha mask + sRGB color space) and pairs it with a UTF-16 text payload. `OpenClipboard → EmptyClipboard → SetClipboardData(CF_DIBV5) → SetClipboardData(CF_UNICODETEXT) → CloseClipboard`. Non-Windows fallback is text-only via arboard.
- `src/cache.rs` — write PNG to `~/.clipbridge/cache/`, purge files older than 7 days.
- `src/inject.rs` — format the text payload.
- `src/runner.rs` — `run -- <cmd>` wrapper: starts watcher, spawns child, stops watcher on exit.

## Taboos

- No comments explaining WHAT (names do that). Comments only for WHY.
- No speculative abstractions.
- Use `anyhow::Result` everywhere — this is a binary, not a library.
- Don't read clipboard image when text is also present — only act on image-only clipboards. This is the loop guard *and* the user-rich-content guard.
- **Never replace** the clipboard image without also re-emitting it. clipbridge's contract is "image stays, text is added." Any future feature that breaks this contract is a regression.
