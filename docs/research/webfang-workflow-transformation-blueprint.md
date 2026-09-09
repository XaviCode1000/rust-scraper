# WebFang Workflow Transformation Blueprint

Date: 2026-09-09
Status: target design
Repository: WebFang
Branch: chore/ci-workflow-overhaul
Worktree: /home/xavi/Projects/Rust/webfang-worktrees/chore-ci-workflow-overhaul

## 1. Purpose

Transform WebFang's CI, testing, PR/merge, and issue workflow into a fast, controlled, and bug-focused system using evidence from two reference groups:

- Mature Rust projects: ripgrep, tokio, serde, ruff, cargo-mutants, typst, rust-analyzer, bevy.
- Crawler/browser/agent projects: Scrapling, agent-browser, Firecrawl, browser-use.

The goal is not to copy every pattern. The goal is to adopt only the mechanisms that improve:

- wall-clock merge time,
- bug detection,
- regression traceability,
- change isolation,
- release stability,
- maintainer review load,
- CI repeatability.

## 2. Current Pain

Observed before the current branch:

- PR CI wall-clock around 20-33 minutes.
- Required checks include expensive contexts.
- `CI Gate` currently aggregates too many jobs.
- AI integration and mutation checks participate in merge friction.
- Many tests run even for docs-only or narrow changes.
- `strict` branch protection creates rebase loops:
  - merge PR A
  - PR B becomes behind
  - PR B rebuilds full CI
  - merge cost grows roughly linearly
- No native GitHub merge queue is currently usable for this repo.

## 3. Design Principles

The WebFang system should move from "CI must prove everything on every PR" to:

> "PR CI proves the change is safe to merge. Heavy verification proves it is safe to release."

### Principle 1: PR gate fast, main gate hard

PR CI should be the fast confidence layer.

Main CI should be the full evidence layer.

This mirrors:

- agent-browser: lightweight tests on PR, native e2e/cross-platform jobs off PR.
- wasmtime and ripgrep: small PR path, stronger verification outside the hottest loop.
- Firecrawl: domain-specific SDK and service checks split by component.

### Principle 2: Change classification first, test execution second

Before deciding what to run, classify the change:

- docs-only
- CI-only
- metadata-only
- core logic
- HTTP/downloader
- scraper/crawler
- AI model
- MCP contract
- database/state
- dependencies
- release

This mirrors Deno's `pre_build` output model and Firecrawl path-aware workflows.

### Principle 3: Required checks must be few and stable

Branch protection should require a smaller number of meaningful contexts.

Avoid making every valuable check blocking. Valuable does not always mean required.

This mirrors:

- rust-analyzer: aggregate result instead of many noisy required checks.
- ripgrep: maintainer-focused minimal gates.
- tokio: `basics` before expensive work.
- agent-browser: squash-only PR flow and one human review gate.

### Principle 4: Tests are risk controls, not ceremony

Each test tier must answer a specific question:

- Does the project still compile?
- Does formatting/style stay stable?
- Does core logic still behave?
- Do crawler fetch decisions remain correct?
- Do AI semantic outputs stay within acceptable boundaries?
- Do MCP contracts remain compatible?
- Does release packaging work across targets?
- Does hot-path behavior survive mutation?

This is the same idea behind Cargo-mutants' `sigs`/`syntax` split: cheap structural confidence early, deeper behavioral confidence later.

### Principle 5: Regressions must be addressable by issue number

Each bug fix should create a named regression test tied to the issue or task.

This mirrors ripgrep:

```text
r<issue-number> or equivalent stable naming convention
```

For WebFang, prefer test names that can be grepped:

```text
issue_1094_...
pr_1249_...
crawler_1070_...
waf_1104_...
```

### Principle 6: Heavy jobs need opt-in or path trigger

Never run these by default on every PR unless paths changed:

- real network,
- ONNX/AI inference,
- coverage,
- full feature matrix,
- mutation beyond hotpaths,
- sanitizers,
- fuzz build/checks,
- cross-platform release build,
- long benchmark runs.

