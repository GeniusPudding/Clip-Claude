$ErrorActionPreference = "Stop"
Set-Location (Split-Path $PSScriptRoot -Parent)

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Installing rustup..."
    $rustupInit = Join-Path $env:TEMP "rustup-init.exe"
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $rustupInit
    & $rustupInit -y --default-toolchain stable --profile minimal
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
}

cargo build --release
& ".\target\release\clipbridge.exe" install
