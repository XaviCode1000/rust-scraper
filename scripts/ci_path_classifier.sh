#!/usr/bin/env bash
#
# ci_path_classifier.sh — Phase 1 path-aware CI foundation.
#
# Design source: docs/research/webfang-workflow-transformation-blueprint.md
# §5 "Tier 0: Static metadata" (changed-file classifier) and Phase 2
# "Path-aware CI" (affected-area outputs). Tier 0 must decide WHAT to run
# before any compilation, so this script is FILENAME-ONLY by design: no
# content scans, no checkout, no cargo. It must stay cheap (<2s) and
# deterministic.
#
# Usage:
#   scripts/ci_path_classifier.sh [--base-ref <ref>] [--head-ref <ref>]
#                                 [--github-output <path>] [--files <list>]
#   scripts/ci_path_classifier.sh classify <base> <head>
#
# Flags:
#   --base-ref <ref>       base for `git diff` (default: origin/main).
#   --head-ref <ref>       head for `git diff` (default: HEAD).
#   --github-output <path> override for $GITHUB_OUTPUT (CI writes here).
#   --files <list>         newline-separated file list; skips `git diff`.
#                          Also honoured via $CI_PATH_CLASSIFIER_FILES.
#                          Intended for unit-testing the matcher without git.
#
# Outputs (one `key=value` line each, `true`/`false`):
#   docs_only, ci_only, code_changed, ai_changed, mcp_changed, cli_changed,
#   core_changed, crawler_changed, downloader_changed, tests_changed,
#   release_changed, lock_changed, all
#
# Destination: $GITHUB_OUTPUT (or --github-output) when set, else stdout.
# A human-readable summary always goes to stderr for the job log.
#
# Exit code: always 0 on successful classification (even when all=true);
# non-zero only for CLI misuse (bad flags) — a fallback verdict must never
# fail the caller, it must only widen the lanes that run downstream.

set -euo pipefail

# ---------------------------------------------------------------------------
# Path semantics. Each output below lists the filename patterns it matches.
# Patterns are matched with bash `case` globs against the repo-relative path.
#
#   docs_only
#     True iff EVERY changed file is docs-eligible AND at least one file
#     changed. Docs-eligible = markdown/prose (*.md, *.mdx, *.rst), anything
#     under docs/, site/, website/, book/, AGENTS.md at any level, and images
#     (png/jpg/svg/gif/webp/ico) that illustrate docs. EXPLICITLY EXCLUDED
#     even when they look doc-ish: .github/**, scripts/** (CI surface),
#     *.rs, crates/**, tests/**, benches/**, examples/**, fuzz/** (code),
#     Cargo.toml / Cargo.lock (build graph). One non-docs file flips it false.
#
#   ci_only
#     True iff every file is CI infrastructure: .github/**, scripts/**,
#     .pre-commit-config.yaml, typos.toml, codecov.yml. A scripts/release*.sh
#     file ALSO sets release_changed — the flags are independent, not exclusive.
#
#   code_changed
#     Any Rust build input: *.rs, crates/**, tests/**, benches/**,
#     examples/**, fuzz/**, Cargo.toml, Cargo.lock, build.rs, clippy.toml,
#     rustfmt.toml, rust-toolchain.toml, deny.toml, nextest.toml, .cargo/**,
#     Dockerfile*/docker-compose (container build inputs).
#
#   ai_changed
#     Heuristic (filename-only, may over-match): crates/webfang_ai/** plus
#     AI-model/inference markers anywhere (ai_integration, clean_ai, onnx,
#     granite, embedding). Content check is deliberately NOT done here; the
#     AI lane treats a match as "run the model tests", a miss costs a lane.
#
#   mcp_changed
#     crates/webfang_mcp/** or any path containing `mcp` (mcp_server,
#     mcp_behavioral_test, handshake fixtures).
#
#   cli_changed
#     crates/webfang_cli/**, cli_harness, cli_reference fixtures.
#
#   core_changed
#     crates/webfang_core/** (domain + application + infrastructure layers).
#
#   crawler_changed
#     Any path containing `crawler` or `sitemap` (engine, politeness, sitemap
#     parsing, checkpoint code). Substring match is intentional: crawler code
#     is scattered across core layers, not one directory.
#
#   downloader_changed
#     Fetch guard-chain paths (see AGENTS.md "Fetch guard-chain"): any path
#     containing `downloader`, `ssrf`, `guard_chain`/`guard-chain`, `waf`,
#     `cookie_bridge`, `hybrid_router`, `spa_detector`, `resource_governor`.
#     Guard order is load-bearing, so ANY file here must run the mock
#     behavioral lane, not just unit tests.
#
#   tests_changed
#     tests/** or */tests/**, *test*.rs, insta snapshots (*.snap,
#     *.snap.new), nextest.toml, test-inventory files/fixtures.
#
#   release_changed
#     Cargo.lock (Cargo.* lock per spec), release-plz.toml, cliff.toml,
#     CHANGELOG*, the release workflow, release/publish/package/dist scripts,
#     container files. Drives the release tier, never the PR fast gate.
#
#   lock_changed
#     Cargo.lock at any level (plus any *.lock as a conservative net).
#     Implies code_changed + release_changed (build graph moved).
#
#   all
#     Conservative fallback: true when `git diff` cannot run (missing refs,
#     not a git checkout), when the file list is EMPTY (nothing to prove
#     narrow scope from), or when ANY file matches none of the docs/ci/code
#     buckets (unknown surface — e.g. .envrc, editor configs, new top-level
#     tooling). Downstream must then run the FULL local gate, never skip.
# ---------------------------------------------------------------------------

