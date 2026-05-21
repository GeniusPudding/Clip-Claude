# Install Clip-Claude: build release binaries, copy to %LOCALAPPDATA%, register
# auto-start, spawn the background daemon. Safe to re-run.

$ErrorActionPreference = 'Stop'
$repoDir = $PSScriptRoot

Write-Host ''
Write-Host '=== Clip-Claude install ==='
Write-Host "Location: $repoDir"

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host 'Installing rustup (stable toolchain, minimal profile)...'
    $rustupInit = Join-Path $env:TEMP 'rustup-init.exe'
    Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile $rustupInit
    & $rustupInit -y --default-toolchain stable --profile minimal
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
}

Push-Location $repoDir
try {
    cargo build --release
    & '.\target\release\clip-claude.exe' install
} finally {
    Pop-Location
}

Write-Host ''
Write-Host '=== Done ==='
Write-Host 'Take a screenshot (Win+Shift+S), then Ctrl+V into any agent CLI.'
Write-Host 'Verify:   & "$env:LOCALAPPDATA\Clip-Claude\clip-claude.exe" status'
Write-Host 'Uninstall: .\uninstall.ps1'
