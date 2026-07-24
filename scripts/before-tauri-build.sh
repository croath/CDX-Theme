#!/usr/bin/env bash
# Tauri beforeBuildCommand / beforeDevCommand helper.
# Stages the cdxtheme CLI sidecar, then runs Trunk (build or serve).
#
# Usage:
#   scripts/before-tauri-build.sh           # trunk build (release packaging)
#   scripts/before-tauri-build.sh --dev     # trunk serve (tauri dev)
#
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODE="build"
if [[ "${1:-}" == "--dev" ]]; then
  MODE="dev"
fi

bash "$ROOT/scripts/prepare-cli-sidecar.sh"

cd "$ROOT/app-ui"
if [[ "$MODE" == "dev" ]]; then
  exec trunk serve
else
  exec trunk build
fi
