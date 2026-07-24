#!/usr/bin/env bash
# CDXTheme — set workspace + Tauri app version
#
# Updates:
#   Cargo.toml              [workspace.package] version
#   app-tauri/tauri.conf.json  "version"
#
# Member crates inherit via version.workspace = true.
#
# Usage:
#   ./scripts/version.sh 0.1.5
#   ./scripts/version.sh --help
#
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

usage() {
  sed -n '2,13p' "$0" | sed 's/^# \{0,1\}//'
  exit 0
}

log()  { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
ok()   { printf '\033[1;32m✓\033[0m %s\n' "$*"; }
die()  { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

if [[ $# -eq 0 ]]; then
  die "version required (e.g. ./scripts/version.sh 0.1.5)"
fi

case "$1" in
  --help|-h) usage ;;
esac

VERSION="$1"

# Semver-ish: MAJOR.MINOR.PATCH with optional pre-release / build metadata
if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.-]+)?$ ]]; then
  die "invalid version '$VERSION' (expected e.g. 0.1.5 or 0.2.0-beta.1)"
fi

CARGO_TOML="$ROOT/Cargo.toml"
TAURI_CONF="$ROOT/app-tauri/tauri.conf.json"

[[ -f "$CARGO_TOML" ]] || die "missing $CARGO_TOML"
[[ -f "$TAURI_CONF" ]] || die "missing $TAURI_CONF"

# Current versions (for logging)
OLD_CARGO="$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$CARGO_TOML" | head -n1)"
OLD_TAURI="$(sed -n 's/^[[:space:]]*"version": "\([^"]*\)".*/\1/p' "$TAURI_CONF" | head -n1)"

log "Setting version → $VERSION"
log "  Cargo.toml:              $OLD_CARGO → $VERSION"
log "  app-tauri/tauri.conf.json: $OLD_TAURI → $VERSION"

# Only the first top-level `version = "..."` (workspace.package), not dependency versions.
# Portable: avoid sed -i (BSD vs GNU).
tmp="$(mktemp)"
awk -v ver="$VERSION" '
  BEGIN { done = 0 }
  !done && /^version = "/ {
    print "version = \"" ver "\""
    done = 1
    next
  }
  { print }
' "$CARGO_TOML" >"$tmp"
mv "$tmp" "$CARGO_TOML"

tmp="$(mktemp)"
awk -v ver="$VERSION" '
  BEGIN { done = 0 }
  !done && /^[[:space:]]*"version": "/ {
    match($0, /^[[:space:]]*/)
    indent = substr($0, RSTART, RLENGTH)
    print indent "\"version\": \"" ver "\","
    done = 1
    next
  }
  { print }
' "$TAURI_CONF" >"$tmp"
mv "$tmp" "$TAURI_CONF"

# Verify
NEW_CARGO="$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$CARGO_TOML" | head -n1)"
NEW_TAURI="$(sed -n 's/^[[:space:]]*"version": "\([^"]*\)".*/\1/p' "$TAURI_CONF" | head -n1)"

[[ "$NEW_CARGO" == "$VERSION" ]] || die "Cargo.toml version not updated (got '$NEW_CARGO')"
[[ "$NEW_TAURI" == "$VERSION" ]] || die "tauri.conf.json version not updated (got '$NEW_TAURI')"

ok "version set to $VERSION"
