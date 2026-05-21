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
echo "Setup complete."
echo "Binary: target/release/clip-claude"
echo "Quick test: ./scripts/dev.sh doctor"
echo "Auto-install: ./install.sh"