BASE_REF="origin/main"
HEAD_REF="HEAD"
OUTPUT_OVERRIDE=""
FILES_OVERRIDE="${CI_PATH_CLASSIFIER_FILES:-}"
# Tracks whether --files / $CI_PATH_CLASSIFIER_FILES was GIVEN at all: an
# explicitly empty list means "zero files changed" ( -> all=true ), which is
# different from "no list provided" ( -> run git diff). Without this flag an
# empty --files would silently fall back to the range diff.
FILES_GIVEN=false
# NOTE: explicit if/then (not `[[ ... ]] && x=true`): under `set -e` a bare
# `predicate && assignment` statement aborts the script when the predicate
# is FALSE. Same rule applies to every flag set below.
if [[ -n "$FILES_OVERRIDE" ]]; then
  FILES_GIVEN=true
fi

usage() {
  sed -n '2,30p' "$0"
}

# --- per-file predicates (return 0 on match) --------------------------------

is_docs_file() {
  local f="$1"
  case "$f" in
    .github/* | scripts/* | crates/* | tests/* | benches/* | examples/* | fuzz/*)
      return 1 ;;
    *.rs | *Cargo.toml | *Cargo.lock)
      return 1 ;;
  esac
  case "$f" in
    # NOTE: AGENTS.md patterns come FIRST: the later *.md glob would
    # otherwise shadow them (same match, lost documentation of intent).
    AGENTS.md | */AGENTS.md | \
      *.md | *.mdx | *.rst | docs/* | site/* | website/* | book/* | \
      *.png | *.jpg | *.jpeg | *.svg | *.gif | *.webp | *.ico)
      return 0 ;;
  esac
  return 1
}

is_ci_file() {
  local f="$1"
  case "$f" in
    .github/* | scripts/* | .pre-commit-config.yaml | typos.toml | codecov.yml)
      return 0 ;;
  esac
  return 1
}

is_code_file() {
  local f="$1"
  case "$f" in
    # NOTE: build.rs forms come FIRST: the *.rs glob below would otherwise
    # shadow them (same match, lost documentation of intent).
    */build.rs | build.rs | \
      *.rs | crates/* | tests/* | */tests/* | benches/* | examples/* | fuzz/* | \
      *Cargo.toml | *Cargo.lock | \
      clippy.toml | rustfmt.toml | rust-toolchain.toml | deny.toml | \
      nextest.toml | .cargo/* | Dockerfile* | *.dockerfile | docker-compose*)
      return 0 ;;
  esac
  return 1
}

# --- main classifier ----------------------------------------------------------
# classify <base> <head>: writes all outputs to $GITHUB_OUTPUT (or the
# --github-output override) when set, else to stdout. Always exits 0.

classify() {
  local base="${1:-$BASE_REF}"
  local head="${2:-$HEAD_REF}"
  local dest="${OUTPUT_OVERRIDE:-${GITHUB_OUTPUT:-}}"

  local -a files=()
  local diff_ok=true

  if $FILES_GIVEN; then
    # Synthetic input (tests): newline-separated list, blanks ignored.
    while IFS= read -r line || [[ -n "$line" ]]; do
      if [[ -n "$line" ]]; then files+=("$line"); fi
    done <<< "$FILES_OVERRIDE"
  else
    local root
    root="$(git rev-parse --show-toplevel 2>/dev/null || dirname "$0")"
    # Resolve the base: requested ref, then main, then HEAD~1. Anything else
    # is fail-closed (all=true) rather than fail-open.
    local resolved=""
    for candidate in "$base" "main" "HEAD~1"; do
      if git -C "$root" rev-parse --verify --quiet "$candidate" >/dev/null 2>&1; then
        resolved="$candidate"
        break
      fi
    done
    if [[ -z "$resolved" ]]; then
      diff_ok=false
    else
      local raw=""
      if raw="$(git -C "$root" diff --name-only -z "${resolved}...${head}" -- 2>/dev/null)"; then
        :
      elif raw="$(git -C "$root" diff --name-only -z "$resolved" "$head" -- 2>/dev/null)"; then
        :
      else
        diff_ok=false
      fi
      if $diff_ok && [[ -n "$raw" ]]; then
        # NUL-separated: safe for spaces in paths. mapfile may return
        # non-zero on empty input, hence `|| true` under `set -e`.
        mapfile -d '' -t files <<< "$raw" || true
      fi
    fi
  fi

  # Drop empty elements from the NUL split, if any. The ${#...} guard
  # keeps `set -u` happy on empty arrays (bash < 4.4 treats "${a[@]}"
  # of an empty array as unbound).
  local -a clean=()
  local f
  if [[ ${#files[@]} -gt 0 ]]; then
    for f in "${files[@]}"; do
      [[ -n "$f" ]] && clean+=("$f")
    done
  fi
  files=()
  if [[ ${#clean[@]} -gt 0 ]]; then
    for f in "${clean[@]}"; do
      files+=("$f")
    done
  fi

  # Defaults: nothing changed.
  local docs_only=false ci_only=false code_changed=false ai_changed=false
  local mcp_changed=false cli_changed=false core_changed=false
  local crawler_changed=false downloader_changed=false tests_changed=false
  local release_changed=false lock_changed=false all=false

  if ! $diff_ok || [[ ${#files[@]} -eq 0 ]]; then
    # Conservative: no evidence of narrow scope -> run everything.
    all=true
  else
    local all_docs=true all_ci=true known=true
    for f in "${files[@]}"; do
      if ! is_docs_file "$f"; then all_docs=false; fi
      if ! is_ci_file "$f"; then all_ci=false; fi
      if is_code_file "$f"; then
        code_changed=true
      elif ! is_docs_file "$f" && ! is_ci_file "$f"; then
        known=false
      fi

      case "$f" in
        *webfang_ai* | *ai_integration* | *clean_ai* | *onnx* | *granite* | *embedding*)
          ai_changed=true ;;
      esac
      case "$f" in
        *webfang_mcp* | *mcp* | *MCP*)
          mcp_changed=true ;;
      esac
      case "$f" in
        *webfang_cli* | *cli_harness* | *cli_reference*)
          cli_changed=true ;;
      esac
      case "$f" in
        *webfang_core*)
          core_changed=true ;;
      esac
      case "$f" in
        *crawler* | *sitemap*)
          crawler_changed=true ;;
      esac
      case "$f" in
        *downloader* | *ssrf* | *guard_chain* | *guard-chain* | *waf* | \
          *cookie_bridge* | *hybrid_router* | *spa_detector* | *resource_governor*)
          downloader_changed=true ;;
      esac
      case "$f" in
        tests/* | */tests/* | *test*.rs | *.snap | *.snap.new | \
          nextest.toml | *test-inventory* | *test_inventory* | fixtures/*)
          tests_changed=true ;;
      esac
      case "$f" in
        *Cargo.lock | release-plz.toml | cliff.toml | CHANGELOG* | \
          .github/workflows/release.yml | scripts/*release* | \
          scripts/*publish* | scripts/*package* | scripts/*dist* | \
          Dockerfile* | *.dockerfile | docker-compose*)
          release_changed=true ;;
      esac
      case "$f" in
        *Cargo.lock | *.lock)
          lock_changed=true ;;
      esac
    done
    # A lockfile moves the build graph: always code + release relevant.
    if $lock_changed; then
      code_changed=true
      release_changed=true
    fi
    if $all_docs; then docs_only=true; fi
    if $all_ci; then ci_only=true; fi
    # Unknown surface anywhere -> widen to everything.
    if ! $known; then all=true; fi
  fi

  # NOTE: never redirect to /dev/stdout by path — some sandboxes and CI
  # capture harnesses expose fd 1 via a non-reopenable handle (ENXIO on
  # open). With no dest we inherit stdout directly, which always works.
  if [[ -n "$dest" ]]; then
    {
      echo "docs_only=$docs_only"
      echo "ci_only=$ci_only"
      echo "code_changed=$code_changed"
      echo "ai_changed=$ai_changed"
      echo "mcp_changed=$mcp_changed"
      echo "cli_changed=$cli_changed"
      echo "core_changed=$core_changed"
      echo "crawler_changed=$crawler_changed"
      echo "downloader_changed=$downloader_changed"
      echo "tests_changed=$tests_changed"
      echo "release_changed=$release_changed"
      echo "lock_changed=$lock_changed"
      echo "all=$all"
    } >> "$dest"
  else
    echo "docs_only=$docs_only"
    echo "ci_only=$ci_only"
    echo "code_changed=$code_changed"
    echo "ai_changed=$ai_changed"
    echo "mcp_changed=$mcp_changed"
    echo "cli_changed=$cli_changed"
    echo "core_changed=$core_changed"
    echo "crawler_changed=$crawler_changed"
    echo "downloader_changed=$downloader_changed"
    echo "tests_changed=$tests_changed"
    echo "release_changed=$release_changed"
    echo "lock_changed=$lock_changed"
    echo "all=$all"
  fi

  # Human summary for the job log (stderr so it never pollutes $GITHUB_OUTPUT
  # parsing or stdout key=value consumers).
  echo "classifier: ${#files[@]} file(s) base=$base head=$head -> docs_only=$docs_only ci_only=$ci_only code=$code_changed ai=$ai_changed mcp=$mcp_changed cli=$cli_changed core=$core_changed crawler=$crawler_changed downloader=$downloader_changed tests=$tests_changed release=$release_changed lock=$lock_changed all=$all" >&2
}

# --- CLI -----------------------------------------------------------------------

if [[ "${1:-}" == "classify" ]]; then
  shift
  classify "${1:-$BASE_REF}" "${2:-$HEAD_REF}"
  exit 0
fi

while [[ $# -gt 0 ]]; do
  case "$1" in
    --base-ref) BASE_REF="${2:?missing value for --base-ref}"; shift 2 ;;
    --head-ref) HEAD_REF="${2:?missing value for --head-ref}"; shift 2 ;;
    --github-output) OUTPUT_OVERRIDE="${2:?missing value for --github-output}"; shift 2 ;;
    --files) FILES_OVERRIDE="$2"; FILES_GIVEN=true; shift 2 ;;
    -h | --help) usage; exit 0 ;;
    *) echo "error: unknown argument '$1' (see --help)" >&2; exit 2 ;;
  esac
done

classify "$BASE_REF" "$HEAD_REF"
