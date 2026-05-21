# Uninstall Clip-Claude: stop the daemon and remove the HKCU Run-key entry.
# Repo files stay on disk; delete %LOCALAPPDATA%\Clip-Claude\ manually for a full wipe.

$ErrorActionPreference = 'Continue'

Write-Host ''
Write-Host '=== Clip-Claude uninstall ==='

$installedExe = Join-Path $env:LOCALAPPDATA 'Clip-Claude\clip-claude.exe'
$repoExe      = Join-Path $PSScriptRoot 'target\release\clip-claude.exe'

if (Test-Path $installedExe) {
    & $installedExe uninstall
} elseif (Test-Path $repoExe) {
    & $repoExe uninstall
} else {
    Write-Host 'clip-claude.exe not found in %LOCALAPPDATA% or target/release.'
    Write-Host 'Nothing to do.'
    exit 0
}

Write-Host ''
Write-Host 'Done. Binaries remain at %LOCALAPPDATA%\Clip-Claude\ — delete manually for a clean wipe.'
