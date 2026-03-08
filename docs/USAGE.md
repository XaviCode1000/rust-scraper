# Usage Guide

## Installation

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

## Basic Usage

### Headless Mode (CI/CD Friendly)

```bash
# Scrape entire website
./target/release/rust_scraper --url https://example.com

# With output directory
./target/release/rust_scraper --url https://example.com --output ./my-scrape

# Download images
./target/release/rust_scraper --url https://example.com --download-images

# Download documents (PDF, DOCX, XLSX)
./target/release/rust_scraper --url https://example.com --download-documents
```

### Interactive Mode (TUI)

```bash
# Select URLs before downloading
./target/release/rust_scraper --url https://example.com --interactive

# With sitemap
./target/release/rust_scraper --url https://example.com \
  --interactive \
  --use-sitemap
```

#### TUI Controls

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate URLs |
| `Space` | Toggle selection |
| `A` | Select all |
| `D` | Deselect all |
| `Enter` | Confirm download |
| `Y` / `N` | Final confirmation |
| `q` | Quit |

## Sitemap Support

### Auto-Discovery

```bash
# Automatically finds sitemap from robots.txt
./target/release/rust_scraper --url https://example.com --use-sitemap
```

### Explicit URL

```bash
# Specify sitemap URL directly
./target/release/rust_scraper --url https://example.com \
  --use-sitemap \
  --sitemap-url https://example.com/sitemap.xml
```

### Compressed Sitemaps

Supports `.xml` and `.xml.gz`:

```bash
# Gzip-compressed sitemap
./target/release/rust_scraper --url https://example.com \
  --use-sitemap \
  --sitemap-url https://example.com/sitemap.xml.gz
```

## Advanced Configuration

### Concurrency

```bash
# Limit concurrent downloads (default: 5)
./target/release/rust_scraper --url https://example.com --concurrency 3
```

### User Agent

```bash
# Custom user agent
./target/release/rust_scraper --url https://example.com \
  --user-agent "MyBot/1.0"
```

### Logging

```bash
# Debug logging
RUST_LOG=rust_scraper=debug ./target/release/rust_scraper --url https://example.com

# Trace logging
RUST_LOG=rust_scraper=trace ./target/release/rust_scraper --url https://example.com
```

## Examples

### Example 1: Scrape Blog

```bash
./target/release/rust_scraper \
  --url https://myblog.com \
  --output ./blog-backup \
  --download-images \
  --use-sitemap
```

### Example 2: Download Documentation

```bash
./target/release/rust_scraper \
  --url https://docs.example.com \
  --output ./docs \
  --download-documents \
  --interactive
```

### Example 3: CI/CD Pipeline

```yaml
# .github/workflows/scrape.yml
- name: Scrape website
  run: |
    ./target/release/rust_scraper \
      --url https://example.com \
      --output ./dataset \
      --use-sitemap
```

### Example 4: Full Options

```bash
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

## Troubleshooting

### Terminal Corruption

If the TUI leaves your terminal in a bad state:

```bash
reset
```

The TUI has a panic hook to restore the terminal, but if something goes wrong, use `reset`.

### WAF Blocking

If you're getting 403 errors:

1. Use `--use-sitemap` (polite crawling)
2. Reduce `--concurrency` (e.g., `--concurrency 2`)
3. Add delays between requests

### Windows Reserved Names

Files like `CON.md`, `PRN.md` are automatically renamed to `CON_safe.md`, `PRN_safe.md` to prevent filesystem errors.

### SSL/Certificate Errors

If you encounter SSL errors:

```bash
# Update system certificates (Arch Linux)
sudo pacman -Sy ca-certificates

# Then rebuild
cargo build --release
```

## API Usage

### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
rust_scraper = "1.0.0"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

### Example

```rust
use rust_scraper::{discover_urls, scrape_urls, CrawlerConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = CrawlerConfig::builder()
        .concurrency(5)
        .build();
    
    // Discover URLs from sitemap
    let urls = discover_urls("https://example.com", &config).await?;
    
    println!("Found {} URLs", urls.len());
    
    // Scrape all URLs
    let results = scrape_urls(&urls, &config).await?;
    
    for result in results {
        println!("Scraped: {} - {}", result.url, result.title.unwrap_or_default());
    }
    
    Ok(())
}
```

### Custom HTTP Client

```rust
use rust_scraper::create_http_client;

fn main() -> anyhow::Result<()> {
    let client = create_http_client()?;
    // Use client for custom scraping logic
    Ok(())
}
```

## Performance Tips

1. **Use sitemaps**: Faster than crawling, polite to servers
2. **Adjust concurrency**: `--concurrency 3` for HDD, `--concurrency 10+` for SSD
3. **Enable compression**: Built-in gzip/brotli support (default)
4. **Batch processing**: Use `--max-pages` for large sites

## Security Features

- **SSRF Prevention**: URL host comparison (not string contains)
- **Input Validation**: `url::Url::parse()` (RFC 3986 compliant)
- **Windows Safe**: Reserved names blocked (`CON` → `CON_safe`)
- **WAF Bypass Prevention**: Chrome 131+ UAs with TTL caching
