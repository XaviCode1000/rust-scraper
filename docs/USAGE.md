# Usage Guide — rust-scraper

**Production-ready web scraper with Clean Architecture**

---

## Quick Start

### Basic Scraping (Headless Mode)

```bash
# Scrape a single URL (default: 10 pages max, markdown format)
cargo run -- --url https://example.com

# With custom output directory
cargo run -- --url https://example.com --output ./my-scrape

# See all available options
cargo run -- --help
```

**Expected Output:**
```
🚀 Rust Scraper v0.4.0 - Clean Architecture + TUI
📌 Target: https://example.com
📁 Output: output
✅ URL validated: https://example.com/
🔍 Discovering URLs...
✅ Found 5 URLs
📡 Headless mode: will scrape all 5 URLs
🕷️  Scraping 5 URLs...
✅ Scraping completed: 5 elements extracted
💾 Exporting results (format: Jsonl)...
🎉 Pipeline completed successfully!
📊 Files generated: output
📈 Total URLs processed: 5
```

---

## Basic Usage

### Required Arguments

| Argument | Description | Example |
|----------|-------------|---------|
| `--url <URL>` | URL to scrape (required) | `--url https://example.com` |

### Common Options

| Flag | Default | Description |
|------|---------|-------------|
| `-o, --output <DIR>` | `output` | Output directory for scraped content |
| `-f, --format <FORMAT>` | `markdown` | Output format: `markdown`, `json`, `text` |
| `--export-format <FORMAT>` | `jsonl` | RAG export: `jsonl`, `auto` |
| `--max-pages <N>` | `10` | Maximum pages to scrape |
| `--delay-ms <MS>` | `1000` | Delay between requests in milliseconds |
| `--concurrency <N>` | `auto` | Parallel requests (auto-detects CPU) |
| `-v, --verbose` | - | Verbosity level (`-v`, `-vv`, `-vvv`) |
| `--resume` | - | Skip already processed URLs |
| `--interactive` | - | TUI mode for URL selection |
| `--force-js-render` | - | Force JS rendering for SPA sites (reserved, no-op) |
| `--force-js-render` | - | Force JS rendering for SPA sites (reserved, no-op) |

### Output Formats

**Individual file formats (`--format`):**

- `markdown` — Markdown with YAML frontmatter (recommended for RAG)
- `json` — Structured JSON with metadata
- `text` — Plain text without formatting

**RAG export formats (`--export-format`):**

- `jsonl` — JSON Lines (one JSON per line), optimal for RAG pipelines
- `auto` — Auto-detect from existing export files

---

## Interactive Mode (TUI)

Select URLs interactively before scraping:

```bash
# Enable TUI mode
cargo run -- --url https://example.com --interactive

# With sitemap discovery
cargo run -- --url https://example.com --interactive --use-sitemap
```

### TUI Controls

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate URLs |
| `Space` | Toggle URL selection |
| `A` | Select all URLs |
| `D` | Deselect all URLs |
| `Enter` | Confirm selection and start scraping |
| `Y` / `N` | Final confirmation prompt |
| `q` | Quit (with confirmation) |

**Note:** The TUI has a panic hook to restore terminal state. If something goes wrong, run `reset` to fix terminal corruption.

---

## Sitemap Support

Auto-discover URLs from `robots.txt` or explicit sitemap:

### Auto-Discovery

```bash
# Automatically finds sitemap from robots.txt
cargo run -- --url https://example.com --use-sitemap
```

### Explicit Sitemap URL

```bash
# Specify sitemap URL directly
cargo run -- --url https://example.com \
  --use-sitemap \
  --sitemap-url https://example.com/sitemap.xml
```

### Compressed Sitemaps

Supports `.xml` and `.xml.gz`:

```bash
# Gzip-compressed sitemap
cargo run -- --url https://example.com \
  --use-sitemap \
  --sitemap-url https://example.com/sitemap.xml.gz
```

---

## AI Semantic Cleaning (Feature-Gated)

**Requires:** `cargo build --features ai`

AI-powered semantic cleaning for better RAG output:

```bash
# Enable AI semantic cleaning
cargo run --features ai -- --url https://example.com --clean-ai

# With custom threshold (if available)
cargo run --features ai -- --url https://example.com \
  --clean-ai \
  --ai-threshold 0.7
```

**What it does:**
- Uses `SemanticCleaner` to process HTML content
- Generates semantic chunks with embeddings
- Exports in JSONL format with embeddings field
- Splits content into semantic segments using embedding-based refinement

**Note:** The `--clean-ai` flag is only available when compiled with `--features ai`. Without the feature, the flag is hidden and will show an error.

---

## RAG Export

Export scraped content optimized for Retrieval-Augmented Generation pipelines:

