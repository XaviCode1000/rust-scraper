# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-03-08

### 🎉 Added - Production Ready Features

#### Core Functionality
- **Web Scraping**: Multi-threaded async web scraper with Tokio
- **Sitemap Support**: Zero-allocation streaming parser (quick-xml)
  - Gzip decompression (async-compression)
  - Sitemap index recursion (max depth 3)
  - Auto-discovery from robots.txt
- **TUI Interactivo**: Ratatui + crossterm URL selector
  - Interactive checkbox selection
  - Confirmation mode before download
  - Terminal restore on panic/exit

#### Architecture
- **Clean Architecture**: Domain → Application → Infrastructure → Adapters
- **Error Handling**: thiserror for libraries, anyhow for applications
- **Dependency Injection**: HTTP client, user agents, concurrency config

#### Performance
- **True Streaming**: Constant ~8KB RAM, no OOM
- **LazyLock**: Syntax highlighting cache (2-10ms → ~0.01ms)
- **Zero-Allocation Parsing**: quick-xml for sitemaps
- **Concurrent Downloads**: Bounded concurrency (configurable)

#### Security
- **SSRF Prevention**: URL host comparison (not string contains)
- **Windows Safe**: Reserved names blocked (CON, PRN, AUX → CON_safe, etc.)
- **WAF Bypass Prevention**: Chrome 131+ UAs with TTL caching
- **Input Validation**: url::Url::parse() (RFC 3986 compliant)

### 📦 Dependencies
- reqwest 0.12 (HTTP client)
- tokio (async runtime)
- scraper (HTML parsing)
- quick-xml 0.37 (XML streaming)
- async-compression 0.4 (gzip)
- ratatui 0.29 (TUI)
- crossterm 0.28 (terminal events)
- thiserror (error handling)
- clap (CLI)
- chrono, dirs, serde_json (UA caching)

### 🧪 Testing
- 198 unit + integration tests
- State-based TUI tests (no rendering)
- Clean Architecture compliance tests

### 📖 Documentation
- README.md with features and usage
- USAGE.md with examples
- API docs with # Examples sections

### 🔧 CI/CD
- GitHub Actions: build, test, clippy, fmt
- Auto-release on tag

## [0.4.0] - 2026-03-08

### Added
- TUI interactive selector
- Clean Architecture orchestration

## [0.3.0] - 2026-03-08

### Added
- Sitemap support with gzip
- Auto-discovery from robots.txt

## [0.2.0] - 2026-03-07

### Added
- Clean Architecture migration
- Domain/Application/Infrastructure layers

## [0.1.0] - 2026-03-06

### Added
- Initial release
- Basic web scraping
