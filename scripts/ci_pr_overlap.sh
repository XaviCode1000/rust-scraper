#!/usr/bin/env bash
#
# ci_pr_overlap.sh — Phase 5 file-overlap check for safe PR batching.
#
# Design source: docs/research/webfang-workflow-transformation-blueprint.md
# Phase 5 "Merge batching without native queue" + AGENTS.md "Batch merge of
# multiple green PRs" (files touched must be fully disjoint).
#
# Usage:
#   scripts/ci_pr_overlap.sh <PR-N1> <PR-N2> [...]
#   scripts/ci_pr_overlap.sh -h | --help
#
# Behaviour: for each PR, reads the changed-file list via read-only
# `gh pr view --json files,baseRefName,state`, prints per-PR file lists and
# the pairwise overlap. Verifies each PR is open and targets `main`.
#
# Exit codes:
#   0  All file sets are disjoint (safe to batch).
#   1  At least one overlapping path exists (do NOT batch; printed on stdout).
#   2  Usage error, gh missing/failure, or a PR is not open / not targeting main.
#
# Scope notes:
#   - Read-only: only `gh pr view` queries. Never comments, merges, or mutates.
#   - Overlap is computed on exact path strings (no renames resolution).
#   - CHANGELOG.md overlap is always reported — per AGENTS.md, work PRs must
#     never touch CHANGELOG.md, so any appearance there is a policy violation
#     to fix first, not a batch candidate.
set -euo pipefail

usage() {
  sed -n '2,22p' "$0"
}

if [[ ${1:-} == "-h" || ${1:-} == "--help" ]]; then
  usage
  exit 0
fi

if [[ $# -lt 2 ]]; then
  echo "error: at least two PR numbers are required" >&2
  usage >&2
  exit 2
fi

for arg in "$@"; do
  if ! [[ "$arg" =~ ^[0-9]+$ ]]; then
    echo "error: invalid PR number '$arg' (expected digits only)" >&2
    usage >&2
    exit 2
  fi
done

if ! command -v gh >/dev/null 2>&1; then
  echo "error: gh CLI is not installed or not on PATH" >&2
  exit 2
fi

TMPDIR_OVERLAP="$(mktemp -d)"
trap 'rm -rf "$TMPDIR_OVERLAP"' EXIT

prs=("$@")

# --- fetch + validate each PR -------------------------------------------------
for pr in "${prs[@]}"; do
  meta="$(gh pr view "$pr" --json baseRefName,state 2>/dev/null || true)"
  if [[ -z "$meta" ]]; then
    echo "error: could not read PR #$pr (gh failure or PR does not exist)" >&2
    exit 2
  fi
  base="$(printf '%s' "$meta" | jq -r '.baseRefName // empty' 2>/dev/null || true)"
  state="$(printf '%s' "$meta" | jq -r '.state // empty' 2>/dev/null || true)"
  if [[ "$state" != "OPEN" ]]; then
    echo "error: PR #$pr is not open (state=${state:-unknown})" >&2
    exit 2
  fi
  if [[ "$base" != "main" ]]; then
    echo "error: PR #$pr targets '$base', not 'main'" >&2
    exit 2
  fi
  files="$(gh pr view "$pr" --json files --jq '.files[].path' 2>/dev/null || true)"
  if [[ -z "$files" ]]; then
    echo "error: could not read file list for PR #$pr" >&2
    exit 2
  fi
  printf '%s\n' "$files" | sort -u > "$TMPDIR_OVERLAP/pr-$pr.txt"
  count="$(wc -l < "$TMPDIR_OVERLAP/pr-$pr.txt" | tr -d ' ')"
  echo "PR #$pr ($count files, base=main, state=OPEN):"
  sed 's/^/  /' "$TMPDIR_OVERLAP/pr-$pr.txt"
done

# --- pairwise overlap ----------------------------------------------------------
overlap_found=0
n=${#prs[@]}
for ((i = 0; i < n; i++)); do
  for ((j = i + 1; j < n; j++)); do
    a="${prs[$i]}"
    b="${prs[$j]}"
    shared="$(comm -12 "$TMPDIR_OVERLAP/pr-$a.txt" "$TMPDIR_OVERLAP/pr-$b.txt" || true)"
    if [[ -n "$shared" ]]; then
      overlap_found=1
      echo "overlap: PR #$a <-> PR #$b:"
      printf '%s\n' "$shared" | sed 's/^/  shared: /'
    else
      echo "disjoint: PR #$a <-> PR #$b"
    fi
  done
done

if [[ $overlap_found -eq 1 ]]; then
  echo "RESULT: OVERLAP — do NOT batch these PRs."
  exit 1
fi
echo "RESULT: DISJOINT — safe to batch."
exit 0
