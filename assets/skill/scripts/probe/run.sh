#!/usr/bin/env bash
# Run a saved probe expression against live Codex (CDP).
#
# Usage (from themes repo root, or any cwd):
#   .//path/to/run.sh work-layout
#   .//path/to/run.sh set-mode work
#   .//path/to/run.sh set-mode chat --wait-ms 500
#   .//path/to/run.sh go-work-home
#   .//path/to/run.sh list
#
# Extra args after the script name are passed to `/path/to/cdxthemex probe`.
# Binary path lives in repo AGENTS.md — do not hardcode it here.
# Override with CDXTHEME=… if needed; default is the `/path/to/cdxthemex` command on PATH.

set -euo pipefail

DIR="$(cd "$(dirname "$0")" && pwd)"
CDXTHEME="${CDXTHEME:-cdxthemex}"

if [[ -x "$CDXTHEME" ]]; then
  :
elif command -v "$CDXTHEME" >/dev/null 2>&1; then
  CDXTHEME="$(command -v "$CDXTHEME")"
else
  echo "cdxthemex not found (put it on PATH, or set CDXTHEME to the path from AGENTS.md)" >&2
  exit 1
fi

usage() {
  cat <<EOF
Usage: $(basename "$0") <script> [probe-flags…]
       $(basename "$0") set-mode <chat|work> [probe-flags…]
       $(basename "$0") list

Scripts (in $DIR):
EOF
  ls -1 "$DIR"/*.js 2>/dev/null | xargs -n1 basename | sed 's/\.js$//' | sed 's/^/  /'
}

if [[ $# -lt 1 ]]; then
  usage
  exit 1
fi

NAME="$1"
shift

if [[ "$NAME" == "list" || "$NAME" == "-h" || "$NAME" == "--help" ]]; then
  usage
  exit 0
fi

MODE_ARG=""
if [[ "$NAME" == "set-mode" ]]; then
  if [[ $# -lt 1 ]]; then
    echo "set-mode requires chat|work" >&2
    exit 1
  fi
  MODE_ARG="$1"
  shift
  NAME="set-mode"
fi

FILE="$DIR/${NAME}.js"
if [[ ! -f "$FILE" ]]; then
  echo "unknown script: $NAME" >&2
  usage
  exit 1
fi

EXPR="$(cat "$FILE")"
if [[ -n "$MODE_ARG" ]]; then
  # Inject mode for set-mode.js
  EXPR="var __CDXTHEME_MODE__ = $(printf '%s' "$MODE_ARG" | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read().strip()))'); $EXPR"
fi

exec "$CDXTHEME" probe --expr "$EXPR" "$@"
