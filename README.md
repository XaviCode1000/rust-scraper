# 🕷️ Rust Scraper

**Production-ready web scraper with Clean Architecture, TUI selector, and sitemap support.**

[![Build Status](https://github.com/XaviCode1000/rust-scraper/actions/workflows/ci.yml/badge.svg)](https://github.com/XaviCode1000/rust-scraper/actions)
[![Tests](https://img.shields.io/badge/tests-198%20passing-brightgreen)](https://github.com/XaviCode1000/rust-scraper)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)](https://www.rust-lang.org)
[![Version](https://img.shields.io/badge/version-1.0.0-blue)](https://github.com/XaviCode1000/rust-scraper/releases)

## ✨ Features

### 🚀 Core
- **Async Web Scraping**: Multi-threaded with Tokio runtime
- **Sitemap Support**: Zero-allocation streaming parser
  - Gzip decompression (`.xml.gz`)
  - Sitemap index recursion (max depth 3)
  - Auto-discovery from `robots.txt`
- **TUI Interactivo**: Select URLs before downloading
  - Checkbox selection (`[✅]` / `[⬜]`)
  - Keyboard navigation (↑↓, Space, Enter)
  - Confirmation mode (Y/N)

### 🏗️ Architecture
- **Clean Architecture**: Domain → Application → Infrastructure → Adapters
- **Error Handling**: `thiserror` for libraries, `anyhow` for applications
- **Dependency Injection**: HTTP client, user agents, concurrency config

### ⚡ Performance
- **True Streaming**: Constant ~8KB RAM, no OOM
- **Zero-Allocation Parsing**: `quick-xml` for sitemaps
- **LazyLock Cache**: Syntax highlighting (2-10ms → ~0.01ms)
- **Bounded Concurrency**: Configurable parallel downloads

### 🔒 Security
- **SSRF Prevention**: URL host comparison (not string contains)
- **Windows Safe**: Reserved names blocked (`CON` → `CON_safe`)
- **WAF Bypass Prevention**: Chrome 131+ UAs with TTL caching
- **RFC 3986 URLs**: `url::Url::parse()` validation

## 📦 Installation

### From Source

```bash
git clone https://github.com/XaviCode1000/rust-scraper.git
cd rust-scraper
cargo build --release
```

The binary will be available at `target/release/rust_scraper`.

### From Cargo (coming soon)

```bash
cargo install rust_scraper
```

## 🚀 Usage

### Basic (Headless Mode)

```bash
# Scrape all URLs from a website
./target/release/rust_scraper --url https://example.com

# With sitemap (auto-discovers from robots.txt)
./target/release/rust_scraper --url https://example.com --use-sitemap

# Explicit sitemap URL
./target/release/rust_scraper --url https://example.com \
  --use-sitemap \
  --sitemap-url https://example.com/sitemap.xml.gz
```

### Interactive Mode (TUI)

```bash
# Select URLs interactively before downloading
./target/release/rust_scraper --url https://example.com --interactive

# With sitemap
./target/release/rust_scraper --url https://example.com \
  --interactive \
  --use-sitemap
```

### TUI Controls

| Key | Action |
|-----|--------|
| `↑↓` | Navigate URLs |
| `Space` | Toggle selection |
| `A` | Select all |
| `D` | Deselect all |
| `Enter` | Confirm download |
| `Y/N` | Final confirmation |
| `q` | Quit |

### Advanced Options

```bash
# Full example with all options
./target/release/rust_scraper \
  --url https://example.com \
  --output ./output \
  --format markdown \
  --download-images \
  --download-documents \
  --use-sitemap \
  --concurrency 5 \
  --delay-ms 1000 \
  --max-pages 100 \
  --verbose
```

### Get Help

```bash
./target/release/rust_scraper --help
```

## 📖 Documentation

- [**Usage Guide**](docs/USAGE.md) - Detailed examples and troubleshooting
- [**Architecture**](docs/ARCHITECTURE.md) - Clean Architecture details
- [**API Docs**](https://docs.rs/rust_scraper) - Rust documentation

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_validate_and_parse_url
```

**Tests:** 198 passing ✅

## 🏗️ Architecture

```
Domain (entities, errors)
    ↓
Application (services, use cases)
    ↓
Infrastructure (HTTP, parsers, converters)
    ↓
Adapters (TUI, CLI, detectors)
```

**Dependency Rule:** Dependencies point inward. Domain never imports frameworks.

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for detailed architecture documentation.

## 🔧 Development

### Requirements

- Rust 1.75+
- Cargo

### Build

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

### Lint

```bash
# Run Clippy (deny warnings)
cargo clippy -- -D warnings

# Check formatting
cargo fmt --all -- --check
```

### Run

```bash
# Run in debug mode
cargo run -- --url https://example.com

# Run in release mode
cargo run --release -- --url https://example.com
```

## 📄 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## 🙏 Acknowledgments

- Built with [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html) principles
- Inspired by [ripgrep](https://github.com/BurntSushi/ripgrep) performance patterns
- Uses [rust-skills](https://github.com/leonardomso/rust-skills) (179 rules)

## 📊 Stats

- **Lines of Code:** ~4000+
- **Tests:** 198 passing
- **Coverage:** High (state-based testing)
- **MSRV:** 1.75.0

## 🗺️ Roadmap

- [ ] v1.1.0: Multi-domain crawling
- [ ] v1.2.0: JavaScript rendering (headless browser)
- [ ] v2.0.0: Distributed scraping

---

**Made with ❤️ using Rust and Clean Architecture**
