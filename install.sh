#!/usr/bin/env bash
# Build Clip-Claude. macOS / Linux: auto-start hook not wired in this release —
# the binary is built but you must add it to your PATH and launch manually.

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

echo
echo '=== Done ==='
echo "Binary: $repo_dir/target/release/clip-claude"
echo
echo 'macOS / Linux: auto-start on login is not wired in this release.'
echo '  - Multi-format clipboard write (image + text coexist) is Windows-only for now.'
echo '  - On macOS, Claude Code already supports native Cmd+V image paste, so this is moot for the Claude case.'
echo '  - For Gemini / Codex on macOS / Linux, run `target/release/clip-claude start` manually'
echo '    (text-only fallback — image will be replaced by the path text on paste).'
