# CLI Reference — rust-scraper

**Version:** 1.0.0  
**MSRV:** 1.80  
**Last Updated:** 2026-03-11

---

## Quick Start

```bash
# Basic scraping (default: markdown output, 10 pages, 1s delay)
cargo run -- --url "https://example.com"

# Scrape with JSON output
cargo run -- --url "https://example.com" -f json

# Scrape for RAG pipeline (JSONL format)
cargo run -- --url "https://example.com" --export-format jsonl

# Interactive mode with TUI selector
cargo run -- --url "https://example.com" --interactive
```

---

## Usage

```bash
rust-scraper [OPTIONS] --url <URL>
```

---

## Required Arguments

| Flag | Description | Required |
|------|-------------|----------|
| `-u, --url <URL>` | Target URL to scrape (must include `http://` or `https://`) | ✅ Yes |

**Example:**
```bash
cargo run -- --url "https://example.com"
```

---

## Output Options

### Individual File Output (`-f, --format`)

Creates separate output files per scraped page — ideal for human-readable output.

| Flag | Values | Default | Description |
|------|--------|---------|-------------|
| `-f, --format <FORMAT>` | `markdown`, `json`, `text` | `markdown` | Output format for individual files |

**Formats:**

| Format | Description | Use Case |
|--------|-------------|----------|
| `markdown` | Markdown with YAML frontmatter | RAG, documentation, human-readable |
| `json` | Structured JSON with metadata | Programmatic processing |
| `text` | Plain text without formatting | Simple text extraction |

**Example:**
```bash
# Markdown (default)
cargo run -- --url "https://example.com" -f markdown

# JSON output
cargo run -- --url "https://example.com" -f json

# Plain text
cargo run -- --url "https://example.com" -f text
```

### RAG Pipeline Export (`--export-format`)

Creates batch export suitable for LLM/RAG pipelines.

| Flag | Values | Default | Description |
|------|--------|---------|-------------|
| `--export-format <FORMAT>` | `jsonl`, `auto` | `jsonl` | Export format for RAG pipeline |

**Formats:**

| Format | Description | Feature Required |
|--------|-------------|------------------|
| `jsonl` | JSON Lines (one JSON per line), optimal for RAG | None (default) |
| `auto` | Auto-detect from existing export files | None |

**Example:**
```bash
# JSONL export (default)
cargo run -- --url "https://example.com" --export-format jsonl

# Auto-detect format
cargo run -- --url "https://example.com" --export-format auto
```

### Output Directory (`-o, --output`)

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `-o, --output <DIR>` | Path | `output` | Output directory for scraped content |

**Example:**
```bash
cargo run -- --url "https://example.com" -o ./my-scrapes
```

---

## Scraping Options

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `-s, --selector <SELECTOR>` | String | `body` | CSS selector for content extraction |
| `--max-pages <N>` | Integer | `10` | Maximum pages to scrape |
| `--delay-ms <MS>` | Integer | `1000` | Delay between requests in milliseconds |
| `--concurrency <VALUE>` | `auto` or Integer | `auto` | Concurrency level (parallel requests) |

### CSS Selector (`-s, --selector`)

Extract specific content using CSS selectors:

```bash
# Extract only article content
cargo run -- --url "https://example.com" -s "article"

# Extract main content by ID
cargo run -- --url "https://example.com" -s "#main-content"

# Extract by class
cargo run -- --url "https://example.com" -s ".content-body"
```

### Page Limit (`--max-pages`)

Control how many pages to scrape:

```bash
# Scrape only 5 pages
cargo run -- --url "https://example.com" --max-pages 5

# Scrape up to 100 pages
cargo run -- --url "https://example.com" --max-pages 100
```

### Request Delay (`--delay-ms`)

Rate limiting to avoid overwhelming servers:

```bash
# 2 second delay between requests
cargo run -- --url "https://example.com" --delay-ms 2000

# Fast scraping (500ms delay)
cargo run -- --url "https://example.com" --delay-ms 500
```

### Concurrency (`--concurrency`)

Hardware-aware concurrency control:

| Value | Description |
|-------|-------------|
| `auto` (default) | Auto-detect based on CPU cores |
| `1-16` | Explicit concurrency value |

