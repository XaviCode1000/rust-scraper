# Contributing to Rust Scraper

**Last Updated:** March 11, 2026  
**Version:** 1.0.5  
**Status:** ✅ 281 tests passing | ✅ CI/CD enabled | ✅ Production-ready

Thank you for considering contributing to rust-scraper! This document provides **VERIFIED** guidelines based on how the project ACTUALLY works.

---

## 📋 Table of Contents

- [Getting Started](#-getting-started)
- [Project Structure](#-project-structure)
- [Development Workflow](#-development-workflow)
- [Git Workflow](#-git-workflow)
- [Testing Guidelines](#-testing-guidelines)
- [Code Style](#-code-style)
- [Documentation](#-documentation)
- [Submitting Changes](#-submitting-changes)
- [Issue Reporting](#-issue-reporting)
- [Volunteer Opportunities](#-volunteer-opportunities)

---

## 🚀 Getting Started

### Prerequisites

| Requirement | Version | How to Check |
|-------------|---------|--------------|
| **Rust** | 1.80+ (MSRV) | `rustc --version` (current: 1.93.0) |
| **Cargo** | 1.80+ | `cargo --version` (current: 1.93.0) |
| **OS** | Linux, macOS, Windows | Tested on CachyOS Linux |
| **Optional (AI features)** | CMake, C++17 | Required for `tract-onnx` |

### Setup Steps

```bash
# 1. Fork the repository
# Visit https://github.com/XaviCode1000/rust-scraper and click "Fork"

# 2. Clone your fork
git clone https://github.com/YOUR_USERNAME/rust-scraper.git
cd rust-scraper

# 3. Fetch dependencies
cargo fetch

# 4. Build in debug mode
cargo build

# 5. Run tests (281 total)
cargo test

# 6. Verify formatting
cargo fmt --all -- --check

# 7. Run Clippy
cargo clippy -- -D clippy::correctness -D clippy::suspicious
```

### Hardware-Aware Setup (CachyOS)

```fish
# For HDD systems (500GB, 8GB RAM, 4C/4T CPU)
# Limit test threads to avoid OOM
cargo test --test-threads=2

# Use ionice for I/O-heavy operations
ionice -c 3 cargo build

# Limit parallel jobs
make -j (math (nproc) - 1)  # ~3 threads max
```

---

## 🏗️ Project Structure

### Clean Architecture Layers (VERIFIED)

```
rust_scraper/
├── src/
│   ├── lib.rs                  # Library root, public API re-exports
│   ├── main.rs                 # CLI entry point (TUI + headless)
│   ├── config.rs               # Logging configuration
│   ├── error.rs                # ScraperError enum (thiserror)
│   ├── url_path.rs             # URL path handling
│   ├── user_agent.rs           # User-Agent rotation
│   ├── export_factory.rs       # Export format factory
│   ├── export_utils.rs         # Export utilities
│   │
│   ├── domain/                 # Domain layer (PURE, no dependencies)
│   │   ├── mod.rs
│   │   ├── entities.rs         # ScrapedContent, DownloadedAsset
│   │   └── value_objects.rs    # ValidUrl (newtype for type safety)
│   │
│   ├── application/            # Application layer (use cases)
│   │   ├── mod.rs
│   │   ├── http_client.rs      # HTTP client with retry logic
│   │   └── scraper_service.rs  # Scraping orchestration
│   │
│   ├── infrastructure/         # Infrastructure layer (technical)
│   │   ├── mod.rs
│   │   ├── http/               # HTTP implementations
│   │   ├── scraper/            # Readability, fallback, asset download
│   │   ├── converter/          # HTML→Markdown, syntax highlighting
│   │   ├── output/             # File saver, frontmatter
│   │   └── ai/                 # AI semantic cleaning (ONNX)
│   │
│   └── adapters/               # Adapters layer (external)
│       ├── mod.rs
│       ├── detector/           # MIME type detection
│       ├── extractor/          # URL extraction
│       └── downloader/         # Asset downloading
│
├── tests/                      # Integration tests
│   ├── integration.rs          # Main integration test suite
│   └── ai_integration.rs       # AI feature tests (requires --features ai)
│
├── docs/                       # Documentation
│   ├── ARCHITECTURE.md         # Clean Architecture design
│   ├── AI-SEMANTIC-CLEANING.md # AI features (v1.0.5+)
│   ├── RAG-EXPORT.md           # JSONL export for RAG pipelines
│   ├── USAGE.md                # Usage examples
│   ├── CLI.md                  # CLI reference
│   └── CONTRIBUTING.md         # This file
│
├── .github/workflows/          # CI/CD pipelines
│   ├── ci.yml                  # Main CI (build, test, clippy, fmt)
│   ├── release.yml             # Release automation
│   └── opencode.yml            # Opencode CI
│
├── rust-skills/                # 179 Rust best practices
│   └── rules/                  # Categorized rules (own-*, err-*, etc.)
│
└── Cargo.toml                  # Dependencies and features
```

### Layer Statistics (VERIFIED)

| Layer | Files | LOC | Dependencies |
|-------|-------|-----|--------------|
| **Domain** | 2 | ~200 | None (pure Rust) |
| **Application** | 2 | ~300 | Domain |
| **Infrastructure** | 10+ | ~1,500 | Domain, Application, external crates |
| **Adapters** | 5+ | ~800 | All layers |
| **Total (src/)** | 20+ | 3,754 | 45+ (core), 60+ (with AI) |

### Feature Flags

| Feature | Description | Tests |
|---------|-------------|-------|
| `images` | Enable image downloading | Included in `full` |
| `documents` | Enable document downloading | Included in `full` |
| `full` | All features except AI | 217 library tests |
| `ai` | AI-powered semantic cleaning | 64 AI integration tests |

---

## 🔧 Development Workflow

### Daily Development Commands

```bash
# Build (debug mode, fast)
cargo build

# Build (release mode, optimized)
cargo build --release

# Run (debug mode)
cargo run -- --url https://example.com

# Run (release mode, faster)
cargo run --release -- --url https://example.com

# Run with AI features
cargo run --release --features ai -- --url https://example.com --clean-ai
```

### Testing Commands (VERIFIED)

```bash
# Run ALL tests (281 total)
cargo test

# Run library tests only (217 tests)
cargo test --lib

# Run with output (see println!)
cargo test -- --nocapture

# Run specific test
cargo test test_validate_and_parse_url

# Run AI integration tests (64 tests, requires --features ai)
cargo test --features ai --test ai_integration -- --test-threads=2

# Run doctests
cargo test --doc

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### Test Results (VERIFIED — March 11, 2026)

| Test Suite | Count | Status | Notes |
|------------|-------|--------|-------|
| **Library Tests** | 217 | ✅ Passing | `cargo test --lib` |
| **Binary Tests** | 0 | ✅ Passing | `cargo test --bin` |
| **Integration Tests** | 8 | ✅ Passing | `cargo test --test integration` |
| **AI Integration Tests** | 64 | ✅ Passing | `--features ai --test-threads=2` |
| **Doctests** | 56 | ✅ Passing | `cargo test --doc` |
| **TOTAL** | **281** | ✅ **All Passing** | 4 ignored |

### Linting Commands

```bash
# Run Clippy (CI configuration)
cargo clippy -- -D clippy::correctness -D clippy::suspicious -D clippy::unused -W clippy::perf -W clippy::style

# Run Clippy (strict, deny all warnings)
cargo clippy -- -D warnings

# Run Clippy (all targets, all features)
cargo clippy --all-targets --all-features -- -D warnings

# Check formatting (CI check)
cargo fmt --all -- --check

# Format code (required before commit)
cargo fmt
```

### CI Workflow (VERIFIED — `.github/workflows/ci.yml`)

```yaml
# CI runs on: push to main, pull requests to main
# Jobs (sequential):
1. build        → cargo build --verbose, cargo build --release --verbose
2. test         → cargo test --verbose, cargo test --release --verbose
3. clippy       → cargo clippy -- -D correctness -D suspicious -D unused -W perf -W style
4. fmt          → cargo fmt --all -- --check
5. checks       → Aggregates all jobs (must pass all)
```

**CI Failure Handling:**
- Error logs exported to `.github/outputs/error_log.md`
- Logs uploaded as artifacts for debugging
- Failure context includes last 50 lines of output

---

## 📝 Git Workflow

### Branch Naming (VERIFIED from existing branches)

| Branch Type | Format | Example |
|-------------|--------|---------|
| **Feature** | `feature/<description>` | `feature/ai-semantic-cleaning-issue9` |
| **Bug Fix** | `fix/<description>` | `fix/embeddings-preservation` |
| **Release** | `release/<version>` | `release/1.0.5` |
| **Hotfix** | `hotfix/<description>` | `hotfix/ci-blocker` |

**Existing Branches:**
- `main` (default, protected)
- `feature/ai-semantic-cleaning-issue9` (current development)
- `feature/rag-export-pipeline` (remote)

### Commit Message Format (VERIFIED from `git log`)

This project uses [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types (from actual commits):**

| Type | Description | Example |
|------|-------------|---------|
| `feat` | New feature | `feat(ai): Module 5 - Full RAG Pipeline Integration` |
| `fix` | Bug fix | `fix(ai): Preserve embeddings during semantic filtering` |
| `docs` | Documentation | `docs: Add AI Semantic Cleaning documentation` |
| `chore` | Maintenance | `chore: Remove test_ai.rs example (CI blocker)` |
| `ci` | CI/CD changes | `ci: Trigger CI rebuild` |
| `refactor` | Code restructuring | `refactor: extract HTML conversion to separate module` |
| `test` | Adding tests | `test: add integration tests for asset download` |
| `style` | Formatting | `style: apply rustfmt` |
| `perf` | Performance | `perf: eliminate unnecessary cloning` |

**Scopes (from actual commits):**
- `ai` — AI/ML features
- `example` — Example code
- `release` — Release process
- General (no scope) — Cross-cutting changes

### Commit Examples (REAL from `git log --format="%s" | head -15`)

```bash
# Feature with scope
git commit -m "feat(ai): Module 5 - Full RAG Pipeline Integration (Issue #9 COMPLETE)"

# Bug fix with scope
git commit -m "fix(ai): Preserve embeddings + fix test isolation (Issue #9)"

# Chore with scope
git commit -m "chore: Remove test_ai.rs example (CI blocker)"

# CI change
git commit -m "ci: Trigger CI rebuild"

# Documentation
git commit -m "docs: Add AI Semantic Cleaning documentation (Issue #9)"

# Fix without scope
git commit -m "fix: Apply rustfmt formatting to AI module files"

# Fix with release scope
git commit -m "fix(release): add check-token job to verify secret existence"
```

### Breaking Changes

For breaking changes, add `BREAKING CHANGE:` in commit body:

```bash
git commit -m "refactor: migrate to Clean Architecture

Major refactoring from monolithic structure to layered architecture.

BREAKING CHANGE: Migrated from anyhow::Result to ScraperError::Result
"
```

### Pull Request Process (VERIFIED)

**Current Open PRs:**
- #11: `feat: AI-Powered Semantic Content Extraction with Embedding Preservation Fix (Issue #9)`
  - Branch: `feature/ai-semantic-cleaning-issue9`
  - Stats: +7,189 additions, -51 deletions
  - Status: Open (created ~15 hours ago)

**Steps:**

1. **Fork** the repository
2. **Create a feature branch** from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```
3. **Make changes** with tests
4. **Ensure tests pass**:
   ```bash
   cargo test --all-features  # Or specific features
   cargo clippy -- -D clippy::correctness -D clippy::suspicious
   cargo fmt
   ```
5. **Commit** with conventional commits
6. **Push** to your fork:
   ```bash
   git push -u origin feature/your-feature-name
   ```
7. **Create PR** with:
   - Clear title (conventional commit format)
   - Description of changes
   - Link to related issue (e.g., "Closes #9")
   - Test evidence (e.g., "281 tests passing")

---

## 🧪 Testing Guidelines

### Test Categories

| Category | Location | Command | Count |
|----------|----------|---------|-------|
| **Unit Tests** | In-source (`#[cfg(test)] mod tests`) | `cargo test --lib` | 217 |
| **Integration Tests** | `tests/integration.rs` | `cargo test --test integration` | 8 |
| **AI Integration** | `tests/ai_integration.rs` | `cargo test --features ai --test ai_integration` | 64 |
| **Doctests** | In documentation (`/// # Examples`) | `cargo test --doc` | 56 |

### Writing Unit Tests

Unit tests go **in the same file** as the code:

```rust
/// Function documentation
///
/// # Examples
///
/// ```
/// use rust_scraper::example;
///
/// let result = example();
/// assert!(result.is_ok());
/// ```
pub fn example() -> Result<(), ScraperError> {
    // Implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_success() {
        // Arrange
        let input = "valid_input";

        // Act
        let result = example(input);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_example_invalid_input() {
        // Arrange
        let input = "";

        // Act
        let result = example(input);

        // Assert
        assert!(result.is_err());
    }
}
```

### Writing Integration Tests

Integration tests go in `tests/` directory:

```rust
// tests/integration.rs

use rust_scraper::{ScraperConfig, ScraperService};

#[tokio::test]
async fn test_full_scraping_pipeline() {
    // Arrange
    let config = ScraperConfig::builder()
        .url("https://example.com")
        .build()
        .unwrap();

    // Act
    let result = ScraperService::scrape(config).await;

    // Assert
    assert!(result.is_ok());
    let content = result.unwrap();
    assert!(!content.is_empty());
}
```

### Writing Doctests

Doctests go in **documentation comments**:

```rust
/// Validates and parses a URL.
///
/// # Examples
///
/// ```
/// use rust_scraper::url_path::validate_and_parse_url;
///
/// let url = validate_and_parse_url("https://example.com")
///     .expect("Valid URL");
/// assert_eq!(url.domain(), "example.com");
/// ```
///
/// # Errors
///
/// Returns `ScraperError::InvalidUrl` if the URL is malformed.
pub fn validate_and_parse_url(url_str: &str) -> Result<Url, ScraperError> {
    // Implementation
}
```

### Test Best Practices (rust-skills applied)

| Rule | Description | Example |
|------|-------------|---------|
| `test-unit-isolation` | Unit tests must be isolated | No shared state between tests |
| `test-async-with-tokio` | Use `#[tokio::test]` for async | `#[tokio::test] async fn test_async() { }` |
| `test-proptest-properties` | Use proptest for property-based | `proptest! { #[test] fn prop_test(x: i32) { } }` |
| `test-mockall-traits` | Use mockall for trait mocking | `mock! { pub HttpClient { fn get(&self) -> Result<String>; } }` |
| `test-criterion-bench` | Use criterion for benchmarks | `criterion_group!(benches, bench_function);` |

### Test Commands Reference

```bash
# Run all tests
cargo test

# Run with output (see println!)
cargo test -- --nocapture

# Run specific test by name
cargo test test_validate_and_parse_url

# Run tests matching a pattern
cargo test -- test_url

# Run only library tests
cargo test --lib

# Run only integration tests
cargo test --test integration

# Run AI tests (requires --features ai)
cargo test --features ai --test ai_integration -- --test-threads=2

# Run doctests only
cargo test --doc

# Run with coverage
cargo tarpaulin --out Html

# Run in release mode (faster for heavy tests)
cargo test --release

# Limit test threads (HDD optimization)
cargo test --test-threads=2
```

---

## 📏 Code Style

### Formatting (rustfmt)

**Required before every commit:**

```bash
# Format all code
cargo fmt

# Check formatting (CI check)
cargo fmt --all -- --check
```

**Configuration:** Uses default rustfmt settings (Rust 2021 edition).

### Linting (Clippy)

**CI Configuration:**

```bash
cargo clippy -- -D clippy::correctness -D clippy::suspicious -D clippy::unused -W clippy::perf -W clippy::style
```

**Categories:**

| Category | Level | Description |
|----------|-------|-------------|
| `correctness` | **DENY** | Bugs, logic errors |
| `suspicious` | **DENY** | Suspicious code patterns |
| `unused` | **DENY** | Unused code |
| `perf` | WARN | Performance issues |
| `style` | WARN | Style issues |

**Local Development:**

```bash
# Strict (deny all warnings)
cargo clippy -- -D warnings

# CI configuration
cargo clippy -- -D clippy::correctness -D clippy::suspicious
```

### rust-skills Compliance (179 Rules)

This project follows the [rust-skills](https://github.com/leonardomso/rust-skills) repository (179 rules).

**Categories:**

| Category | Rules | Priority | Examples |
|----------|-------|----------|----------|
| **Ownership & Borrowing** | 12 | CRITICAL | `own-borrow-over-clone`, `own-slice-over-vec` |
| **Error Handling** | 12 | CRITICAL | `err-thiserror-lib`, `err-no-unwrap-prod` |
| **Memory Optimization** | 15 | CRITICAL | `mem-with-capacity`, `mem-smallvec` |
| **API Design** | 15 | HIGH | `api-builder-pattern`, `api-must-use` |
| **Async/Await** | 15 | HIGH | `async-no-lock-await`, `async-spawn-blocking` |
| **Compiler Optimization** | 12 | HIGH | `opt-lto-release`, `opt-inline` |
| **Naming Conventions** | 16 | MEDIUM | `name-camel-case`, `name-snake-case` |
| **Type Safety** | 10 | MEDIUM | `type-newtype-ids`, `type-option-result` |
| **Testing** | 13 | MEDIUM | `test-proptest-properties`, `test-tokio-async` |
| **Documentation** | 11 | MEDIUM | `doc-all-public`, `doc-examples-section` |
| **Performance Patterns** | 11 | MEDIUM | `perf-iterators`, `perf-entry-api` |
| **Project Structure** | 11 | LOW | `proj-lib-minimal`, `proj-feature-modules` |
| **Clippy & Linting** | 11 | LOW | `lint-deny-correctness`, `lint-warn-perf` |
| **Anti-patterns** | 15 | LOW | `anti-unwrap-abuse`, `anti-lock-across-await` |

**NEVER (violations will be rejected):**

| Anti-pattern | Rule | Example |
|--------------|------|---------|
| ❌ `.unwrap()` in production | `anti-unwrap-abuse` | Use `?` or `expect("reason")` |
| ❌ Lock across `.await` | `anti-lock-across-await` | Clone data, release lock first |
| ❌ `&Vec<T>` when `&[T]` works | `own-slice-over-vec` | Accept `&[T]` not `&Vec<T>` |
| ❌ `format!()` in hot paths | `anti-format-hot-path` | Use `write!()` or pre-allocate |
| ❌ Unnecessary clones | `anti-clone-excessive` | Borrow with `&T` |
| ❌ `expect()` without reason | `anti-expect-lazy` | `expect("database connection failed")` |

### Code Review Checklist

Before submitting a PR, ensure:

- [ ] Code is formatted with `cargo fmt`
- [ ] No clippy warnings (`cargo clippy -- -D clippy::correctness -D clippy::suspicious`)
- [ ] All tests pass (`cargo test`)
- [ ] New code has tests (unit, integration, or doctest)
- [ ] No `.unwrap()` in production code (use `?` or `expect("reason")`)
- [ ] Error types are well-defined (`thiserror` for libraries)
- [ ] Documentation is updated (/// comments, # Examples, # Errors)
- [ ] rust-skills rules are followed (179 rules)
- [ ] No locks held across `.await` points
- [ ] Ownership model is clear (borrow vs clone)

---

## 📖 Documentation

### Where to Document

| Location | Purpose |
|----------|---------|
| `docs/` | High-level documentation (architecture, features) |
| `README.md` | Project overview, quick start |
| `src/*.rs` | API documentation (/// comments) |
| `CHANGELOG.md` | Version history |
| `docs/CHANGES.md` | Detailed changelog |

### Documentation Standards (rust-skills)

| Rule | Description | Example |
|------|-------------|---------|
| `doc-all-public` | Document all public items | `/// Validates a URL` |
| `doc-examples-section` | Include `# Examples` | `/// # Examples` with runnable code |
| `doc-errors-section` | Include `# Errors` for fallible functions | `/// # Errors` |
| `doc-panics-section` | Include `# Panics` for panicking functions | `/// # Panics` |
| `doc-safety-section` | Include `# Safety` for unsafe functions | `/// # Safety` |
| `doc-intra-doc-links` | Use intra-doc links | `[Vec]`, `[crate::MyType]` |

### Documentation Example

```rust
/// Validates and parses a URL string.
///
/// This function ensures the URL is well-formed according to RFC 3986
/// and extracts the domain for SSRF prevention.
///
/// # Arguments
///
/// * `url_str` - The URL string to validate and parse
///
/// # Returns
///
/// * `Ok(Url)` - A validated URL object
/// * `Err(ScraperError::InvalidUrl)` - If the URL is malformed
///
/// # Examples
///
/// ```
/// use rust_scraper::url_path::validate_and_parse_url;
///
/// let url = validate_and_parse_url("https://example.com/path")
///     .expect("Valid URL");
/// assert_eq!(url.domain(), "example.com");
/// ```
///
/// # Errors
///
/// Returns `ScraperError::InvalidUrl` if:
/// - The URL scheme is missing or invalid
/// - The domain is empty
/// - The URL contains invalid characters
///
/// # SSRF Prevention
///
/// This function validates the URL but does NOT check for SSRF.
/// Use `validate_url_host()` for SSRF prevention.
pub fn validate_and_parse_url(url_str: &str) -> Result<Url, ScraperError> {
    // Implementation
}
```

### Building Documentation

```bash
# Generate HTML documentation
cargo doc

# Open documentation in browser
cargo doc --open

# Include private items
cargo doc --document-private-items

# No dependencies (faster)
cargo doc --no-deps
```

---

## 📤 Submitting Changes

### PR Checklist

Before submitting a PR, verify:

```bash
# 1. All tests pass
cargo test

# 2. Clippy passes (CI configuration)
cargo clippy -- -D clippy::correctness -D clippy::suspicious -D clippy::unused -W clippy::perf -W clippy::style

# 3. Formatting is correct
cargo fmt --all -- --check

# 4. Build succeeds (debug and release)
cargo build
cargo build --release

# 5. AI tests pass (if applicable)
cargo test --features ai --test ai_integration -- --test-threads=2
```

**PR Template:**

```markdown
## Description
[Clear description of changes]

## Type of Change
- [ ] Bug fix (non-breaking change)
- [ ] New feature (non-breaking change)
- [ ] Breaking change (fix or feature requiring API changes)
- [ ] Documentation update

## Related Issue
Closes #<issue-number>

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] All existing tests pass (281 total)
- [ ] Clippy passes
- [ ] Formatting correct

## Test Evidence
```
cargo test --lib 2>&1 | grep "test result"
# test result: ok. 217 passed; 0 failed
```

## Checklist
- [ ] Code follows rust-skills guidelines (179 rules)
- [ ] No `.unwrap()` in production code
- [ ] No locks across `.await`
- [ ] Documentation updated (/// comments, # Examples)
- [ ] Commit message follows Conventional Commits
```

### CI Requirements (VERIFIED)

All PRs must pass:

| Job | Command | Status |
|-----|---------|--------|
| **build** | `cargo build --verbose`, `cargo build --release --verbose` | Required |
| **test** | `cargo test --verbose`, `cargo test --release --verbose` | Required |
| **clippy** | `cargo clippy -- -D correctness -D suspicious -D unused -W perf -W style` | Required |
| **fmt** | `cargo fmt --all -- --check` | Required |

### Review Process

1. **Automated Checks** — CI must pass (all 4 jobs)
2. **Code Review** — Maintainer reviews code for:
   - rust-skills compliance (179 rules)
   - Clean Architecture adherence
   - Test coverage
   - Documentation quality
3. **Approval** — Maintainer approves PR
4. **Merge** — PR merged to `main`

### Current Open Issues/PRs (VERIFIED)

**Pull Requests:**
- #11: `feat: AI-Powered Semantic Content Extraction with Embedding Preservation Fix (Issue #9)`
  - Status: Open
  - Branch: `feature/ai-semantic-cleaning-issue9`

**Issues:**
- #9: `[Feature] AI-Powered Semantic Content Extraction via Local SLM Inference`
  - Status: Open
  - Labels: feature

---

## 🐛 Issue Reporting

### How to Report Bugs

Open an issue at https://github.com/XaviCode1000/rust-scraper/issues with:

**Bug Report Template:**

```markdown
## Description
[Clear description of the bug]

## Steps to Reproduce
1. [First step]
2. [Second step]
3. [and so on...]

## Expected Behavior
[What should happen]

## Actual Behavior
[What actually happens]

## Environment
- Rust version: [e.g., 1.93.0]
- OS: [e.g., CachyOS Linux]
- Version: [e.g., 1.0.5]
- Features: [e.g., ai, full]

## Logs
```
[Error messages or stack traces]
```

## Additional Context
[Any other context, screenshots, etc.]
```

### Issue Labels

| Label | Description |
|-------|-------------|
| `bug` | Something isn't working |
| `feature` | New feature request |
| `documentation` | Documentation improvements |
| `good first issue` | Good for newcomers |
| `help wanted` | Extra attention needed |
| `question` | Further information requested |

### Feature Request Template

```markdown
## Problem
[What problem does this solve?]

## Proposal
[How should it work?]

## Alternatives
[What other solutions exist?]

## Use Cases
[Who will use this and how?]

## Additional Context
[Any other context, screenshots, mockups]
```

---

## 🙌 Volunteer Opportunities

### Current Needs (VERIFIED)

| Area | Description | Priority |
|------|-------------|----------|
| **Unit Tests** | More coverage for infrastructure layer | HIGH |
| **Benchmarks** | Criterion benchmarks for hot paths | HIGH |
| **Documentation** | More `# Examples` in API docs | MEDIUM |
| **CI/CD** | Improve CI workflows, add caching | MEDIUM |
| **Performance** | Optimize for HDD/low-resource systems | MEDIUM |
| **AI Features** | Test with different ONNX models | LOW |

### Planned Features (Roadmap)

| Version | Feature | Status |
|---------|---------|--------|
| **v1.0.5** | AI-powered semantic cleaning | ✅ Complete |
| **v1.0.5** | Embeddings preservation bug fix | ✅ Complete (PR #11) |
| **v1.1.0** | Multi-domain crawling | 🚧 Planned |
| **v1.2.0** | JavaScript rendering (headless browser) | 🚧 Planned |
| **v2.0.0** | Distributed scraping | 🚧 Planned |

### How to Help

1. **Pick an issue** — Check https://github.com/XaviCode1000/rust-scraper/issues
2. **Comment** — "I'd like to work on this"
3. **Create branch** — `git checkout -b fix/issue-<number>`
4. **Implement** — Follow this guide
5. **Submit PR** — Link to issue

### Good First Issues

Look for issues labeled `good first issue`:
- Documentation improvements
- Simple bug fixes
- Test additions
- Clippy warning fixes

---

## 📚 Resources

### Essential Reading

| Resource | Description |
|----------|-------------|
| [Rust Book](https://doc.rust-lang.org/book/) | Official Rust book |
| [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) | Rust API design |
| [Rust Performance Book](https://nnethercote.github.io/perf-book/) | Optimization guide |
| [rust-skills](https://github.com/leonardomso/rust-skills) | 179 Rust best practices |
| [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html) | Architecture pattern |
| [Conventional Commits](https://www.conventionalcommits.org/) | Commit message format |
| [Tokio Documentation](https://tokio.rs/tokio/tutorial) | Async runtime guide |

### Project Documentation

| Document | Description |
|----------|-------------|
| [`README.md`](../README.md) | Project overview |
| [`docs/ARCHITECTURE.md`](ARCHITECTURE.md) | Clean Architecture design |
| [`docs/AI-SEMANTIC-CLEANING.md`](AI-SEMANTIC-CLEANING.md) | AI features (v1.0.5+) |
| [`docs/RAG-EXPORT.md`](RAG-EXPORT.md) | JSONL export for RAG |
| [`docs/USAGE.md`](USAGE.md) | Usage examples |
| [`docs/CLI.md`](CLI.md) | CLI reference |
| [`docs/CHANGES.md`](CHANGES.md) | Changelog |

### rust-skills Catalog

| Category | Rules | Key Rules |
|----------|-------|-----------|
| **Ownership** | 12 | `own-borrow-over-clone`, `own-slice-over-vec`, `own-arc-for-shared` |
| **Error** | 12 | `err-thiserror-lib`, `err-no-unwrap-prod`, `err-question-mark` |
| **Memory** | 15 | `mem-with-capacity`, `mem-smallvec`, `mem-zero-copy` |
| **API** | 15 | `api-builder-pattern`, `api-must-use`, `api-newtype-ids` |
| **Async** | 15 | `async-no-lock-await`, `async-spawn-blocking`, `async-join-parallel` |
| **Optimization** | 12 | `opt-lto-release`, `opt-inline`, `opt-pgo` |
| **Testing** | 13 | `test-proptest-properties`, `test-tokio-async`, `test-criterion-bench` |
| **Documentation** | 11 | `doc-all-public`, `doc-examples-section`, `doc-errors-section` |

**Full catalog:** [`rust-skills/README.md`](../rust-skills/README.md)

---

## 🙏 Thank You

Every contribution, no matter how small, is appreciated!

**Current Status:**
- ✅ 281 tests passing
- ✅ CI/CD enabled
- ✅ Production-ready (v1.0.5)
- ✅ rust-skills compliant (179 rules)

---

**Questions?** Open an issue at https://github.com/XaviCode1000/rust-scraper/issues or reach out to the maintainers.

**Happy Contributing! 🦀**
