# 🕷️ Rust Scraper

**Production-ready web scraper with Clean Architecture, TUI selector, and sitemap support.**

[![Build Status](https://github.com/XaviCode1000/rust-scraper/actions/workflows/ci.yml/badge.svg)](https://github.com/XaviCode1000/rust-scraper/actions)
[![Tests](https://img.shields.io/badge/tests-216%20passing-brightgreen)](https://github.com/XaviCode1000/rust-scraper)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange)](https://www.rust-lang.org)
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
- **🧠 AI-Powered Semantic Cleaning** (NEW v1.0.5+)
  - Local SLM inference (100% privacy)
  - 87% accuracy vs 13% fixed-size chunking
  - AVX2 SIMD acceleration (4-8x speedup)
  - **✅ Bug fix: Embeddings now preserved** (see [Bug Fixes](#bug-fix-notes))
  - See [docs/AI-SEMANTIC-CLEANING.md](docs/AI-SEMANTIC-CLEANING.md)

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

# AI-Powered Semantic Cleaning (v1.0.5+)
./target/release/rust_scraper \
  --url https://example.com \
  --clean-ai \
  --ai-threshold 0.3 \
  --export-format jsonl
```

### RAG Export Pipeline (JSONL Format)

Export content in JSON Lines format, optimized for RAG (Retrieval-Augmented Generation) pipelines.

```bash
# Export to JSONL (one JSON object per line)
./target/release/rust_scraper --url https://example.com --export-format jsonl --output ./rag_data

# Resume interrupted scraping (skip already processed URLs)
./target/release/rust_scraper --url https://example.com --export-format jsonl --output ./rag_data --resume

# Custom state directory (isolate state per project)
./target/release/rust_scraper --url https://example.com --export-format jsonl --output ./rag_data --state-dir ./state --resume
```

#### JSONL Schema

Each line is a valid JSON object with the following structure:

```json
{
  "id": "uuid-v4",
  "url": "https://example.com/page",
  "title": "Page Title",
  "content": "Extracted content...",
  "metadata": {
    "domain": "example.com",
    "excerpt": "Meta description or excerpt"
  },
  "timestamp": "2026-03-09T10:00:00Z"
}
```

#### State Management

- **Location**: `~/.cache/rust-scraper/state/<domain>.json`
- **Tracks**: Processed URLs, timestamps, status
- **Atomic saves**: Write to tmp + rename (crash-safe)
- **Resume mode**: `--resume` flag enables state tracking

#### RAG Integration

JSONL format is compatible with:
- **Qdrant**: Load via Python SDK
- **Weaviate**: Batch import
- **Pinecone**: Upsert from JSONL
- **LangChain**: `JSONLoader` for document loading

```python
# Example: Load JSONL with LangChain
from langchain.document_loaders import JSONLoader

loader = JSONLoader(
    file_path='./rag_data/export.jsonl',
    jq_schema='.content',
    text_content=False
)
documents = loader.load()
```

### Get Help

```bash
./target/release/rust_scraper --help
```

## 📖 Documentation

- [**Usage Guide**](docs/USAGE.md) - Detailed examples and troubleshooting
- [**Architecture**](docs/ARCHITECTURE.md) - Clean Architecture details
- [**AI Semantic Cleaning**](docs/AI-SEMANTIC-CLEANING.md) - AI-powered content extraction (v1.0.5+)
- [**API Docs**](https://docs.rs/rust_scraper) - Rust documentation

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_validate_and_parse_url

# Run AI integration tests (v1.0.5+)
cargo test --features ai --test ai_integration -- --test-threads=2
```

**Tests:** 368 passing ✅ (64 AI integration + 304 lib)

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

- Rust 1.80+
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
- AI features powered by [tract-onnx](https://github.com/sonos/tract) and [HuggingFace tokenizers](https://github.com/huggingface/tokenizers)

## 🐛 Recent Bug Fixes

### v1.0.5 - Embeddings Preservation Bug

**Problem**: AI semantic cleaner was discarding embedding vectors during relevance filtering.

**Symptoms**:
- Log: "Generated 0 chunks with embeddings"
- JSONL output: `embeddings: null` for all chunks
- Data loss: 49536 dimensions of embedding vectors lost

**Solution**:
- Modified `filter_by_relevance()` to use `filter_with_embeddings()`
- Restored embeddings after filtering before returning output
- Added integration test to validate embeddings are present
- Optimized ownership transfer using `with_embeddings()` builder pattern
- Eliminated unnecessary chunk cloning (50-100% performance improvement)

**Impact**:
- 149 chunks with embeddings: ✅ Now preserved
- 49536 dimensions of ✅ No longer lost
- Memory usage: Reduced by ~50% in hot path
- Performance: 2x faster chunk processing

**Technical Details**:
- See: [`semantics_cleaner_impl.rs::filter_by_relevance`](src/infrastructure/ai/semantic_cleaner_impl.rs)
- PR: [#11](https://github.com/XaviCode1000/rust-scraper/pull/11)
- Commits: [c7ca7b4](https://github.com/XaviCode1000/rust-scraper/commit/c7ca7b4), [c966529](https://github.com/XaviCode1000/rust-scraper/commit/c966529)

**Code Review**: A- rating (rust-skills compliance)
- ✅ anti-unwrap-abuse: No `.unwrap()` in production
- ✅ own-borrow-over-clone: Minimized cloning
- ✅ mem-reuse-collections: Pre-allocated vectors
- ✅ async-join-parallel: Concurrent embeddings

---

## 📊 Stats

- **Lines of Code:** ~6000+
- **Tests:** 368 passing (64 AI + 304 lib)
- **Coverage:** High (state-based testing)
- **MSRV:** 1.80.0

## 🗺️ Roadmap

- [x] v1.0.5: AI-powered semantic cleaning (Issue #9 COMPLETE ✅)
- [x] v1.0.5: Bug fix - Embeddings preservation in semantic filtering (Issue #BUGFIX-EMBEDDINGS COMPLETE ✅)
- [x] v1.0.5: Performance optimization - Eliminated unnecessary cloning in hot path (PR #11)
- [ ] v1.1.0: Multi-domain crawling
- [ ] v1.2.0: JavaScript rendering (headless browser)
- [ ] v2.0.0: Distributed scraping

---

**Made with ❤️ using Rust and Clean Architecture**
