# Implementation Task Breakdown: Vault Auto-Detect, Quick-Save Mode, and Rich Metadata for Obsidian

## Tasks

### 1. Add `whatlang` dependency to Cargo.toml
- **Files to modify**: `Cargo.toml`
- **Description**: Add the `whatlang` crate for language detection to support rich metadata generation
- **Acceptance criteria**:
  - `whatlang = "0.6.0"` (or latest compatible version) added to dependencies section
  - Cargo builds successfully with new dependency
- **Estimated complexity**: S

### 2. Create `src/infrastructure/obsidian/` module structure (mod.rs)
- **Files to modify**: 
  - Create `src/infrastructure/obsidian/mod.rs`
  - Update `src/infrastructure/mod.rs` to include the obsidian module
- **Description**: Create module structure for Obsidian-specific functionality
- **Acceptance criteria**:
  - New directory `src/infrastructure/obsidian/` created
  - `mod.rs` file exists with basic module declaration
  - Parent module includes obsidian module
- **Estimated complexity**: S

### 3. Implement vault_detector.rs with tests
- **Files to modify**:
  - Create `src/infrastructure/obsidian/vault_detector.rs`
  - Create `src/infrastructure/obsidian/vault_detector_tests.rs` (or inline tests)
- **Description**: Implement automatic Obsidian vault detection with priority: CLI > env var > config > auto-scan
- **Acceptance criteria**:
  - VaultDetector struct with detection logic implemented
  - Function to detect vault path following priority order
  - Unit tests covering all detection scenarios
  - Integration with existing config system
- **Estimated complexity**: M

### 4. Implement metadata.rs with tests
- **Files to modify**:
  - Create `src/infrastructure/obsidian/metadata.rs`
  - Create `src/infrastructure/obsidian/metadata_tests.rs` (or inline tests)
- **Description**: Implement rich metadata generation including readingTime, language, wordCount, contentType, status
- **Acceptance criteria**:
  - Metadata struct with all required fields
  - Language detection using whatlang crate
  - Reading time calculation based on word count
  - Word count and content type detection
  - Status field implementation
  - Unit tests for all metadata generation functions
- **Estimated complexity**: M

### 5. Implement uri.rs with tests
- **Files to modify**:
  - Create `src/infrastructure/obsidian/uri.rs`
  - Create `src/infrastructure/obsidian/uri_tests.rs` (or inline tests)
- **Description**: Implement Obsidian URI handling to open notes after saving
- **Acceptance criteria**:
  - Function to generate Obsidian URI from vault path and file path
  - Function to open note using Obsidian URI (platform-appropriate)
  - Unit tests for URI generation
  - Integration test for URI opening (mocked if necessary)
- **Estimated complexity**: M

### 6. Add `--vault` and `--quick-save` CLI flags to Args
- **Files to modify**: `src/lib.rs` (Args struct)
- **Description**: Add command-line arguments for vault path and quick-save mode
- **Acceptance criteria**:
  - `--vault` flag added to Args struct with appropriate help text
  - `--quick-save` flag added to Args struct (boolean flag)
  - Both flags properly documented with help text
  - Args struct compiles successfully with new fields
- **Estimated complexity**: S

### 7. Add vault_path to ConfigDefaults
- **Files to modify**: `src/cli/config.rs` (ConfigDefaults struct)
- **Description**: Add vault path configuration to TOML config file support
- **Acceptance criteria**:
  - vault_path field added to ConfigDefaults struct
  - Field properly typed as Option<String> or similar
  - Default implementation included
  - TOML deserialization works with new field
- **Estimated complexity**: S

### 8. Wire vault detection in main.rs
- **Files to modify**: `src/main.rs`
- **Description**: Integrate vault detection logic into application startup
- **Acceptance criteria**:
  - Vault detection called during initialization
  - Detected vault path made available to application services
  - Proper error handling when no vault can be detected
  - Integration with existing configuration loading
- **Estimated complexity**: M

### 9. Wire quick-save branch in main.rs
- **Files to modify**: `src/main.rs`
- **Description**: Implement quick-save mode that scrapes and exports directly to vault inbox
- **Acceptance criteria**:
  - When `--quick-save` flag is used, bypass normal output directory logic
  - Content saved directly to vault inbox folder
  - Uses detected vault path for destination
  - Proper handling of inbox folder creation if needed
  - Maintains all other scraping functionality
- **Estimated complexity**: L

### 10. Extend frontmatter.rs with rich metadata fields
- **Files to modify**: `src/infrastructure/output/frontmatter.rs`
- **Description**: Add new fields to Frontmatter struct for rich metadata
- **Acceptance criteria**:
  - Frontmatter struct extended with readingTime, language, wordCount, contentType, status fields
  - All new fields properly serialized to YAML
  - Backward compatibility maintained (existing fields unchanged)
  - Unit tests updated to cover new fields
- **Estimated complexity**: M

### 11. Integrate metadata generation in file_saver.rs
- **Files to modify**: `src/infrastructure/output/file_saver.rs`
- **Description**: Generate rich metadata and pass it to frontmatter generation
- **Acceptance criteria**:
  - Metadata generation called during file saving process
  - Rich metadata passed to frontmatter::generate function
  - ObsidianOptions extended to support new metadata fields if needed
  - Proper error handling for metadata generation failures
  - Integration tested with existing save workflow
- **Estimated complexity**: L

### 12. Wire Obsidian URI opening in main.rs
- **Files to modify**: `src/main.rs`
- **Description**: Open saved note in Obsidian after successful save in quick-save mode
- **Acceptance criteria**:
  - After successful save in quick-save mode, generate Obsidian URI
  - Attempt to open note using platform-appropriate method
  - Non-blocking operation (doesn't wait for Obsidian to close)
  - Graceful handling when Obsidian is not installed or URI fails
  - Only triggers in quick-save mode, not regular operation
- **Estimated complexity**: M

### 13. Run full test suite and clippy
- **Files to modify**: None (verification task)
- **Description**: Ensure all new functionality works correctly and doesn't break existing code
- **Acceptance criteria**:
  - All existing tests still pass
  - All new unit tests pass
  - cargo clippy passes with no new warnings
  - cargo nextest run --test-threads 2 completes successfully
  - Manual verification of quick-save mode functionality
- **Estimated complexity**: L

## Dependencies Notes
- Task 1 must be completed before Task 4 (metadata.rs depends on whatlang)
- Tasks 2-5 can be done in parallel after Task 1
- Tasks 6-7 should be done before Tasks 8-12
- Tasks 8-12 depend on Tasks 2-7 being completed
- Task 13 should be last to verify everything works