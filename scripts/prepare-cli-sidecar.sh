#!/usr/bin/env bash
# Build the `cdxtheme` CLI and stage it for Tauri `bundle.externalBin`.
#
# Output: app-tauri/binaries/cdxthemex-<target-triple>[.exe]
#
# IMPORTANT: The staged name must NOT case-collide with the app main binary
# (`CDXTheme`). macOS/Windows default filesystems are case-insensitive, so
# bundling as plain `cdxtheme` overwrites `CDXTheme` and breaks notarization
# ("The signature of the binary is invalid").
#
# Honors:
#   TAURI_ENV_TARGET_TRIPLE  — set by `cargo tauri` during build/dev
#   TAURI_ENV_DEBUG          — "true" → debug profile; else release
#   CLI_SIDECAR_PROFILE      — override: release | debug
#   CLI_SIDECAR_TARGET       — override cargo --target triple
#
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_DIR="$ROOT/app-tauri/binaries"
mkdir -p "$BIN_DIR"

host_triple() {
  if rustc --print host-tuple >/dev/null 2>&1; then
    rustc --print host-tuple
  else
    rustc -vV | sed -n 's/^host: //p'
  fi
}

if [[ -n "${CLI_SIDECAR_TARGET:-}" ]]; then
  TRIPLE="$CLI_SIDECAR_TARGET"
elif [[ -n "${TAURI_ENV_TARGET_TRIPLE:-}" ]]; then
  TRIPLE="$TAURI_ENV_TARGET_TRIPLE"
else
  TRIPLE="$(host_triple)"
fi

if [[ -z "${TRIPLE:-}" ]]; then
  echo "error: could not determine target triple" >&2
  exit 1
fi

if [[ -n "${CLI_SIDECAR_PROFILE:-}" ]]; then
  PROFILE="$CLI_SIDECAR_PROFILE"
elif [[ "${TAURI_ENV_DEBUG:-}" == "true" ]]; then
  PROFILE="debug"
else
  PROFILE="release"
fi

EXT=""
case "$TRIPLE" in
  *windows*) EXT=".exe" ;;
esac

HOST="$(host_triple)"
USE_TARGET=0
if [[ "$TRIPLE" != "$HOST" ]] || [[ -n "${CLI_SIDECAR_TARGET:-}" ]]; then
  USE_TARGET=1
fi

echo "==> Building cdxtheme CLI ($PROFILE / $TRIPLE)…"

CARGO_ARGS=(build -p cdx-theme-cli --manifest-path "$ROOT/Cargo.toml")
if [[ "$PROFILE" == "release" ]]; then
  CARGO_ARGS+=(--release)
fi
if [[ "$USE_TARGET" -eq 1 ]]; then
  CARGO_ARGS+=(--target "$TRIPLE")
  SRC="$ROOT/target/$TRIPLE/$PROFILE/cdxtheme$EXT"
else
  SRC="$ROOT/target/$PROFILE/cdxtheme$EXT"
fi

cargo "${CARGO_ARGS[@]}"

if [[ ! -f "$SRC" ]]; then
  # Host builds sometimes still land under target/<triple>/ when RUSTUP_TOOLCHAIN forces it.
  ALT="$ROOT/target/$TRIPLE/$PROFILE/cdxtheme$EXT"
  if [[ -f "$ALT" ]]; then
    SRC="$ALT"
  else
    echo "error: CLI binary not found at $SRC" >&2
    exit 1
  fi
fi

# Stage as cdxthemex (not cdxtheme) — see header comment about case collision.
DEST="$BIN_DIR/cdxthemex-${TRIPLE}${EXT}"
# Remove legacy staged names (case-collision with CDXTheme, or old cdxtheme-cli).
rm -f "$BIN_DIR/cdxtheme-${TRIPLE}${EXT}" "$BIN_DIR/cdxtheme-cli-${TRIPLE}${EXT}"
cp -f "$SRC" "$DEST"
chmod +x "$DEST" 2>/dev/null || true

echo "✓ CLI sidecar → $DEST"
