# Toolchain

| Tool      | Purpose                              | Where it lives             | Install                                  |
|-----------|--------------------------------------|----------------------------|------------------------------------------|
| `rustup`  | Manages Rust toolchain versions      | `~/.rustup` + `~/.cargo`   | `./scripts/setup.{ps1,sh}` (auto)        |
| `cargo`   | Build, run, test, dependency manager | `~/.cargo/bin/cargo`       | Bundled with rustup                      |
| `rustc`   | The Rust compiler                    | `~/.cargo/bin/rustc`       | Bundled with rustup                      |
| `clippy`  | Linter                               | `~/.cargo/bin/cargo-clippy`| `rustup component add clippy`            |
| `rustfmt` | Formatter                            | `~/.cargo/bin/rustfmt`     | `rustup component add rustfmt`           |

## Runtime dependencies

The compiled binary is **statically linked** for the standard library and Rust crates. No runtime install needed on a user's machine.

Per-platform system libraries the binary links against (already present on every supported OS):

| Platform | System library                                | Provides                              |
|----------|-----------------------------------------------|---------------------------------------|
| Windows  | `user32.dll`, `kernel32.dll`                  | Win32 clipboard + foreground-window + process info |
| macOS    | `AppKit.framework`, `Foundation.framework`    | NSPasteboard + NSWorkspace            |
| Linux    | `libxcb`                                      | X11 clipboard (Wayland not yet wired) |

External binaries the daemon `Command::spawn`s at runtime:

| Binary       | When used                            | Platform |
|--------------|--------------------------------------|----------|
| `scp`        | SSH bridge — upload PNG to remote `/tmp/` | All     |
| `id`         | Read UID for `launchctl bootstrap`   | macOS    |
| `launchctl`  | Register / unregister LaunchAgent    | macOS    |
| `reg`        | Register / unregister HKCU Run-key   | Windows  |
| `taskkill`   | Stop running daemon during install   | Windows  |
| `tasklist`   | Check daemon-running state           | Windows  |

## Pipeline

1. **Install** (`install.{ps1,sh}` at repo root): one-line wrapper. Installs rustup if missing, builds release, then runs `clip-claude install`. Windows drops binaries into `%LOCALAPPDATA%\Clip-Claude\` + HKCU Run-key. macOS drops binary into `~/Library/Application Support/Clip-Claude/` + writes `~/Library/LaunchAgents/com.clip-claude.daemon.plist` + `launchctl bootstrap gui/$UID`. Both start the daemon immediately.
2. **Setup** (`scripts/setup.{ps1,sh}`): installs rustup if missing, runs `cargo build --release`. Stops there — no auto-install.
3. **Dev** (`scripts/dev.{ps1,sh}`): `cargo run --bin clip-claude --` with passthrough args (debug build).
4. **Build release**: `cargo build --release`. Output at `target/release/clip-claude{.exe}` and `clip-claude-bg{.exe}`.
5. **Test**: `cargo test`.
6. **Lint**: `cargo clippy --all-targets -- -D warnings`.
7. **Format**: `cargo fmt`.
