# Specification: Migrate to wreq for TLS Fingerprint Impersonation

## Requirements

### REQ-1: TLS Fingerprint Impersonation with Chrome 131 Default
**Priority:** MUST
**Description:** The HTTP client must send a TLS ClientHello that is byte-identical to Chrome 131's fingerprint. This is achieved via `wreq::Client::builder().impersonate(Impersonate::Chrome131)`. The impersonation preset must be configurable via `HttpClientConfig` to allow switching between Chrome 131, Firefox 133, and other browser presets supported by wreq.
**Acceptance:**
- Default impersonation preset is Chrome 131
- `HttpClientConfig` has an `impersonate` field with `Impersonate` enum variant
- TLS fingerprint verified as Chrome 131 (not Rust/rustls) against ja3er.com or equivalent JA3 checker
- All HTTP requests from `HttpClient::get()`, `create_http_client()`, and `SitemapParser` use the configured impersonation preset

### REQ-2: ScraperError Decoupled from reqwest Types
**Priority:** MUST
**Description:** `ScraperError::Http` must use `wreq::StatusCode` instead of `reqwest::StatusCode`. `ScraperError::Network` must use `wreq::Error` instead of `reqwest::Error`. The `#[from]` attribute must be updated for automatic error conversion from wreq types. The helper function `ScraperError::http()` must accept `wreq::StatusCode`.
**Acceptance:**
- `src/error.rs` has zero imports of `reqwest` crate
- `ScraperError::Http { status: wreq::StatusCode, url: String }` compiles
- `ScraperError::Network(#[from] wreq::Error)` compiles
- `ScraperError::http(status: wreq::StatusCode, url: &str)` accepts wreq types
- All existing error tests pass with wreq types

### REQ-3: SitemapError Migrated to wreq Types
**Priority:** MUST
**Description:** `SitemapError::HttpError` must use `#[from] wreq::Error` instead of `#[from] reqwest::Error`. The `SitemapParser` struct must hold a `wreq::Client` instead of `reqwest::Client`. All HTTP operations within the sitemap parser must use wreq's streaming API.
**Acceptance:**
- `src/infrastructure/crawler/sitemap_parser.rs` has zero imports of `reqwest` crate
- `SitemapParser.client` field is `wreq::Client`
- `SitemapError::HttpError(#[from] wreq::Error)` compiles
- Streaming response via `response.bytes_stream()` works identically
- All sitemap unit tests pass without modification to test logic

### REQ-4: Preserve All Existing HTTP Client Behavior
**Priority:** MUST
**Description:** The migration must preserve every existing behavior of the HTTP client layer: exponential backoff retry, 403 UA rotation, 429 rate limit handling with Retry-After header, 5xx automatic retry, WAF/CAPTCHA detection (19 signatures), custom headers (Accept-Language, Accept, Referer, Cache-Control), cookie store, gzip/brotli decompression, connection pooling, and timeout configuration.
**Acceptance:**
- `HttpClient::get()` retry logic for 403/429/5xx is unchanged in behavior
- `detect_waf_challenge()` still scans 19 WAF signatures on HTTP 200 responses
- Custom headers are applied to every request
- Cookie store is enabled (`cookie_store(true)`)
- Gzip and brotli compression are enabled
- Connection pool settings preserved: `pool_max_idle_per_host`, `pool_idle_timeout`
- Timeout settings preserved: 30s request timeout, 10s connect timeout
- All wiremock tests pass without modification to test assertions

### REQ-5: reqwest-middleware Replacement with Hand-Rolled Retry
**Priority:** MUST
**Description:** `reqwest-middleware` 0.4 and `reqwest-retry` 0.7 must be removed from `Cargo.toml`. The `create_http_client()` legacy function must return `wreq::Client` directly (not `ClientWithMiddleware`). The hand-rolled retry logic already present in `HttpClient::get()` must cover all cases previously handled by `reqwest-retry` middleware (transient errors, 5xx, timeouts).
**Acceptance:**
- `Cargo.toml` has no `reqwest-middleware`, `reqwest-retry`, or `retry-policies` dependencies
- `create_http_client()` returns `Result<wreq::Client, ScraperError>`
- `pub use reqwest_middleware::ClientWithMiddleware` is removed from `http_client.rs`
- No callers of `create_http_client()` break (update call sites to use `wreq::Client` directly)
- Hand-rolled retry covers: transient network errors, 5xx server errors, timeouts

### REQ-6: Crawler HTTP Client Migrated to wreq
**Priority:** MUST
**Description:** `src/infrastructure/crawler/http_client.rs` must use `wreq::Client` instead of `reqwest::Client`. The `create_rate_limited_client()` function must return `wreq::Client`. The `fetch_url()` function must handle `wreq::Error` in its error mapping.
**Acceptance:**
- `create_rate_limited_client(delay_ms: u64) -> anyhow::Result<wreq::Client>` compiles
- `fetch_url()` maps `wreq::Error` to `CrawlError::Network` correctly
- `wreq::Error::is_timeout()` and `wreq::Error::is_connect()` methods are used (or wreq equivalents)
- Pool size and timeout settings are preserved

