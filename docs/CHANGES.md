# 📋 CHANGELOG - Rust Scraper

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.3.0] - 2026-03-08 - Clean Architecture & Production Ready

### 🎉 Major Changes

**Complete architectural refactoring** from monolithic structure to Clean Architecture:

- **Before**: `scraper.rs` (1035 lines) - monolithic file
- **After**: 4-layer architecture (Domain, Application, Infrastructure, Adapters)

### ✨ Added

#### Clean Architecture Layers

**Domain Layer** (`src/domain/`):
- `entities.rs` - Core business entities (`ScrapedContent`, `DownloadedAsset`)
- `value_objects.rs` - Type-safe primitives (`ValidUrl`)

**Application Layer** (`src/application/`):
- `http_client.rs` - HTTP client with retry + User-Agent rotation
- `scraper_service.rs` - Use cases with bounded concurrency

**Infrastructure Layer** (`src/infrastructure/`):
- `http/` - HTTP client infrastructure
- `scraper/` - Readability, fallback, asset downloading
- `converter/` - HTML→Markdown, syntax highlighting
- `output/` - File saving, YAML frontmatter

**Adapters Layer** (`src/adapters/`):
- `detector/` - MIME type detection
- `extractor/` - URL extraction from HTML
- `downloader/` - Asset downloading

#### Production Features

- **Retry Logic** - Exponential backoff (3 retries) via `reqwest-middleware`
- **Bounded Concurrency** - `buffer_unordered(3)` for HDD systems
- **User-Agent Rotation** - 14 modern browsers with weighted selection
- **Lazy Statics** - CSS selectors compiled once with `once_cell::Lazy`

#### Error Handling

- **Structured Errors** - `ScraperError` enum with 14 variants
- **Type-Safe API** - `ScraperError::Result` instead of `anyhow::Result`
- **From Traits** - Automatic conversion for `std::io::Error`, `reqwest::Error`, etc.

### 🔧 Changed

#### Breaking Changes

- **Migrated from `anyhow::Result` to `ScraperError::Result`** in library API
  - Users can now match on specific error types
  - Better error handling and reporting
  - `anyhow` still used in `main.rs` (application layer)

- **Module Reorganization**:
  - `scraper.rs` removed (split into 15+ modular files)
  - `extractor/` and `detector/` moved to `adapters/` layer
  - New `domain/`, `application/`, `infrastructure/` layers

- **Public API Changes**:
  - `scraper::create_http_client()` → `create_http_client()`
  - `scraper::scrape_with_config()` → `scrape_with_config()`
  - `scraper::save_results()` → `save_results()`

#### Version

- Updated from `0.2.0` to `0.3.0` (semver breaking change)

### 🐛 Fixed

- **Production Panics** - Eliminated all `unwrap()` calls in production code
  - CSS selectors use `Lazy<Selector>` with `expect()`
  - Regex patterns use `Lazy<Regex>` with `expect()`
  - Only tests use `unwrap()`

- **No Retry on Transient Failures** - Now retries on 5xx, timeouts, connection errors

- **Unbounded Concurrency** - Now limits to 3 concurrent requests (HDD-safe)

- **Monolithic File** - Split 1035-line `scraper.rs` into 15+ focused modules

### 📦 Dependencies Added

```toml
[dependencies]
reqwest-middleware = "0.4"    # HTTP client middleware
reqwest-retry = "0.7"         # Retry logic
retry-policies = "0.4"        # Exponential backoff policy
once_cell = "1"               # Lazy statics
rand = "0.8"                  # Random user-agent selection
```

### 📦 Dependencies Changed

- `thiserror = "2"` - Already present, now fully utilized
- `anyhow = "1"` - Moved to application layer only

### 🧪 Testing

- **83 tests passing** (70 unit + 11 doctests + 2 integration)
- New tests for:
  - `ScraperError` variants
  - User-agent rotation
  - Lazy static initialization
  - Bounded concurrency
  - Clean Architecture layers

### 📚 Documentation

- **ARCHITECTURE.md** - Updated with Clean Architecture diagrams
- **CHANGELOG.md** - This file
- Module-level documentation for all layers
- Examples in public API docs

### 🔐 Security

- **User-Agent Rotation** - Reduces bot detection risk
- **Retry with Backoff** - Handles transient network failures gracefully
- **TLS Configuration** - rustls with system certificates

---

## [0.2.0] - 2026-03-07 - Asset Download & Modern Stack

### ✨ Added

