#!/usr/bin/env bash
# Uninstall Clip-Claude: stop daemon, remove LaunchAgent (macOS).

set -euo pipefail
repo_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd "$repo_dir"

echo
echo '=== Clip-Claude uninstall ==='

case "$(uname -s)" in
    Darwin)
        installed_bin="$HOME/Library/Application Support/Clip-Claude/clip-claude"
        if [ -x "$installed_bin" ]; then
            "$installed_bin" uninstall
        elif [ -x ./target/release/clip-claude ]; then
            ./target/release/clip-claude uninstall
        else
            echo 'No installed binary found; falling back to pkill.'
            pkill -f 'clip-claude( start)?$' 2>/dev/null || true
        fi
        ;;
    *)
        if pgrep -f 'clip-claude( start)?$' >/dev/null 2>&1; then
            pkill -f 'clip-claude( start)?$' || true
            echo 'Stopped running watcher process.'
        else
            echo 'No running watcher found.'
        fi
        echo
        echo 'Done. Build artifacts at target/release/ remain — delete manually for a clean wipe.'
        ;;
esac