**Auto-detection logic:**
- 1-2 cores: 1 worker
- 3-4 cores: 3 workers (HDD-aware default)
- 5-7 cores: 5 workers
- 8+ cores: `min(cores - 1, 8)` workers

**Example:**
```bash
# Auto-detect (default)
cargo run -- --url "https://example.com" --concurrency auto

# Explicit concurrency
cargo run -- --url "https://example.com" --concurrency 5

# Single-threaded (safe for slow networks)
cargo run -- --url "https://example.com" --concurrency 1
```

---

## Asset Download

| Flag | Default | Description |
|------|---------|-------------|
| `--download-images` | `false` | Download images (PNG, JPG, GIF, WEBP, SVG, BMP) |
| `--download-documents` | `false` | Download documents (PDF, DOCX, XLSX, PPTX, etc.) |

**Feature Requirements:**
- Requires `--features images` for `--download-images`
- Requires `--features documents` for `--download-documents`
- Or use `--features full` for all features

**Example:**
```bash
# Download images only
cargo run --features images -- --url "https://example.com" --download-images

# Download documents only
cargo run --features documents -- --url "https://example.com" --download-documents

# Download both
cargo run --features full -- --url "https://example.com" --download-images --download-documents
```

**Output Structure:**
```
output/
├── images/
│   ├── 027e504eabfc.png
│   ├── 0c2f4f0301fe.png
│   └── e15cbdd2d653.svg
└── documents/
    └── 9870371a7a8c.pdf
```

**Asset Download Features:**
- **MIME Detection:** Automatic detection from URL extension
- **File Size Limit:** 50MB maximum per file
- **Timeout:** 30 seconds per download
- **Unique Filenames:** SHA256 content hash (first 12 chars)
- **Directory Organization:** Separate folders for images/documents
- **Concurrency Limit:** 3 concurrent downloads (HDD-safe)

---

## State Management

| Flag | Default | Description |
|------|---------|-------------|
| `--resume` | `false` | Resume mode - skip URLs already processed |
| `--state-dir <DIR>` | `~/.cache/rust-scraper/state` | Custom state directory for resume mode |

### Resume Mode (`--resume`)

Avoids re-processing URLs already scraped successfully:

```bash
# First run
cargo run -- --url "https://example.com" --max-pages 50 --resume

# Interrupted? Resume from where you left off
cargo run -- --url "https://example.com" --max-pages 50 --resume
```

### Custom State Directory (`--state-dir`)

```bash
# Use custom state directory
cargo run -- --url "https://example.com" --resume --state-dir ./my-state
```

---

## Sitemap Options

| Flag | Default | Description |
|------|---------|-------------|
| `--use-sitemap` | `false` | Use sitemap for URL discovery |
| `--sitemap-url <URL>` | Auto-discover | Explicit sitemap URL |

### Sitemap Discovery (`--use-sitemap`)

Automatically discovers sitemap from `robots.txt`:

```bash
# Auto-discover sitemap
cargo run -- --url "https://example.com" --use-sitemap
```

### Explicit Sitemap URL (`--sitemap-url`)

```bash
# Specify explicit sitemap URL
cargo run -- --url "https://example.com" --use-sitemap --sitemap-url "https://example.com/sitemap.xml"
```

**Sitemap Features:**
- Auto-discovery from `robots.txt`
- Sitemap index recursion (max depth 3)
- Gzip decompression support
- Zero-allocation streaming parser (quick-xml)

---

## Interactive Mode

| Flag | Default | Description |
|------|---------|-------------|
| `--interactive` | `false` | Interactive mode with TUI URL selector |

### TUI Interactive Mode (`--interactive`)

Launch interactive TUI for URL selection:

```bash
cargo run -- --url "https://example.com" --interactive
```

**Features:**
- Interactive checkbox selection of URLs
- Confirmation mode before download
- Terminal restore on panic/exit
- Ratatui + crossterm backend

---

## AI Options (Feature-Gated)

| Flag | Default | Description | Feature Required |
|------|---------|-------------|------------------|
| `--clean-ai` | `false` | Use AI-powered semantic cleaning for RAG output | `ai` |

