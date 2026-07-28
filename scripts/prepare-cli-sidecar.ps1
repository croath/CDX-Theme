# Build the `cdxthemex` CLI and stage it for Tauri `bundle.externalBin`.
#
# Cargo bin name is `cdxthemex` (cli/Cargo.toml). Staged as:
#   app-tauri\binaries\cdxthemex-<target-triple>.exe
#
# IMPORTANT: Binary must not case-collide with main binary `CDXTheme`
# (Windows NTFS is case-insensitive). Use `cdxthemex`, not `cdxtheme`.
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

$CliBin = 'cdxthemex'

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

Write-Host "==> Building $CliBin CLI ($Profile / $Triple)…"

$cargoArgs = @('build', '-p', 'cdx-theme-cli', '--manifest-path', (Join-Path $Root 'Cargo.toml'))
if ($Profile -eq 'release') { $cargoArgs += '--release' }
if ($UseTarget) {
  $cargoArgs += @('--target', $Triple)
  $Src = Join-Path $Root "target\$Triple\$Profile\$CliBin$Ext"
} else {
  $Src = Join-Path $Root "target\$Profile\$CliBin$Ext"
}

& cargo @cargoArgs
if ($LASTEXITCODE -ne 0) { throw "cargo build failed with exit $LASTEXITCODE" }

if (-not (Test-Path $Src)) {
  $Alt = Join-Path $Root "target\$Triple\$Profile\$CliBin$Ext"
  if (Test-Path $Alt) { $Src = $Alt }
  else { throw "CLI binary not found at $Src" }
}

$Dest = Join-Path $BinDir "$CliBin-$Triple$Ext"
# Remove legacy staged names (case-collision with CDXTheme, or old names).
foreach ($legacyName in @("cdxtheme-$Triple$Ext", "cdxtheme-cli-$Triple$Ext")) {
  $legacy = Join-Path $BinDir $legacyName
  if (Test-Path $legacy) { Remove-Item -Force $legacy }
}
Copy-Item -Force -Path $Src -Destination $Dest
Write-Host "OK  CLI sidecar → $Dest"
