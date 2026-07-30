#!/usr/bin/env bash
# Stage the Bun runtime for Tauri `bundle.externalBin`.
#
# Staged as:
#   app-tauri/binaries/bun-<target-triple>[.exe]
#
# If the staged binary already exists, this is a no-op (unless BUN_SIDECAR_FORCE=1).
# Otherwise downloads the requested Bun version (default: latest) and extracts
# the platform binary into binaries/.
#
# Honors:
#   TAURI_ENV_TARGET_TRIPLE  — set by `cargo tauri` during build/dev
#   BUN_SIDECAR_TARGET       — override Rust target triple
#   BUN_VERSION              — version pin, e.g. "1.2.18" or "latest" (default)
#   BUN_SIDECAR_FORCE        — "1" → re-download even if staged binary exists
#
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_DIR="$ROOT/app-tauri/binaries"
mkdir -p "$BIN_DIR"

BUN_VERSION="${BUN_VERSION:-latest}"
# Normalize: allow "v1.2.18" / "bun-v1.2.18" / "1.2.18"
if [[ "$BUN_VERSION" != "latest" ]]; then
  BUN_VERSION="${BUN_VERSION#bun-}"
  BUN_VERSION="${BUN_VERSION#v}"
fi

host_triple() {
  if rustc --print host-tuple >/dev/null 2>&1; then
    rustc --print host-tuple
  else
    rustc -vV | sed -n 's/^host: //p'
  fi
}

if [[ -n "${BUN_SIDECAR_TARGET:-}" ]]; then
  TRIPLE="$BUN_SIDECAR_TARGET"
elif [[ -n "${TAURI_ENV_TARGET_TRIPLE:-}" ]]; then
  TRIPLE="$TAURI_ENV_TARGET_TRIPLE"
else
  TRIPLE="$(host_triple)"
fi

if [[ -z "${TRIPLE:-}" ]]; then
  echo "error: could not determine target triple" >&2
  exit 1
fi

EXT=""
case "$TRIPLE" in
  *windows*) EXT=".exe" ;;
esac

# Map Rust target triple → Bun release asset target (bun-{target}.zip).
bun_asset_target() {
  case "$1" in
    aarch64-apple-darwin)          echo "darwin-aarch64" ;;
    x86_64-apple-darwin)           echo "darwin-x64" ;;
    aarch64-unknown-linux-gnu)     echo "linux-aarch64" ;;
    aarch64-unknown-linux-musl)    echo "linux-aarch64" ;;
    x86_64-unknown-linux-gnu)      echo "linux-x64" ;;
    x86_64-unknown-linux-musl)     echo "linux-x64" ;;
    x86_64-pc-windows-msvc|x86_64-pc-windows-gnu) echo "windows-x64" ;;
    aarch64-pc-windows-msvc|aarch64-pc-windows-gnu) echo "windows-aarch64" ;;
    *)
      echo "error: unsupported triple for Bun sidecar: $1" >&2
      return 1
      ;;
  esac
}

BUN_TARGET="$(bun_asset_target "$TRIPLE")"
DEST="$BIN_DIR/bun-${TRIPLE}${EXT}"

if [[ -f "$DEST" && "${BUN_SIDECAR_FORCE:-}" != "1" ]]; then
  # Require a plausible binary size (Bun is multi-MB).
  SIZE="$(wc -c <"$DEST" | tr -d ' ')"
  if [[ "$SIZE" -gt 1000000 ]]; then
    echo "✓ Bun sidecar already present → $DEST (${SIZE} bytes)"
    exit 0
  fi
  echo "warn: staged Bun looks too small (${SIZE} bytes); re-downloading…" >&2
fi

echo "==> Preparing Bun sidecar (version=$BUN_VERSION / $BUN_TARGET → $TRIPLE)…"

if [[ "$BUN_VERSION" == "latest" ]]; then
  ZIP_NAME="bun-${BUN_TARGET}.zip"
  URLS=(
    "https://github.com/oven-sh/bun/releases/latest/download/${ZIP_NAME}"
    "https://npmmirror.com/mirrors/bun/latest/${ZIP_NAME}"
  )
else
  ZIP_NAME="bun-${BUN_TARGET}.zip"
  TAG="bun-v${BUN_VERSION}"
  URLS=(
    "https://github.com/oven-sh/bun/releases/download/${TAG}/${ZIP_NAME}"
    "https://npmmirror.com/mirrors/bun/${BUN_VERSION}/${ZIP_NAME}"
  )
fi

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/cdxtheme-bun.XXXXXX")"
cleanup() { rm -rf "$TMP_DIR"; }
trap cleanup EXIT

ZIP_PATH="$TMP_DIR/bun.zip"
download_ok=0
last_err=""
for url in "${URLS[@]}"; do
  echo "    trying $url"
  if command -v curl >/dev/null 2>&1; then
    if curl -fsSL --retry 3 --retry-delay 2 -o "$ZIP_PATH" "$url"; then
      download_ok=1
      break
    fi
    last_err="curl failed for $url"
  elif command -v wget >/dev/null 2>&1; then
    if wget -q -O "$ZIP_PATH" "$url"; then
      download_ok=1
      break
    fi
    last_err="wget failed for $url"
  else
    echo "error: need curl or wget to download Bun" >&2
    exit 1
  fi
done

if [[ "$download_ok" -ne 1 ]]; then
  echo "error: failed to download Bun zip ($last_err)" >&2
  exit 1
fi

ZIP_SIZE="$(wc -c <"$ZIP_PATH" | tr -d ' ')"
if [[ "$ZIP_SIZE" -lt 1000000 ]]; then
  echo "error: downloaded zip too small (${ZIP_SIZE} bytes) — not a Bun release?" >&2
  exit 1
fi

echo "    extracting…"
EXTRACT_DIR="$TMP_DIR/extract"
mkdir -p "$EXTRACT_DIR"
if command -v unzip >/dev/null 2>&1; then
  unzip -q -o "$ZIP_PATH" -d "$EXTRACT_DIR"
else
  # Python fallback (stdlib zipfile).
  python3 - "$ZIP_PATH" "$EXTRACT_DIR" <<'PY'
import sys, zipfile
zf = zipfile.ZipFile(sys.argv[1])
zf.extractall(sys.argv[2])
PY
fi

# Release layout: bun-<target>/bun[.exe]
FOUND=""
EXE_NAME="bun${EXT}"
while IFS= read -r -d '' f; do
  base="$(basename "$f")"
  if [[ "$base" == "$EXE_NAME" ]]; then
    FOUND="$f"
    break
  fi
done < <(find "$EXTRACT_DIR" -type f -print0 2>/dev/null || true)

if [[ -z "$FOUND" ]]; then
  # Non-null find fallback for older bash / busybox
  FOUND="$(find "$EXTRACT_DIR" -type f -name "$EXE_NAME" 2>/dev/null | head -n1 || true)"
fi

if [[ -z "$FOUND" || ! -f "$FOUND" ]]; then
  echo "error: $EXE_NAME not found inside Bun zip" >&2
  find "$EXTRACT_DIR" -type f | head -n 40 >&2 || true
  exit 1
fi

cp -f "$FOUND" "$DEST"
chmod +x "$DEST" 2>/dev/null || true

DEST_SIZE="$(wc -c <"$DEST" | tr -d ' ')"
if [[ "$DEST_SIZE" -lt 1000000 ]]; then
  echo "error: staged Bun binary too small (${DEST_SIZE} bytes)" >&2
  rm -f "$DEST"
  exit 1
fi

echo "✓ Bun sidecar → $DEST (${DEST_SIZE} bytes, version=$BUN_VERSION)"
