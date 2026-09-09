#!/usr/bin/env bash
#
# ci_status.sh — Phase 5 compact read-only PR status aggregator.
#
# Design source: docs/research/webfang-workflow-transformation-blueprint.md
# Phase 5 "Add simple ci-status aggregator if needed".
#
# Usage:
#   scripts/ci_status.sh <PR-N>
#   scripts/ci_status.sh -h | --help
#
# Behaviour: prints one compact summary — required-check buckets via
# `gh pr checks --json name,bucket` plus mergeStateStatus/mergeable/state
# via `gh pr view --json` — and exits 0. Read-only: never merges, comments,
# or mutates anything.
#
# Exit codes:
#   0  Status printed (regardless of whether checks are green or red).
#   2  Usage error or gh failure.
set -euo pipefail

usage() {
  sed -n '2,17p' "$0"
}

if [[ ${1:-} == "-h" || ${1:-} == "--help" ]]; then
  usage
  exit 0
fi

if [[ $# -ne 1 || ! "${1:-}" =~ ^[0-9]+$ ]]; then
  echo "error: exactly one PR number is required" >&2
  usage >&2
  exit 2
fi

if ! command -v gh >/dev/null 2>&1; then
  echo "error: gh CLI is not installed or not on PATH" >&2
  exit 2
fi

pr="$1"

checks="$(gh pr checks "$pr" --json name,bucket 2>/dev/null || true)"
if [[ -z "$checks" ]]; then
  echo "error: could not read checks for PR #$pr" >&2
  exit 2
fi

meta="$(gh pr view "$pr" --json number,title,state,baseRefName,mergeStateStatus,mergeable 2>/dev/null || true)"
if [[ -z "$meta" ]]; then
  echo "error: could not read metadata for PR #$pr" >&2
  exit 2
fi

title="$(printf '%s' "$meta" | jq -r '.title // "?"')"
state="$(printf '%s' "$meta" | jq -r '.state // "?"')"
base="$(printf '%s' "$meta" | jq -r '.baseRefName // "?"')"
mstate="$(printf '%s' "$meta" | jq -r '.mergeStateStatus // "?"')"
mergeable="$(printf '%s' "$meta" | jq -r '.mergeable // "?"')"

echo "PR #$pr: $title"
echo "  state=$state base=$base mergeStateStatus=$mstate mergeable=$mergeable"
echo "  checks:"
printf '%s' "$checks" | jq -r '.[] | "    \(.bucket)\t\(.name)"' | sort
exit 0