### REQ-7: BoringSSL Build Compatibility
**Priority:** MUST
**Description:** The build must succeed without OpenSSL/BoringSSL symbol conflicts. On Linux, wreq's BoringSSL dependency must use the `prefix-symbols` feature (or equivalent) to avoid conflicts with any system `openssl-sys` that may be pulled in by other crates. The release build must compile cleanly.
**Acceptance:**
- `cargo build --release` succeeds without linker errors
- No `openssl-sys` symbol conflicts in the build output
- `cargo clippy --all-targets --all-features -- -D warnings` passes with zero warnings
- Build works on target hardware (Intel Haswell, Linux CachyOS)

### REQ-8: wiremock Test Compatibility
**Priority:** MUST
**Description:** All `wiremock` 0.6 tests must continue to pass without modification. Since wiremock operates at the HTTP level (not TLS), the mock server behavior is unchanged. Only type imports in test files need updating from `reqwest::StatusCode` to `wreq::StatusCode`.
**Acceptance:**
- All wiremock tests in `src/application/http_client.rs` pass:
  - `test_403_returns_error`
  - `test_429_returns_error`
  - `test_500_returns_error`
  - `test_500_exhausts_retries`
  - `test_404_returns_client_error`
  - `test_200_returns_body`
  - `test_200_cloudflare_challenge_returns_waf_error`
  - `test_200_recaptcha_challenge_returns_waf_error`
  - `test_200_normal_page_returns_body`
- No wiremock assertion logic changes required (only type imports)

### REQ-9: Unit Test Type Compatibility
**Priority:** MUST
**Description:** All unit tests that reference `reqwest::StatusCode` or `reqwest::Error` must be updated to use wreq equivalents. Test logic and assertions must remain unchanged — only type imports change.
**Acceptance:**
- `test_http_error()` in `src/error.rs` uses `wreq::StatusCode::from_u16(404)`
- All test modules compile without reqwest imports
- `cargo nextest run --test-threads 2` passes all tests
- No test logic changes — only type substitutions

### REQ-10: No reqwest Imports Remain in Source Code
**Priority:** MUST
**Description:** After migration, no source file in `src/` may import from the `reqwest` crate. This includes direct imports, re-exports, and transitive usage. The only reference to reqwest may be in `Cargo.toml` during the transition (if kept as a dev-dependency for any reason), but ideally it should be fully removed.
**Acceptance:**
- `grep -r "use reqwest" src/` returns zero results
- `grep -r "reqwest::" src/` returns zero results
- `Cargo.toml` has no `reqwest` dependency (replaced entirely by `wreq`)

### REQ-11: Configurable Impersonation Presets
**Priority:** SHOULD
**Description:** `HttpClientConfig` should expose an `impersonate` field that allows selecting between different browser impersonation presets (Chrome 131, Firefox 133, Safari, etc.). The default should be Chrome 131. This enables flexibility for sites that may fingerprint specific browser versions.
**Acceptance:**
- `HttpClientConfig` has `impersonate: Impersonate` field (or equivalent wreq type)
- Default config uses `Impersonate::Chrome131`
- Client builder applies the configured preset
- Documentation explains available presets

### REQ-12: Downloader Adapter Header Compatibility
**Priority:** SHOULD
**Description:** Any adapter code that imports HTTP header constants (e.g., `reqwest::header::CONTENT_TYPE`) must be updated to use wreq's header module (`wreq::header::CONTENT_TYPE`). This includes `src/adapters/downloader/mod.rs` if it references reqwest headers.
**Acceptance:**
- No `reqwest::header` imports in adapter code
- Header constants use `wreq::header::*` equivalents
- Downloader functionality unchanged

### REQ-13: Version Bump to v1.1.0
**Priority:** SHOULD
**Description:** Since this is a significant internal change that adds TLS fingerprint impersonation capability (Layer 2 WAF Evasion), the package version should be bumped from `1.0.7` to `1.1.0` to reflect the new minor version feature.
**Acceptance:**
- `Cargo.toml` version is `1.1.0`
- Git tag or changelog entry created for v1.1.0

### REQ-14: Fallback to Standard TLS if Impersonation Fails
**Priority:** COULD
**Description:** If a specific impersonation preset fails to establish a connection (e.g., server rejects the specific TLS fingerprint), the client should gracefully fall back to a standard TLS connection rather than failing the request entirely. This is a safety net for edge cases.
**Acceptance:**
- Connection failures due to TLS fingerprint are retried with standard TLS
- Fallback is logged at debug level
- No infinite retry loops between impersonation and standard TLS

## Scenarios

### Scenario: Default Chrome 131 TLS Fingerprint
**Given:** A user creates an `HttpClient` with default `HttpClientConfig`
**When:** The client makes a GET request to any HTTPS URL
**Then:** The TLS ClientHello sent matches Chrome 131's fingerprint exactly
**And:** The JA3 hash matches Chrome 131's known JA3 hash

