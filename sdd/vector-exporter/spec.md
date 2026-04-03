# Vector Exporter Specification

## Purpose

Add vector export capability to the Rust Scraper, enabling export of `DocumentChunk` instances (with embeddings) to a structured JSON file format suitable for vector database ingestion. The format includes a metadata header with model info and dimensions, followed by a documents array containing text content and embedding vectors.

---

## 1. Requirements

| ID | Type | Description | Priority |
|----|------|-------------|----------|
| R1 | ADDED | System SHALL support `ExportFormat::Vector` as a new export format variant | MUST |
| R2 | ADDED | System SHALL implement `VectorExporter` struct that implements the `Exporter` trait | MUST |
| R3 | ADDED | System SHALL provide `cosine_similarity(a: &[f32], b: &[f32]) -> f32` as a pure Rust scalar function | MUST |
| R4 | ADDED | Vector export output SHALL conform to the JSON schema defined in Section 3 (Data Contract) | MUST |
| R5 | ADDED | VectorExporter SHALL write a metadata header on first write (new file) containing `format_version`, `model_name`, `dimensions`, `total_documents`, and `created_at` | MUST |
| R6 | ADDED | VectorExporter SHALL append documents to existing files in append mode without rewriting the metadata header | MUST |
| R7 | ADDED | VectorExporter SHALL reject documents without embeddings when exporting in strict mode | SHOULD |
| R8 | ADDED | VectorExporter SHALL use file locking (`fs2::FileExt`) to prevent concurrent write corruption | MUST |
| R9 | ADDED | VectorExporter SHALL use `BufWriter` for streaming writes, not in-memory buffering of entire dataset | MUST |
| R10 | ADDED | `cosine_similarity` SHALL return `0.0` for zero-magnitude vectors (either input) | MUST |
| R11 | ADDED | `cosine_similarity` SHALL return `1.0` for identical normalized vectors | MUST |
| R12 | ADDED | VectorExporter SHALL create output directories if they do not exist | MUST |
| R13 | ADDED | VectorExporter SHALL serialize embeddings as JSON arrays of `f32` values | MUST |
| R14 | ADDED | `ExportFactory::create_exporter` SHALL return `VectorExporter` when `ExportFormat::Vector` is requested | MUST |
| R15 | ADDED | VectorExporter SHALL use `serde_json` for serialization, matching existing exporter patterns | MUST |

---

## 2. Scenarios

### Scenario: Export single document with embeddings (happy path)

- GIVEN a `VectorExporter` configured with output directory `/tmp/output` and filename `vectors`
- AND a `DocumentChunk` with `embeddings = Some([0.1, 0.2, ..., 0.384])` (384 dimensions)
- WHEN `exporter.export(chunk)` is called
- THEN the output file `/tmp/output/vectors.json` is created
- AND the file contains valid JSON with `metadata` and `documents` keys
- AND `metadata.format_version` equals `1`
- AND `metadata.total_documents` equals `1`
- AND `documents[0].embeddings` contains the 384-element array

### Scenario: Export batch of documents with embeddings

- GIVEN a `VectorExporter` with a fresh output path
- AND 5 `DocumentChunk` instances, each with 384-dimensional embeddings
- WHEN `exporter.export_batch(chunks)` is called
- THEN the output file contains valid JSON
- AND `metadata.total_documents` equals `5`
- AND `documents` array contains exactly 5 entries
- AND each entry has `id`, `url`, `title`, `content`, `metadata`, `timestamp`, and `embeddings` fields

### Scenario: Export document without embeddings (non-strict mode)

- GIVEN a `VectorExporter` in default (non-strict) mode
- AND a `DocumentChunk` with `embeddings = None`
- WHEN `exporter.export(chunk)` is called
- THEN the export succeeds
- AND the document's `embeddings` field in the output JSON is `null`

### Scenario: Export empty batch

- GIVEN a `VectorExporter` with a fresh output path
- AND an empty `Vec<DocumentChunk>`
- WHEN `exporter.export_batch(chunks)` is called
- THEN the output file is created with valid JSON
- AND `metadata.total_documents` equals `0`
- AND `documents` is an empty array `[]`

### Scenario: Cosine similarity with zero-magnitude vector

- GIVEN `a = [0.0, 0.0, 0.0]` (zero vector)
- AND `b = [0.1, 0.2, 0.3]` (non-zero vector)
- WHEN `cosine_similarity(&a, &b)` is called
- THEN the result is `0.0`

