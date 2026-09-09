#!/usr/bin/env bash
#
# check_regression_naming.sh — advisory regression-naming check (Phase 4).
#
# Lists test files changed in the working diff vs origin/main whose ADDED
# test functions lack any issue/PR number reference (`issue_<N>` / `pr_<N>`).
# Advisory only: prints `::notice::` lines in CI (plain `NOTICE:` locally)
# and ALWAYS exits 0 — it must never turn a lane red. See
# docs/regression-test-naming.md for the convention.
#
# What counts:
#   - Test file = changed `*.rs` under `tests/` or `*/tests/*`, or named
#     `*test*.rs` (covers unit tests colocated in src/, e.g.
#     `crates/webfang_core/src/adapters/downloader/mod.rs`).
#   - Added test function = an added `fn <name>(` line in the diff where
#     <name> looks like a test: it contains `test`, or starts with one of
#     the convention prefixes (`issue_`/`pr_`/`crawler_`/`waf_`). Plain
#     helpers (fixtures, `mount_*`, builders) are ignored.
#   - Flagged when the added test name contains NO `issue_<digits>` and NO
#     `pr_<digits>` reference.
#
# Usage: scripts/check_regression_naming.sh [--base <ref>]
# Exit code: always 0.

set -uo pipefail

BASE="origin/main"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --base) BASE="${2:?missing value for --base}"; shift 2 ;;
    -h | --help) sed -n '2,20p' "$0"; exit 0 ;;
    *) echo "error: unknown argument '$1' (see --help)" >&2; exit 0 ;;
  esac
done

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || echo ".")"

# Resolve the base; without one there is nothing to diff against.
RESOLVED=""
for candidate in "$BASE" "main" "HEAD~1"; do
  if git -C "$ROOT" rev-parse --verify --quiet "$candidate" >/dev/null 2>&1; then
    RESOLVED="$candidate"
    break
  fi
done
if [[ -z "$RESOLVED" ]]; then
  echo "regression-naming: no base ref available; nothing to check."
  exit 0
fi

is_test_file() {
  local f="$1"
  case "$f" in
    *.rs) ;;
    *) return 1 ;;
  esac
  case "$f" in
    tests/* | */tests/* | *test*.rs) return 0 ;;
  esac
  return 1
}

# Looks like a test fn (vs. a helper): contains `test` or carries a
# convention prefix.
is_test_fn() {
  local name="$1"
  case "$name" in
    *test* | issue_* | pr_* | crawler_* | waf_*) return 0 ;;
  esac
  return 1
}

# Carries an issue/PR number reference.
has_ref() {
  local name="$1"
  if [[ "$name" =~ (issue_[0-9]+|pr_[0-9]+) ]]; then
    return 0
  fi
  return 1
}

notify() {
  local file="$1" fn="$2"
  local msg="test '${fn}' in ${file} has no issue/PR number reference (see docs/regression-test-naming.md)"
  if [[ "${GITHUB_ACTIONS:-}" == "true" ]]; then
    echo "::notice file=${file}::${msg}"
  else
    echo "NOTICE: ${msg}"
  fi
}

# Changed tracked files (worktree vs base) + untracked files.
CHANGED_TMP="$(mktemp)"
trap 'rm -f "$CHANGED_TMP"' EXIT
{
  git -C "$ROOT" diff --name-only "$RESOLVED" -- 2>/dev/null || true
  git -C "$ROOT" ls-files --others --exclude-standard 2>/dev/null || true
} | grep -v '^$' | sort -u > "$CHANGED_TMP" || true

FLAGGED=0
while IFS= read -r file || [[ -n "$file" ]]; do
  [[ -n "$file" ]] || continue
  if ! is_test_file "$file"; then
    continue
  fi
  # Added `fn name(` lines: from the diff for tracked files, from the whole
  # file for untracked ones.
  ADDED=""
  if git -C "$ROOT" ls-files --error-unmatch "$file" >/dev/null 2>&1; then
    ADDED="$(git -C "$ROOT" diff -U0 "$RESOLVED" -- "$file" 2>/dev/null | grep -E '^\+\s*(async\s+)?fn [A-Za-z0-9_]+' || true)"
  else
    if [[ -f "$ROOT/$file" ]]; then
      ADDED="$(grep -E '^\s*(async\s+)?fn [A-Za-z0-9_]+' "$ROOT/$file" 2>/dev/null | sed 's/^/+ /' || true)"
    fi
  fi
  [[ -n "$ADDED" ]] || continue
  while IFS= read -r line || [[ -n "$line" ]]; do
    [[ -n "$line" ]] || continue
    # Extract the fn name: strip leading `+`, whitespace, `async`, `fn `.
    name="$(echo "$line" | sed -E 's/^\+\s*//; s/^async\s+//; s/^fn ([A-Za-z0-9_]+).*/\1/')"
    case "$name" in
      fn | async | "" ) continue ;;
    esac
    if is_test_fn "$name" && ! has_ref "$name"; then
      notify "$file" "$name"
      FLAGGED=$((FLAGGED + 1))
    fi
  done <<< "$ADDED"
done < "$CHANGED_TMP"

if [[ "$FLAGGED" -eq 0 ]]; then
  echo "regression-naming: OK — no added test functions without issue/PR reference."
else
  echo "regression-naming: $FLAGGED added test function(s) without issue/PR reference (advisory only)."
fi

exit 0