### Scenario: Scraping Cloudflare-Protected Site
**Given:** A target website is protected by Cloudflare WAF
**And:** The scraper previously failed with `ScraperError::WafBlocked` using reqwest
**When:** The scraper makes a request using the wreq-based client with Chrome 131 impersonation
**Then:** The request succeeds with HTTP 200 and real content
**And:** No WAF challenge is detected in the response body

### Scenario: 403 Forbidden Triggers UA Rotation
**Given:** An `HttpClient` is configured with multiple user agents
**When:** A GET request returns HTTP 403
**Then:** The client retries once with a different user agent
**And:** If the retry also returns 403, `HttpError::Forbidden` is returned
**And:** The TLS fingerprint remains Chrome 131 on both attempts

### Scenario: 429 Rate Limit with Retry-After Header
**Given:** An `HttpClient` receives HTTP 429 with `Retry-After: 5` header
**When:** The client processes the response
**Then:** It waits 5 seconds before retrying
**And:** Uses exponential backoff for subsequent 429 responses
**And:** Returns `HttpError::RateLimited(5)` after exhausting retries

### Scenario: WAF Challenge Detected in HTTP 200 Response
**Given:** A server returns HTTP 200 with a Cloudflare challenge page body
**When:** The response body is scanned by `detect_waf_challenge()`
**Then:** The function returns `Some("Cloudflare")`
**And:** `HttpClient::get()` returns `HttpError::WafChallenge("Cloudflare")`
**And:** The UA is rotated once before returning the error

### Scenario: Sitemap Parsing with wreq Client
**Given:** A `SitemapParser` is created with default configuration
**When:** `parse_from_url()` is called with a valid sitemap URL
**Then:** The sitemap is fetched using `wreq::Client` with Chrome 131 impersonation
**And:** XML parsing extracts all `<loc>` URLs correctly
**And:** Gzip-compressed sitemaps are decompressed and parsed

### Scenario: wiremock Test with wreq Client
**Given:** A wiremock `MockServer` is started
**And:** A mock is configured to return HTTP 200 with a specific body
**When:** `HttpClient::get()` is called with the mock server URI
**Then:** The response body matches the mock's configured body
**And:** The test assertion passes without modification

### Scenario: Error Type Conversion from wreq::Error
**Given:** A network request fails with a wreq connection error
**When:** The error is propagated through the application layer
**Then:** `wreq::Error` is automatically converted to `ScraperError::Network` via `#[from]`
**And:** The error message preserves the original wreq error description

### Scenario: Legacy create_http_client Returns wreq::Client
**Given:** A caller uses the legacy `create_http_client()` function
**When:** The function is invoked
**Then:** It returns a `wreq::Client` (not `ClientWithMiddleware`)
**And:** The client has a random user agent from the pool
**And:** Timeout, gzip, and brotli settings are applied

### Scenario: Build Succeeds Without OpenSSL Conflicts
**Given:** The project is built with `cargo build --release`
**When:** The linker processes all object files
**Then:** No symbol conflicts between BoringSSL and OpenSSL are reported
**And:** The binary is produced successfully

### Scenario: Configurable Impersonation Preset
**Given:** A user sets `HttpClientConfig { impersonate: Impersonate::Firefox133, ..Default::default() }`
**When:** An `HttpClient` is created with this config
**Then:** All requests use Firefox 133's TLS fingerprint
**And:** The JA3 hash matches Firefox 133's known fingerprint

### Scenario: Sitemap Response Size Limit Enforcement
**Given:** A sitemap response exceeds 50MB
**When:** `SitemapParser` streams the response
**Then:** Parsing stops at the size limit
**And:** `SitemapError::ResponseTooLarge(52_428_800)` is returned
**And:** No OOM occurs

## Non-Functional Requirements

### Performance
- TLS impersonation must not add measurable latency overhead (< 5ms per connection)
- Memory usage must remain at ~8KB constant RAM for streaming responses (unchanged from current behavior)
- Connection pooling settings preserved: pool_max_idle_per_host = max(3, num_cpus - 1), pool_idle_timeout = 60s
- Build time must not increase significantly (wreq compilation cached by sccache)

### Security
- TLS impersonation must use real browser fingerprints, not custom/fake ones that could be more easily detected
- All existing WAF detection (19 signatures) must continue to function
- robots.txt respect is unchanged
- No `.unwrap()` in production code paths — all network errors handled via `Result`
- BoringSSL `prefix-symbols` feature prevents symbol collision attacks

### Compatibility
- MSRV remains 1.88+ (wreq 5.3.0 must support this)
- wiremock 0.6 tests work without logic changes (HTTP-level mocking)
- All feature flags (`images`, `documents`, `full`, `ai`) continue to work
- Target hardware: Intel Haswell i5-4590, 8GB DDR3, HDD — all defaults must be HDD-aware
- Linux CachyOS with ZRAM swap — BoringSSL build must succeed on this platform

### Reliability
- Hand-rolled retry logic must cover all cases previously handled by reqwest-middleware:
  - Transient network errors (connection refused, reset)
  - 5xx server errors (with exponential backoff)
  - Timeouts (with retry)
- No regression in error handling: all error variants must be preserved
- Rollback plan: full revert achievable in < 5 minutes via git revert
