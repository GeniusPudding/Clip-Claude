#!/usr/bin/env bash
# Uninstall Clip-Claude (macOS / Linux). Currently this only kills any running
# foreground watcher process; auto-start is not wired on these platforms.

set -euo pipefail

echo
echo '=== Clip-Claude uninstall ==='

if pgrep -f 'clip-claude( start)?$' >/dev/null 2>&1; then
    pkill -f 'clip-claude( start)?$' || true
    echo 'Stopped running watcher process.'
else
    echo 'No running watcher found.'
fi

echo
echo 'Done. Build artifacts at target/release/ remain — delete manually for a clean wipe.'
