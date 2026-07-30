---
name: create-pr
description: >
  Create a GitHub pull request from the current branch: inspect status/commits/diff,
  write a Summary + Test plan body, push upstream, and open the PR with `gh`.
  Use when the user asks to "create PR", "open a PR", "generate github pr",
  "提 PR", "创建 PR", or runs /create-pr.
metadata:
  short-description: "Push branch and open a GitHub PR"
---

# Create GitHub PR

Open a pull request for the **current branch** against the repo default base (`master` or `main`).

Requires: `git`, `gh` (authenticated), network for push/create.

## When to use

- User: "generate github pr", "create PR", "open PR", "提 PR", `/create-pr`
- Work is already committed (or you just finished committing)

## Safety

- **Never force-push** unless the user explicitly asks.
- **Never push** if the user only asked for a draft body / dry-run.
- Confirm before pushing if the branch already tracks a remote that diverged, or if you would rewrite history.
- Do not amend published commits unless the user asks.

## Workflow

### 1. Gather context (always run first)

From the repo root:

```bash
# Preferred: one-shot context dump (this skill)
bash .agents/skills/github/scripts/pr-context.sh
```

Or the equivalent manual checks:

```bash
git status
git branch -vv
git remote -v
git rev-parse --abbrev-ref HEAD

# Base branch: prefer origin/HEAD, else master, else main
BASE=$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@')
BASE=${BASE:-master}
git show-ref --verify --quiet "refs/remotes/origin/$BASE" || BASE=main

git log --oneline "${BASE}..HEAD"
git diff --stat "${BASE}...HEAD"
git diff "${BASE}...HEAD"   # skim for summary; keep large diffs out of the PR body
gh auth status
gh repo view --json nameWithOwner,defaultBranchRef
```

Interpret:

| Check | Action |
| --- | --- |
| Dirty working tree | Commit or stash first (ask user if unclear). Do not open a PR that omits unfinished edits unless they say so. |
| No commits vs base | Nothing to PR — stop and tell the user. |
| Branch has no upstream | Will `git push -u origin HEAD` in step 3. |
| Already has open PR for this head | Update existing PR or report the URL — do not open a duplicate. |

Check for an existing PR:

```bash
gh pr view --json url,number,state,title 2>/dev/null || true
# or:
gh pr list --head "$(git rev-parse --abbrev-ref HEAD)" --json number,url,state,title
```

### 2. Write title and body

**Title:** short, imperative, explains *why* / product effect (not a dump of file names).

**Body** (Markdown) — always include:

```markdown
## Summary

- <1–3 bullets of what changed and why>

## Test plan

- [ ] <concrete verification steps>
```

Rules:

- Base the summary on the **commit list + diffstat**, not memory alone.
- Test plan items should be checkboxes the author can actually run.
- Keep the body proportional to the change size.

### 3. Push and create

**Option A — helper script** (title + body via env or flags):

```bash
bash .agents/skills/github/scripts/create-pr.sh \
  --title "Short imperative title" \
  --body-file /tmp/pr-body.md
```

Or pass body inline:

```bash
bash .agents/skills/github/scripts/create-pr.sh \
  --title "Short imperative title" \
  --body "$(cat <<'EOF'
## Summary

- …

## Test plan

- [ ] …
EOF
)"
```

**Option B — manual (same semantics):**

```bash
git push -u origin HEAD

gh pr create --base "$BASE" --title "…" --body "$(cat <<'EOF'
## Summary

- …

## Test plan

- [ ] …

EOF
)"
```

Optional flags (when the user asks):

| Flag | Meaning |
| --- | --- |
| `--draft` | Open as draft |
| `--base <branch>` | Override base (default: origin default / master / main) |
| `--fill` | Use commit messages only — **avoid** unless body is trivial |

### 4. Report back

Always return:

1. PR URL (from `gh pr create` stdout)
2. Base ← head branch names
3. One-line reminder of what the PR covers

## Helper scripts

| Script | Role |
| --- | --- |
| `scripts/pr-context.sh` | Print status, base, commits, diffstat, existing PR (read-only) |
| `scripts/create-pr.sh` | Push `-u` if needed, then `gh pr create` |

Both live under `.agents/skills/github/scripts/` (repo-local copy of the user skill).

## Failure handling

| Error | What to do |
| --- | --- |
| `gh` not logged in | Run `gh auth status`; tell user to `gh auth login` |
| Push rejected (non-fast-forward) | Do **not** force-push; show status and ask |
| No permission / wrong remote | Show `git remote -v` and `gh repo view` |
| Empty body/title | Abort create; fix and retry |

## Do not

- Create a PR with zero commits ahead of base
- Force-push by default
- Put secrets, `.env`, or signing material in the PR body
- Open a second PR for the same head when one already exists (share the existing URL instead)
