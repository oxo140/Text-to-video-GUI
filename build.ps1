param([switch]$Run)
$ErrorActionPreference = 'Stop'
Set-Location -LiteralPath $PSScriptRoot
$localTools = Join-Path (Split-Path $PSScriptRoot -Parent) '.tools'
if (-not (Get-Command cargo -ErrorAction SilentlyContinue) -and (Test-Path "$localTools\cargo\bin\cargo.exe")) {
    $env:CARGO_HOME = "$localTools\cargo"
    $env:RUSTUP_HOME = "$localTools\rustup"
    $env:PATH = "$localTools\cargo\bin;$env:PATH"
}
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { throw 'Rust est requis : https://rustup.rs' }
if ($IsWindows -or $env:OS -eq 'Windows_NT') {
    $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vswhere) {
        $vsPath = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
        if ($vsPath) { & "$vsPath\Common7\Tools\Launch-VsDevShell.ps1" -Arch amd64 -HostArch amd64 -SkipAutomaticLocation }
    }
}
cargo test --locked
if ($LASTEXITCODE -ne 0) { throw 'Les tests ont échoué.' }
cargo build --release --locked
if ($LASTEXITCODE -ne 0) { throw 'La compilation a échoué.' }
New-Item -ItemType Directory -Force dist | Out-Null
Copy-Item -LiteralPath target\release\text-video-studio.exe -Destination 'dist\Text-Video-Studio-GUI.exe'
Copy-Item -LiteralPath target\release\text-video-cmd.exe -Destination 'dist\Text-Video-Studio-CMD.exe'
Copy-Item -LiteralPath README.md,THIRD_PARTY.md,Ubuntu-FONT-LICENCE.txt -Destination dist
$cargoRoot = $env:CARGO_HOME
if (-not $cargoRoot) { $cargoRoot = Join-Path $env:USERPROFILE '.cargo' }
$registry = Join-Path $cargoRoot 'registry\src'
$fonts = Resolve-Path "$registry\*\epaint_default_fonts-0.29.1\fonts" -ErrorAction SilentlyContinue | Select-Object -First 1
if ($fonts) {
    New-Item -ItemType Directory -Force dist\licenses | Out-Null
    Copy-Item -LiteralPath (Join-Path $fonts.Path 'Hack-Regular.txt'),(Join-Path $fonts.Path 'OFL.txt'),(Join-Path $fonts.Path 'emoji-icon-font-mit-license.txt') -Destination dist\licenses
}
if ($Run) { Start-Process -FilePath (Join-Path $PSScriptRoot 'dist\Text-Video-Studio-GUI.exe') -WindowStyle Normal }
