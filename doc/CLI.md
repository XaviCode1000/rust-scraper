# CLI Reference

## Usage

```bash
rust-scraper [OPTIONS] --url <URL>
```

## Options

### Required

| Flag | Description |
|------|-------------|
| `-u, --url <URL>` | Target URL to scrape (must include http:// or https://) |

### Optional

| Flag | Description | Default |
|------|-------------|---------|
| `-o, --output <DIR>` | Output directory | `output` |
| `-f, --format <FORMAT>` | Output format (markdown/json/text) | `markdown` |
| `-s, --selector <SELECTOR>` | CSS selector for content | `body` |
| `--download-images` | Download images to `output/images/` | - |
| `--download-documents` | Download documents to `output/documents/` | - |
| `--delay-ms <MS>` | Delay between requests (ms) | `1000` |
| `--max-pages <N>` | Maximum pages to scrape | `10` |
| `-v, --verbose` | Increase verbosity (-vv for debug) | - |
| `-h, --help` | Show help | - |
| `-V, --version` | Show version | - |

## Output Formats

### markdown

Creates structured Markdown with YAML frontmatter:

```bash
cargo run -- --url "https://example.com" -f markdown
```

Output: `output/example.com/index.md`

```yaml
---
title: Example Domain
url: https://example.com/
date: 2026-03-07
author: null
excerpt: This domain is for use in documentation...
---

# Example Domain

Content of the page...
```

### json

Creates JSON file with all metadata:

```bash
cargo run -- --url "https://example.com" -f json
```

Output: `output/results.json`

```json
[
  {
    "title": "Example Domain",
    "content": "...",
    "url": "https://example.com/",
    "excerpt": "...",
    "author": null,
    "date": null
  }
]
```

### text

Creates plain text file:

```bash
cargo run -- --url "https://example.com" -f text
```

Output: `output/example.com/index.txt`

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

Download all documents (PDF, DOCX, etc.) found on the page:

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

- **Automatic MIME type detection** - Uses content-type headers
- **File size limit** - 50MB maximum per file
- **Timeout** - 30 seconds per download
- **Unique filenames** - Generated from SHA256 hash
- **Directory organization** - Separate folders for images and documents

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

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | Error (invalid URL, network error, etc.) |

## Notes

- URL must include protocol (`http://` or `https://`)
- HTTPS uses system TLS certificates (rustls with native roots)
- The scraper extracts main content using Readability algorithm
- Files are organized by domain to avoid collisions
- Asset downloads respect file size limits (50MB max)
- Asset downloads timeout after 30 seconds per file
- Asset filenames are unique based on content hash (SHA256)