# Exploration Report: Issue #23 - Vault auto-detect, quick-save mode, and rich metadata for Obsidian

## Current State of Obsidian Integration (Issue #22)

### 1. Obsidian Options Structure
Located in `src/infrastructure/output/file_saver.rs`:
```rust
#[derive(Debug, Clone, Default)]
pub struct ObsidianOptions {
    /// Convert same-domain links to [[wiki-link]] syntax
    pub wiki_links: bool,
    /// Tags to include in YAML frontmatter
    pub tags: Vec<String>,
    /// Rewrite asset paths as relative to the .md file
    pub relative_assets: bool,
}
```

### 2. Frontmatter Generation
Located in `src/infrastructure/output/frontmatter.rs`:
```rust
pub fn generate(
    title: &str,
    url: &str,
    date: Option<&str>,
    author: Option<&str>,
    excerpt: Option<&str>,
    tags: &[String],
) -> String {
    // Generates YAML with: title, url, date, author, excerpt, tags
}
```

### 3. CLI Arguments
Located in `src/lib.rs` (Args struct):
- `obsidian_wiki_links`: boolean flag
- `obsidian_tags`: comma-separated tags (Option<Vec<String>>)
- `obsidian_relative_assets`: boolean flag

### 4. Configuration Defaults
Located in `src/cli/config.rs` (ConfigDefaults):
- `obsidian_wiki_links`: Option<bool>
- `obsidian_tags`: Option<String>
- `obsidian_relative_assets`: Option<bool>

### 5. Main Pipeline
Located in `src/main.rs`:
- Obsidian options constructed from CLI args
- Passed to `save_results()` function

## What Needs to be Implemented for Issue #23

### Feature 1: Auto-detect Vault Path
**Search Order**: CLI flag > env var OBSIDIAN_VAULT > config file > auto-scan for `.obsidian/` in common locations

### Feature 2: Quick-Save Mode
**Behavior**: `--obsidian --quick-save` → scrape + export directly to vault inbox, no TUI, no confirmation

### Feature 3: Rich Metadata for Dataview
**Fields to Add to Frontmatter**:
- `readingTime`: Estimated reading time in minutes
- `language`: Detected language code (e.g., "en", "es")
- `wordCount`: Total word count
- `contentType`: Content type (article, documentation, etc.)
- `status`: Processing status (e.g., "processed")

### Feature 4: Obsidian URI Integration
**Action**: Open note in Obsidian after saving using `obsidian://open?vault=...&file=...`

## Key Files That Need Changes

### 1. `src/lib.rs` - Add New CLI Arguments
```rust
// In Args struct:
/// Path to Obsidian vault for direct saving
#[arg(long, env = "OBSIDIAN_VAULT")]
pub vault: Option<std::path::PathBuf>,

/// Enable quick-save mode (bypass TUI and confirmation)
#[arg(long, env = "OBSIDIAN_QUICK_SAVE")]
pub quick_save: bool,

/// Inbox folder within vault for quick-save (default: "Inbox")
#[arg(long, default_value = "Inbox", env = "OBSIDIAN_INBOX_FOLDER")]
pub inbox_folder: String,
```

### 2. `src/cli/config.rs` - Extend ConfigDefaults
```rust
#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(default)]
pub struct ConfigDefaults {
    // ... existing fields ...
    /// Default Obsidian vault path
    pub vault_path: Option<String>,
    /// Default quick-save mode setting
    pub quick_save: Option<bool>,
    /// Default inbox folder name
    pub inbox_folder: Option<String>,
}
```

### 3. `src/main.rs` - Modify Pipeline
Key modifications needed:
- Add vault detection function (CLI > env > config > auto-scan)
- Add quick-save branch that bypasses TUI
- Modify save path when vault is provided and quick-save is enabled
- Add Obsidian URI opening after successful save

### 4. `src/infrastructure/output/frontmatter.rs` - Extend Frontmatter
```rust
#[derive(Debug, Serialize)]
struct Frontmatter {
    // ... existing fields ...
    /// Estimated reading time in minutes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reading_time: Option<u32>,
    /// Detected language (ISO 639-1 code)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Word count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub word_count: Option<usize>,
    /// Content type (article, documentation, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    /// Processing status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}
```

### 5. `src/infrastructure/output/file_saver.rs` - Modify save_results
- Accept vault path parameter
- When vault path provided and quick-save enabled, save to vault/inbox
- Construct and attempt to open Obsidian URI after save

## Dependencies Needed

### 1. Language Detection
Add to `Cargo.toml`:
```toml
[dependencies]
whatlang = "0.16"
```

### 2. Word Counting
Can be implemented manually or use:
```toml
wordcount = "0.1"
```

