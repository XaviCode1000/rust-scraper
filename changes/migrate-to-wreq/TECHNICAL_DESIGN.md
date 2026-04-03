# Technical Design: Migrate to wreq for TLS Fingerprint Impersonation

## Architecture Overview

The HTTP client layer sits in **Infrastructure** (`src/infrastructure/http/`) and is consumed by **Application** (`src/application/http_client.rs`). The migration replaces `reqwest` 0.12 + `reqwest-middleware` 0.4 with `wreq` 5.3.0, adding TLS fingerprint impersonation at the Infrastructure layer while preserving the existing `HttpClient` wrapper in Application.

```
┌─────────────────────────────────────────┐
│  Adapters (TUI, CLI)                    │  ← No changes
├─────────────────────────────────────────┤
│  Application (HttpClient wrapper)       │  ← Change Client type, keep logic
├─────────────────────────────────────────┤
│  Infrastructure (wreq HTTP client)      │  ← Replace reqwest → wreq
├─────────────────────────────────────────┤
│  Domain (ScraperError)                  │  ← Update status/error types
└─────────────────────────────────────────┘
```

**Key principle:** The `HttpClient` struct in Application already implements hand-rolled retry logic (403 UA rotation, 429 backoff, 5xx retry). This means `reqwest-middleware` and `reqwest-retry` are **redundant** and will be removed entirely. The migration simplifies the dependency tree.

---

## Key Decisions

### Decision 1: wreq Version — Stable 5.3.0

**Context:** wreq has a pre-release 6.0.0-rc.28 and a stable 5.3.0.

**Options considered:**
- `wreq 6.0.0-rc.28` — latest, but unstable API, breaking changes possible
- `wreq 5.3.0` — stable, well-tested, sufficient for our needs

**Decision:** Pin to `wreq 5.3.0` (stable).

**Rationale:** This is a production scraper. Pre-release versions risk API instability and breaking changes. The stable 5.3.0 already supports TLS impersonation with Chrome 131 presets, which is our primary requirement.

**Consequences:** We may miss newer impersonation presets added in 6.x, but can upgrade later when 6.0.0 is stable.

---

### Decision 2: BoringSSL Coexistence — `prefix-symbols` Feature

**Context:** wreq uses BoringSSL internally. If any other crate in the dependency tree uses `openssl-sys`, linker conflicts occur.

**Options considered:**
- Use `prefix-symbols` feature on wreq's BoringSSL — prefixes all BoringSSL symbols to avoid collisions
- Remove all openssl-using crates — impractical, other crates may depend on it
- Use `boring2` crate directly with `prefix-symbols`

**Decision:** Enable `prefix-symbols` feature on wreq. On Linux, this is mandatory.

**Rationale:** The project uses `tract-onnx` (optional, behind `ai` feature) and other crates that may transitively depend on OpenSSL. The `prefix-symbols` feature is the standard way to resolve BoringSSL/openssl-sys coexistence.

**Consequences:** Slightly larger binary due to duplicated SSL symbols, but no runtime conflicts.

---

### Decision 3: Error Type Strategy — Abstract to Primitives

**Context:** Current `ScraperError` holds `reqwest::StatusCode` and `reqwest::Error` directly, coupling Domain to the HTTP library.

**Options considered:**
- Keep wreq types in ScraperError — minimal code changes, but couples Domain to wreq
- Abstract to primitives (u16 for status, String for messages) — cleaner separation, more work
- Define a domain-level StatusCode newtype — middle ground

**Decision:** Abstract to primitives. `ScraperError::Http` stores `u16` for status code. `ScraperError::Network` stores `String` for the error message. Remove `#[from] reqwest::Error` and replace with manual conversion.

**Rationale:** Following Clean Architecture, Domain must NOT depend on external crates. The current coupling to `reqwest::StatusCode` and `reqwest::Error` is a violation. This migration is the right time to fix it.

**Consequences:**
- `ScraperError::Http` loses `reqwest::StatusCode` methods (`.is_success()`, etc.) — callers must use `u16` comparisons
- `ScraperError::Network` loses `reqwest::Error` introspection (`.is_timeout()`, `.is_connect()`) — but `HttpClient` already maps these to `HttpError` variants before they reach ScraperError
- Tests that construct `reqwest::StatusCode::from_u16(404)` must use raw `404u16` instead

---

### Decision 4: Remove reqwest-middleware — Leverage Existing Hand-Rolled Retry

**Context:** The project uses `reqwest-middleware` 0.4 + `reqwest-retry` 0.7 for automatic retry. But `HttpClient` in Application already implements comprehensive retry logic (403 UA rotation, 429 backoff, 5xx retry).

