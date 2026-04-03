# Tasks: VectorExporter Implementation

## Phase 1: Foundation — Domain Changes

- [x] 1.1 Add `Vector` variant to `ExportFormat` enum in `src/domain/entities.rs` with `.json` extension and "Vector" name; update `extension()` and `name()` match arms.
- [x] 1.2 Add `DimensionMismatch { expected: usize, actual: usize }` variant to `ExporterError` in `src/domain/exporter.rs` with thiserror display message.

## Phase 2: Core Implementation — VectorExporter

- [x] 2.1 Create `src/infrastructure/export/vector_exporter.rs` with `VectorExporter` struct holding `ExporterConfig`; implement constructor `new()` and `new_with_path()`.
- [x] 2.2 Implement `get_writer()` method: create dirs, acquire fs2 file lock, open file with BufWriter (append or truncate based on config).
- [x] 2.3 Implement `write_metadata_header()` helper: writes `{ "format_version": "1.0", "model_name": null, "dimensions": null, "total_documents": 0, "created_at": "<ISO8601>", "documents": [` — only on new files; in append mode, detect existing header and position writer before closing `]`.
- [x] 2.4 Implement `serialize_document()` helper: serializes `DocumentChunk` to JSON with dimension validation (returns `DimensionMismatch` if embeddings present and length differs from header dimensions).
- [x] 2.5 Implement `Exporter::export()`: get writer, write metadata if first doc, serialize document with null embeddings support (R13), write comma-separated JSON into documents array, flush.
- [x] 2.6 Implement `Exporter::export_batch()`: get writer, write metadata if first batch, iterate docs with commas between entries, serialize each with dimension validation, flush once at end.
- [x] 2.7 Implement pure Rust `cosine_similarity(a: &[f32], b: &[f32]) -> f32` as a module-level function: returns 0.0 for zero-magnitude vectors, validates equal dimensions.

## Phase 3: Integration — Wiring

- [x] 3.1 Add `pub mod vector_exporter;` and `pub use vector_exporter::VectorExporter;` to `src/infrastructure/export/mod.rs`.
- [x] 3.2 Update `create_exporter()` in `src/export_factory.rs`: add `ExportFormat::Vector` match arm returning `VectorExporter`; update `Auto` detection to check for existing `.json` vector export files.
- [x] 3.3 Update `Args::export_format` in `src/lib.rs`: add "vector" to the CLI help text description for ExportFormat values.
- [x] 3.4 Re-export `VectorExporter` in `src/lib.rs` under `infrastructure::export`.

## Phase 4: Testing

- [x] 4.1 Write tests for `ExportFormat::Vector`: extension is "json", name is "Vector" (update existing test in `src/domain/exporter.rs`).
- [x] 4.2 Write test for single doc export with embeddings in `vector_exporter.rs`: verify JSON structure, metadata header, documents array.
- [x] 4.3 Write test for batch export with multiple docs: verify all docs present, proper comma separation, valid JSON.
- [x] 4.4 Write test for append mode: write first batch, then append second batch, verify header preserved and all docs present.
- [x] 4.5 Write test for export without embeddings (R13): verify `embeddings` field is null in JSON output.
- [x] 4.6 Write test for empty batch export: verify no crash, valid JSON structure with empty documents array.
- [x] 4.7 Write test for dimension mismatch: export doc with different dimension than first doc, verify `DimensionMismatch` error.
- [x] 4.8 Write test for `cosine_similarity`: normal vectors, zero-magnitude returns 0.0, identical vectors return 1.0, orthogonal return 0.0.
- [x] 4.9 Write test for directory creation failure: pass invalid path, verify `DirectoryCreation` error.
- [x] 4.10 Write test for serialization failure (NaN in embeddings): verify error is propagated, not panicked.