### 3. Content Type Detection
Start with simple heuristics; may enhance later

### 4. Obsidian URI Opening
Use platform-appropriate methods:
- Linux: `xdg-open`
- macOS: `open`
- Windows: `start`

## Implementation Approach

### Phase 1: Vault Detection & Quick-Save
1. Implement vault detection function:
   ```rust
   fn detect_vault_path(cli_arg: Option<PathBuf>, env_var: Option<String>, config_opt: Option<String>) -> Option<PathBuf>
   ```
2. Add quick-save logic in main pipeline:
   - Skip TUI when `--quick-save` is set
   - Save directly to `<vault>/<inbox_folder>/` instead of output directory

### Phase 2: Rich Metadata
1. Extend `Frontmatter` struct and `generate` function
2. Add word counting utility:
   ```rust
   fn count_words(text: &str) -> usize {
       text.split_whitespace().count()
   }
   ```
3. Add reading time calculation:
   ```rust
   fn calculate_reading_time(word_count: usize) -> u32 {
       ((word_count as f32 / 200.0).ceil() as u32).max(1)
   }
   ```
4. Add language detection using `whatlang`:
   ```rust
   fn detect_language(text: &str) -> Option<String> {
       whatlang::detect(text)
           .map(|lang| lang.lang().to_string())
   }
   ```
5. Implement simple content type detection:
   ```rust
   fn detect_content_type(url: &url::Url) -> Option<String> {
       let path = url.path().to_lowercase();
       if path.contains("/blog/") || path.contains("/article/") {
           Some("article".to_string())
       } else if path.contains("/docs/") || path.contains("/documentation/") {
           Some("documentation".to_string())
       } else if path.contains("/api/") {
           Some("api_reference".to_string())
       } else {
           Some("webpage".to_string())
       }
   }
   ```

### Phase 3: Obsidian URI Integration
1. After successful save, construct URI:
   ```rust
   fn construct_obsidian_uri(vault_name: &str, file_path_in_vault: &str) -> String {
       format!("obsidian://open?vault={}&file={}", vault_name, file_path_in_vault)
   }
   ```
2. Open URI using platform-appropriate method:
   ```rust
   #[cfg(target_os = "linux")]
   fn open_uri(uri: &str) -> Result<()> {
       std::process::Command::new("xdg-open").arg(uri).status()?;
       Ok(())
   }
   // Similar for macOS and Windows
   ```

## Potential Challenges & Solutions

### Challenge 1: Vault Detection Accuracy
- **Problem**: Multiple `.obsidian` directories might exist (e.g., backups, templates)
- **Solution**: Prioritize most recently modified vault, or require explicit selection when ambiguous

### Challenge 2: Language Detection Performance
- **Problem**: `whatlang` might add significant binary size/compile time
- **Solution**: Make it optional feature, or only enable when needed for frontmatter

### Challenge 3: Content Type Detection Accuracy
- **Problem**: Simple URL heuristics may misclassify content
- **Solution**: Start simple, enhance later with HTML/meta tag analysis if needed

### Challenge 4: Cross-Platform URI Opening
- **Problem**: Different commands needed for Linux/macOS/Windows
- **Solution**: Use conditional compilation with `#[cfg(target_os = "...")]`

### Challenge 5: Quick-Save Mode Interactions
- **Problem**: Ensuring quick-save still respects resume mode, filtering, etc.
- **Solution**: Apply quick-save after URL filtering/resume logic, just change output destination

## Recommendations

1. **Start Simple**: Implement vault detection and quick-save first as they provide immediate value
2. **Leverage Existing Patterns**: Follow the same patterns used for existing Obsidian options
3. **Feature Flags**: Consider making advanced features (language detection) optional via feature flags
4. **Testing**: Add unit tests for vault detection logic and frontmatter generation
5. **User Feedback**: Provide clear logging when vault is auto-detected or when quick-save mode is active
6. **Error Handling**: Gracefully fall back to standard output directory if vault detection fails

## Files Summary

### New Dependencies Needed:
- `whatlang` for language detection
- Optional: `wordcount` for word counting (can implement manually)

### Files to Modify:
1. `src/lib.rs` - Add CLI args
2. `src/cli/config.rs` - Add config defaults
3. `src/main.rs` - Modify pipeline logic
4. `src/infrastructure/output/frontmatter.rs` - Extend frontmatter
5. `src/infrastructure/output/file_saver.rs` - Modify save logic

### Estimated Effort:
- Vault detection & quick-save: 1-2 days
- Rich metadata: 2-3 days
- Obsidian URI integration: 0.5-1 day
- Testing & polishing: 1 day

Total: ~4-6 days