This mirrors agent-browser, wasmtime, ruff, and Deno.

## 4. Borrowed Patterns By Source

### From ripgrep

What we adopt:

- Small required CI surface.
- Regression tests named by issue.
- PR discipline driven by maintainer judgment, not CI volume.
- Stable test commands documented in the repo.
- Minimal permissions and clear workflow ownership.

What we do not copy:

- One-maintainer-only backlog tolerance. WebFang already has too much velocity to rely purely on manual serialization.

### From tokio

What we adopt:

- `basics` or lightweight preflight gating expensive work.
- Separation between normal tests and risk lanes like loom, stress, unsafe, semver.
- Labels that carry workflow meaning:
  - area,
  - risk,
  - state,
  - review status.

What we do not copy:

- Multi-maintainer org routing.
- Full feature/cross-platform/external downstream testing in every PR flow.

### From ruff

What we adopt:

- Path-aware test execution.
- Nextest-style profiles for CI.
- Snapshot discipline with cleanup tooling.
- Fast PR feedback for large test suites.

What we do not copy:

- LSP or JavaScript test infrastructure assumptions.

### From cargo-mutants

What we adopt:

- Tiered gates:
  - structural/type checks,
  - focused behavioral confidence,
  - deeper mutation later.
- Hot-path mutation as high confidence, not universal PR blocking ceremony.
- Clear generated artifacts with provenance.

What we do not copy:

- Blocking every PR on broad mutation before the fast gate is healthy.

### From typst

What we adopt:

- Snapshot corpus discipline.
- Small PRs.
- Fast deterministic tests first.
- Clean human review workflow.

What we do not copy:

- Large custom runner fleet requirements.

### From rust-analyzer

What we adopt:

- Aggregate CI status.
- Aggregator scripts to avoid 20 noisy required contexts.
- Code generation or metadata checks when relevant.
- Fail fast on basic contract checks.

What we do not copy:

- Very large monorepo CI surface without a team.

### From bevy

What we adopt:

- Issue/label taxonomy:
  - regression,
  - S-blocked,
  - A-area,
  - C-category,
  - D-design,
  - E-effort.
- PR review and CI expectation discipline.

What we do not copy:

- Massive label set at this stage.

### From Scrapling

What we adopt:

- Aggressive `paths-ignore` for docs-only changes.
- Test lane separated from code-quality lane.
- Concurrency cancels obsolete branch runs.
- Release PR pattern for versioned publish.
- Single-maintainer-friendly pipeline style.

What we do not copy:

- macOS-only required matrix without local platform checks.

### From agent-browser

What we adopt:

- PR runs fast deterministic tests.
- Heavy native/browser tests run outside PR flow.
- Cross-platform and release artifacts are main/tag jobs.
- Version synchronization checks for release correctness.
- Explicit ignored test category for expensive e2e behavior.

What we do not copy:

- Signed branch/tag requirement while the merge system is still unstable.

### From Firecrawl

What we adopt:

- Component-path workflows:
  - API tests only when API paths change,
  - SDK tests only when SDK paths change,
  - deploy only when deployment surfaces change.
- Local service/harness for tests where practical.
- Cache native and browser dependencies.
- Separate audits and releases from code correctness gates.

What we do not copy:

- Enterprise monorepo sprawl and multi-SDK fanout.

### From browser-use

What we adopt:

- Test shards by module/file for parallelism.
- Issue templates requiring reproduction and debug evidence.
- Stale automation for issues and PRs.
- Separate evaluation lane for agent behavior.
- Strong caching of browser/runtime dependencies.

What we do not copy:

- Relying on test parallelism while preserving expensive required PR checks.

## 5. Target CI Tiers

### Tier 0: Static metadata

Trigger: every PR.
Required: yes.
Budget: < 2 minutes.

Content:

