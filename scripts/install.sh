#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

if ! command -v cargo >/dev/null 2>&1; then
    echo "Installing rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
    export PATH="$HOME/.cargo/bin:$PATH"
fi

cargo build --release

echo
echo "Build complete. Binary: target/release/clipbridge"
echo
echo "Note: auto-start on login is only wired for Windows in this release."
echo "On macOS / Linux, copy target/release/clipbridge somewhere on PATH"
echo "and run it manually with \`clipbridge\` when you want it active."
