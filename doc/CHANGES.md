# 📋 CHANGELOG - Rust Scraper

All notable changes to this project will be documented in this file.

---

## v0.3.0 - Asset Download & TLS Improvements (Current)

### 🆕 New Features

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
   - No external CA bundle dependencies

### 🔧 Improvements

| Before | After |
|--------|-------|
| No asset download | Automatic image/document download |
| No TLS configuration | System certificate support |
| Fixed filenames | Unique hash-based filenames |
| No MIME detection | Automatic MIME type detection |

### 📦 Dependencies Updated

```toml
# Added
sha2 = "0.10"  # File hashing for unique filenames
mime = "0.3"   # MIME type detection

# Updated
reqwest = { version = "0.12", features = ["rustls-tls-native-roots", "gzip", "brotli"] }
```

### 🧪 Testing

- **New tests** covering:
  - Asset download functionality
  - MIME type detection
  - File naming with hash
  - File size validation
  - Timeout handling

### 📊 Validation

```
✅ cargo build --release  # Compiles
✅ cargo test             # All tests passing
✅ cargo clippy           # No warnings
```

### 🏗️ Usage Examples

```bash
# Download images only
cargo run --release -- --url "https://example.com" --download-images

# Download documents only
cargo run --release -- --url "https://example.com" --download-documents

# Download both images and documents
cargo run --release -- --url "https://example.com" --download-images --download-documents

# Custom output directory
cargo run --release -- --url "https://example.com" --download-images -o ./my-downloads
```

### 📁 Output Structure

```
output/
├── example.com/
│   └── index.md
├── images/
│   ├── 027e504eabfc.png
│   ├── 0c2f4f0301fe.png
│   └── e15cbdd2d653.svg
└── documents/
    └── 9870371a7a8c.pdf
```

---

## v0.2.0 - Modern Scraper Stack (Major Refactor)

### ⚠️ Breaking Changes

- **URL is now a required CLI argument** - No more hardcoded default URLs
- **Removed Brave Browser dependency** - Now uses pure HTTP client

### 🆕 New Features

1. **CLI with clap**
   - URL is required: `--url` or `-u`
   - Configurable output format: `--format` (markdown/json/text)
   - Configurable output directory: `--output` or `-o`
   - Verbose logging: `-v` or `--verbose`

2. **Readability Algorithm (legible)**
   - Extracts clean content like Firefox Reader Mode
   - Filters out: navigation, ads, sidebars, footer content, scripts
   - Preserves: article title, byline, excerpt, main content

3. **Fallback HTML→Markdown (htmd)**
   - Used when readability fails
   - Proper conversion of HTML elements to Markdown

4. **Modern HTTP Client (reqwest)**
   - TLS support with rustls
   - Gzip/Brotli compression
   - Configurable timeouts

### 🔧 Improvements

| Before | After |
|--------|-------|
| Hardcoded URL | CLI required argument |
| spider + Brave | reqwest + legible |
| Naive replace() | Readability algorithm |
| No CLI | Full clap CLI |
| No tests | 38 tests (30 unit + 8 integration) |

### 📦 Dependencies Updated

```toml
# Added
clap = { version = "4", features = ["derive"] }  # CLI
reqwest = { version = "0.12", features = ["rustls-tls", "gzip", "brotli"] }  # HTTP
legible = "0.4"  # Readability
htmd = "0.5"     # HTML→Markdown
serde = { version = "1", features = ["derive"] }  # Serialization

# Removed
spider = { version = "2", features = ["chrome"] }  # No longer needed
supermarkdown = "0.0.5"  # Replaced by htmd
thiserror = "1"  # Using anyhow instead
```

### 🧪 Testing

- **30 unit tests** covering:
  - URL validation (14 tests)
  - HTTP client creation
  - Fallback HTML parsing
  - Save results (Markdown, JSON, Text)
  - Logging initialization

- **8 integration tests** covering:
  - Full scraping pipeline
  - Error handling (404, invalid URL)
  - Special characters handling

### 🔨 Code Quality

- Fixed: `anti-unwrap-abuse` - No more silent fallbacks
- Fixed: Removed unnecessary `unsafe` blocks
- Fixed: Proper error propagation with anyhow
- Added: Comprehensive documentation

### 📊 Validation

```
✅ cargo build --release  # Compiles
✅ cargo test             # 38 tests passing
✅ cargo clippy           # No warnings
```

### 🏗️ Migration from v0.1.x

```bash
# Before (v0.1.x)
cargo run

# After (v0.2.0+)
cargo run -- --url "https://example.com"
```

---

## v0.1.2 - Rust 2024 Edition

### Changes

- Updated to Rust Edition 2024 (edition = "2021" in Cargo.toml)
- Added unsafe block for env::set_var() to comply with Rust 2024

### Fixed

- Brave path correction on Linux (`/usr/bin/brave`)
- Type safety improvements in get_brave_path()

---

## v0.1.1 - Path Correction & Type Safety

### Fixed

- Corrected Brave path on Linux
- Improved type safety in return types

---

## v0.1.0 - Initial Refactor

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

| Version | Date | Status |
|---------|------|--------|
| v0.3.0 | 2026-03 | 🟢 Current |
| v0.2.0 | 2026-03 | 🟢 Previous |
| v0.1.2 | 2024 | 🔵 Previous |
| v0.1.1 | 2024 | 🔵 Previous |
| v0.1.0 | 2024 | 🔵 Previous |

---

**Latest Version**: v0.3.0
**Rust Edition**: 2021
**Status**: ✅ Production Ready