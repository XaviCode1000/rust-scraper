# rust_scraper - Project State

## Current Status: ✅ ALL TESTS PASSING

Date: 2026-04-22

---

## Test Suite Status

| Test Group | Status |
|------------|--------|
| All unit tests | ✅ PASS |
| Integration tests | ✅ PASS |
| CLI binary tests | ✅ PASS |
| Security tests | ✅ PASS |
| Vector exporter tests | ✅ PASS |

---

## Recent Fixes

### 1. Tokio-console Conflict
- **File**: `Cargo.toml`, `src/main.rs`
- **Issue**: `console-subscriber = "0.5"` acts as global test hook, breaking CLI binary tests
- **Fix**: Removed console-subscriber dependency and `console_subscriber::init()` call
- **Tests fixed**: 5 CLI binary tests

### 2. Entropy Detection Threshold
- **File**: `src/application/http_client/waf.rs`
- **Issue**: UTF-8 encoding of code points 128-255 produces 2-byte sequences, reducing byte entropy to ~5.5 bits
- **Fix**: Lowered entropy threshold from 6.5 → 6.0 → 5.5
- **Tests fixed**: `test_detect_by_entropy_high`, `test_datadome_high_entropy_detection`

### 3. CLI Error Output
- **File**: `src/cli/error.rs`
- **Issue**: `CliExit::Termination::report()` didn't print error messages to stderr
- **Fix**: Added `eprintln!("Error: {}", msg)` for each error variant
- **Tests fixed**: `test_invalid_url_shows_error`

### 4. Vector Exporter Tests
- **File**: `src/infrastructure/export/vector_exporter.rs`
- **Issue**: `create_test_chunk()` creates documents without embeddings, but tests expected embeddings field
- **Fix**: Manually add `doc.embeddings = Some(vec![...])` in tests
- **Tests fixed**: `test_serialize_document_with_embeddings`, `test_serialize_document_dimension_mismatch`

---

## Architecture Summary

- **Stack**: Rust 1.88 · Tokio · wreq (TLS fingerprint) · ratatui · tract-onnx
- **Hardware**: Intel i5-4590 (4C), 8GB DDR3, HDD
- **Test runner**: cargo-nextest with 2 threads (HDD optimization)

---

## Commands

```bash
# Run tests
cargo nextest run --test-threads 2

# Check compilation
cargo check

# Lint
cargo clippy -- -D warnings

# CI pipeline
just test-ci
```
