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

1. **Setup** (`scripts/setup.{ps1,sh}`): installs rustup if missing, runs `cargo build --release`.
2. **Dev** (`scripts/dev.{ps1,sh}`): `cargo run --` with passthrough args (debug build).
3. **Build release**: `cargo build --release`. Output at `target/release/clipbridge{.exe}`.
4. **Test**: `cargo test`.
5. **Lint**: `cargo clippy --all-targets -- -D warnings`.
6. **Format**: `cargo fmt`.

## Cross-platform release builds

For shipping prebuilt binaries via GitHub Releases, use a CI matrix:

| Runner            | Target triple                    | Output extension |
|-------------------|----------------------------------|------------------|
| `windows-latest`  | `x86_64-pc-windows-msvc`         | `.exe`           |
| `macos-latest`    | `aarch64-apple-darwin`           | (none)           |
| `macos-13`        | `x86_64-apple-darwin`            | (none)           |
| `ubuntu-latest`   | `x86_64-unknown-linux-gnu`       | (none)           |

(`.github/workflows/release.yml` is not yet committed — add when the first tagged release is cut.)