### AI Semantic Cleaning (`--clean-ai`)

Requires `--features ai` to be enabled at compile time:

```bash
cargo run --features ai -- --url "https://example.com" --clean-ai
```

**What it does:**
- Uses `SemanticCleaner` to process HTML content
- Generates semantic chunks with embeddings
- Exports in JSONL format with embeddings field

**AI Feature Dependencies:**
- ONNX runtime (tract-onnx)
- Tokenizers (sentence-transformers)
- HuggingFace Hub for model downloads
- Memory-mapped file loading (zero-copy)
- Multi-dimensional arrays for embeddings

---

## Logging & Verbosity

| Flag | Description |
|------|-------------|
| `-v` | Info level logging |
| `-vv` | Debug level logging |
| `-vvv` | Trace level logging |

### Verbosity Flags

```bash
# Info level
cargo run -- --url "https://example.com" -v

# Debug level
cargo run -- --url "https://example.com" -vv

# Trace level (most verbose)
cargo run -- --url "https://example.com" -vvv
```

### Environment Variable (`RUST_LOG`)

For fine-grained control:

```bash
# Debug for specific module
RUST_LOG=rust_scraper=debug cargo run -- --url "https://example.com"

# Trace for entire application
RUST_LOG=trace cargo run -- --url "https://example.com"

# Multiple levels
RUST_LOG=rust_scraper=debug,reqwest=info cargo run -- --url "https://example.com"
```

---

## Help & Version

| Flag | Description |
|------|-------------|
| `-h, --help` | Print help (see summary with `-h`) |
| `--version` | Print version information |

```bash
# Full help
cargo run -- --help

# Quick help summary
cargo run -- -h

# Version
cargo run -- --version
```

---

## Complete Examples

### 1. Basic Scraping

```bash
# Default settings (markdown output, 10 pages, 1s delay)
cargo run -- --url "https://example.com"
```

### 2. Custom Output Format

```bash
# JSON output
cargo run -- --url "https://example.com" -f json

# Plain text output
cargo run -- --url "https://example.com" -f text
```

### 3. RAG Pipeline Export

```bash
# JSONL format (optimal for RAG)
cargo run -- --url "https://example.com" --export-format jsonl

# With custom output directory
cargo run -- --url "https://example.com" --export-format jsonl -o ./rag-data
```

### 4. Asset Downloads

```bash
# Download images only
cargo run --features images -- --url "https://example.com" --download-images

# Download documents only
cargo run --features documents -- --url "https://example.com" --download-documents

# Download both images and documents
cargo run --features full -- --url "https://example.com" --download-images --download-documents
```

### 5. Rate Limiting & Concurrency

```bash
# Slower scraping (2s delay)
cargo run -- --url "https://example.com" --delay-ms 2000

# Limit to 5 pages
cargo run -- --url "https://example.com" --max-pages 5

# Custom concurrency
cargo run -- --url "https://example.com" --concurrency 2
```

### 6. Resume Mode

```bash
# First run with resume enabled
cargo run -- --url "https://example.com" --max-pages 100 --resume

# Resume after interruption
cargo run -- --url "https://example.com" --max-pages 100 --resume
```

### 7. Sitemap Discovery

```bash
# Auto-discover sitemap from robots.txt
cargo run -- --url "https://example.com" --use-sitemap

# Explicit sitemap URL
cargo run -- --url "https://example.com" --use-sitemap --sitemap-url "https://example.com/sitemap.xml"
```

### 8. Interactive Mode

```bash
# Launch TUI for URL selection
cargo run -- --url "https://example.com" --interactive
```

### 9. AI Semantic Cleaning

```bash
# Enable AI-powered cleaning
cargo run --features ai -- --url "https://example.com" --clean-ai
```

### 10. Production Dataset Creation

```bash
# Full production run with all features
cargo run --features full -- \
  --url "https://example.com" \
  --export-format jsonl \
  --download-images \
  --download-documents \
  --delay-ms 2000 \
  --max-pages 100 \
  --concurrency 3 \
  --resume \
  -o ./production-dataset \
  -vv
```

### 11. CSS Selector Extraction

