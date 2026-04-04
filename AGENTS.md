# Agent Instructions — Rust Scraper

Production-ready web scraper with Clean Architecture, TUI, and AI semantic cleaning.

**Stack:** Rust 1.88+ · Tokio · wreq (TLS fingerprint) · ratatui · tract-onnx (feature-gated)
**Hardware:** Intel i5-4590 (4C), 8GB DDR3, HDD — all commands are HDD-optimized

---

## Key Commands

```bash
cargo check                                          # Verify compilation (FAST — use this)
cargo check --features ai                            # Verify with AI feature
cargo clippy -- -D warnings                          # Lint (quick pass)
cargo clippy --all-targets --all-features -- -D warnings  # Full lint
cargo nextest run --test-threads 2                   # Run tests
cargo nextest run --test-threads 2 --features ai     # Tests with AI
cargo llvm-cov --html --output-dir coverage-llvm     # Coverage
cargo fmt --check                                    # Format check
bacon                                                # Background checker (auto-runs clippy)
```

**⚠️ HDD timeout rules:** First `cargo check` takes ~4 min (cold compile, 300 crates). After that, `sccache` makes everything fast. **ALWAYS set explicit timeouts** for heavy commands. Prefer `cargo check` over `cargo build` during development. Never run `cargo build --release` unless explicitly asked.

---

## Code Style

```rust
// Error handling: thiserror for domain/infra, anyhow for app/binary
#[derive(Error, Debug)]
pub enum ScraperError {
    #[error("URL inválida: {0}")]
    InvalidUrl(String),
    #[error("error de red: {0}")]
    Network(String),
}

// Async: never hold locks across .await, use JoinSet for parallel tasks
async fn scrape_urls(&self, urls: &[Url]) -> Vec<Result<Content>> {
    let mut tasks = JoinSet::new();
    for url in urls {
        let client = self.client.clone();
        tasks.spawn(async move { fetch(&client, url).await });
    }
    // collect results...
}

// Feature gating: #[cfg(feature = "ai")] for ONNX-dependent code
#[cfg(feature = "ai")]
pub use domain::semantic_cleaner::SemanticCleaner;
```

**Conventions:** Named exports only, no defaults. `snake_case` functions, `UpperCamelCase` types. No `get_` prefix. Error messages lowercase, no trailing punctuation.

---

## Non-Obvious Patterns

### Crate version conflicts (DO NOT try to unify)
- `dashmap` 5.x (via governor) + 6.x (direct) — both needed
- `quick-xml` 0.37 (direct) + 0.38 (via syntect→plist) — both needed
- `scraper` 0.22 → selectors 0.26, `legible` → dom_query → selectors 0.35 — both needed

### HTTP client: `wreq` not `reqwest`
Uses TLS fingerprint emulation (Chrome 131) for WAF evasion. Layer 2 evasion built in.

### WAF detection on HTTP 200
Responses are scanned for 19 WAF signatures (Cloudflare, reCAPTCHA, hCaptcha, DataDome, PerimeterX, Akamai). If detected, UA is rotated and retried once. Still blocked → `ScraperError::WafBlocked`.

### AI feature (`--features ai`)
- Loads ~90MB ONNX model (all-MiniLM-L6-v2) into memory
- `SemanticCleanerImpl::new()` is **async** — loads model once, reuses
- `cleaner.clean(html)` is **async** — returns `Vec<DocumentChunk>` with embeddings
- One page → multiple chunks when AI cleaning is active
- Model cached in `~/.cache/rust-scraper/models/`

---

## Testing Rules

```bash
cargo nextest run --test-threads 2                   # Standard test run
cargo nextest run --test-threads 2 --features ai     # With AI tests
```

- Tests use `#[cfg(test)] mod tests { }` pattern
- Async tests: `#[tokio::test]`
- Structure: Arrange → Act → Assert
- Use `use super::*;` in test modules
- Mock external dependencies with `mockall`
- **Never delete a failing test** — only fix or extend

---

## Clean Architecture

```
Domain (pure, no frameworks) → Application (orchestration) → Infrastructure (implementations) → Adapters (TUI, CLI)
```

**VIOLATION = REJECT:** Domain layer importing `tokio`, `wreq`, or any I/O crate.

Error handling by layer: Domain → `thiserror`, Application → `anyhow`, Infrastructure → `thiserror`, Binary → `anyhow`.

---

## Boundaries

### ✅ Always
- Run `cargo check` before marking any task complete
- Run `cargo clippy -- -D warnings` before committing
- Use `cargo nextest run` (never `cargo test`)
- Use `cargo llvm-cov` (never `cargo tarpaulin`)
- Use `bacon` for background checking (never `cargo-watch`)

### ⚠️ Ask first
- Adding or removing dependencies
- Changing feature flag structure
- Modifying `Cargo.toml` profiles
- Database or schema changes

### 🚫 Never
- Commit secrets, `.env` files, or credentials
- Use `.unwrap()` in production code — use `?` or `match`
- Hold `Mutex`/`RwLock` across `.await` points
- Force push to main or protected branches
- Modify `target/`, `dist/`, or `build/` directories
- Run `cargo build --release` during development (use `cargo check`)

---

## GitNexus — Code Intelligence

This project is indexed by GitNexus: **3515 symbols, 6453 relationships, 300 execution flows**.

```bash
bunx gitnexus analyze              # Re-index after code changes
bunx gitnexus status               # Check index freshness
```

**Before editing any symbol:** Run impact analysis to check blast radius.
**Before committing:** Run `gitnexus_detect_changes()` to verify scope.
**When exploring:** Use `gitnexus_query()` instead of grep for execution flows.
**When debugging:** Use `gitnexus_context()` for 360° symbol view.

> If index is stale, run `bunx gitnexus analyze` first.

---

## Resources

- [rust-skills](rust-skills/SKILL.md) — 179 Rust rules (project local)
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — Architecture details
- [DEVELOPMENT.md](DEVELOPMENT.md) — Dev workflow and tooling
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