### JSONL Format (Recommended)

```bash
# Export to JSONL (default)
cargo run -- --url https://example.com --export-format jsonl

# With AI embeddings
cargo run --features ai -- --url https://example.com \
  --export-format jsonl \
  --clean-ai
```

**Output format:**
```json
{"url": "https://example.com/page1", "title": "Page Title", "content": "...", "embedding": [0.1, 0.2, ...]}
{"url": "https://example.com/page2", "title": "Another Page", "content": "...", "embedding": [0.3, 0.4, ...]}
```

### Resume Mode

Skip already processed URLs:

```bash
# Enable resume mode
cargo run -- --url https://example.com --resume

# Custom state directory
cargo run -- --url https://example.com \
  --resume \
  --state-dir /path/to/state
```

**State storage:**
- Default: `~/.cache/rust-scraper/state`
- Override with `XDG_CACHE_HOME` environment variable
- State is organized by domain

---

## Advanced Options

### Concurrency Control

Auto-detects based on CPU cores (HDD-aware):

| Cores | Default Concurrency |
|-------|---------------------|
| 1-2 | 1 |
| 4 | 3 (HDD-aware) |
| 8+ | `min(cores - 1, 8)` |

```bash
# Override auto-detection
cargo run -- --url https://example.com --concurrency 5

# Limit for HDD (recommended)
cargo run -- --url https://example.com --concurrency 3
```

### Rate Limiting

```bash
# Custom delay between requests (default: 1000ms)
cargo run -- --url https://example.com --delay-ms 2000

# Aggressive scraping (not recommended for production)
cargo run -- --url https://example.com --delay-ms 100 --max-pages 5
```

### Asset Downloads

```bash
# Download images
cargo run -- --url https://example.com --download-images

# Download documents (PDF, DOCX, XLSX, etc.)
cargo run -- --url https://example.com --download-documents

# Download both
cargo run -- --url https://example.com \
  --download-images \
  --download-documents
```

### Verbosity Levels

```bash
# Default (info level)
cargo run -- --url https://example.com

# Debug logging
cargo run -- --url https://example.com -v

# Trace logging (maximum detail)
cargo run -- --url https://example.com -vvv
```

---

## Output Formats

### Markdown (Default)

```bash
cargo run -- --url https://example.com --format markdown
```

**Output:**
```markdown
---
url: https://example.com
title: Example Domain
scraped_at: 2026-03-11T10:00:00Z
---

# Example Domain

This domain is for use in illustrative examples...
```

### JSON

```bash
cargo run -- --url https://example.com --format json
```

**Output:**
```json
{
  "url": "https://example.com",
  "title": "Example Domain",
  "content": "This domain is for use in illustrative examples...",
  "scraped_at": "2026-03-11T10:00:00Z",
  "assets": [...]
}
```

### Text

```bash
cargo run -- --url https://example.com --format text
```

**Output:**
```
Example Domain

This domain is for use in illustrative examples...
```

---

## Troubleshooting

### Common Errors

#### `URL inválida: <mensaje>`

**Cause:** Invalid URL format or missing scheme.

**Solution:**
```bash
# Include scheme (http/https)
cargo run -- --url https://example.com  # ✅
cargo run -- --url example.com          # ❌
```

#### `HTTP error 403 al acceder a <URL>`

**Cause:** WAF blocking or rate limiting.

**Solutions:**
```bash
# Use sitemap (polite crawling)
cargo run -- --url https://example.com --use-sitemap

# Reduce concurrency
cargo run -- --url https://example.com --concurrency 2

# Add delays
cargo run -- --url https://example.com --delay-ms 3000
```

#### `Error de red: <detalle>`

**Cause:** Network timeout or connection failure.

**Solutions:**
- Check internet connection
- Increase timeout (modify source or retry)
- Use `--delay-ms` to avoid overwhelming server

#### `Error de legibilidad: <detalle>`

**Cause:** Readability algorithm failed to extract content.

**Solutions:**
- Try a different `--selector`
- Check if page uses JavaScript rendering (not supported)
- Verify page has actual content

#### SPA Detection Warning: `{domain} returned minimal content ({N} chars)`

**Cause:** The page returned less than 50 characters of content after extraction, which typically indicates a Single Page Application (SPA) that requires JavaScript rendering.

**What this means:**
- The scraper fetched the HTML successfully, but the content is rendered client-side
- Common with React, Vue, Angular, and other SPA frameworks
- The HTML likely contains only mount points like `<div id="root">` or `<div id="app">`

**Current behavior (v1.3.0):**
- A warning is logged via `tracing::warn!` during scraping
- The minimal content is still returned (not discarded)
- The `--force-js-render` flag is reserved but has no effect yet