### Scenario: Cosine similarity with identical vectors

- GIVEN `a = [0.5, 0.5, 0.7071]` (normalized)
- AND `b = [0.5, 0.5, 0.7071]` (same values)
- WHEN `cosine_similarity(&a, &b)` is called
- THEN the result is approximately `1.0` (within `f32::EPSILON` tolerance)

### Scenario: Cosine similarity with orthogonal vectors

- GIVEN `a = [1.0, 0.0, 0.0]`
- AND `b = [0.0, 1.0, 0.0]`
- WHEN `cosine_similarity(&a, &b)` is called
- THEN the result is `0.0`

### Scenario: Directory creation fails

- GIVEN a `VectorExporter` configured with output directory `/root/no-permission`
- WHEN `exporter.export(chunk)` is called
- THEN the result is `Err(ExporterError::DirectoryCreation(_))`

### Scenario: File lock acquisition fails

- GIVEN a `VectorExporter` with a valid output path
- AND another process holding an exclusive lock on the same file
- WHEN `exporter.export(chunk)` is called
- THEN the result is `Err(ExporterError::WriteError(_))` with a message containing "lock"

### Scenario: Serialization fails (invalid float)

- GIVEN a `DocumentChunk` with `embeddings = Some(vec![f32::NAN])`
- WHEN `exporter.export(chunk)` is called
- THEN the result is `Err(ExporterError::Serialization(_))`

### Scenario: Append mode preserves existing documents

- GIVEN a `VectorExporter` with `append = false` that exports 2 documents to `vectors.json`
- AND a second `VectorExporter` with `append = true` pointing to the same file
- WHEN the second exporter exports 1 additional document
- THEN the file contains 3 documents total
- AND the metadata header `total_documents` reflects the count from the most recent write

### Scenario: Metadata header written only on new file

- GIVEN an existing `vectors.json` file with valid metadata and 2 documents
- AND a `VectorExporter` with `append = true`
- WHEN `exporter.export(chunk)` is called with a new document
- THEN the existing metadata header is NOT overwritten
- AND the new document is appended to the `documents` array

### Scenario: Dimension mismatch detection

- GIVEN a `VectorExporter` that has already written documents with 384-dimensional embeddings
- AND a `DocumentChunk` with `embeddings = Some(vec![0.1; 768])` (wrong dimensions)
- WHEN `exporter.export(chunk)` is called
- THEN the result is `Err(ExporterError::InvalidConfig(_))` with a message about dimension mismatch

---

## 3. Data Contract

### Output JSON Schema

```json
{
  "metadata": {
    "format_version": 1,
    "model_name": "all-MiniLM-L6-v2",
    "dimensions": 384,
    "total_documents": 0,
    "created_at": "2026-04-01T00:00:00Z"
  },
  "documents": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "url": "https://example.com/page",
      "title": "Page Title",
      "content": "Clean text content...",
      "metadata": {
        "author": "John Doe",
        "domain": "example.com"
      },
      "timestamp": "2026-04-01T00:00:00Z",
      "embeddings": [0.012, -0.034, 0.056, ...]
    }
  ]
}
```

### Field Specifications

| Path | Type | Required | Description |
|------|------|----------|-------------|
| `metadata.format_version` | `u32` | YES | Schema version, currently `1` |
| `metadata.model_name` | `String` | YES | Embedding model identifier, default `"all-MiniLM-L6-v2"` |
| `metadata.dimensions` | `u32` | YES | Embedding vector dimension count, default `384` |
| `metadata.total_documents` | `u64` | YES | Count of documents in the `documents` array |
| `metadata.created_at` | `String` (ISO 8601) | YES | UTC timestamp of first write |
| `documents` | `array` | YES | Array of document objects |
| `documents[].id` | `String` (UUID) | YES | Chunk unique identifier |
| `documents[].url` | `String` | YES | Source URL |
| `documents[].title` | `String` | YES | Page/article title |
| `documents[].content` | `String` | YES | Clean text content |
| `documents[].metadata` | `object` | YES | Key-value metadata map |
| `documents[].timestamp` | `String` (ISO 8601) | YES | Scrape timestamp |
| `documents[].embeddings` | `array<f32>` \| `null` | NO | Embedding vector or null if not available |

### Constants

