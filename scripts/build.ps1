#Requires -Version 5.1
<#
.SYNOPSIS
  Build CDXTheme (Tauri 2 + Trunk + Leptos) on Windows.

.EXAMPLE
  .\scripts\build.ps1
  .\scripts\build.ps1 -Debug
  .\scripts\build.ps1 -Clean
  .\scripts\build.ps1 -Check
  .\scripts\build.ps1 -Frontend
#>
[CmdletBinding()]
param(
  [switch]$Release,
  [switch]$Debug,
  [switch]$Check,
  [switch]$Frontend,
  [switch]$Clean,
  [switch]$VerboseBuild
)

$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

function Write-Step([string]$Message) {
  Write-Host "==> $Message" -ForegroundColor Cyan
}
function Write-Ok([string]$Message) {
  Write-Host "OK  $Message" -ForegroundColor Green
}
function Die([string]$Message) {
  Write-Host "error: $Message" -ForegroundColor Red
  exit 1
}
function Test-Cmd([string]$Name) {
  return [bool](Get-Command $Name -ErrorAction SilentlyContinue)
}

$Mode = 'release'
if ($Debug) { $Mode = 'debug' }
if ($Check) { $Mode = 'check' }
if ($Frontend) { $Mode = 'frontend' }
if ($Release) { $Mode = 'release' }

# ---------------------------------------------------------------------------
# Toolchain
# ---------------------------------------------------------------------------
Write-Step 'Checking toolchain…'

if (-not (Test-Cmd 'cargo')) { Die 'cargo not found — install Rust from https://rustup.rs' }
if (-not (Test-Cmd 'rustc')) { Die 'rustc not found' }
if (-not (Test-Cmd 'rustup')) { Die 'rustup not found' }

$wasmInstalled = & rustup target list --installed 2>$null
if ($wasmInstalled -notcontains 'wasm32-unknown-unknown') {
  Write-Step 'Installing rustup target wasm32-unknown-unknown'
  rustup target add wasm32-unknown-unknown
}

if (-not (Test-Cmd 'trunk')) {
  Write-Step 'Installing trunk…'
  cargo install trunk --locked
}

$script:TauriCmd = $null
$script:TauriArgsPrefix = @()
if (Test-Cmd 'cargo-tauri') {
  $script:TauriCmd = 'cargo'
  $script:TauriArgsPrefix = @('tauri')
} elseif (& cargo tauri --version 2>$null) {
  $script:TauriCmd = 'cargo'
  $script:TauriArgsPrefix = @('tauri')
} elseif (Test-Cmd 'npm') {
  $script:TauriCmd = 'npm'
  $script:TauriArgsPrefix = @('exec', '--yes', '--package', '@tauri-apps/cli@2', '--', 'tauri')
} else {
  Write-Step 'Installing tauri-cli…'
  cargo install tauri-cli --version '^2' --locked
  $script:TauriCmd = 'cargo'
  $script:TauriArgsPrefix = @('tauri')
}

Write-Ok ("cargo  {0}" -f (cargo --version))
Write-Ok ("rustc  {0}" -f (rustc --version))
if (Test-Cmd 'trunk') { Write-Ok ("trunk  {0}" -f (trunk --version)) }

if ((Test-Path (Join-Path $Root 'package.json')) -and (Test-Cmd 'npm')) {
  if (-not (Test-Path (Join-Path $Root 'node_modules'))) {
    Write-Step 'npm install…'
    npm install --no-fund --no-audit
  }
}

function Invoke-Tauri([string[]]$Args) {
  $all = @($script:TauriArgsPrefix) + $Args
  Write-Step ("{0} {1}" -f $script:TauriCmd, ($all -join ' '))
  & $script:TauriCmd @all
  if ($LASTEXITCODE -ne 0) { Die "tauri exited with $LASTEXITCODE" }
}

if ($Clean) {
  Write-Step 'Cleaning…'
  cargo clean --manifest-path (Join-Path $Root 'Cargo.toml')
  if (Test-Path (Join-Path $Root 'dist')) { Remove-Item -Recurse -Force (Join-Path $Root 'dist') }
  if (Test-Path (Join-Path $Root 'target')) { Remove-Item -Recurse -Force (Join-Path $Root 'target') }
  Write-Ok 'clean done'
}

switch ($Mode) {
  'check' {
    Write-Step 'Typecheck backend…'
    cargo check --manifest-path (Join-Path $Root 'app-tauri\Cargo.toml')
    if ($LASTEXITCODE -ne 0) { Die 'backend check failed' }
    Write-Step 'Typecheck frontend (wasm32)…'
    cargo check --target wasm32-unknown-unknown --manifest-path (Join-Path $Root 'Cargo.toml')
    if ($LASTEXITCODE -ne 0) { Die 'frontend check failed' }
    Write-Ok 'check passed'
  }
  'frontend' {
    Write-Step 'Building frontend (trunk release)…'
    trunk build --release
    if ($LASTEXITCODE -ne 0) { Die 'trunk build failed' }
    Write-Ok ("frontend → {0}" -f (Join-Path $Root 'dist'))
  }
  default {
    $tauriArgs = @('build')
    if ($Mode -eq 'debug') { $tauriArgs += '--debug' }
    if ($VerboseBuild) { $tauriArgs += '--verbose' }

    Write-Step ("Building CDXTheme ({0})…" -f $Mode)
    Invoke-Tauri $tauriArgs

    $bundleRoot = Join-Path $Root 'target\release\bundle'
    if ($Mode -eq 'debug') {
      $bundleRoot = Join-Path $Root 'target\debug\bundle'
    }

    Write-Ok 'Build finished'
    if (Test-Path $bundleRoot) {
      Write-Step "Bundles under: $bundleRoot"
      Get-ChildItem -Path $bundleRoot -Recurse -Include *.msi,*.exe,*.nsis.zip -ErrorAction SilentlyContinue |
        ForEach-Object { Write-Host ("  - {0}" -f $_.FullName) }
    }
  }
}
