#!/usr/bin/env bash
# Push current branch and open a GitHub PR with gh.
#
# Usage:
#   create-pr.sh --title "Title" --body "## Summary\n..."
#   create-pr.sh --title "Title" --body-file ./pr-body.md
#   create-pr.sh --title "Title" --body-file ./pr-body.md --draft
#   create-pr.sh --title "Title" --body-file ./pr-body.md --base master
#   create-pr.sh --dry-run --title "Title" --body-file ./pr-body.md
#
# Env overrides:
#   PR_TITLE, PR_BODY, PR_BASE, PR_DRAFT=1
set -euo pipefail

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "error: not inside a git repository" >&2
  exit 1
fi

if ! command -v gh >/dev/null 2>&1; then
  echo "error: gh CLI not found" >&2
  exit 1
fi

ROOT=$(git rev-parse --show-toplevel)
cd "$ROOT"

HEAD=$(git rev-parse --abbrev-ref HEAD)
if [[ "$HEAD" == "HEAD" ]]; then
  echo "error: detached HEAD — checkout a branch first" >&2
  exit 1
fi

TITLE=${PR_TITLE:-}
BODY=${PR_BODY:-}
BODY_FILE=
BASE=${PR_BASE:-}
DRAFT=${PR_DRAFT:-0}
DRY_RUN=0
PUSH=1

usage() {
  sed -n '2,14p' "$0" | sed 's/^# \{0,1\}//'
  exit 2
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --title)
      TITLE=${2:-}
      shift 2
      ;;
    --body)
      BODY=${2:-}
      shift 2
      ;;
    --body-file)
      BODY_FILE=${2:-}
      shift 2
      ;;
    --base)
      BASE=${2:-}
      shift 2
      ;;
    --draft)
      DRAFT=1
      shift
      ;;
    --no-push)
      PUSH=0
      shift
      ;;
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    -h | --help)
      usage
      ;;
    *)
      echo "error: unknown arg: $1" >&2
      usage
      ;;
  esac
done

if [[ -n "$BODY_FILE" ]]; then
  if [[ ! -f "$BODY_FILE" ]]; then
    echo "error: body file not found: $BODY_FILE" >&2
    exit 1
  fi
  BODY=$(cat "$BODY_FILE")
fi

if [[ -z "$TITLE" ]]; then
  echo "error: --title is required" >&2
  exit 1
fi
if [[ -z "$BODY" ]]; then
  echo "error: --body or --body-file is required" >&2
  exit 1
fi

detect_base() {
  local explicit=${1:-}
  if [[ -n "$explicit" ]]; then
    echo "$explicit"
    return
  fi
  local base
  base=$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@' || true)
  if [[ -n "$base" ]]; then
    echo "$base"
    return
  fi
  if git show-ref --verify --quiet refs/remotes/origin/master || git show-ref --verify --quiet refs/heads/master; then
    echo master
    return
  fi
  if git show-ref --verify --quiet refs/remotes/origin/main || git show-ref --verify --quiet refs/heads/main; then
    echo main
    return
  fi
  echo master
}

BASE=$(detect_base "$BASE")

# Prefer local base, else origin/base for ahead-count.
REF="$BASE"
if ! git rev-parse --verify "$BASE" >/dev/null 2>&1; then
  if git rev-parse --verify "origin/$BASE" >/dev/null 2>&1; then
    REF="origin/$BASE"
  else
    echo "error: base branch '$BASE' not found (tried $BASE and origin/$BASE)" >&2
    exit 1
  fi
fi

AHEAD=$(git rev-list --count "${REF}..HEAD")
if [[ "$AHEAD" -eq 0 ]]; then
  echo "error: no commits ahead of $REF — nothing to PR" >&2
  exit 1
fi

if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "warning: working tree is dirty — uncommitted changes will NOT be in the PR" >&2
fi

# Avoid duplicate open PRs for this head.
EXISTING=$(gh pr list --head "$HEAD" --state open --json url,number --jq '.[0].url // empty' 2>/dev/null || true)
if [[ -n "$EXISTING" ]]; then
  echo "open PR already exists for head '$HEAD':"
  echo "$EXISTING"
  exit 0
fi

echo "head:  $HEAD"
echo "base:  $BASE (ref $REF)"
echo "ahead: $AHEAD commit(s)"
echo "title: $TITLE"
if [[ "$DRAFT" == "1" ]]; then
  echo "draft: yes"
fi
echo

if [[ "$DRY_RUN" == "1" ]]; then
  echo "=== dry-run: would push and create PR ==="
  echo "git push -u origin HEAD"
  echo "gh pr create --base $BASE --title … --body …"
  echo
  echo "=== body preview ==="
  printf '%s\n' "$BODY"
  exit 0
fi

if [[ "$PUSH" == "1" ]]; then
  echo "→ git push -u origin HEAD"
  git push -u origin HEAD
  echo
fi

ARGS=(pr create --base "$BASE" --title "$TITLE" --body "$BODY")
if [[ "$DRAFT" == "1" ]]; then
  ARGS+=(--draft)
fi

echo "→ gh ${ARGS[*]//$BODY/[body]}"
URL=$(gh "${ARGS[@]}")
echo
echo "PR: $URL"