**Options considered:**
- Keep reqwest-middleware — incompatible with wreq, would need a middleware adapter
- Find wreq-compatible middleware — none exist in the ecosystem
- Remove reqwest-middleware entirely — hand-rolled retry already covers all cases

**Decision:** Remove `reqwest-middleware`, `reqwest-retry`, and `retry-policies` entirely.

**Rationale:** The `HttpClient::get()` method already handles all retry scenarios. The middleware was redundant. Removing it simplifies the dependency tree and eliminates a migration blocker.

**Consequences:** The legacy `create_http_client()` function that returns `ClientWithMiddleware` must be updated to return `wreq::Client` directly. All callers of `create_http_client()` must be audited.

---

### Decision 5: TLS Impersonation Preset — Chrome 131 Default, Configurable

**Context:** wreq supports TLS impersonation via `Impersonate` enum. We need a sensible default with the ability to change it.

**Options considered:**
- Hardcode Chrome 131 — simple, but inflexible
- Make it a CLI flag — flexible, but adds complexity
- Default to Chrome 131, configurable via `HttpClientConfig` — best of both

**Decision:** Default to Chrome 131 (`Impersonate::Chrome131`), expose via `HttpClientConfig::tls_preset` field.

**Rationale:** Chrome 131 is the current stable Chrome version at the time of writing. Making it configurable allows testing against different fingerprints without code changes.

**Consequences:** `HttpClientConfig` gains a new field. The `Default` impl uses Chrome 131. Future presets (Firefox 133, Safari 18) can be added without breaking changes.

---

### Decision 6: Streaming Compatibility — Preserve with wreq

**Context:** The project uses `reqwest` streaming for large payloads (sitemap parsing, downloader). wreq has compatible streaming APIs.

**Options considered:**
- Keep reqwest for streaming only — defeats the purpose of migration
- Migrate all streaming to wreq — wreq supports `bytes()` streaming identically

**Decision:** Migrate all streaming to wreq. The `reqwest::Client::get().send().await?.bytes()` pattern is identical in wreq.

**Rationale:** wreq is API-compatible with reqwest for the streaming patterns used in this project. No functional changes needed.

**Consequences:** `asset_download.rs` and `sitemap_parser.rs` must update their `use reqwest::Client` imports to `use wreq::Client`.

---

## Migration Plan

### Phase 1: Cargo.toml & Dependencies (Highest Risk)

**Steps:**
1. Replace `reqwest 0.12` with `wreq 5.3.0` in `[dependencies]`
2. Remove `reqwest-middleware 0.4`, `reqwest-retry 0.7`, `retry-policies 0.4`
3. Add `wreq` features: `["boring-tls", "gzip", "brotli", "stream", "json", "cookies"]`
4. Run `cargo update` to resolve dependency tree
5. Verify no `openssl-sys` conflicts with `cargo build`

**Files affected:**
- `Cargo.toml`
- `Cargo.lock` (auto-regenerated)

**Verification:**
```bash
cargo check 2>&1 | grep -i "error"  # Should show type errors, not linker errors
cargo build --release 2>&1 | grep -i "openssl\|boring"  # Should show no conflicts
```

---

### Phase 2: Domain Error Types (Medium Risk)

**Steps:**
1. Change `ScraperError::Http { status: reqwest::StatusCode, ... }` → `status: u16`
2. Change `ScraperError::Network(#[from] reqwest::Error)` → `Network(String)`
3. Remove `#[from]` on Network variant (manual conversion in HttpClient)
4. Update `ScraperError::http()` constructor to accept `u16`
5. Update `test_http_error()` in error.rs tests

**Files affected:**
- `src/error.rs`

**Verification:**
```bash
cargo nextest run error --test-threads 2
```

---

### Phase 3: Application HTTP Client (Medium Risk)

**Steps:**
1. Update `use reqwest::Client` → `use wreq::Client` in `src/application/http_client.rs`
2. Add TLS impersonation to client builder:
   ```rust
   let builder = Client::builder()
       .impersonate(Impersonate::Chrome131)
       .timeout(Duration::from_secs(30))
       // ... rest of config
   ```
3. Update error mapping: `wreq::Error` → `HttpError` variants (same logic, different type)
4. Remove `ClientWithMiddleware` re-export and `create_http_client()` middleware logic
5. Update `create_http_client()` to return `wreq::Client` directly

**Files affected:**
- `src/application/http_client.rs`

**Verification:**
```bash
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run http_client --test-threads 2
```

---

### Phase 4: Infrastructure & Adapters (Low Risk)

**Steps:**
1. Update `src/infrastructure/scraper/asset_download.rs`: `use reqwest::Client` → `use wreq::Client`
2. Update any `reqwest::header::*` imports to `wreq::header::*`
3. Update `src/infrastructure/http/mod.rs` re-exports if needed
4. Audit all files for remaining `reqwest::` references

