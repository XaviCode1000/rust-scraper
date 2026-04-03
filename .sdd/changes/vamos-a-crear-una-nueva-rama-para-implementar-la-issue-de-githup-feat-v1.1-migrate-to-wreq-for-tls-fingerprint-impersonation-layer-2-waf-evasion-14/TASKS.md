# Tasks: Migrate to wreq for TLS Fingerprint Impersonation

## Phase 1: Cargo.toml & Dependencies (Highest Risk)

- [ ] **TASK-001: Update Cargo.toml dependencies**
  - Replace `reqwest 0.12` with `wreq 5.3.0` and features: `["boring-tls", "gzip", "brotli", "stream", "json", "cookies"]`
  - Remove `reqwest-middleware 0.4`, `reqwest-retry 0.7`, `retry-policies 0.4`
  - Run `cargo update` to regenerate Cargo.lock
  - Verify: `cargo check` shows type errors (expected), not linker errors
  - Files: `Cargo.toml`, `Cargo.lock`

- [ ] **TASK-002: Verify BoringSSL build compatibility**
  - Run `cargo build --release`
  - Check for openssl-sys symbol conflicts in build output
  - If conflicts: ensure `boring-tls` feature handles prefix-symbols on Linux
  - Verify: Binary produced, no linker errors
  - Files: None (build verification only)

## Phase 2: Domain Error Types (Medium Risk)

- [ ] **TASK-003: Decouple ScraperError from reqwest types**
  - Change `ScraperError::Http { status: reqwest::StatusCode, ... }` → `status: u16`
  - Change `ScraperError::Network(#[from] reqwest::Error)` → `Network(String)` (remove `#[from]`)
  - Update `ScraperError::http()` to accept `u16` instead of `reqwest::StatusCode`
  - Update error messages to lowercase per `err-lowercase-msg` rule
  - Update `test_http_error()` in test module: use `404u16` instead of `reqwest::StatusCode::from_u16(404).unwrap()`
  - Verify: `cargo nextest run error --test-threads 2` passes
  - Files: `src/error.rs`

## Phase 3: Application HTTP Client (Medium Risk)

- [ ] **TASK-004: Migrate HttpClient to wreq**
  - Update `use reqwest::Client` → `use wreq::Client`
  - Add `use wreq::Impersonate` import
  - Add `.impersonate(Impersonate::Chrome131)` to client builder
  - Update error mapping: `wreq::Error` → `HttpError` variants (same logic, different type)
  - Update doc comments: "reqwest::Client" → "wreq::Client"
  - Verify: `cargo clippy --all-targets --all-features -- -D warnings` passes on http_client.rs
  - Files: `src/application/http_client.rs`

- [ ] **TASK-005: Replace reqwest-middleware in create_http_client()**
  - Remove `pub use reqwest_middleware::ClientWithMiddleware` re-export
  - Change `create_http_client()` return type from `ClientWithMiddleware` → `wreq::Client`
  - Remove `reqwest_middleware::ClientBuilder`, `reqwest_retry` imports and middleware logic
  - Add `.impersonate(Impersonate::Chrome131)` and `.cookie_store(true)` to builder
  - Update error message to lowercase
  - Verify: All callers of `create_http_client()` compile
  - Files: `src/application/http_client.rs`

- [ ] **TASK-006: Update HttpClient wiremock tests**
  - No logic changes needed — wiremock operates at HTTP level
  - Verify all wiremock tests pass: test_403, test_429, test_500, test_500_exhausts_retries, test_404, test_200, test_200_cloudflare, test_200_recaptcha, test_200_normal
  - Verify: `cargo nextest run http_client --test-threads 2` passes
  - Files: `src/application/http_client.rs` (test modules)

- [ ] **TASK-007: Migrate scraper_service.rs**
  - Remove `use reqwest_middleware::ClientWithMiddleware` import
  - Update any function signatures that accept `ClientWithMiddleware` to use `wreq::Client`
  - Verify: Compiles clean
  - Files: `src/application/scraper_service.rs`