- issue link validation,
- label validation,
- PR title validation,
- actionlint/shellcheck,
- repo architecture guards,
- generated diff checks,
- changed-file classifier.

Outputs:

- `change_type`
- `affected_areas`
- `needs_core`
- `needs_downloader`
- `needs_crawler`
- `needs_ai`
- `needs_mcp`
- `needs_db`
- `needs_release`
- `needs_mutation_hotpath`

### Tier 1: PR fast correctness gate

Trigger: every ready PR.
Required: yes.
Budget: target < 8 minutes.

Content:

- `cargo fmt --check`
- focused `cargo clippy`
- `cargo check --all-targets`
- unit tests
- mock integration tests
- behavioral snapshot tests for affected area
- MCP contract tests if MCP paths changed
- downloader guard-chain checks if downloader paths changed
- docs-only PRs run static metadata plus docs only

Required contexts:

```text
Validate PR metadata
WebFang Fast Gate
```

Optional temporary required context:

```text
cargo-mutants PR diff
```

This should be advisory before Tier 2.

### Tier 2: Main integration evidence

Trigger: push to `main`.
Required: none directly unless branch protection is moved to required checks on main queue later.
Budget: target 20-25 minutes.

Content:

- full feature matrix
- coverage
- AI integration
- sanitizers when relevant
- longer snapshot matrix
- release smoke check for default feature set

Purpose: detect PR interactions that the fast gate cannot catch.

### Tier 3: Release confidence

Trigger: release tag or release PR.
Budget: allowed to be slower.

Content:

- cross-platform build matrix
- package smoke tests
- binary size checks
- CLI help/snapshot checks
- release notes validation
- install smoke tests

### Tier 4: Deep assurance

Trigger: nightly, weekly, manual label, hot-path change, security labels.
Purpose: catch latent defects after the fast/medium tiers passed.

Content:

- mutation full or deep
- fuzz build/checks
- long network simulator scenarios
- memory/stress profiles
- benchmark regression reports
- dependency/security audit jobs

## 6. Required Checks Target

Short term:

```text
Validate PR metadata
WebFang Fast Gate
```

Mid term, if risk appetite allows:

```text
Validate PR metadata
WebFang Fast Gate
cargo-mutants PR diff for hot paths only
```

Avoid as direct required checks after the fast gate is stable:

```text
Tests (AI integration)
Coverage
Feature matrix
Cross-platform builds
Miri
Sanitizers
Full mutants
```

## 7. Test Taxonomy

WebFang should classify each test in stable metadata:

```text
static
unit
integration_mock
integration_db
network_sim
ai_model_semantic
mcp_contract
mutation_sensitive
release_smoke
long_stability
```

Where possible, tags should live in:

- Rust attributes,
- helper naming,
- test directory layout,
- manifest metadata,
- or a generated inventory report.

The CI classifier should read changed files and select a job set from this taxonomy.

### Required bug-detection classes

Every subsystem should answer "how would this fail?" at three layers:

1. Contract/unit tests for invariants.
2. Mock integration tests for guard chains.
3. End-to-end or model tests for acceptance behavior.

For example, downloader changes should be tested by:

- URL validation unit tests,
- guard chain order integration tests,
- WAF/redirect/cookie mock behavioral tests,
- AI semantic cleaning only if AI paths are affected.

## 8. PR and Issue Workflow

### Branch rules

Allowed branch prefixes:

```text
feat/
fix/
refactor/
chore/
ci/
docs/
release/
```

Recommended branch names:

```text
fix/downloader-ssrf-1243
feat/cli-mcp-serve-1260
chore/ci-fast-gate-1265
```

### Issue labels

Adopt a practical subset of Bevy-style labels:

Categories:

```text
C-bug
C-enhancement
C-feature-request
```

Areas:

```text
A-fetch
A-guard_chain
A-crawler
A-sitemap
A-cli
A-mcp
A-ai
A-export
A-testing
A-security
A-ci
```

Workflow:

