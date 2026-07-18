#!/usr/bin/env bash
# CDXTheme — release / debug desktop build (Tauri 2 + Trunk + Leptos)
#
# Usage:
#   ./scripts/build.sh              # release bundle
#   ./scripts/build.sh --debug      # debug build
#   ./scripts/build.sh --clean      # cargo + trunk clean, then release
#   ./scripts/build.sh --check      # typecheck only (no bundle)
#   ./scripts/build.sh --frontend   # trunk build only
#   ./scripts/build.sh --help
#
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

MODE="release"   # release | debug | check | frontend
DO_CLEAN=0
VERBOSE=0

usage() {
  sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//'
  exit 0
}

log()  { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
ok()   { printf '\033[1;32m✓\033[0m %s\n' "$*"; }
die()  { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }
have() { command -v "$1" >/dev/null 2>&1; }

while [[ $# -gt 0 ]]; do
  case "$1" in
    --release|-r) MODE="release" ;;
    --debug|-d)   MODE="debug" ;;
    --check|-c)   MODE="check" ;;
    --frontend|-f) MODE="frontend" ;;
    --clean)      DO_CLEAN=1 ;;
    --verbose|-v) VERBOSE=1 ;;
    --help|-h)    usage ;;
    *) die "unknown option: $1 (try --help)" ;;
  esac
  shift
done

# ---------------------------------------------------------------------------
# Toolchain
# ---------------------------------------------------------------------------
ensure_tools() {
  log "Checking toolchain…"

  have cargo  || die "cargo not found — install Rust from https://rustup.rs"
  have rustc  || die "rustc not found — install Rust from https://rustup.rs"
  have rustup || die "rustup not found — install Rust from https://rustup.rs"

  if [[ -f "$ROOT/rust-toolchain.toml" ]]; then
    log "Using rust-toolchain.toml"
    rustup show active-toolchain || true
  fi

  # WASM target for Trunk / Leptos CSR
  if ! rustup target list --installed | grep -qx 'wasm32-unknown-unknown'; then
    log "Installing rustup target wasm32-unknown-unknown"
    rustup target add wasm32-unknown-unknown
  fi

  if ! have trunk; then
    log "Installing trunk…"
    cargo install trunk --locked
  fi

  # Prefer cargo-installed tauri CLI; fall back to npx
  TAURI_CMD=()
  if have cargo-tauri; then
    TAURI_CMD=(cargo tauri)
  elif cargo tauri --version >/dev/null 2>&1; then
    TAURI_CMD=(cargo tauri)
  elif have npm; then
    TAURI_CMD=(npm exec --yes --package @tauri-apps/cli@2 -- tauri)
  else
    log "Installing tauri-cli…"
    cargo install tauri-cli --version "^2" --locked
    TAURI_CMD=(cargo tauri)
  fi

  ok "cargo  $(cargo --version)"
  ok "rustc  $(rustc --version)"
  ok "trunk  $(trunk --version 2>/dev/null || echo installed)"
  ok "tauri  $(${TAURI_CMD[*]} --version 2>/dev/null || echo 'via cargo/npm')"

  # Frontend CSS deps (optional — Trunk can download tailwind CLI itself)
  if [[ -f "$ROOT/package.json" ]] && have npm; then
    if [[ ! -d "$ROOT/node_modules" ]]; then
      log "npm install (frontend CSS tools)…"
      npm install --no-fund --no-audit
    fi
  fi
}

run_clean() {
  log "Cleaning…"
  cargo clean --manifest-path "$ROOT/Cargo.toml" || true
  rm -rf "$ROOT/dist" "$ROOT/target" 2>/dev/null || true
  ok "clean done"
}

run_check() {
  log "Typecheck backend…"
  cargo check --manifest-path "$ROOT/app-tauri/Cargo.toml"
  log "Typecheck frontend (wasm32)…"
  cargo check --target wasm32-unknown-unknown --manifest-path "$ROOT/Cargo.toml"
  ok "check passed"
}

run_frontend() {
  log "Building frontend (trunk release)…"
  trunk build --release
  ok "frontend → $ROOT/dist"
}

run_tauri_build() {
  local args=(build)
  if [[ "$MODE" == "debug" ]]; then
    args+=(--debug)
  fi
  if [[ "$VERBOSE" -eq 1 ]]; then
    args+=(--verbose)
  fi

  log "Building CDXTheme (${MODE})…"
  log "Command: ${TAURI_CMD[*]} ${args[*]}"
  "${TAURI_CMD[@]}" "${args[@]}"

  # Print likely artifacts
  local release_dir="$ROOT/target/release"
  local debug_dir="$ROOT/target/debug"
  local bundle_root="$ROOT/target/release/bundle"
  if [[ "$MODE" == "debug" ]]; then
    bundle_root="$ROOT/target/debug/bundle"
  fi

  echo
  ok "Build finished"
  if [[ -d "$bundle_root" ]]; then
    log "Bundles under: $bundle_root"
    find "$bundle_root" -maxdepth 3 \( -name '*.app' -o -name '*.dmg' -o -name '*.msi' -o -name '*.exe' -o -name '*.AppImage' -o -name '*.deb' \) 2>/dev/null \
      | sed 's/^/  - /' || true
  fi

  case "$(uname -s)" in
    Darwin)
      local app
      app="$(find "$bundle_root/macos" -maxdepth 1 -name 'CDXTheme.app' 2>/dev/null | head -1 || true)"
      if [[ -n "${app:-}" ]]; then
        ok "macOS app: $app"
      fi
      local bin="$release_dir/CDXTheme"
      [[ "$MODE" == "debug" ]] && bin="$debug_dir/CDXTheme"
      if [[ -x "$bin" ]]; then
        ok "Binary: $bin"
      fi
      ;;
    MINGW*|MSYS*|CYGWIN*|Windows_NT)
      log "Windows installer/exe under: $bundle_root"
      ;;
  esac
}

# ---------------------------------------------------------------------------
main() {
  ensure_tools

  if [[ "$DO_CLEAN" -eq 1 ]]; then
    run_clean
  fi

  case "$MODE" in
    check)    run_check ;;
    frontend) run_frontend ;;
    release|debug) run_tauri_build ;;
    *) die "invalid mode: $MODE" ;;
  esac
}

main