- [ ] **TASK-008: Migrate crawler_service.rs**
  - Update `scrape_single_url()` parameter from `&reqwest_middleware::ClientWithMiddleware` → `&wreq::Client`
  - Replace `reqwest::get(robots_url)` → `wreq::get(robots_url)` for robots.txt check
  - Replace `reqwest::Client::new().head(...)` → `wreq::Client::new().head(...)` for sitemap check
  - Verify: Compiles clean
  - Files: `src/application/crawler_service.rs`

## Phase 4: Infrastructure & Adapters (Low Risk)

- [ ] **TASK-009: Migrate crawler HTTP client**
  - Update `use reqwest::Client` → `use wreq::Client`
  - Update `fetch_url()` error mapping (wreq::Error → CrawlError::Network)
  - Update doc comments: "reqwest" → "wreq"
  - Verify: `cargo nextest run crawler --test-threads 2` passes
  - Files: `src/infrastructure/crawler/http_client.rs`

- [ ] **TASK-010: Migrate sitemap parser**
  - Update `SitemapParser.client` field from `reqwest::Client` → `wreq::Client`
  - Update `SitemapError::HttpError(#[from] reqwest::Error)` → `#[from] wreq::Error`
  - Update `SitemapParser::new()` and `with_config()` builders
  - Verify streaming still works via `response.bytes_stream()`
  - Verify: `cargo nextest run sitemap --test-threads 2` passes
  - Files: `src/infrastructure/crawler/sitemap_parser.rs`

- [ ] **TASK-011: Migrate asset download**
  - Update `use reqwest::Client` → `use wreq::Client` in `download_single_asset()`
  - Verify: Compiles clean
  - Files: `src/infrastructure/scraper/asset_download.rs`

- [ ] **TASK-012: Migrate downloader adapter**
  - Update `use reqwest::{Client, Response}` → `use wreq::{Client, Response}`
  - Update `reqwest::header::CONTENT_TYPE` → `wreq::header::CONTENT_TYPE`
  - Update `reqwest::Result<bytes::Bytes>` → `wreq::Result<bytes::Bytes>` in `into_stream()`
  - Verify: Compiles clean
  - Files: `src/adapters/downloader/mod.rs`

- [ ] **TASK-013: Migrate user agent fetcher**
  - Update `use reqwest::Client` → `use wreq::Client`
  - Verify: Compiles clean, cache logic unchanged
  - Files: `src/user_agent.rs`

- [ ] **TASK-014: Audit re-exports and remaining references**
  - Review `src/infrastructure/http/mod.rs` — no changes needed (re-exports from application)
  - Run `rg "reqwest" src/` — should return zero results (excluding comments/doc strings)
  - Update `src/config.rs` log filter strings if needed (trivial, only in test strings)
  - Verify: No `use reqwest` or `reqwest::` imports remain in source code
  - Files: `src/infrastructure/http/mod.rs`, `src/config.rs`

## Phase 5: Full Test Suite & Validation

- [ ] **TASK-015: Run full test suite**
  - `cargo nextest run --test-threads 2`
  - Fix any remaining test failures
  - Verify: All tests pass (unit + wiremock + integration)
  - Files: All test modules

- [ ] **TASK-016: Run clippy and format check**
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo fmt --check`
  - Verify: Zero warnings, formatting clean
  - Files: All source files

- [ ] **TASK-017: Coverage check**
  - `cargo llvm-cov --html --output-dir coverage-llvm`
  - Verify: Coverage not regressed from baseline
  - Files: None (verification only)

## Phase 6: Integration & Validation

- [ ] **TASK-018: Release build verification**
  - `cargo build --release`
  - Verify: Binary produced, no linker errors
  - Files: None (build verification only)

- [ ] **TASK-019: AI feature build verification**
  - `cargo build --release --features ai`
  - Verify: No BoringSSL/openssl-sys conflicts with tract-onnx
  - Files: None (build verification only)

- [ ] **TASK-020: Version bump to v1.1.0**
  - Update `Cargo.toml` version from `1.0.7` to `1.1.0`
  - Verify: `cargo build --release` succeeds with new version
  - Files: `Cargo.toml`