1. **Asset Download**
   - `--download-images` - Download images to `output/images/`
   - `--download-documents` - Download documents to `output/documents/`
   - Automatic MIME type detection
   - File size limit (50MB max)
   - Timeout per download (30s)
   - Unique filenames based on SHA256 hash

2. **TLS Configuration**
   - System certificate support via rustls
   - Native roots for cross-platform compatibility

3. **Production Features**
   - Retry logic with exponential backoff
   - Bounded concurrency (3 for HDD)
   - User-Agent rotation pool

### 🔧 Changed

- **URL is now a required CLI argument** - No more hardcoded default URLs
- **Removed Brave Browser dependency** - Now uses pure HTTP client
- **Version bumped** from `0.1.x` to `0.2.0`

### 📦 Dependencies Updated

```toml
# Added
sha2 = "0.10"                 # File hashing for unique filenames
reqwest-middleware = "0.4"    # Retry middleware
reqwest-retry = "0.7"         # Retry logic
once_cell = "1"               # Lazy statics
rand = "0.8"                  # User-agent rotation

# Updated
reqwest = { version = "0.12", features = ["rustls-tls-native-roots", "gzip", "brotli"] }
```

### 🧪 Testing

- **70+ tests** covering all functionality
- Integration tests with real HTTP requests
- TempDir for isolated file operations

---

## [0.1.2] - Rust 2024 Edition

### Changed

- Updated to Rust Edition 2024
- Added unsafe block for `env::set_var()` to comply with Rust 2024

### Fixed

- Brave path correction on Linux (`/usr/bin/brave`)
- Type safety improvements in `get_brave_path()`

---

## [0.1.1] - Path Correction & Type Safety

### Fixed

- Corrected Brave path on Linux
- Improved type safety in return types

---

## [0.1.0] - Initial Refactor

### Added

- Complete rewrite with modular structure
- HTML to Markdown conversion
- Structured logging with tracing
- Custom error types with thiserror

### Fixed

- Cargo.toml edition error
- Unnecessary unsafe blocks
- Type mismatches in spider API
- Missing imports in dependencies

---

## 📌 Version History

| Version | Date | Status | Key Feature |
|---------|------|--------|-------------|
| [0.3.0] | 2026-03-08 | 🟢 Current | Clean Architecture |
| [0.2.0] | 2026-03-07 | 🟢 Previous | Asset Download + Production Features |
| [0.1.2] | 2024 | 🔵 Previous | Rust 2024 Edition |
| [0.1.1] | 2024 | 🔵 Previous | Path Correction |
| [0.1.0] | 2024 | 🔵 Previous | Initial Refactor |

---

## Migration Guide

### v0.2.0 → v0.3.0 (Breaking Changes)

#### For Library Users

**Before:**
```rust
use rust_scraper::{scraper, validate_and_parse_url};

let client = scraper::create_http_client()?;
let results = scraper::scrape_with_config(&client, &url, &config).await?;
```

**After:**
```rust
use rust_scraper::{create_http_client, scrape_with_config, validate_and_parse_url};

let client = create_http_client()?;
let results = scrape_with_config(&client, &url, &config).await?;
```

#### Error Handling

**Before:**
```rust
use rust_scraper::anyhow::Result;

fn scrape() -> Result<()> { ... }
```

**After:**
```rust
use rust_scraper::{Result, ScraperError};

fn scrape() -> Result<()> { ... }
// or
fn scrape() -> Result<(), ScraperError> { ... }
```

#### Match on Errors

**New capability in v0.3.0:**
```rust
use rust_scraper::{ScraperError, scrape_with_config};

match scrape_with_config(&client, &url, &config).await {
    Ok(results) => { /* success */ }
    Err(ScraperError::InvalidUrl(msg)) => { /* handle invalid URL */ }
    Err(ScraperError::Http { status, url }) => { /* handle HTTP error */ }
    Err(ScraperError::Network(e)) => { /* handle network error */ }
    Err(ScraperError::Readability(e)) => { /* handle parsing error */ }
    _ => { /* other errors */ }
}
```

### v0.1.x → v0.2.0+ (CLI Usage)

**Before (v0.1.x):**
```bash
cargo run  # Used hardcoded URL
```

**After (v0.2.0+):**
```bash
cargo run -- --url "https://example.com"
```

---

**Latest Version**: [0.3.0] - Clean Architecture  
**Rust Edition**: 2021  
**Status**: ✅ Production Ready  
**Tests**: 83 passing  
**Clippy**: ✅ Clean