**Files affected:**
- `src/infrastructure/scraper/asset_download.rs`
- `src/infrastructure/http/mod.rs`

**Verification:**
```bash
rg "reqwest" src/  # Should return zero results
cargo nextest run --test-threads 2
```

---

### Phase 5: Test Migration (Low Risk)

**Steps:**
1. Update `test_http_error()` in `src/error.rs`: replace `reqwest::StatusCode::from_u16(404).unwrap()` with `404u16`
2. Update test assertions that reference `reqwest` types
3. Verify `wiremock` tests still pass (HTTP-level mocking, unaffected by TLS changes)
4. Run full test suite

**Files affected:**
- `src/error.rs` (test module)
- `src/application/http_client.rs` (test module)
- `tests/` directory (if any integration tests reference reqwest types)

**Verification:**
```bash
cargo nextest run --test-threads 2
cargo llvm-cov --html --output-dir coverage-llvm
```

---

### Phase 6: Integration & Validation

**Steps:**
1. Build release: `cargo build --release`
2. Manual test against Cloudflare-protected site
3. Verify TLS fingerprint matches Chrome 131 (use `ja3er.com/json` endpoint)
4. Confirm WAF detection (19 signatures) still functions on HTTP 200 responses
5. Test with `--features ai` to verify no BoringSSL conflicts

**Verification:**
```bash
# Verify TLS fingerprint
curl -s https://ja3er.com/json | jq '.ja3_hash'  # Compare with Chrome 131 hash

# Full feature build
cargo build --release --features ai

# Full test suite
cargo nextest run --test-threads 2
cargo clippy --all-targets --all-features -- -D warnings
```

---

## Code Changes

### Cargo.toml

**Before:**
```toml
reqwest = { version = "0.12", features = ["rustls-tls-native-roots", "gzip", "brotli", "stream", "json", "cookies"] }
reqwest-middleware = "0.4"
reqwest-retry = "0.7"
retry-policies = "0.4"
```

**After:**
```toml
wreq = { version = "5.3.0", features = ["boring-tls", "gzip", "brotli", "stream", "json", "cookies"] }
```

**Notes:**
- `boring-tls` feature enables BoringSSL with `prefix-symbols` automatically on Linux
- `rustls-tls-native-roots` is replaced by BoringSSL (which includes its own root certs)
- `reqwest-middleware`, `reqwest-retry`, `retry-policies` removed entirely

---

### Error Types

**Before (`src/error.rs`):**
```rust
#[derive(Error, Debug)]
pub enum ScraperError {
    #[error("HTTP error {status} al acceder a {url}")]
    Http {
        status: reqwest::StatusCode,
        url: String,
    },
    #[error("Error de red: {0}")]
    Network(#[from] reqwest::Error),
    // ...
}

impl ScraperError {
    #[must_use]
    pub fn http(status: reqwest::StatusCode, url: &str) -> Self {
        Self::Http { status, url: url.to_string() }
    }
}
```

**After (`src/error.rs`):**
```rust
#[derive(Error, Debug)]
pub enum ScraperError {
    #[error("http error {status} al acceder a {url}")]
    Http {
        status: u16,
        url: String,
    },
    #[error("error de red: {0}")]
    Network(String),
    // ...
}

impl ScraperError {
    #[must_use]
    pub fn http(status: u16, url: &str) -> Self {
        Self::Http { status, url: url.to_string() }
    }
}
```

**Rationale for lowercase error messages:** Following `err-lowercase-msg` rule — error messages should be lowercase, no trailing punctuation.

---

### HTTP Client Builder

**Before (`src/application/http_client.rs`):**
```rust
let builder = Client::builder()
    .timeout(Duration::from_secs(30))
    .connect_timeout(Duration::from_secs(10))
    .pool_max_idle_per_host(pool_size)
    .pool_idle_timeout(Duration::from_secs(60))
    .gzip(true)
    .brotli(true)
    .cookie_store(true);

let client = builder.build()
    .map_err(|e| ScraperError::Config(format!("Failed to create HTTP client: {}", e)))?;
```

**After (`src/application/http_client.rs`):**
```rust
use wreq::{Client, Impersonate};

let builder = Client::builder()
    .impersonate(Impersonate::Chrome131)  // TLS fingerprint impersonation
    .timeout(Duration::from_secs(30))
    .connect_timeout(Duration::from_secs(10))
    .pool_max_idle_per_host(pool_size)
    .pool_idle_timeout(Duration::from_secs(60))
    .gzip(true)
    .brotli(true)
    .cookie_store(true);

let client = builder.build()
    .map_err(|e| ScraperError::Config(format!("failed to create http client: {}", e)))?;
```

