# Exploration: LanceDB Vector DB Integration (Issue #15)

## Current State

The project is a production-ready Rust web scraper (v1.0.7) with Clean Architecture:
- **Domain layer**: `Exporter` trait (sync), `ExporterConfig`, `DocumentChunk` entity with `embeddings: Option<Vec<f32>>` field
- **Infrastructure layer**: Only `JsonlExporter` exists at `src/infrastructure/export/jsonl_exporter.rs`
- **Export factory**: `src/export_factory.rs` creates exporters based on `ExportFormat` enum (only `Jsonl` and `Auto` variants)
- **CLI**: `--export-format` flag via clap, currently accepts `jsonl` or `auto`
- **Feature flags**: `ai`, `images`, `documents`, `full` — all optional
- **MSRV**: 1.88, Rust edition 2021
- **HTTP**: Uses `wreq` (not reqwest), `tokio` for async runtime
- **Hardware target**: Intel Haswell i5-4590 (4 cores, no HT), 8GB DDR3, HDD storage

## Affected Areas

- `Cargo.toml` — new dependency + feature flag `vector-db`
- `src/domain/entities.rs` — `ExportFormat` enum needs `Vector` variant
- `src/domain/exporter.rs` — `Exporter` trait is sync; LanceDB is fully async
- `src/infrastructure/export/mod.rs` — new `vector_exporter` module
- `src/infrastructure/export/vector_exporter.rs` — NEW file
- `src/export_factory.rs` — new match arm for `ExportFormat::Vector`
- `src/lib.rs` — feature-gated re-exports
- `src/main.rs` / `src/lib.rs` (Args) — new `--export-format vector` CLI option
- `rust-version` in Cargo.toml — bump from 1.88 to 1.91

## Approaches

### 1. **LanceDB Embedded (as proposed in issue)**
   - **Pros**: Production-grade, supports IVF-PQ indexing, full-text search + SQL, multi-modal, active development (47 releases), 9.7K GitHub stars
   - **Cons**: 
     - MSRV 1.91 (requires bump from 1.88)
     - ~200+ transitive dependencies (Arrow 57.2 + DataFusion 52.1 + 12 lance-* crates)
     - Binary size impact: +80-150MB (issue estimate of +50-80MB is likely conservative; Arrow+DataFusion tree alone is massive)
     - ALL operations are async — conflicts with sync `Exporter` trait
     - `lzma-sys` transitive FFI dependency (dynamic linking by default)
     - Compile time on Haswell HDD: likely 10-20+ minutes first build
     - rand 0.9 dependency conflicts with project's rand 0.8
   - **Effort**: High

### 2. **sqlite-vec**
   - **Pros**: Extremely lightweight (222KB crate), SQLite extension runs anywhere, 7.3K stars, 1M+ downloads, supports KNN with vec0 virtual tables, zero-copy vector passing via zerocopy
   - **Cons**: 
     - FFI-based (embeds C source, compiles via `cc` crate)
     - Requires `rusqlite` with `bundled` feature (another FFI dependency)
     - Pre-v1 API (breaking changes expected)
     - No built-in indexing beyond flat/HNSW in vec0
     - Rust SDK docs are work-in-progress
   - **Effort**: Medium

### 3. **usearch (USearch)**
   - **Pros**: Single-file HNSW implementation, extremely fast, 4K stars, Rust bindings available, minimal dependencies, supports SIMD
   - **Cons**: 
     - C++ FFI (via bindgen)
     - No persistence layer — just in-memory index (would need custom serialization)
     - No metadata storage — vectors only
     - Not a database, just a search engine
   - **Effort**: Medium-High (need to build persistence layer)

### 4. **qdrant-client**
   - **Pros**: Production-grade, 30K stars, excellent Rust SDK, mature API
   - **Cons**: 
     - Requires separate server process (Docker or cloud)
     - NOT embedded — disqualifies it for a CLI tool
     - Overkill for this use case
   - **Effort**: Low (but wrong architecture for CLI tool)

### 5. **tantivy + vector extensions**
   - **Pros**: 15K stars, 100% Rust (zero FFI), excellent full-text search, already Rust-native
   - **Cons**: 
     - Vector search is experimental/not first-class
     - Would need custom vector integration
     - Complex to set up hybrid search
   - **Effort**: High

### 6. **Custom flat index (cosine similarity on Vec<f32>)**
   - **Pros**: Zero dependencies, 100% Rust, trivial to implement, works with existing `DocumentChunk.embeddings` field, no MSRV bump, no FFI, instant compile
   - **Cons**: 
     - O(n) search — fine for <10K vectors, degrades beyond
     - No persistence beyond JSONL/JSON files
     - No advanced indexing (IVF, HNSW, PQ)
     - No hybrid search
   - **Effort**: Low

## Recommendation

**NO-GO for LanceDB as currently proposed.** The dependency cost is prohibitive for a CLI tool targeting 8GB RAM / HDD systems.

**Recommended approach: Phased strategy**

### Phase 1 (v1.2): Custom flat index with JSON persistence
- Implement `VectorExporter` using flat cosine similarity on `DocumentChunk.embeddings`
- Store as `.lancedb/` directory with JSON files (compatible naming, future-proof)
- Zero new dependencies, zero MSRV bump, zero FFI
- O(n) search is perfectly fine for typical scraper output (<5K documents)
- Can be implemented in ~200 lines of Rust

### Phase 2 (future): Upgrade path to LanceDB or sqlite-vec
- When users need >10K vectors or advanced indexing, they can:
  - Enable `--features vector-db` for LanceDB
  - Or use the flat index as export format and import into any vector DB externally

This keeps the default build lean while providing an opt-in heavy path.

## Risks

- **LanceDB dependency bloat**: Arrow 57 + DataFusion 52 + 12 lance crates = ~200+ transitive deps. On Haswell HDD, first compile could take 15-25 minutes.
- **rand version conflict**: LanceDB pulls rand 0.9, project uses rand 0.8. Both will coexist (like dashmap 5/6), increasing binary size.
- **lzma-sys FFI**: Transitive dependency requiring dynamic linking — conflicts with project's zero-FFI principle.
- **Async/sync mismatch**: `Exporter` trait is sync; LanceDB is fully async. Would need `tokio::block_on` wrapper or trait redesign.
- **MSRV bump**: 1.88 → 1.91 breaks compatibility with older toolchains.

## Ready for Proposal

**Yes** — the exploration is complete. The orchestrator should proceed to create a change proposal with the phased approach (flat index first, LanceDB optional behind feature flag). The key insight is that LanceDB's dependency tree is too heavy for the project's hardware target and CLI nature, but a lightweight flat index satisfies 95% of use cases with zero cost.