```bash
# Extract only article content
cargo run -- --url "https://example.com/blog" -s "article.post-content"

# Extract main content by ID
cargo run -- --url "https://example.com" -s "#main"
```

### 12. Verbose Debugging

```bash
# Debug logging
cargo run -- --url "https://example.com" -vv

# Trace logging with custom RUST_LOG
RUST_LOG=rust_scraper=trace cargo run -- --url "https://example.com" -vvv
```

---

## Feature Flags

rust-scraper supports optional features for extended functionality:

| Feature | Description | Enables |
|---------|-------------|---------|
| `images` | Image downloading support | `mime-type-detector` |
| `documents` | Document downloading support | `mime-type-detector` |
| `ai` | AI semantic cleaning | `ort`, `tokenizers`, `tract-onnx`, etc. |
| `full` | All features | `images`, `documents` |

### Using Feature Flags

```bash
# Enable single feature
cargo run --features images -- --url "https://example.com" --download-images

# Enable multiple features
cargo run --features "images,documents" -- --url "https://example.com" --download-images --download-documents

# Enable all features
cargo run --features full -- --url "https://example.com" --download-images --download-documents

# Build with features (faster subsequent runs)
cargo build --release --features full
./target/release/rust-scraper --url "https://example.com" --download-images --download-documents
```

---

## Troubleshooting

### Invalid URL Error

```bash
# ❌ Wrong (missing protocol)
cargo run -- --url "example.com"

# ✅ Correct
cargo run -- --url "https://example.com"
```

**Error Message:**
```
Error: Invalid URL: Failed to parse URL 'example.com': relative URL without a base
```

### SSL/TLS Certificate Errors

```bash
# Update system certificates (Arch Linux / CachyOS)
sudo pacman -Sy ca-certificates

# Update system certificates (Debian/Ubuntu)
sudo update-ca-certificates
```

**Error Message:**
```
Error: error sending request: certificate validation failed
```

### Permission Denied

```bash
# Check directory permissions
ls -la ./output

# Create directory with correct permissions
mkdir -p ./output && chmod 755 ./output
```

**Error Message:**
```
Error: Failed to write output: Permission denied (os error 13)
```

### Network Timeouts

For slow networks, increase delay and reduce concurrency:

```bash
cargo run -- --url "https://example.com" --delay-ms 3000 --concurrency 1
```

### Feature Not Enabled

```bash
# ❌ Wrong (trying to use feature without enabling)
cargo run -- --url "https://example.com" --download-images

# ✅ Correct
cargo run --features images -- --url "https://example.com" --download-images
```

**Error Message:**
```
Error: Feature 'images' is not enabled
```

### AI Feature Compilation Errors

The `ai` feature requires additional system dependencies:

```bash
# Install build dependencies (Arch Linux / CachyOS)
sudo pacman -Sy cmake llvm clang

# Install C++ toolchain
sudo pacman -Sy gcc gcc-libs
```

**Common Errors:**
- `CMake not found` → Install CMake
- `Cannot find -lstdc++` → Install GCC
- `ONNX runtime not found` → Build with `--features ai`

### Memory Issues on Large Scrapes

For systems with limited RAM (8GB or less):

```bash
# Reduce concurrency
cargo run -- --url "https://example.com" --concurrency 1 --max-pages 20

# Process in batches
cargo run -- --url "https://example.com" --max-pages 10 --resume
```

---

## Exit Codes

| Code | Description |
|------|-------------|
| `0` | Success |
| `1` | General error (invalid URL, network error, etc.) |
| `2` | CLI argument parsing error |

---

## Full Help Output

<details>
<summary><strong>Click to expand full --help output (verified 2026-03-11)</strong></summary>

