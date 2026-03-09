# RAG Export Pipeline

**Status:** ✅ Complete (Issue #1 - 95%)
**Format:** JSON Lines (JSONL)
**Feature:** State management with resume support

---

## Overview

The RAG Export Pipeline exports scraped content in **JSON Lines (JSONL)** format, optimized for ingestion into vector databases and RAG (Retrieval-Augmented Generation) systems.

### Key Features

- **Streaming writes**: Constant memory usage (~8KB), no OOM risks
- **Resume support**: `--resume` flag tracks processed URLs
- **State persistence**: Atomic saves with crash recovery
- **RAG-ready**: Compatible with Qdrant, Weaviate, Pinecone, LangChain

---

## Quick Start

### Basic Export

```bash
# Export to JSONL
./target/release/rust_scraper \
  --url https://example.com \
  --export-format jsonl \
  --output ./rag_data
```

### Resume Mode

```bash
# Resume interrupted scraping
./target/release/rust_scraper \
  --url https://example.com \
  --export-format jsonl \
  --output ./rag_data \
  --resume
```

### Custom State Directory

```bash
# Isolate state per project
./target/release/rust_scraper \
  --url https://example.com \
  --export-format jsonl \
  --output ./rag_data \
  --state-dir ./project-state \
  --resume
```

---

## JSONL Schema

### Document Structure

Each line in the output file is a valid JSON object:

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "url": "https://example.com/docs/getting-started",
  "title": "Getting Started Guide",
  "content": "This guide will help you get started with...",
  "metadata": {
    "domain": "example.com",
    "excerpt": "Meta description or auto-extracted excerpt"
  },
  "timestamp": "2026-03-09T10:00:00.000000Z"
}
```

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | UUID v4 | Unique document identifier |
| `url` | String | Source URL (RFC 3986 validated) |
| `title` | String | Page title (from `<title>` tag) |
| `content` | String | Extracted content (Readability algorithm) |
| `metadata.domain` | String | Domain name for multi-site tracking |
| `metadata.excerpt` | String | Meta description or first paragraph |
| `timestamp` | ISO 8601 | UTC timestamp of extraction |

---

## State Management

### Storage Location

Default: `~/.cache/rust-scraper/state/<domain>.json`

Custom: `--state-dir /path/to/state`

### State File Structure

```json
{
  "domain": "https://example.com",
  "processed_urls": [
    "https://example.com/",
    "https://example.com/docs",
    "https://example.com/about"
  ],
  "last_export": "2026-03-09T10:00:00.000000Z",
  "total_exported": 3
}
```

### Atomic Saves

State is saved atomically using write-to-temp + rename pattern:

1. Write JSON to `<domain>.tmp`
2. `fs::rename()` to `<domain>.json`
3. Crash-safe: partial writes are never visible

---

## RAG Integration

### LangChain (Python)

```python
from langchain.document_loaders import JSONLoader
from langchain.text_splitter import RecursiveCharacterTextSplitter
from langchain.embeddings import OpenAIEmbeddings
from langchain.vectorstores import Qdrant

# Load JSONL
loader = JSONLoader(
    file_path='./rag_data/export.jsonl',
    jq_schema='.content',
    text_content=False,
    metadata_func=lambda d, m: {"url": d["url"], "title": d["title"]}
)
documents = loader.load()

# Split into chunks
text_splitter = RecursiveCharacterTextSplitter(
    chunk_size=500,
    chunk_overlap=50
)
chunks = text_splitter.split_documents(documents)

# Embed and store
embeddings = OpenAIEmbeddings()
vectorstore = Qdrant.from_documents(
    chunks,
    embeddings,
    url="http://localhost:6333",
    collection_name="rust_scraper"
)
```

### LlamaIndex (Python)

```python
from llama_index import SimpleDirectoryReader, VectorStoreIndex
from llama_index.readers.file import JSONLReader

# Load JSONL
reader = JSONLReader()
documents = reader.load_data(file_path='./rag_data/export.jsonl')

