#!/usr/bin/env bash
# Build Clip-Claude, install binary, register LaunchAgent (macOS), start daemon.
# Safe to re-run — idempotent.

set -euo pipefail
repo_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd "$repo_dir"

echo
echo '=== Clip-Claude install ==='
echo "Location: $repo_dir"

if ! command -v cargo >/dev/null 2>&1; then
    echo 'Installing rustup (stable toolchain, minimal profile)...'
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
    export PATH="$HOME/.cargo/bin:$PATH"
fi

cargo build --release

case "$(uname -s)" in
    Darwin)
        ./target/release/clip-claude install
        echo
        echo '=== Done ==='
        echo 'Take a screenshot (Cmd+Shift+4), then Cmd+V into any agent CLI.'
        echo 'Verify:   "$HOME/Library/Application Support/Clip-Claude/clip-claude" status'
        echo 'Uninstall: ./uninstall.sh'
        ;;
    *)
        echo
        echo '=== Done ==='
        echo "Binary: $repo_dir/target/release/clip-claude"
        echo 'Linux / BSD: auto-start not wired in this release.'
        echo 'Run target/release/clip-claude start manually.'
        ;;
esac