```rust
pub const VECTOR_FORMAT_VERSION: u32 = 1;
pub const DEFAULT_MODEL_NAME: &str = "all-MiniLM-L6-v2";
pub const DEFAULT_DIMENSIONS: u32 = 384;
```

---

## 4. API Contract

### 4.1 `ExportFormat` modification

```rust
// In src/domain/entities.rs — ADD variant to existing enum
pub enum ExportFormat {
    Jsonl,
    Auto,
    Vector,  // NEW: Vector JSON format with embeddings
}
```

The `extension()` method SHALL return `"json"` for `Vector`.
The `name()` method SHALL return `"Vector"` for `Vector`.
The `clap::ValueEnum` derive SHALL auto-generate CLI parsing for `"vector"`.

### 4.2 `VectorExporter` struct

```rust
// In src/infrastructure/export/vector_exporter.rs

use std::path::PathBuf;

pub struct VectorExporter {
    config: ExporterConfig,
    model_name: String,
    dimensions: u32,
}

impl VectorExporter {
    /// Create a new VectorExporter with the given configuration.
    ///
    /// # Arguments
    /// * `config` - ExporterConfig with output path and format settings
    ///
    /// # Example
    /// ```ignore
    /// let config = ExporterConfig::new(output_dir, ExportFormat::Vector, "vectors");
    /// let exporter = VectorExporter::new(config);
    /// ```
    #[must_use]
    pub fn new(config: ExporterConfig) -> Self;

    /// Create a VectorExporter with output directory and filename, using defaults
    /// for model name and dimensions.
    ///
    /// # Arguments
    /// * `output_dir` - Directory where the output file will be written
    /// * `filename` - Base filename without extension
    ///
    /// # Example
    /// ```ignore
    /// let exporter = VectorExporter::new_with_path(PathBuf::from("./output"), "vectors");
    /// ```
    #[must_use]
    pub fn new_with_path(output_dir: PathBuf, filename: impl Into<String>) -> Self;

    /// Create a VectorExporter with custom model name and dimensions.
    ///
    /// # Arguments
    /// * `config` - ExporterConfig
    /// * `model_name` - Embedding model identifier (e.g., "all-MiniLM-L6-v2")
    /// * `dimensions` - Expected embedding dimension count (e.g., 384)
    #[must_use]
    pub fn with_model(config: ExporterConfig, model_name: impl Into<String>, dimensions: u32) -> Self;
}

impl crate::domain::exporter::Exporter for VectorExporter {
    fn export(&self, document: DocumentChunk) -> ExportResult<()>;
    fn export_batch(&self, documents: Vec<DocumentChunk>) -> ExportResult<()>;
    fn config(&self) -> &ExporterConfig;
    fn format(&self) -> ExportFormat;
}
```

### 4.3 `cosine_similarity` function

```rust
// In src/infrastructure/export/vector_exporter.rs (or a dedicated math module)

/// Compute cosine similarity between two vectors.
///
/// Returns a value in the range `[-1.0, 1.0]` where:
/// - `1.0` means identical direction
/// - `0.0` means orthogonal (no similarity)
/// - `-1.0` means opposite direction
///
/// Returns `0.0` if either vector has zero magnitude.
///
/// # Arguments
/// * `a` - First vector (must be same length as `b`)
/// * `b` - Second vector (must be same length as `a`)
///
/// # Panics
/// Panics if `a.len() != b.len()`.
///
/// # Example
/// ```
/// use rust_scraper::infrastructure::export::vector_exporter::cosine_similarity;
///
/// let a = [1.0, 0.0, 0.0];
/// let b = [0.0, 1.0, 0.0];
/// assert!((cosine_similarity(&a, &b) - 0.0).abs() < f32::EPSILON);
/// ```
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32;
```

### 4.4 Internal types

```rust
// In src/infrastructure/export/vector_exporter.rs

/// Metadata header for vector export files.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VectorMetadata {
    format_version: u32,
    model_name: String,
    dimensions: u32,
    total_documents: u64,
    created_at: String,
}

/// Complete vector export file structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VectorExport {
    metadata: VectorMetadata,
    documents: Vec<DocumentChunk>,
}
```

---

## 5. Error Contract

### New `ExporterError` variant

```rust
// In src/domain/exporter.rs — ADD to existing enum

#[derive(Error, Debug)]
pub enum ExporterError {
    // ... existing variants ...

