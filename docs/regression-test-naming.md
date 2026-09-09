# Regression test naming convention

Behavior-change tests must be traceable to the issue/task that motivated
them. Use these prefixes for test function names (unit, integration, and
behavioral):

| Prefix | Meaning | Real examples (verified in-tree) |
| :--- | :--- | :--- |
| `issue_<N>_*` | Regression for issue `<N>` | `issue_1151_post_handshake_stdout_close_exits_cleanly` (`crates/webfang_mcp/tests/stdio_handshake.rs`), `issue_1132_zero_limits_unrepresentable_and_mapping_is_one_to_one` (`crates/webfang_mcp/tests/mcp_protocol.rs`), `issue_1117_hostile_urls_cannot_reach_download` (`crates/webfang_core/src/adapters/downloader/mod.rs`), `issue_1162_invalid_url_rejected_before_fetch` (`crates/webfang_core/src/application/crawler/sitemap_discovery.rs`) |
| `pr_<N>_*` | Regression scoped to PR `<N>` (no issue) | Reserved — zero current users in the tree (verified 2026-09-09). Use when the fix lands directly from review without an issue. |
| `waf_*` | WAF/guard-chain behavior | `waf_gauntlet_403_429_200_success`, `waf_gauntlet_persistent_403_fails` (`crates/webfang_core/tests/behavioral/cli/waf_gauntlet_test.rs`), `waf_sequence_is_deterministic_across_fresh_serves` (`crates/webfang_benchmark/tests/corpus_determinism_test.rs`) |
| `crawler_*` | Crawler surface behavior | `crawler_http_download_and_feature_flags_parse_identically` (`crates/webfang_core/src/cli/args/crawler.rs`), `crawler_max_depth_keeps_zero_valid_and_enforces_the_shared_cap` (`crates/webfang_core/src/domain/options_spec/mod.rs`) |

## Rules

1. **New behavior-change tests carry the issue/task number.** A test that
   pins fixed behavior is named `issue_<N>_<what_it_proves>` (or `pr_<N>_…`
   when there is no issue). Area prefixes (`waf_*`, `crawler_*`) describe
   the surface; combine them after the number when both apply
   (`issue_<N>_waf_…`).
2. **Snapshots move with the test.** Behavioral changes that alter
   Markdown/JSON/stderr output update the insta snapshots alongside
   (`cargo insta review`, never commit `.snap.new`; sanitize with
   `redact_nondeterministic()`).
3. **Advisory enforcement.** `scripts/check_regression_naming.sh` lists
   changed test files whose added test functions lack any issue/PR number
   reference as `::notice::` lines. It never fails — the convention is
   reviewed by humans, not enforced by gates.
