# Mode D re-verification — evidence

**HEAD:** `9b145d99` (branch `feat/mode-d-reverify`)
**Date:** 2026-09-09 · **Method:** real execution, ai-featured dev build
**Build:** `cargo build -p webfang_cli --features ai` — clean, 0 errors, 6m10s
(isolated target dir `~/.cache/cargo-target/feat-mode-d-reverify`,
`RUSTC_WRAPPER` unset — sccache breaks isolated builds, see Gate 4 #1267
follow-up). Model: Granite-97M from native hf_hub cache
(`~/.cache/huggingface/hub/`, 397 MB) — no download needed.
**Fixture:** local `python3 -m http.server` on `127.0.0.1:18989/article.html`
(200, article markup). SSRF guards disabled via the official test env vars
(`WEBFANG_DISABLE_SSRF_ENTRY_GUARD/RESOLVER/REDIRECT_GUARD=1`,
`domain/ssrf_guard.rs:53-82`) — same method as the auditor; with guards on,
loopback exits 69 with zero pages (invalid evidence either way).

## Results

| Check | Result | Evidence |
|---|---|---|
| `--clean-ai` determinism | ✅ PASS | 2 runs, exit 0 both; `checksum_sha256` identical `4644a774…eebd14`; `word_count` 18 = 18 (`/tmp/moded-r1/export.jsonl`, `/tmp/moded-r2/export.jsonl`) |
| Fail-fast preflight (invalid `--ai-model`) | ✅ PASS | exit 64 + Spanish error `Modelo AI inválido para --ai-model: Unknown AI model … Valid values: granite-97m, granite-311m` — rejected before any fetch (`/tmp/moded-r3.log`) |
| MCP+AI (`semantic_cleaner` with ai build) | ✅ PASS | ai MCP server (`--enable-ai`, granite-97m lazy wiring) returned real chunks with 384-dim embeddings for the fixture URL (live `tools/call`, `isError` absent) |
| Chromium / `--js-strategy full` (F-52) | ❌ NOT TESTED | No Chrome in this environment — stays BLOCKED, unchanged from AUDIT-02 §14 |

## Verdict
Mode D is CLOSED except JS rendering: CLI+AI (determinism + preflight) and
MCP+AI (live `semantic_cleaner` with embeddings) verified with executable
evidence on the ai-featured build. JS rendering stays BLOCKED on environment
(no Chrome), not on code. Lesson: the MCP server needs `--enable-ai` at
runtime — without it the ai build honestly reports tools unavailable (by
design); and MCP loopback fixtures need `WEBFANG_MCP_DISABLE_SSRF=1` on top
of the dial-level disables.