    /// Embedding dimension mismatch detected
    #[error("dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: u32, actual: usize },

    /// File already exists with incompatible format
    #[error("existing file has incompatible format")]
    IncompatibleFormat,
}
```

### Error behavior summary

| Error | Trigger | Recovery |
|-------|---------|----------|
| `DirectoryCreation` | `fs::create_dir_all` fails | Caller must fix permissions/path |
| `WriteError` | File open, lock, or write fails | Caller must retry or fix path |
| `Serialization` | `serde_json::to_string` fails (e.g., NaN) | Caller must sanitize input |
| `DimensionMismatch` | Document embedding length != configured dimensions | Caller must re-embed or reconfigure |
| `InvalidConfig` | Missing required config values | Caller must provide valid config |

---

## 6. File Changes

| File | Action | Description |
|------|--------|-------------|
| `src/domain/entities.rs` | MODIFY | Add `Vector` variant to `ExportFormat` enum; update `extension()` and `name()` match arms |
| `src/domain/exporter.rs` | MODIFY | Add `DimensionMismatch` and `IncompatibleFormat` variants to `ExporterError` |
| `src/infrastructure/export/vector_exporter.rs` | CREATE | New file: `VectorExporter` struct, `cosine_similarity` function, `VectorMetadata`/`VectorExport` types, `impl Exporter` |
| `src/infrastructure/export/mod.rs` | MODIFY | Add `pub mod vector_exporter;` and `pub use vector_exporter::VectorExporter;` |
| `src/export_factory.rs` | MODIFY | Add `ExportFormat::Vector` match arm in `create_exporter()` returning `VectorExporter` |
| `src/lib.rs` | MODIFY | Add `VectorExporter` to public re-exports from infrastructure layer |

---

## 7. Test Requirements

### Minimum Coverage

- All public functions MUST have at least one test
- `cosine_similarity` MUST have property-based tests or exhaustive edge-case coverage
- `VectorExporter::export` and `VectorExporter::export_batch` MUST test both file creation and content validation

### Required Test Cases

| Test Name | What It Validates |
|-----------|-------------------|
| `test_vector_exporter_single_document` | Single doc export creates valid JSON with metadata header |
| `test_vector_exporter_batch_documents` | Batch export writes all documents with correct count |
| `test_vector_exporter_append_mode` | Append mode adds to existing file without overwriting header |
| `test_vector_exporter_empty_batch` | Empty batch produces valid JSON with zero documents |
| `test_vector_exporter_no_embeddings` | Document without embeddings exports with `null` embeddings field |
| `test_vector_exporter_dimension_mismatch` | Wrong dimension count returns `DimensionMismatch` error |
| `test_vector_exporter_directory_creation_fails` | Invalid path returns `DirectoryCreation` error |
| `test_vector_exporter_serialization_nan` | NaN in embeddings returns `Serialization` error |
| `test_cosine_similarity_identical_vectors` | Identical vectors return `1.0` |
| `test_cosine_similarity_orthogonal_vectors` | Orthogonal vectors return `0.0` |
| `test_cosine_similarity_zero_magnitude` | Zero vector returns `0.0` |
| `test_cosine_similarity_opposite_vectors` | Opposite vectors return `-1.0` |
| `test_cosine_similarity_different_lengths_panics` | Mismatched lengths panic |
| `test_cosine_similarity_single_dimension` | Single-element vectors compute correctly |
| `test_vector_exporter_new_with_path_defaults` | `new_with_path` sets correct defaults for model/dimensions |
| `test_vector_exporter_with_model_custom` | `with_model` sets custom model name and dimensions |
| `test_export_factory_creates_vector_exporter` | Factory returns `VectorExporter` for `ExportFormat::Vector` |
| `test_vector_metadata_header_fields` | Metadata contains all required fields with correct types |

### Test Constraints

- All tests MUST use `tempfile::TempDir` for isolation — no filesystem pollution
- All tests MUST follow Arrange → Act → Assert structure
- Test names MUST be descriptive: `test_{unit}_{condition}_{expected_result}`
- Async tests MUST use `#[tokio::test]` (not applicable here — all sync)
- No `.unwrap()` in test assertions — use `assert!`, `assert_eq!`, `assert!(result.is_ok())`
- Float comparisons MUST use epsilon tolerance: `(actual - expected).abs() < f32::EPSILON`
