# Build the `cdxtheme` CLI and stage it for Tauri `bundle.externalBin`.
#
# Output: app-tauri\binaries\cdxtheme-<target-triple>.exe
#
# Honors:
#   TAURI_ENV_TARGET_TRIPLE, TAURI_ENV_DEBUG
#   CLI_SIDECAR_PROFILE, CLI_SIDECAR_TARGET
#
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent $PSScriptRoot
$BinDir = Join-Path $Root 'app-tauri\binaries'
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null

function Get-HostTriple {
  $out = & rustc --print host-tuple 2>$null
  if ($LASTEXITCODE -eq 0 -and $out) { return $out.Trim() }
  $line = (& rustc -vV) | Where-Object { $_ -match '^host: ' } | Select-Object -First 1
  if (-not $line) { throw 'could not determine host triple' }
  return ($line -replace '^host:\s*', '').Trim()
}

$Triple = $env:CLI_SIDECAR_TARGET
if ([string]::IsNullOrEmpty($Triple)) { $Triple = $env:TAURI_ENV_TARGET_TRIPLE }
if ([string]::IsNullOrEmpty($Triple)) { $Triple = Get-HostTriple }

$Profile = $env:CLI_SIDECAR_PROFILE
if ([string]::IsNullOrEmpty($Profile)) {
  if ($env:TAURI_ENV_DEBUG -eq 'true') { $Profile = 'debug' } else { $Profile = 'release' }
}

$Ext = ''
if ($Triple -match 'windows') { $Ext = '.exe' }

$HostTriple = Get-HostTriple
$UseTarget = ($Triple -ne $HostTriple) -or -not [string]::IsNullOrEmpty($env:CLI_SIDECAR_TARGET)

Write-Host "==> Building cdxtheme CLI ($Profile / $Triple)…"

$cargoArgs = @('build', '-p', 'cdx-theme-cli', '--manifest-path', (Join-Path $Root 'Cargo.toml'))
if ($Profile -eq 'release') { $cargoArgs += '--release' }
if ($UseTarget) {
  $cargoArgs += @('--target', $Triple)
  $Src = Join-Path $Root "target\$Triple\$Profile\cdxtheme$Ext"
} else {
  $Src = Join-Path $Root "target\$Profile\cdxtheme$Ext"
}

& cargo @cargoArgs
if ($LASTEXITCODE -ne 0) { throw "cargo build failed with exit $LASTEXITCODE" }

if (-not (Test-Path $Src)) {
  $Alt = Join-Path $Root "target\$Triple\$Profile\cdxtheme$Ext"
  if (Test-Path $Alt) { $Src = $Alt }
  else { throw "CLI binary not found at $Src" }
}

$Dest = Join-Path $BinDir "cdxtheme-$Triple$Ext"
Copy-Item -Force -Path $Src -Destination $Dest
Write-Host "OK  CLI sidecar → $Dest"
