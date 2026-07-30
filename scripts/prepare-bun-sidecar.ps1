# Stage the Bun runtime for Tauri `bundle.externalBin`.
#
# Staged as:
#   app-tauri\binaries\bun-<target-triple>.exe
#
# If the staged binary already exists, this is a no-op (unless BUN_SIDECAR_FORCE=1).
# Otherwise downloads the requested Bun version (default: latest).
#
# Honors:
#   TAURI_ENV_TARGET_TRIPLE, BUN_SIDECAR_TARGET
#   BUN_VERSION          — "latest" (default) or e.g. "1.2.18"
#   BUN_SIDECAR_FORCE    — "1" → re-download even if present
#
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent $PSScriptRoot
$BinDir = Join-Path $Root 'app-tauri\binaries'
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null

$BunVersion = $env:BUN_VERSION
if ([string]::IsNullOrEmpty($BunVersion)) { $BunVersion = 'latest' }
if ($BunVersion -ne 'latest') {
  $BunVersion = $BunVersion -replace '^bun-', '' -replace '^v', ''
}

function Get-HostTriple {
  $out = & rustc --print host-tuple 2>$null
  if ($LASTEXITCODE -eq 0 -and $out) { return $out.Trim() }
  $line = (& rustc -vV) | Where-Object { $_ -match '^host: ' } | Select-Object -First 1
  if (-not $line) { throw 'could not determine host triple' }
  return ($line -replace '^host:\s*', '').Trim()
}

function Get-BunAssetTarget([string]$Triple) {
  switch -Wildcard ($Triple) {
    'aarch64-apple-darwin' { return 'darwin-aarch64' }
    'x86_64-apple-darwin' { return 'darwin-x64' }
    'aarch64-unknown-linux-gnu' { return 'linux-aarch64' }
    'aarch64-unknown-linux-musl' { return 'linux-aarch64' }
    'x86_64-unknown-linux-gnu' { return 'linux-x64' }
    'x86_64-unknown-linux-musl' { return 'linux-x64' }
    'x86_64-pc-windows-msvc' { return 'windows-x64' }
    'x86_64-pc-windows-gnu' { return 'windows-x64' }
    'aarch64-pc-windows-msvc' { return 'windows-aarch64' }
    'aarch64-pc-windows-gnu' { return 'windows-aarch64' }
    default { throw "unsupported triple for Bun sidecar: $Triple" }
  }
}

$Triple = $env:BUN_SIDECAR_TARGET
if ([string]::IsNullOrEmpty($Triple)) { $Triple = $env:TAURI_ENV_TARGET_TRIPLE }
if ([string]::IsNullOrEmpty($Triple)) { $Triple = Get-HostTriple }

$Ext = ''
if ($Triple -match 'windows') { $Ext = '.exe' }

$BunTarget = Get-BunAssetTarget $Triple
$Dest = Join-Path $BinDir "bun-$Triple$Ext"

if ((Test-Path $Dest) -and ($env:BUN_SIDECAR_FORCE -ne '1')) {
  $size = (Get-Item $Dest).Length
  if ($size -gt 1000000) {
    Write-Host "OK  Bun sidecar already present → $Dest ($size bytes)"
    exit 0
  }
  Write-Warning "staged Bun looks too small ($size bytes); re-downloading…"
}

Write-Host "==> Preparing Bun sidecar (version=$BunVersion / $BunTarget → $Triple)…"

$ZipName = "bun-$BunTarget.zip"
if ($BunVersion -eq 'latest') {
  $Urls = @(
    "https://github.com/oven-sh/bun/releases/latest/download/$ZipName",
    "https://npmmirror.com/mirrors/bun/latest/$ZipName"
  )
} else {
  $Tag = "bun-v$BunVersion"
  $Urls = @(
    "https://github.com/oven-sh/bun/releases/download/$Tag/$ZipName",
    "https://npmmirror.com/mirrors/bun/$BunVersion/$ZipName"
  )
}

$TmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ("cdxtheme-bun-" + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Force -Path $TmpDir | Out-Null
try {
  $ZipPath = Join-Path $TmpDir 'bun.zip'
  $downloaded = $false
  $lastErr = ''
  foreach ($url in $Urls) {
    Write-Host "    trying $url"
    try {
      # TLS 1.2+ for older PowerShell
      [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
      Invoke-WebRequest -Uri $url -OutFile $ZipPath -UseBasicParsing
      if ((Test-Path $ZipPath) -and ((Get-Item $ZipPath).Length -gt 1000000)) {
        $downloaded = $true
        break
      }
      $lastErr = "${url}: download too small or missing"
    } catch {
      $lastErr = "${url}: $($_.Exception.Message)"
    }
  }

  if (-not $downloaded) {
    throw "failed to download Bun zip ($lastErr)"
  }

  $ExtractDir = Join-Path $TmpDir 'extract'
  New-Item -ItemType Directory -Force -Path $ExtractDir | Out-Null
  Expand-Archive -Path $ZipPath -DestinationPath $ExtractDir -Force

  $ExeName = "bun$Ext"
  $Found = Get-ChildItem -Path $ExtractDir -Recurse -File -Filter $ExeName |
    Select-Object -First 1
  if (-not $Found) {
    throw "$ExeName not found inside Bun zip"
  }

  Copy-Item -Force -Path $Found.FullName -Destination $Dest
  $DestSize = (Get-Item $Dest).Length
  if ($DestSize -lt 1000000) {
    Remove-Item -Force $Dest -ErrorAction SilentlyContinue
    throw "staged Bun binary too small ($DestSize bytes)"
  }

  Write-Host "OK  Bun sidecar → $Dest ($DestSize bytes, version=$BunVersion)"
} finally {
  Remove-Item -Recurse -Force $TmpDir -ErrorAction SilentlyContinue
}