**Configurable preset (future):**
```rust
#[derive(Debug, Clone, Default)]
pub enum TlsPreset {
    #[default]
    Chrome131,
    Firefox133,
    Safari18,
}

// In HttpClientConfig:
pub tls_preset: TlsPreset,

// In builder:
.impersonate(match config.tls_preset {
    TlsPreset::Chrome131 => Impersonate::Chrome131,
    TlsPreset::Firefox133 => Impersonate::Firefox133,
    TlsPreset::Safari18 => Impersonate::Safari18,
})
```

---

### create_http_client() — Simplified

**Before:**
```rust
pub fn create_http_client() -> Result<ClientWithMiddleware, ScraperError> {
    let agents = UserAgentCache::fallback_agents();
    let user_agent = get_random_user_agent_from_pool(&agents);
    
    let base_client = Client::builder()
        .user_agent(user_agent)
        .timeout(Duration::from_secs(30))
        .gzip(true)
        .brotli(true)
        .build()
        .map_err(|e| ScraperError::Config(...))?;
    
    use reqwest_middleware::ClientBuilder;
    use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
    
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client = ClientBuilder::new(base_client)
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();
    
    Ok(client)
}
```

**After:**
```rust
pub fn create_http_client() -> Result<Client, ScraperError> {
    let agents = UserAgentCache::fallback_agents();
    let user_agent = get_random_user_agent_from_pool(&agents);
    
    let client = Client::builder()
        .impersonate(Impersonate::Chrome131)
        .user_agent(user_agent)
        .timeout(Duration::from_secs(30))
        .gzip(true)
        .brotli(true)
        .cookie_store(true)
        .build()
        .map_err(|e| ScraperError::Config(format!("failed to create http client: {}", e)))?;
    
    Ok(client)
}
```

---

### asset_download.rs — Import Update

**Before:**
```rust
use reqwest::Client;
```

**After:**
```rust
use wreq::Client;
```

No other changes needed — the `Client::builder()`, `.get()`, `.send()`, `.bytes()` API is identical.

---

## Testing Strategy

| Phase | Test Type | Command | Pass Criteria |
|-------|-----------|---------|---------------|
| Phase 1 | Compilation | `cargo check` | Zero linker errors, no openssl conflicts |
| Phase 2 | Unit tests | `cargo nextest run error --test-threads 2` | All error tests pass with u16 types |
| Phase 3 | Unit + wiremock | `cargo nextest run http_client --test-threads 2` | HttpClient retry logic unchanged |
| Phase 4 | Unit tests | `cargo nextest run --test-threads 2` | All tests pass, no reqwest imports |
| Phase 5 | Full suite + coverage | `cargo nextest run --test-threads 2` + `cargo llvm-cov --html` | 100% of existing tests pass |
| Phase 6 | Manual | `cargo run --release -- --url <CF-protected-url>` | Successfully scrapes previously-blocked site |

**wiremock compatibility:** wiremock operates at the HTTP level (mocks TCP responses), not the TLS level. Since wreq's HTTP API is reqwest-compatible, all wiremock tests pass without modification.

---

## Rollback Plan

If any phase fails and cannot be resolved within 30 minutes:

1. **Revert Cargo.toml:**
   ```bash
   git checkout HEAD -- Cargo.toml
   cargo update  # Restore original Cargo.lock
   ```

2. **Revert source changes:**
   ```bash
   git checkout HEAD -- src/error.rs src/application/http_client.rs src/infrastructure/
   ```

3. **Verify pre-migration state:**
   ```bash
   cargo nextest run --test-threads 2
   cargo clippy --all-targets --all-features -- -D warnings
   ```

4. **Timeline:** Full rollback achievable in < 5 minutes since each phase is independently revertible.

**Per-phase rollback:** Each phase modifies a distinct set of files. If Phase 3 fails, you can revert only `src/application/http_client.rs` while keeping Phases 1-2.

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| wreq API differs from reqwest in subtle ways | Medium | High | Phase 1 isolates compilation; wiremock tests catch behavioral differences |
| BoringSSL prefix-symbols doesn't resolve all conflicts | Low | High | Test with `--features ai` early; fallback to separate build profiles |
| TLS impersonation fails on some WAFs | Low | Medium | Keep Chrome 131 as default; configurable preset allows switching |
| wiremock tests false-positive pass | Low | Low | wiremock tests HTTP semantics, not TLS — false positives unlikely |
| `wreq::Error` methods differ from `reqwest::Error` | Medium | Medium | `HttpClient` already maps errors to `HttpError` variants before they propagate |