```text
S-waiting-on-author
S-waiting-on-review
S-blocked
S-regression
S-needs-reproduction
S-duplicate
```

Risk:

```text
Risk-low
Risk-medium
Risk-high
```

Release:

```text
release-blocker
backport-patch
```

No need to import all Bevy labels. Keep only what changes behavior.

### PR policy

Every PR must include:

```text
Closes #N
Fixes #N
Resolves #N
```

or explicitly link to a parent umbrella issue without closing it.

Every PR must include:

- intent,
- changed behavior,
- tests run,
- CI tier impact,
- rollback story,
- risk label.

PRs requiring docs-only change should not run full Rust compile/test unless lockfiles or generated docs depend on it.

### Stale policy

Adopt browser-use-like stale policy with clear exemptions.

Initial proposal:

- issues stale after 60 days,
- PRs stale after 45 days,
- close stale after 14 days with no activity,
- exempt:
  - security,
  - regression,
  - release-blocker,
  - good-first-issue,
  - pinned,
  - dependencies.

### AI contribution policy

Because WebFang is used with agents and may receive AI-generated contributions, adopt Scrapling-style disclosure:

```text
If AI assistance was used, disclose the extent in the PR or issue.
```

Also adopt deno/browser-use-style guard:

- no undisclosed AI spam,
- trivial AI typo fixes do not need long disclosure,
- AI-authored PRs with no reproduction or human understanding are closed.

## 9. Worktree and Agent Workflow

The existing sibling worktree model is good and should be extended.

### Worktree rules

- One task, one worktree.
- Do not switch branches in a worktree.
- Never use `git stash`; shared stash is dangerous here.
- Every worktree gets:
  - `.envrc` from main,
  - `direnv allow`,
  - `.env` when needed,
  - CodeGraph init,
  - CodeDB reindex.

### Local pre-push gate

Before pushing or opening a PR, locally run the equivalent of PR Tier 1.

This mirrors the Hacker News local merge queue pattern and Deno-like developer loops.

Expected WebFang command:

```bash
./scripts/ci-fast-gate.sh
```

Proposed behavior:

- fmt check,
- repo guards,
- affected crate check/clippy,
- affected tests,
- mock integration tests,
- no AI/ONNX by default,
- no coverage by default,
- no release build default.

### PR batching without native queue

Until native queue works, use explicit manual batches:

1. Select independent PRs:
   - docs + code,
   - different crates,
   - different tests,
   - different release behavior.
2. Confirm disjoint paths.
3. Create a temporary integration branch.
4. Merge remote PR head SHAs into it.
5. Run one fast gate locally.
6. If green, merge sequentially using an automation script or one batch PR if repo policy allows.

WebFang already has docs recommending batch-merge of disjoint files. Keep that pattern; do not replace fast CI with batch hacks.

## 10. Target Workflow for a Normal Change

Example: a downloader SSRF fix.

1. Create worktree.
2. Open PR as draft if implementation is still in progress.
3. Draft or static CI runs only cheap checks.
4. When ready:
   - run local fast gate,
   - mark PR ready,
   - PR CI runs Tier 0 + Tier 1.
5. After fast gate green, request review.
6. Maintainer merges or queues batch.
7. Main Tier 2 runs and catches interactions.
8. Nightly Tier 4 catches deep risk.

Expected result:

```text
PR developer wait: 2-8 min
main confidence wait: 20-25 min
deep assurance: async
```

## 11. Implementation Backlog

### Phase 1: Stop the bleeding

Goal: reduce PR wall-clock and prevent docs-only churn.

Tasks:

1. Keep `CI Gate` off AI tests.
2. Update repo docs and comments to make `CI Gate` the aggregate contract.
3. Add `WebFang Fast Gate` name or rename `CI Gate` comments if needed.
4. Remove `Tests (AI integration)` from required checks if not already done.
5. Make `cargo-mutants (PR diff)` advisory until Tier 1 is stable.
6. Add path-ignore for docs-only and metadata-only workflows.
7. Add changed-file classifier script.

