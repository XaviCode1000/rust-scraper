# Architecture

## Overview

The rust-scraper follows a layered architecture with clear separation of concerns:

```
┌─────────────────────────────────────────┐
│              CLI (main.rs)              │
│  - Argument parsing with clap           │
│  - Orchestration of workflow           │
│  - Asset download coordination         │
└────────────────┬──────────────────────┘
                 │
┌────────────────▼──────────────────────┐
│           Library (lib.rs)              │
│  - Public API re-exports               │
│  - OutputFormat enum                   │
│  - Args struct                         │
└────────────────┬──────────────────────┘
                 │
     ┌────────────┴────────────┐
     │                         │
┌───▼────────────┐  ┌────────▼─────────┐
│   scraper.rs    │  │   downloader.rs  │
│                │  │                  │
│ - HTTP client  │  │ - Image download │
│ - Readability  │  │ - Document download│
│ - HTML→MD     │  │ - MIME detection │
│ - Saving      │  │ - File naming    │
└───────────────┘  └──────────────────┘
     │
     ▼
┌─────────────────────────────────────┐
│         detector.rs                 │
│                                     │
│ - MIME type detection               │
│ - Asset classification              │
│ - File extension extraction         │
└─────────────────────────────────────┘
```

## Core Modules

### scraper.rs

The main scraping engine:

1. **HTTP Client** - Uses reqwest with:
   - Custom User-Agent
   - Gzip/Brotli compression
   - 30s timeout
   - TLS support via rustls with system certificates

2. **Content Extraction** - Two-layer approach:
   - **Primary**: legible (Readability algorithm)
   - **Fallback**: htmd for basic HTML stripping

3. **Markdown Conversion** - Uses html-to-markdown-rs:
   - Preserves heading hierarchy (h1-h6)
   - Code blocks with language detection
   - Lists (ordered/unordered)
   - Emphasis (bold, italic)
   - Links and images

4. **Output Generation**:
   - YAML frontmatter with metadata
   - Domain-based folder structure
   - URL-based file naming

### downloader.rs

Handles downloading of images and documents:

1. **Download Configuration**:
   - Output directory for downloaded files
   - Separate subdirectories for images and documents
   - Maximum file size limit (50MB)
   - Timeout per download (30s)

2. **Asset Classification**:
   - MIME type detection from headers
   - Automatic routing to appropriate subdirectory
   - File extension extraction from URL

3. **File Naming**:
   - Unique filenames based on SHA256 hash
   - Preserves original file extension
   - Prevents filename collisions

4. **Error Handling**:
   - File size validation
   - Download timeout handling
   - Network error propagation

### detector.rs

Provides MIME type detection and asset classification:

1. **MIME Type Detection**:
   - Uses content-type headers from HTTP response
   - Maps common MIME types to categories
   - Fallback to file extension detection

2. **Asset Classification**:
   - Image types: png, jpg, jpeg, gif, svg, webp, etc.
   - Document types: pdf, doc, docx, xls, xlsx, ppt, pptx, etc.
   - Other types: bin, unknown

3. **File Extension Extraction**:
   - Parses URL path for extension
   - Handles query parameters
   - Provides fallback extension

## Data Flow

### Content Scraping

```
URL Input
    │
    ▼
┌─────────────┐
│ Validation  │  url::Url parsing
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ HTTP Fetch  │  reqwest client with TLS
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Readability │  legible crate
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Markdown    │  html-to-markdown-rs
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Frontmatter │  serde_yaml
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ File Save   │  std::fs
└─────────────┘
```

### Asset Download

```
URL Input
    │
    ▼
┌─────────────┐
│ HTTP Fetch  │  reqwest client
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ MIME Detect │  detector::detect_from_url
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Classify    │  Image vs Document
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Size Check  │  Max 50MB validation
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Hash File   │  SHA256 content hash
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Save File   │  output/images/ or output/documents/
└─────────────┘
```

## Design Decisions

### Why Readability?

The Readability algorithm (used by Firefox Reader Mode, Pocket, Instapaper) is specifically designed to extract the main content from a web page while filtering out:
- Navigation menus
- Advertisements
- Sidebars
- Footer content
- Scripts and styles

This makes it ideal for RAG pipelines where clean, relevant content is essential.

### Type-Safe URL Handling

Instead of using raw `String` for paths, we use newtypes:
- Prevents invalid filenames
- Validates at construction time
- Makes APIs self-documenting

### Why html-to-markdown-rs?

Compared to alternatives:
- Preserves heading hierarchy
- Supports code blocks with language hints
- Actively maintained (v2.28.0 in 2026)
- Rich configuration options

### Why Separate Download Module?

1. **Separation of Concerns** - Scraping vs downloading are distinct operations
2. **Reusability** - Downloader can be used independently
3. **Testability** - Easier to test download logic separately
4. **Configuration** - Different settings for downloads vs scraping

### TLS Configuration

Uses rustls with native roots:
- **Security**: Modern TLS implementation
- **Compatibility**: System certificate store
- **No External Dependencies**: No need for CA bundles
- **Cross-Platform**: Works on Linux, macOS, Windows

## Dependencies

### Core
- **reqwest** - HTTP client with TLS support
- **legible** - Readability algorithm
- **html-to-markdown-rs** - HTML→Markdown conversion

### CLI
- **clap** - Argument parsing

### Output
- **serde_yaml** - YAML frontmatter
- **chrono** - Date formatting
- **syntect** - Syntax highlighting

### Asset Download
- **sha2** - File hashing for unique filenames
- **mime** - MIME type detection

### Testing
- **mockall** - Mocking framework
- **tokio-test** - Async testing utilities
- **tempfile** - Temporary directory management
- **walkdir** - Directory traversal

## Testing Strategy

- **Unit tests** - Individual functions
- **Integration tests** - Full workflow
- **TempDir** - Isolated file operations
- **walkdir** - Verify nested output structure
- **Mock HTTP responses** - Test download logic without network

## Performance Considerations

1. **Async I/O** - Non-blocking downloads
2. **Connection Pooling** - Reuse HTTP connections
3. **Compression** - Gzip/Brotli support
4. **File Hashing** - SHA256 for unique filenames
5. **Timeouts** - Prevent hanging downloads