```
Production-ready web scraper with Clean Architecture                    

Usage: rust_scraper [OPTIONS] --url <URL>                               

Options:                                                                
  -u, --url <URL>                                                       
          URL to scrape (required)                                      
                                                                        
  -s, --selector <SELECTOR>                                             
          CSS selector for content extraction                           
                                                                        
          [default: body]                                               
                                                                        
  -o, --output <OUTPUT>                                                 
          Output directory for scraped content                          
                                                                        
          [default: output]                                             
                                                                        
  -f, --format <FORMAT>                                                 
          Output format for individual files (markdown, text, json)     
                                                                        
          Creates separate output files for each scraped page: - markdown: Markdown with YAML frontmatter (default) - text: Plain text without formatting - json: Structured JSON with metadata
                                                                        
          Use this for human-readable output or when you need individual files per page.
                                                                        
          Possible values:                                              
          - markdown: Markdown format with YAML frontmatter (recommended for RAG)
          - json:     Structured JSON with metadata                     
          - text:     Plain text without formatting                     
                                                                        
          [default: markdown]                                           
                                                                        
      --export-format <EXPORT_FORMAT>                                   
          Export format for RAG pipeline (jsonl, auto)            
                                                                   
          Creates output suitable for retrieval-augmented generation: - jsonl: JSON Lines format (one JSON per line), optimal for RAG - auto: Detect from existing export files
                                                                        
          Use this for LLM/RAG pipelines that need batch export.        
                                                                        
          Possible values:                                              
          - jsonl: JSONL format (JSON Lines - one JSON object per line) Optimal for RAG pipelines and vector database ingestion
          - auto:  Auto-detect format from existing export files        
                                                                        
          [default: jsonl]                                              
                                                                        
      --resume                                                          
          Resume mode - skip URLs already processed                     
                                                                        
          Saves processing status to cache directory (~/.cache/rust-scraper/state) Avoids re-processing URLs already scraped successfully.
                                                                        
      --state-dir <STATE_DIR>                                           
          Custom state directory for resume mode                        
                                                                        
          Default: ~/.cache/rust-scraper/state                          
                                                                        
      --delay-ms <DELAY_MS>                                             
          Delay between requests in milliseconds                        
                                                                        
          [default: 1000]                                               
                                                                        
      --max-pages <MAX_PAGES>                                           
          Maximum pages to scrape                                       
                                                                        
          [default: 10]                                                 
                                                                        
      --download-images                                                 
          Download images from the page                                 
                                                                        
      --download-documents                                              
          Download documents from the page (PDF, DOCX, XLSX, etc.)      
                                                                        
  -v, --verbose...                                                      
          Verbosity level (use multiple times for more detail: -v, -vv, -vvv)
                                                                        
      --concurrency <CONCURRENCY>                                       
          Concurrency level (number of parallel requests)               
                                                                        
          Default: auto-detect based on CPU cores: - 1-2 cores: 1 - 4 cores: 3 (HDD-aware) - 8+ cores: min(CPU cores - 1, 8)
                                                                        
          Note: Can be overridden via CLI or detected at runtime. The actual value used is determined at startup.
                                                                        
          [default: auto]                                               
                                                                        
      --use-sitemap                                                     
          Use sitemap for URL discovery (auto-discovers from robots.txt if URL not provided)
                                                                        
      --sitemap-url <SITEMAP_URL>                                       
          Explicit sitemap URL (optional, auto-discovers if not provided)
                                                                        
      --interactive                                                     
          Interactive mode with TUI URL selector                        
                                                                        
  -h, --help                                                            
          Print help (see a summary with '-h')
```

</details>

*To regenerate: `cargo run -- --help 2>&1 | tee /tmp/cli_full_help.txt`*

---

## Related Documentation

- [Architecture](./ARCHITECTURE.md) — Clean Architecture layers
- [Configuration](./CONFIGURATION.md) — Advanced configuration options
- [RAG Pipeline](./RAG_PIPELINE.md) — Using rust-scraper for RAG datasets
- [TUI Guide](./TUI.md) — Interactive mode guide

---

## Version History

### v1.0.0 (2026-03-11)

- ✅ Full CLI documentation with all verified flags
- ✅ Feature flags documented (`ai`, `images`, `documents`)
- ✅ Concurrency auto-detection (hardware-aware)
- ✅ Sitemap support with auto-discovery
- ✅ TUI interactive mode
- ✅ State management with resume capability
- ✅ AI semantic cleaning (feature-gated)

---

**Last Verified:** 2026-03-11 with `cargo run -- --help`  
**rust-scraper** v1.0.0 — Production-ready web scraper with Clean Architecture