Acceptance:

- docs-only PR wall-clock below normal test suite.
- required checks list no longer includes `Tests (AI integration)`.

### Phase 2: Path-aware CI

Goal: run the right tests for the right changes.

Tasks:

1. Create classifier job.
2. Add affected-area outputs:
   - core,
   - downloader,
   - crawler,
   - ai,
   - mcp,
   - docs,
   - ci.
3. Gate jobs on outputs instead of running everything on every PR.
4. Add always-present wrapper for required checks so path-skipped jobs do not block forever.
5. Add snapshot path detection.
6. Add AI path detection for ONNX/AI tests.

Acceptance:

- docs PR skips compile-heavy jobs.
- MCP PR runs MCP contract jobs.
- AI PR runs AI lane; non-AI PR does not.

### Phase 3: Test inventory and budgets

Goal: know what tests exist and what each protects.

Tasks:

1. Generate JSON/text test inventory.
2. Add metadata:
   - area,
   - tier,
   - required_on,
   - tags,
   - runtime expectation.
3. Create test-budget report in PR:
   - count by tier,
   - changed files,
   - estimated CI job set.
4. Add dead-test detection.
5. Add duplicate-test detection as advisory.
6. Enforce no orphan snapshots.

Acceptance:

- PR shows changed test classes and affected CI lanes.
- nightly detects dead tests or duplicate coverage hotspots.

### Phase 4: Mutation and quality gates

Goal: make mutation testing a risk engine, not a merge tax.

Tasks:

1. Keep hot-path mutation on PR only if changed area demands it.
2. Add area-based mutation scope.
3. Add deep mutation on nightly and release.
4. Add quality budgets with clear baseline and failure policy.
5. Add regression test naming convention.
6. Enforce linked issue in test names or test metadata report.

Acceptance:

- mutation fails PR only for risk-bearing changed functions.
- main/nightly catches broader drift.

### Phase 5: Merge batching without native queue

Goal: handle multiple ready PRs without rebase hell.

Tasks:

1. Add script to compute PR file overlap.
2. Add local integration branch helper.
3. Add merge queue documentation.
4. Add stale automation.
5. Add simple `ci-status` aggregator if needed.

Acceptance:

- two docs PRs can be processed without triggering code CI for both after one merge.
- one docs + one code PR can be batched safely when disjoint.

### Phase 6: Observability and SLOs

Goal: prove the workflow is actually faster and safer.

Metrics:

- median PR fast gate wall-clock.
- p90 PR fast gate wall-clock.
- required check list.
- average time from ready PR to merge.
- number of required checks.
- PR CI job minutes.
- main tier run wall-clock.
- nightly failure count.
- mutation score by area.
- number of PRs marked behind due strict mode.

SLO targets:

```text
Tier 0 p90:     2 min
Tier 1 p90:     10 min
Tier 2 p90:     30 min
Docs-only p90:  3 min
AI path p90:    accepted separately
```

## 12. Non-Negotiable Guardrails

These guardrails should stay strict:

```bash
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings -W clippy::cognitive_complexity -W clippy::too_many_lines
cargo fmt
```

And:

```bash
cargo nextest run
```

For behavior changes:

- add regression test,
- name it with issue/task number,
- update snapshot or explain why no snapshot needed.

For fetch/security changes:

- guard-chain order must assert through mock integration tests,
- SSRF/redirect behavior must not be inferred only from comments.

For release changes:

- version sync and generated docs must pass before publish.
- cross-platform binaries run only in release tier.

## 13. Decision Summary

The optimized WebFang system should look like this:

```text
PR:  small, fast, relevant
main:  full and rigorous
nightly/release:  deep assurance
issues:  traceable
tests:  risk controls
snapshots:  governed
mutation:  focused by risk
agents/worktrees:  isolated and disciplined
```

The key change is conceptual:

```text
from: every PR must pay for every signal
to:   every PR pays for its blast radius, and main pays for the whole map
```
