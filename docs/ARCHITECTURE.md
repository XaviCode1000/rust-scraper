# Architecture

## Overview

The rust-scraper follows **Clean Architecture** with clear separation of concerns and dependency rule (dependencies point inward):

```
┌─────────────────────────────────────────────────────────┐
│                    CLI (main.rs)                        │
│  - Argument parsing with clap                           │
│  - Orchestration of workflow                            │
│  - Logging initialization                               │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│                  Library (lib.rs)                       │
│  - Public API re-exports                                │
│  - ScraperConfig, Args, OutputFormat                    │
│  - validate_and_parse_url()                             │
└────────────────────┬────────────────────────────────────┘
                     │
     ┌───────────────┴────────────────┐
     │                                │
┌────▼──────────┐            ┌────────▼────────┐
│   DOMAIN      │            │  APPLICATION    │
│   (pure)      │            │  (use cases)    │
│               │            │                 │
│ - entities    │            │ - http_client   │
│ - value_objs  │            │ - scraper_svc   │
└───────────────┘            └─────────────────┘
                                      │
                    ┌─────────────────┼─────────────────┐
                    │                 │                 │
             ┌──────▼──────┐  ┌──────▼──────┐  ┌──────▼──────┐
             │INFRASTRUCTURE│  │  ADAPTERS   │  │   OUTPUT    │
             │             │  │             │  │             │
             │ - http      │  │ - detector  │  │ - file_saver│
             │ - scraper   │  │ - extractor │  │ - frontmatter│
             │ - converter │  │ - downloader│  │             │
             └─────────────┘  └─────────────┘  └─────────────┘
```

## Clean Architecture Layers

### Domain Layer (`src/domain/`)

**Pure business logic** - no external dependencies (no reqwest, no tokio, no serde frameworks).

#### `entities.rs`
Core business entities:
- `ScrapedContent` - Main output type with title, content, URL, metadata, assets
- `DownloadedAsset` - Downloaded image/document with URL, local path, size

#### `value_objects.rs`
Type-safe primitives:
- `ValidUrl` - Newtype around `url::Url` guaranteeing validity at type level
  - Prevents invalid URLs at compile time
  - Self-documenting APIs
  - No runtime validation needed after construction

### Application Layer (`src/application/`)

**Use cases and orchestration** - depends on domain, not on infrastructure.

#### `http_client.rs`
HTTP client creation with production features:
- User-Agent rotation (14 modern browsers, weighted selection)
- Exponential backoff retry (3 retries, 100ms→200ms→400ms)
- Gzip/Brotli compression
- 30s timeout
- TLS via rustls with system certificates

#### `scraper_service.rs`
Main scraping orchestration:
- `scrape_with_readability()` - Clean content extraction
- `scrape_with_config()` - Scraping with asset download options
- `scrape_multiple_with_limit()` - Bounded concurrency (3 for HDD systems)
- Error handling with `ScraperError` type

### Infrastructure Layer (`src/infrastructure/`)

**Technical implementations** - depends on domain, implements application interfaces.

#### `http/`
HTTP client infrastructure (re-exports from application).

#### `scraper/`
- `readability.rs` - legible crate wrapper for content extraction
- `fallback.rs` - htmd fallback when Readability fails
- `asset_download.rs` - Image/document downloading with SHA256 hashing

#### `converter/`
- `html_to_markdown.rs` - HTML→Markdown with html-to-markdown-rs
- `syntax_highlight.rs` - Code block highlighting with syntect

#### `output/`
- `file_saver.rs` - Save results (Markdown/Text/JSON) with domain-based folders
- `frontmatter.rs` - YAML frontmatter generation with metadata

### Adapters Layer (`src/adapters/`)

**External integrations** - feature-gated, optional functionality.

#### `detector/`
MIME type detection and asset classification:
- `detect_from_url()` - Classify by file extension
- `detect_from_path()` - Path-based detection
- `AssetType` enum (Image, Document, Unknown)
- `get_extension()` - Extract extension from URL

#### `extractor/`
URL extraction from HTML:
- `extract_images()` - Find `<img>`, `<picture>`, `<source>` tags
- `extract_documents()` - Find links to PDF, DOCX, XLSX, etc.
- CSS selectors compiled once with `once_cell::Lazy`

#### `downloader/`
Asset downloading (feature-gated):
- Bounded concurrency (3 concurrent downloads)
- SHA256 content hashing for unique filenames
- File size validation (50MB max)
- Timeout handling (30s per download)

## Data Flow

### Content Scraping (Clean Architecture)

