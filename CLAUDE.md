# clipbridge

Cross-platform clipboard daemon that turns image-on-clipboard into a path-on-clipboard so terminal AI agents (Claude Code, Gemini CLI, Codex) can read screenshots as if they natively supported image paste.

## Intent

Take a screenshot (Win+Shift+S / Cmd+Shift+4 to clipboard), Ctrl+V into an agent's prompt, the agent sees the image. The trick: a tiny native daemon watches the clipboard, saves any image to disk, and replaces clipboard content with a multi-line text payload that points at the saved file. Agents collapse the multi-line paste into a `[Pasted text #1]` token visually, and their Read tool opens the path multimodally — so the user gets near-native image-paste UX without modifying the agent CLIs.

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
- `src/watcher.rs` — polling loop that detects image-only clipboards, gated on focus.
- `src/focus.rs` — foreground-window process-tree walk (Windows: user32 + sysinfo). Identifies agent CLIs (`claude`, `gemini`, `codex`, or `node` running them). Cached with a 2s TTL keyed on foreground PID.
- `src/cache.rs` — write PNG to `~/.clipbridge/cache/`, purge files older than 7 days.
- `src/inject.rs` — format the text payload that replaces the clipboard image.
- `src/runner.rs` — `run -- <cmd>` wrapper: starts watcher (focus-aware), spawns child, stops watcher on exit.

## Taboos

- No comments explaining WHAT (names do that). Comments only for WHY.
- No speculative abstractions.
- Use `anyhow::Result` everywhere — this is a binary, not a library.
- Don't read clipboard image when text is also present (web copies, etc.) — only act on image-only clipboards.
- Default behavior must be safe: never convert when foreground isn't an agent CLI. The `--all-windows` flag is the only opt-out.
