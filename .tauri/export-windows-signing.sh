#!/usr/bin/env bash
# Source this file to export WINDOWS_CERTIFICATE* for Tauri bundling.
#   source .tauri/export-windows-signing.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PFX_B64="$ROOT/windows-codesign.pfx.base64"
PASS_FILE="$ROOT/windows-codesign.password"

if [[ ! -f "$PFX_B64" || ! -f "$PASS_FILE" ]]; then
  echo "error: missing $PFX_B64 or $PASS_FILE — run .tauri/generate-windows-codesign.sh" >&2
  return 1 2>/dev/null || exit 1
fi

export WINDOWS_CERTIFICATE="$(tr -d '\n' < "$PFX_B64")"
export WINDOWS_CERTIFICATE_PASSWORD="$(tr -d '\n' < "$PASS_FILE")"
echo "Exported WINDOWS_CERTIFICATE and WINDOWS_CERTIFICATE_PASSWORD (from .tauri/)"