```
URL Input
    │
    ▼
┌─────────────┐
│ Application │  validate_and_parse_url()
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Application │  create_http_client() + retry middleware
└──────┬──────┘
       │
       ▼
┌─────────────┐
│Infrastructure│  reqwest HTTP fetch
└──────┬──────┘
       │
       ▼
┌─────────────┐
│Infrastructure│  legible::parse() (Readability)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│Infrastructure│  html_to_markdown::convert()
└──────┬──────┘
       │
       ▼
┌─────────────┐
│Infrastructure│  syntax_highlight::highlight()
└──────┬──────┘
       │
       ▼
┌─────────────┐
│Infrastructure│  frontmatter::generate()
└──────┬──────┘
       │
       ▼
┌─────────────┐
│Infrastructure│  file_saver::save_results()
└─────────────┘
```

### Asset Download Flow

```
HTML Content
    │
    ▼
┌─────────────┐
│  Adapters   │  extractor::extract_images()
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Adapters   │  detector::detect_from_url()
└──────┬──────┘
       │
       ▼
┌─────────────┐
│Infrastructure│  asset_download::download_all()
└──────┬──────┘
       │
       ▼
┌─────────────┐
│Infrastructure│  SHA256 hash + file save
└─────────────┘
```

## Design Decisions

### Why Clean Architecture?

1. **Separation of Concerns** - Domain logic isolated from frameworks
2. **Testability** - Mock infrastructure, test domain/application in isolation
3. **Maintainability** - Changes to HTTP client don't affect domain entities
4. **Reusability** - Domain entities usable in different contexts (CLI, web API, library)

### Why `ValidUrl` Newtype?

Instead of `String` or raw `url::Url`:
- **Type Safety** - Can't accidentally pass invalid URL
- **Self-Documenting** - API signature guarantees validity
- **Compile-Time Validation** - Errors caught early

### Why Bounded Concurrency?

Hardware-aware design for target system (Intel i5-4590, 8GB RAM, HDD):
- **Prevents FD Exhaustion** - 100 URLs ≠ 100 open files
- **Avoids HDD Thrashing** - Sequential writes on mechanical drives
- **Reduces Bot Detection** - Doesn't look like DDoS

### Why Retry with Exponential Backoff?

Production resilience:
- **Handles Transient Failures** - 5xx errors, timeouts, connection resets
- **Respectful** - Backoff prevents hammering servers
- **User-Friendly** - Scraping succeeds despite network hiccups

### Why User-Agent Rotation?

Anti-bot evasion:
- **14 Modern Browsers** - Chrome (40%), Firefox (20%), Safari (20%), Edge (20%)
- **Weighted Selection** - Mimics real traffic distribution
- **Per-Request Rotation** - No patterns for detection

### Why `once_cell::Lazy` for CSS Selectors?

- **Compile Once** - `Selector::parse()` is expensive
- **No unwrap() in Prod** - `expect()` with clear error message
- **Thread-Safe** - Static initialization

## Dependencies by Layer

### Domain
- `serde` (derive only) - For serialization
- `url` - URL parsing (minimal dependency)

### Application
- `reqwest-middleware` - HTTP client with retry
- `reqwest-retry` - Exponential backoff
- `futures` - Stream utilities for concurrency

### Infrastructure
- `reqwest` - HTTP client
- `legible` - Readability algorithm
- `html-to-markdown-rs` - HTML→Markdown
- `syntect` - Syntax highlighting
- `serde_yaml` - YAML frontmatter
- `chrono` - Date formatting
- `sha2` - Content hashing

### Adapters
- `scraper` - HTML parsing (CSS selectors)
- `mimetype-detector` (optional) - MIME detection from bytes
- `rand` - Random selection

## Testing Strategy

### Unit Tests
- **Domain** - Entity creation, value object validation
- **Application** - HTTP client creation, service orchestration
- **Infrastructure** - Converter tests, file saver tests
- **Adapters** - Extractor tests, detector tests

### Integration Tests
- Full scraping pipeline (real HTTP requests)
- Error handling (404, invalid URL, timeout)
- Asset download (with real files)

### Test Isolation
- `TempDir` for file operations
- No cross-test state
- Mock HTTP not needed (tests are fast enough)

## Performance Considerations

1. **Async I/O** - Tokio runtime for non-blocking operations
2. **Connection Pooling** - Reqwest reuses connections
3. **Compression** - Gzip/Brotli support reduces bandwidth
4. **Bounded Concurrency** - Prevents resource exhaustion
5. **Retry Backoff** - Reduces server load on failures
6. **Lazy Statics** - CSS selectors compiled once
7. **SHA256 Hashing** - Fast unique filenames

## Module Dependency Graph

```
main.rs
  │
  ▼
lib.rs ───────────────┐
  │                   │
  ▼                   │
domain ◄──────────────┘
  │
  ▼
application
  │
  ├──────► infrastructure
  │
  └──────► adapters
```

**Dependency Rule**: Dependencies point inward. Domain knows nothing about infrastructure or adapters.
