# CLI Reference

## Usage

```bash
rust-scraper [OPTIONS] --url <URL>
```

## Options

### Required

| Flag | Description |
|------|-------------|
| `-u, --url <URL>` | Target URL to scrape (must include `http://` or `https://`) |

### Optional

| Flag | Description | Default |
|------|-------------|---------|
| `-o, --output <DIR>` | Output directory | `output` |
| `-f, --format <FORMAT>` | Output format (markdown/json/text) | `markdown` |
| `-s, --selector <SELECTOR>` | CSS selector for content | `body` |
| `--download-images` | Download images to `output/images/` | ❌ |
| `--download-documents` | Download documents to `output/documents/` | ❌ |
| `--delay-ms <MS>` | Delay between requests (ms) | `1000` |
| `--max-pages <N>` | Maximum pages to scrape | `10` |
| `-v, --verbose` | Increase verbosity (`-vv` for debug) | ❌ |
| `-h, --help` | Show help | - |
| `-V, --version` | Show version | - |

## Output Formats

### markdown (Default)

Creates structured Markdown with YAML frontmatter:

```bash
cargo run -- --url "https://example.com" -f markdown
```

**Output:** `output/example.com/index.md`

```yaml
---
title: Example Domain
url: https://example.com/
date: 2026-03-08
author: null
excerpt: This domain is for use in documentation...
---

# Example Domain

Content of the page...

```rust
// Code blocks with syntax highlighting
fn main() {
    println!("Hello, world!");
}
```
```

### json

Creates JSON file with all metadata:

```bash
cargo run -- --url "https://example.com" -f json
```

**Output:** `output/results.json`

```json
[
  {
    "title": "Example Domain",
    "content": "...",
    "url": "https://example.com/",
    "excerpt": "...",
    "author": null,
    "date": null,
    "assets": []
  }
]
```

### text

Creates plain text file without formatting:

```bash
cargo run -- --url "https://example.com" -f text
```

**Output:** `output/example.com/index.txt`

```
Example Domain

Content of the page...
```

## Asset Download

### Download Images

Download all images found on the page:

```bash
cargo run -- --url "https://example.com" --download-images
```

Images are saved to `output/images/` with unique filenames based on content hash:

```
output/images/
├── 027e504eabfc.png
├── 0c2f4f0301fe.png
└── e15cbdd2d653.svg
```

### Download Documents

Download all documents (PDF, DOCX, XLSX, etc.) found on the page:

```bash
cargo run -- --url "https://example.com" --download-documents
```

Documents are saved to `output/documents/`:

```
output/documents/
└── 9870371a7a8c.pdf
```

### Download Both Images and Documents

```bash
cargo run -- --url "https://example.com" --download-images --download-documents
```

### Asset Download Features

| Feature | Description |
|---------|-------------|
| **MIME Detection** | Automatic detection from URL extension |
| **File Size Limit** | 50MB maximum per file |
| **Timeout** | 30 seconds per download |
| **Unique Filenames** | SHA256 content hash (first 12 chars) |
| **Directory Organization** | Separate folders for images/documents |
| **Concurrency Limit** | 3 concurrent downloads (HDD-safe) |

## Examples

### Basic Usage

```bash
# Scrape a simple page
cargo run -- --url "https://example.com"

# Specify output directory
cargo run -- --url "https://example.com" -o ./data

# Get JSON output
cargo run -- --url "https://example.com" -f json
```

### Verbose Output

```bash
# Show info logs
cargo run -- --url "https://example.com" -v

# Show debug logs
cargo run -- --url "https://example.com" -vv

# Show trace logs
cargo run -- --url "https://example.com" -vvv
```

### Output Location

```bash
# Default: ./output/domain/path.md
cargo run -- --url "https://example.com/docs"

# Custom directory
cargo run -- --url "https://example.com" -o ./my-scrapes
```

### Asset Downloads

```bash
# Download only images
cargo run -- --url "https://example.com" --download-images

# Download only documents
cargo run -- --url "https://example.com" --download-documents

# Download both images and documents
cargo run -- --url "https://example.com" --download-images --download-documents

# Custom output directory for assets
cargo run -- --url "https://example.com" --download-images -o ./my-downloads
```

### Production Usage

```bash
# Scrape multiple pages with rate limiting
cargo run -- --url "https://example.com" \
  --download-images \
  --download-documents \
  --delay-ms 2000 \
  --max-pages 50 \
  -o ./rag-dataset \
  -vv
```

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | Error (invalid URL, network error, etc.) |

## Production Features (v0.3.0)

### Retry Logic

The scraper automatically retries failed requests with exponential backoff:

- **Retries:** 3 attempts
- **Backoff:** 100ms → 200ms → 400ms
- **Triggers:** 5xx errors, timeouts, connection resets

No configuration needed - works automatically.

### Concurrency Control

Bounded concurrency prevents resource exhaustion:

- **Limit:** 3 concurrent requests
- **Purpose:** Prevents HDD thrashing, FD exhaustion
- **Hardware-aware:** Optimized for 4C/8GB RAM systems

### User-Agent Rotation

Automatic rotation of User-Agent headers:

- **Pool:** 14 modern browsers
- **Distribution:** 40% Chrome, 20% Firefox, 20% Safari, 20% Edge
- **Purpose:** Reduces bot detection

No configuration needed - works automatically.

## Notes

- URL must include protocol (`http://` or `https://`)
- HTTPS uses system TLS certificates (rustls with native roots)
- The scraper extracts main content using Readability algorithm
- Files are organized by domain to avoid collisions
- Code blocks are syntax-highlighted automatically
- YAML frontmatter includes metadata (title, URL, date, author, excerpt)

## Troubleshooting

### Invalid URL Error

```bash
# ❌ Wrong (missing protocol)
cargo run -- --url "example.com"

# ✅ Correct
cargo run -- --url "https://example.com"
```

### SSL/TLS Errors

If you encounter SSL certificate errors:

```bash
# Update system certificates (Arch Linux)
sudo pacman -Sy ca-certificates

# Update system certificates (Debian/Ubuntu)
sudo update-ca-certificates
```

### Permission Denied

If you get permission errors writing to output directory:

```bash
# Check directory permissions
ls -la ./output

# Create directory with correct permissions
mkdir -p ./output && chmod 755 ./output
```

### Network Timeouts

For slow networks, increase timeout (requires code change):

```rust
// In src/application/http_client.rs
const TIMEOUT_SECS: u64 = 60; // Increase from 30 to 60
```