# Create index
index = VectorStoreIndex.from_documents(documents)

# Query
query_engine = index.as_query_engine()
response = query_engine.query("What is Rust?")
print(response)
```

### Direct Qdrant Upload (curl)

```bash
# Convert JSONL to Qdrant batch format
cat export.jsonl | jq -s 'map({
  id: .id,
  vector: [],  # Add embeddings here
  payload: {
    url: .url,
    title: .title,
    content: .content
  }
})' > qdrant_batch.json

# Upload to Qdrant
curl -X PUT "http://localhost:6333/collections/rust_scraper/points" \
  -H "Content-Type: application/json" \
  -d @qdrant_batch.json
```

---

## Performance Considerations

### HDD Optimization

For mechanical hard drives (HDD):

```bash
# Use ionice for background priority
ionice -c 3 ./target/release/rust_scraper \
  --url https://example.com \
  --export-format jsonl \
  --output ./rag_data
```

### Concurrency Settings

| Storage | Concurrency | Command |
|---------|-------------|---------|
| HDD | 3 (default) | `--concurrency 3` |
| SSD | 5-8 | `--concurrency 5` |
| NVMe | 10+ | `--concurrency 10` |

### Memory Usage

- **Streaming writes**: ~8KB constant RAM
- **BufWriter**: 8KB buffer (matches HDD sector size)
- **No intermediate collections**: Documents exported immediately

---

## Troubleshooting

### State File Not Created

**Problem:** `--resume` doesn't track URLs

**Solution:** Ensure state directory is writable:
```bash
mkdir -p ~/.cache/rust-scraper/state
chmod 755 ~/.cache/rust-scraper/state
```

### JSONL Validation

**Problem:** Invalid JSON in output

**Solution:** Validate with jq:
```bash
# Check each line
cat export.jsonl | while read line; do
  echo "$line" | jq . > /dev/null || echo "Invalid: $line"
done
```

### Resume Not Skipping URLs

**Known Issue:** Current implementation tracks processed URLs but doesn't skip them before scraping. This is a design limitation - URLs are scraped first, then marked as processed.

**Workaround:** Use `--max-pages` to limit re-scraping:
```bash
./target/release/rust_scraper \
  --url https://example.com \
  --export-format jsonl \
  --output ./rag_data \
  --resume \
  --max-pages 10
```

---

## Hardware-Aware Recommendations

### For HDD (Mechanical Drives)

```bash
# Low I/O priority
ionice -c 3 ./target/release/rust_scraper \
  --url https://example.com \
  --export-format jsonl \
  --output ./rag_data \
  --concurrency 3 \
  --delay-ms 1000
```

### For SSD/NVMe

```bash
# Higher concurrency
./target/release/rust_scraper \
  --url https://example.com \
  --export-format jsonl \
  --output ./rag_data \
  --concurrency 8 \
  --delay-ms 500
```

### Release Build (Recommended)

```bash
# Build with LTO for best performance
cargo build --release

# Binary size comparison
ls -lh target/debug/rust_scraper target/release/rust_scraper
# Debug: ~50MB, Release: ~5MB (10x smaller)
```

---

## Future Enhancements

- [ ] Pre-scrape URL skipping (skip before HTTP request)
- [ ] Batch state saves (reduce I/O operations)
- [ ] Zvec format support (Alibaba vector format)
- [ ] Direct vector database upload
- [ ] Incremental exports (only new/changed content)

---

## References

- [JSON Lines Specification](https://jsonlines.org/)
- [LangChain JSONLoader](https://python.langchain.com/docs/integrations/document_loaders/json_loader)
- [Qdrant Documentation](https://qdrant.tech/documentation/)
- [rust-skills: mem-with-capacity](https://github.com/leonardomso/rust-skills/blob/main/mem-with-capacity.md)
- [rust-skills: async-tokio-fs](https://github.com/leonardomso/rust-skills/blob/main/async-tokio-fs.md)
