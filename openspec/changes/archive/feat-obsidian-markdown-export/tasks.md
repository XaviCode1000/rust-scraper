# Task Breakdown: Obsidian-compatible Markdown Export

## Task 1: Add pathdiff dependency
**Description:** Add the pathdiff crate to Cargo.toml for computing relative paths
**Files to modify:** Cargo.toml
**Acceptance criteria:**
- pathdiff = "0.2.1" added to dependencies section
- cargo check passes without errors
**Estimated complexity:** S

## Task 2: Add CLI flags to Args struct
**Description:** Add Obsidian-specific CLI arguments to the Args struct in src/lib.rs
**Files to modify:** src/lib.rs
**Acceptance criteria:**
- Added --obsidian-tags flag accepting comma-separated tags
- Added --obsidian-wiki-links boolean flag to enable wiki-link conversion
- Added --obsidian-relative-assets boolean flag to enable relative asset paths
- All flags properly documented with help text
- cargo check passes
**Estimated complexity:** S

## Task 3: Add Obsidian configuration to ConfigDefaults
**Description:** Add Obsidian-specific configuration options to ConfigDefaults struct
**Files to modify:** src/cli/config.rs
**Acceptance criteria:**
- Added obsidian_tags: Option<String> field
- Added obsidian_wiki_links: bool field (default: false)
- Added obsidian_relative_assets: bool field (default: false)
- Updated default values and deserialization logic
- cargo check passes
**Estimated complexity:** S

## Task 4: Create Obsidian converter module
**Description:** Create new converter module for Obsidian-specific transformations
**Files to modify:**
- src/infrastructure/converter/obsidian.rs (new file)
- src/infrastructure/converter/mod.rs (update to export new module)
**Acceptance criteria:**
- Created obsidian.rs with functions for:
  - Converting standard markdown links to wiki-links: [text](url) → [[slug|text]]
  - Processing assets to convert absolute paths to relative paths
  - Proper handling of code blocks to avoid conversion inside them
- Added comprehensive unit tests covering:
  - Wiki-link conversion for same-domain links
  - Preservation of external links
  - Asset path relativization
  - Code block protection
- All tests pass
**Estimated complexity:** M

## Task 5: Add tags support to frontmatter
**Description:** Extend frontmatter generation to include Obsidian tags
**Files to modify:** src/infrastructure/output/frontmatter.rs
**Acceptance criteria:**
- Modified Frontmatter struct to include tags: Option<Vec<String>>
- Updated generate() function to accept tags parameter
- Modified serialization to include tags as YAML list when present
- Updated all calls to generate() to pass tags (empty when not specified)
- Added tests for tags serialization
- cargo check passes
**Estimated complexity:** S

## Task 6: Integrate wiki-links in file_saver
**Description:** Apply wiki-link conversion to markdown content in file_saver
**Files to modify:** src/infrastructure/output/file_saver.rs
**Acceptance criteria:**
- Added logic to detect when wiki-link conversion is enabled
- Integrated call to obsidian converter wiki-link function
- Applied conversion after markdown conversion but before syntax highlighting
- Preserved existing functionality when feature is disabled
- Added tests verifying wiki-link conversion
- cargo check passes
**Estimated complexity:** M

## Task 7: Integrate relative assets in file_saver
**Description:** Apply relative asset path conversion in file_saver
**Files to modify:** src/infrastructure/output/file_saver.rs
**Acceptance criteria:**
- Added logic to detect when relative asset conversion is enabled
- Integrated call to obsidian converter asset path function
- Used pathdiff crate to compute relative paths from .md file to assets
- Applied conversion after wiki-link processing
- Preserved existing functionality when feature is disabled
- Added tests verifying asset path relativization
- cargo check passes
**Estimated complexity:** M

## Task 8: Wire Obsidian features in scrape pipeline
**Description:** Connect CLI args/config to the file saving functionality
**Files to modify:**
- src/cli/mod.rs (or wherever scrape pipeline is orchestrated)
- src/lib.rs (if needed to pass args through)
**Acceptance criteria:**
- Obsidian CLI args are parsed and passed through to save_results function
- Configuration values are properly loaded and merged with CLI args
- save_results function signature updated to accept Obsidian options
- All three Obsidian features (tags, wiki-links, relative assets) properly wired
- cargo check passes
**Estimated complexity:** M

## Task 9: Run full test suite and clippy
**Description:** Verify implementation doesn't break existing functionality
**Files to modify:** None (verification only)
**Acceptance criteria:**
- cargo nextest run --test-threads 2 passes
- cargo clippy -- -D warnings passes
- cargo doc --no-deps passes
- Manual verification of Obsidian features works as expected
**Estimated complexity:** L (time-wise, but straightforward)