**Workarounds:**
- Check if the site has a public API you can query directly
- Use the site's sitemap if available (may have SSR versions)
- Wait for v1.4 which will include headless browser rendering

**Track implementation:** [Issue #16](https://github.com/XaviCode1000/rust-scraper/issues/16)

#### `Modo offline: modelo '<repo>' no está en caché`

**Cause:** AI feature enabled but model not downloaded.

**Solution:**
```bash
# Run once online to download model
cargo run --features ai -- --url https://example.com --clean-ai

# Then offline mode will work if model is cached
```

#### `Chunk <id> excede límite de tokens: <n> > 512`

**Cause:** Content chunk exceeds model's token limit (all-MiniLM-L6-v2).

**Solutions:**
- Split content manually before scraping
- Use `--format markdown` without `--clean-ai`
- Modify chunker configuration in source

### Terminal Corruption

If the TUI leaves your terminal in a bad state:

```bash
reset
```

The TUI has a panic hook to restore the terminal, but if something goes wrong, use `reset`.

### Windows Reserved Names

Files like `CON.md`, `PRN.md` are automatically renamed to `CON_safe.md`, `PRN_safe.md` to prevent filesystem errors. No action needed.

### SSL/Certificate Errors

If you encounter SSL errors:

```bash
# Update system certificates (Arch Linux)
sudo pacman -Sy ca-certificates

# Then rebuild
cargo build --release
```

---

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `XDG_CACHE_HOME` | `~/.cache` | Base directory for cache (state store, models) |
| `RUST_LOG` | - | Override logging level (e.g., `rust_scraper=debug`) |

**State store path resolution:**
1. `--state-dir` (if provided)
2. `$XDG_CACHE_HOME/rust-scraper/state`
3. `~/.cache/rust-scraper/state` (default)

---

## Examples

### Example 1: Scrape Blog with Sitemap

```bash
cargo run -- --url https://myblog.com \
  --output ./blog-backup \
  --download-images \
  --use-sitemap \
  --max-pages 50
```

### Example 2: Download Documentation (Interactive)

```bash
cargo run -- --url https://docs.example.com \
  --output ./docs \
  --download-documents \
  --interactive \
  --concurrency 3
```

### Example 3: RAG Dataset with AI Embeddings

```bash
cargo run --features ai -- --url https://example.com \
  --export-format jsonl \
  --clean-ai \
  --resume \
  --max-pages 100
```

### Example 4: CI/CD Pipeline (GitHub Actions)

```yaml
# .github/workflows/scrape.yml
- name: Scrape website
  run: |
    ./target/release/rust_scraper \
      --url https://example.com \
      --output ./dataset \
      --use-sitemap \
      --export-format jsonl \
      --resume
```

### Example 5: Full Options (Production)

```bash
cargo run -- --url https://example.com \
  --output ./output \
  --format markdown \
  --export-format jsonl \
  --download-images \
  --download-documents \
  --use-sitemap \
  --concurrency 3 \
  --delay-ms 1000 \
  --max-pages 100 \
  --resume \
  -vv
```

---

## Performance Tips

1. **Use sitemaps**: Faster than crawling, polite to servers
2. **Adjust concurrency**: `--concurrency 3` for HDD, `--concurrency 10+` for SSD
3. **Enable resume mode**: `--resume` to avoid re-processing
4. **Batch processing**: Use `--max-pages` for large sites
5. **Enable compression**: Built-in gzip/brotli support (default)

---

## Security Features

- **SSRF Prevention**: URL host comparison (not string contains)
- **Input Validation**: `url::Url::parse()` (RFC 3986 compliant)
- **Windows Safe**: Reserved names blocked (`CON` → `CON_safe`)
- **WAF Bypass Prevention**: Chrome 131+ UAs with TTL caching

---

## API Usage

### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
rust_scraper = "0.4.0"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

### Example

```rust
use rust_scraper::{discover_urls_for_tui, scrape_urls_for_tui, CrawlerConfig, ScraperConfig};
use url::Url;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let seed = Url::parse("https://example.com")?;
    
    let crawler_config = CrawlerConfig::builder(seed)
        .concurrency(5)
        .use_sitemap(true)
        .build();

    // Discover URLs
    let urls = discover_urls_for_tui("https://example.com", &crawler_config).await?;
    println!("Found {} URLs", urls.len());

    // Scrape
    let scraper_config = ScraperConfig::default();
    let results = scrape_urls_for_tui(&urls, &scraper_config).await?;

    for result in results {
        println!("Scraped: {} - {}", result.url, result.title.unwrap_or_default());
    }

    Ok(())
}
```

---

## License

MIT License — See [LICENSE](../LICENSE) for details.

---

**Last updated:** April 2026  
**Version:** rust-scraper v1.3.0
