#!/usr/bin/env bash
#
# check_orphan_snapshots.sh — Phase 3 orphan-snapshot guard (no cargo).
#
# Purpose: enforce no orphan snapshots. insta names integration baselines
# `<file_stem>__<snapshot>.snap`, so a `.snap` whose stem matches no test
# file is a dead expectation: it never runs, never fails, and rots.
#
# Rules:
#   1. Any `*.snap.new` (pending snapshot) is a violation — pending
#      snapshots must never be committed (CI runs with --check semantics).
#   2. A `*.snap` under `crates/<crate>/tests/**` is valid iff the stem
#      before the first `__` matches an existing test target in the SAME
#      crate's `tests/` tree: either a `<stem>.rs` file anywhere under it
#      or a `<stem>/` directory (multi-file targets such as
#      `tests/behavioral/` with `main.rs` snapshot as `behavioral__...`).
#   3. A `*.snap` under `crates/<crate>/src/**/snapshots/` is a unit-test
#      baseline (compiled with the crate, e.g.
#      `src/infrastructure/axtree/snapshots/` owned by `playwright.rs`).
#      It is valid iff a sibling `.rs` file in the snapshots dir's parent
#      has its file stem contained in the snapshot basename. This is the
#      one principled exception to "tests tree only": without it the guard
#      would fail on the clean tree.
#   4. Any other `*.snap` location is a violation.
#
# Test seam: set ORPHAN_SNAP_ROOT to point the guard at a synthetic tree
# (used by the negative test) instead of the git toplevel.
#
# Usage: bash scripts/check_orphan_snapshots.sh
# Exit: 1 with offending paths listed on violation, 0 when clean.
#
set -euo pipefail

ROOT="${ORPHAN_SNAP_ROOT:-$(git rev-parse --show-toplevel 2>/dev/null || (cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd))}"
FAIL=0

report() {
  echo "::error::$1"
  FAIL=1
}

# --- rule 1: pending snapshots ------------------------------------------------
while IFS= read -r -d '' pending; do
  report "pending snapshot must never be committed: ${pending#"$ROOT"/}"
done < <(find "$ROOT" -path "$ROOT/.git" -prune -o -type f -name '*.snap.new' -print0 2>/dev/null)

# --- rules 2-4: orphan baselines -----------------------------------------------
while IFS= read -r -d '' snap; do
  rel="${snap#"$ROOT"/}"
  base="$(basename "$snap" .snap)"

  # Crate is the second path component (crates/<crate>/...).
  rest="${rel#crates/}"
  crate="${rest%%/*}"
  if [[ "$rel" != crates/* ]] || [[ "$rest" == "$rel" ]] || [[ -z "$crate" ]]; then
    report "snapshot outside crates/: $rel"
    continue
  fi

  if [[ "$rel" == crates/*/tests/* ]]; then
    # Rule 2: stem before the first `__` must resolve in the same crate's
    # tests/ tree (file `<stem>.rs` anywhere, or directory `<stem>/`).
    if [[ "$base" == *__* ]]; then
      stem="${base%%__*}"
    else
      stem="$base"
    fi
    if [[ -z "$stem" ]]; then
      report "snapshot with empty stem: $rel"
      continue
    fi
    if [[ -n "$(find "$ROOT/crates/$crate/tests" \( -name "${stem}.rs" -o -name "$stem" \) -print -quit 2>/dev/null)" ]]; then
      continue
    fi
    report "orphan snapshot (no $stem.rs or $stem/ in crates/$crate/tests/): $rel"
  elif [[ "$rel" == crates/*/src/*/snapshots/* ]]; then
    # Rule 3: unit-test baseline — a sibling module must own it.
    parent="$(dirname "$(dirname "$snap")")"
    owned=false
    for sibling in "$parent"/*.rs; do
      [[ -e "$sibling" ]] || break
      sstem="$(basename "$sibling" .rs)"
      if [[ "$base" == *"$sstem"* ]]; then
        owned=true
        break
      fi
    done
    if ! $owned; then
      report "orphan src snapshot (no owning sibling module in $parent): $rel"
    fi
  else
    # Rule 4: snapshots live in tests/ or src unit snapshots/ only.
    report "snapshot outside crates/<crate>/{tests,src}/**: $rel"
  fi
done < <(find "$ROOT" -path "$ROOT/.git" -prune -o -type f -name '*.snap' -print0 2>/dev/null)

if [[ $FAIL -ne 0 ]]; then
  echo "FAILED: orphan snapshot issue(s) found"
  exit 1
fi

echo "OK: no orphan snapshots"
exit 0
