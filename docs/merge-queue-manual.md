# Manual merge queue (no native queue)

This repo has **no native GitHub merge queue**: the repo object exposes no
`merge_queue_enabled` flag and no `merge_queue` branch-protection rule
applies (a REST `merge_queue` rule is rejected as invalid for this repo).
Batching below is local tooling + maintainer discipline, not a server-side
queue. It exists to dodge the **strict-mode rebase cost**: branch
protection runs with `strict: true`, so merging PR A marks every other
ready PR `BEHIND`, and each rebase re-runs the full CI (~27 min). N
sequential merges cost ~N x CI; one disjoint batch costs ~1 x CI.

## Procedure

### 1. Overlap check

```bash
scripts/ci_pr_overlap.sh <PR-N1> <PR-N2> [...]
```

Exit `0` = file sets disjoint (safe to batch); exit `1` = overlap printed
(do NOT batch — merge sequentially via step 4 instead); exit `2` = a PR is
not open, not targeting `main`, or a `gh` failure. `CHANGELOG.md` must not
appear in any list (work PRs never touch it — see AGENTS.md).

### 2. Batch branch

```bash
# SHAs are remote PR head SHAs already fetched locally:
gh pr view <N> --json headRefOid --jq .headRefOid
scripts/ci_batch_branch.sh fix/batch-<topic> <SHA1> <SHA2>
# Preview only: scripts/ci_batch_branch.sh --dry-run fix/batch-<topic> <SHA1>
```

Creates the branch from current `main` in a sibling worktree
(`~/Projects/Rust/webfang-worktrees/<dir>`) without switching the current
one, then merges each SHA with `git merge --no-ff`. On conflict it stops,
leaves the worktree in place, and prints recovery instructions — never
auto-resolves. Refuses if the branch or directory already exists.

### 3. Fast gate (in the new worktree)

```bash
cd ~/Projects/Rust/webfang-worktrees/fix-batch-<topic>
bash scripts/ci_fast_gate.sh
```

Write the CHANGELOG entries here — the ONE place they are written.

### 4a. One batch PR (disjoint slices)

Push, open the PR linking ALL issues (`Closes #A`, `Closes #B`), close the
originals as superseded, then merge with a **merge commit** (preserves
per-fix revert granularity):

```bash
scripts/merge-when-green.sh <batch-PR> --merge
```

### 4b. Sequential merges (overlapping slices)

Merge each PR on its own with the default squash strategy, rebasing the
next one onto updated `main` between merges:

```bash
scripts/merge-when-green.sh <PR-N1>          # squash (default)
# rebase next slice, then:
scripts/merge-when-green.sh <PR-N2>
```

### 5. Post-merge runbook (from the main checkout)

```bash
gh pr view <N> --json state,mergeCommit          # state: MERGED
git fetch origin && git merge --ff-only origin/main
git worktree remove ~/Projects/Rust/webfang-worktrees/<dir>
git branch -D <branch>
git worktree prune
git worktree list    # only main remains
git status --short   # empty
```

## Quick status

```bash
scripts/ci_status.sh <PR-N>   # required checks + mergeStateStatus, read-only
```

See also: AGENTS.md "Batch merge of multiple green PRs" (full procedure
with preconditions) and `scripts/merge-when-green.sh --help` (exit-code
contract: `2` = checks failed, `3` = BEHIND/BLOCKED, `4` = gh failure).
