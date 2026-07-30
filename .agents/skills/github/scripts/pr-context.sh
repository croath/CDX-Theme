#!/usr/bin/env bash
# Read-only dump of git/gh context for drafting a GitHub PR.
# Usage: pr-context.sh [base-branch]
set -euo pipefail

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "error: not inside a git repository" >&2
  exit 1
fi

ROOT=$(git rev-parse --show-toplevel)
cd "$ROOT"

HEAD=$(git rev-parse --abbrev-ref HEAD)
if [[ "$HEAD" == "HEAD" ]]; then
  echo "error: detached HEAD — checkout a branch first" >&2
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

BASE=$(detect_base "${1:-}")

echo "=== repo ==="
echo "root: $ROOT"
echo "head: $HEAD"
echo "base: $BASE"
if git rev-parse --abbrev-ref --symbolic-full-name '@{u}' >/dev/null 2>&1; then
  echo "upstream: $(git rev-parse --abbrev-ref --symbolic-full-name '@{u}')"
else
  echo "upstream: (none)"
fi
echo

echo "=== remotes ==="
git remote -v
echo

echo "=== status ==="
git status -sb
echo

echo "=== branch -vv (current) ==="
git branch -vv | sed -n "/^\* /p"
echo

echo "=== commits (${BASE}..HEAD) ==="
if git rev-parse --verify "$BASE" >/dev/null 2>&1 || git rev-parse --verify "origin/$BASE" >/dev/null 2>&1; then
  REF="$BASE"
  git rev-parse --verify "$BASE" >/dev/null 2>&1 || REF="origin/$BASE"
  COUNT=$(git rev-list --count "${REF}..HEAD" 2>/dev/null || echo 0)
  echo "count: $COUNT"
  if [[ "$COUNT" != "0" ]]; then
    git log --oneline "${REF}..HEAD"
    echo
    echo "=== diffstat (${REF}...HEAD) ==="
    git diff --stat "${REF}...HEAD"
  else
    echo "(no commits ahead of $REF)"
  fi
else
  echo "warning: base '$BASE' not found locally or on origin"
fi
echo

echo "=== existing PR for this head ==="
if command -v gh >/dev/null 2>&1; then
  if gh auth status >/dev/null 2>&1; then
    gh pr list --head "$HEAD" --json number,url,state,title,baseRefName 2>/dev/null \
      || echo "(gh pr list failed)"
    echo
    echo "=== repo (gh) ==="
    gh repo view --json nameWithOwner,defaultBranchRef 2>/dev/null \
      || echo "(gh repo view failed)"
  else
    echo "gh is not authenticated (gh auth login)"
  fi
else
  echo "gh not installed"
fi
