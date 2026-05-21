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

| Platform | System library | Provides                              |
|----------|----------------|---------------------------------------|
| Windows  | `user32.dll`, `kernel32.dll` | Win32 clipboard API     |
| macOS    | `AppKit.framework`           | NSPasteboard            |
| Linux    | `libxcb`                     | X11 clipboard (Wayland not yet wired) |

## Pipeline

1. **Install** (`scripts/install.{ps1,sh}`): one-line wrapper. Installs rustup if missing, builds release, then (Windows) runs `clip-claude install` to drop binaries into `%LOCALAPPDATA%\Clip-Claude\` + register HKCU Run-key + start daemon.
2. **Setup** (`scripts/setup.{ps1,sh}`): installs rustup if missing, runs `cargo build --release`. Stops there — no auto-install.
3. **Dev** (`scripts/dev.{ps1,sh}`): `cargo run --bin clip-claude --` with passthrough args (debug build).
4. **Build release**: `cargo build --release`. Output at `target/release/clip-claude{.exe}` and `clip-claude-bg{.exe}`.
5. **Test**: `cargo test`.
6. **Lint**: `cargo clippy --all-targets -- -D warnings`.
7. **Format**: `cargo fmt`.
