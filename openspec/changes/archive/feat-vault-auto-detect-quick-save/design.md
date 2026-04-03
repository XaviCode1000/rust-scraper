# Technical Design: Vault Auto-Detect, Quick-Save Mode, and Rich Metadata for Obsidian

> **Issue:** #23
> **Branch:** `feat/vault-auto-detect-quick-save`
> **Depends on:** #22 (Obsidian Markdown export) — ALREADY MERGED
> **Version target:** v1.6

---

## 1. Architecture Overview

### 1.1 Clean Architecture Placement

This feature adds three new capabilities, all sitting at the **Infrastructure layer** (vault detection, metadata generation) and **Adapters layer** (CLI flags, URI opening, quick-save orchestration):

```
┌─────────────────────────────────────────────────┐
│  Adapters                                       │
│  src/main.rs         ← quick-save branch, URI   │
│  src/lib.rs          ← --vault, --quick-save    │
│  src/cli/config.rs   ← vault_path config        │
├─────────────────────────────────────────────────┤
│  Infrastructure                                 │
│  src/infrastructure/obsidian/                   │
│    vault_detector.rs  ← detect_vault()           │
│    metadata.rs        ← ObsidianRichMetadata     │
│    uri.rs             ← open_in_obsidian()       │
├─────────────────────────────────────────────────┤
│  Infrastructure (existing, modified)            │
│  src/infrastructure/output/frontmatter.rs       │
│    ← Rich metadata fields added to Frontmatter  │
│  src/infrastructure/converter/obsidian.rs       │
│    ← Unchanged (wiki-links, asset paths)        │
├─────────────────────────────────────────────────┤
│  Application                                    │
│    ← No changes (pure orchestration)            │
├─────────────────────────────────────────────────┤
│  Domain                                         │
│    ← No changes (pure business logic)           │
└─────────────────────────────────────────────────┘
```

### 1.2 Why a New `infrastructure/obsidian/` Module?

The existing `infrastructure/converter/obsidian.rs` handles **content transformations** (wiki-link conversion, asset path resolution). The new `infrastructure/obsidian/` module handles **Obsidian ecosystem integration** (vault detection, rich metadata, URI protocol). These are orthogonal concerns:

| Module | Responsibility | Layer |
|--------|---------------|-------|
| `converter/obsidian.rs` | Markdown transformations | Infrastructure |
| `obsidian/vault_detector.rs` | Filesystem vault discovery | Infrastructure |
| `obsidian/metadata.rs` | Rich metadata generation | Infrastructure |
| `obsidian/uri.rs` | Obsidian URI protocol | Adapters (platform-specific) |

**Decision:** Place `uri.rs` in `infrastructure/obsidian/` rather than `adapters/` because it's a simple `Command::new("xdg-open")` call — not a full adapter pattern. The Adapters layer is reserved for TUI, CLI, and detector integrations.

### 1.3 Module Registration

```rust
// src/infrastructure/mod.rs — add:
pub mod obsidian;

// src/infrastructure/obsidian/mod.rs — new:
pub mod metadata;
pub mod uri;
pub mod vault_detector;
```

---

## 2. Module Design

### 2.1 `vault_detector.rs` — Vault Auto-Detection

```rust
//! Obsidian vault auto-detection.
//!
//! Searches for Obsidian vaults using a priority-ordered strategy:
//! 1. Explicit CLI `--vault` flag
//! 2. `OBSIDIAN_VAULT` environment variable
//! 3. TOML config file `vault_path`
//! 4. Auto-scan common locations for `.obsidian/` marker

use std::path::{Path, PathBuf};

/// Result of vault detection.
#[derive(Debug, Clone)]
pub enum VaultDetection {
    /// Vault found at explicit path (CLI, env, or config).
    Explicit(PathBuf),
    /// Vault auto-detected at path.
    AutoDetected(PathBuf),
    /// No vault found — will use output directory as fallback.
    NotFound,
}

impl VaultDetection {
    /// Get the vault root path if detected.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::Explicit(p) | Self::AutoDetected(p) => Some(p),
            Self::NotFound => None,
        }
    }

    /// Check if a vault was found (explicit or auto-detected).
    #[must_use]
    pub fn is_found(&self) -> bool {
        matches!(self, Self::Explicit(_) | Self::AutoDetected(_))
    }
}

/// Detect an Obsidian vault using priority-ordered search.
///
/// # Search Order
/// 1. `explicit_path` — from CLI `--vault` flag
/// 2. `OBSIDIAN_VAULT` environment variable
/// 3. `config_path` — from TOML config `vault_path` field
/// 4. Auto-scan common locations (see `scan_common_locations()`)
///
/// # Arguments
/// - `explicit_path` — Optional explicit vault path from CLI
/// - `config_path` — Optional vault path from config file
///
/// # Returns
/// `VaultDetection` indicating the result
pub fn detect_vault(
    explicit_path: Option<&Path>,
    config_path: Option<&str>,
) -> VaultDetection {
    // Priority 1: CLI flag
    if let Some(path) = explicit_path {
        if is_valid_vault(path) {
            return VaultDetection::Explicit(path.to_path_buf());
        }
        // Invalid explicit path — log warning, fall through
        tracing::warn!("Explicit vault path not valid: {}", path.display());
    }

    // Priority 2: Environment variable
    if let Ok(env_path) = std::env::var("OBSIDIAN_VAULT") {
        let path = PathBuf::from(&env_path);
        if is_valid_vault(&path) {
            return VaultDetection::Explicit(path);
        }
        tracing::warn!("OBSIDIAN_VAULT env var not valid: {}", env_path);
    }

    // Priority 3: Config file
    if let Some(config_str) = config_path {
        let path = PathBuf::from(config_str);
        if is_valid_vault(&path) {
            return VaultDetection::Explicit(path);
        }
        tracing::warn!("Config vault_path not valid: {}", config_str);
    }

    // Priority 4: Auto-scan
    if let Some(path) = scan_common_locations() {
        return VaultDetection::AutoDetected(path);
    }

    VaultDetection::NotFound
}

/// Check if a path is a valid Obsidian vault (contains `.obsidian/` directory).
fn is_valid_vault(path: &Path) -> bool {
    path.join(".obsidian").is_dir()
}

/// Scan common locations for Obsidian vaults.
///
/// Search order:
/// 1. `~/Obsidian/` (default vault folder)
/// 2. `~/Documents/Obsidian/`
/// 3. `~/.obsidian/` (hidden vault at home)
/// 4. Current working directory (if it contains `.obsidian/`)
///
/// Returns the first valid vault found, or None.
fn scan_common_locations() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let candidates = [
        home.join("Obsidian"),
        home.join("Documents").join("Obsidian"),
        home.join(".obsidian"),
        std::env::current_dir().ok()?,
    ];

    candidates.iter().find(|p| is_valid_vault(p)).cloned()
}

/// Resolve the save target directory within a vault.
///
/// When `--quick-save` is used, content goes to `<vault>/Inbox/`.
/// When `--vault` is used without quick-save, content goes to
/// `<vault>/<output_subdir>/` preserving the existing output structure.
///
/// # Arguments
/// - `vault_path` — Root of the Obsidian vault
/// - `quick_save` — If true, use Inbox folder
/// - `domain` — Domain name for subfolder organization
///
/// # Returns
/// Resolved output directory path
#[must_use]
pub fn resolve_vault_output(
    vault_path: &Path,
    quick_save: bool,
    domain: &str,
) -> PathBuf {
    if quick_save {
        vault_path.join("Inbox")
    } else {
        vault_path.join(domain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_is_valid_vault_true() {
        let tmp = std::env::temp_dir().join("test_vault_valid");
        fs::create_dir_all(tmp.join(".obsidian")).unwrap();
        assert!(is_valid_vault(&tmp));
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_is_valid_vault_false() {
        let tmp = std::env::temp_dir().join("test_vault_invalid");
        fs::create_dir_all(&tmp).unwrap();
        assert!(!is_valid_vault(&tmp));
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_detect_vault_explicit_path() {
        let tmp = std::env::temp_dir().join("test_vault_explicit");
        fs::create_dir_all(tmp.join(".obsidian")).unwrap();
        let result = detect_vault(Some(&tmp), None);
        assert!(matches!(result, VaultDetection::Explicit(_)));
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_detect_vault_not_found() {
        let result = detect_vault(None, None);
        // In test env, no vault should be found
        assert!(matches!(result, VaultDetection::NotFound));
    }

    #[test]
    fn test_resolve_vault_output_quick_save() {
        let vault = PathBuf::from("/home/user/Obsidian");
        let result = resolve_vault_output(&vault, true, "example.com");
        assert_eq!(result, PathBuf::from("/home/user/Obsidian/Inbox"));
    }

    #[test]
    fn test_resolve_vault_output_domain() {
        let vault = PathBuf::from("/home/user/Obsidian");
        let result = resolve_vault_output(&vault, false, "example.com");
        assert_eq!(result, PathBuf::from("/home/user/Obsidian/example.com"));
    }
}
```

### 2.2 `metadata.rs` — Rich Metadata Generation

```rust
//! Rich Obsidian metadata generation.
//!
//! Generates extended metadata fields for Obsidian frontmatter:
//! - `wordCount` — Total word count of content
//! - `readingTime` — Estimated reading time in minutes (200 WPM)
//! - `language` — Detected language (ISO 639-1 code)
//! - `contentType` — Inferred content type (article, documentation, etc.)
//! - `scrapeDate` — ISO 8601 timestamp of when content was scraped
//! - `source` — Original URL (alias for Dataview compatibility)

use chrono::Utc;
use serde::Serialize;

/// Content type classification for scraped pages.
///
/// Used for template matching and Dataview queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    /// Standard article or blog post
    Article,
    /// Technical documentation or API reference
    Documentation,
    /// Forum post or discussion thread
    Forum,
    /// Product page or landing page
    Product,
    /// Generic/unknown content type
    Other,
}

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Article => write!(f, "article"),
            Self::Documentation => write!(f, "documentation"),
            Self::Forum => write!(f, "forum"),
            Self::Product => write!(f, "product"),
            Self::Other => write!(f, "other"),
        }
    }
}

/// Rich metadata for Obsidian frontmatter.
///
/// Extends the basic frontmatter fields with Obsidian-specific
/// metadata for Dataview queries and power-user workflows.
#[derive(Debug, Clone, Serialize)]
pub struct ObsidianRichMetadata {
    /// Total word count of the content
    pub word_count: usize,
    /// Estimated reading time in minutes (200 WPM average)
    pub reading_time: usize,
    /// Detected language (ISO 639-1 code, e.g. "en", "es", "fr")
    pub language: String,
    /// Inferred content type
    pub content_type: ContentType,
    /// Scrape timestamp (ISO 8601)
    pub scrape_date: String,
    /// Source URL (alias for Dataview compatibility)
    pub source: String,
}

impl ObsidianRichMetadata {
    /// Generate rich metadata from scraped content.
    ///
    /// # Arguments
    /// - `content` — The cleaned text content
    /// - `url` — Original URL of the scraped page
    ///
    /// # Performance
    /// Language detection runs on the first 1024 bytes only —
    /// typically <1ms for short texts.
    pub fn from_content(content: &str, url: &str) -> Self {
        let word_count = count_words(content);
        let reading_time = estimate_reading_time(word_count);
        let language = detect_language(content);
        let content_type = infer_content_type(content, url);
        let scrape_date = Utc::now().format("%Y-%m-%dT%H:%M:%S%z").to_string();

        Self {
            word_count,
            reading_time,
            language,
            content_type,
            scrape_date,
            source: url.to_string(),
        }
    }
}

/// Count words in text using whitespace separation.
///
/// Uses a simple split-based approach — no Unicode segmentation
/// needed for word count estimation.
#[inline]
fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Estimate reading time in minutes at 200 WPM (average adult).
///
/// Rounds up to nearest minute, minimum 1 minute.
#[inline]
fn estimate_reading_time(word_count: usize) -> usize {
    if word_count == 0 {
        return 1;
    }
    (word_count as f64 / 200.0).ceil() as usize
}

/// Detect the language of text using whatlang.
///
/// Analyzes only the first 1024 bytes for performance.
/// Falls back to "unknown" if detection fails or confidence
/// is below 0.3.
fn detect_language(text: &str) -> String {
    use whatlang::detect;

    // Limit to first 1024 bytes for performance
    let sample = if text.len() > 1024 {
        &text[..1024]
    } else {
        text
    };

    match detect(sample) {
        Some(info) if info.confidence() >= 0.3 => {
            info.lang().code().to_string()
        }
        _ => "unknown".to_string(),
    }
}

/// Infer content type from URL patterns and content heuristics.
///
/// Uses a simple rule-based approach:
/// - URLs containing `/doc`, `/docs`, `/api` → Documentation
/// - URLs containing `/forum`, `/thread`, `/discussion` → Forum
/// - URLs containing `/product`, `/shop`, `/store` → Product
/// - Content with >500 words and `<article>` or `<main>` tags → Article
/// - Default → Other
fn infer_content_type(content: &str, url: &str) -> ContentType {
    let url_lower = url.to_lowercase();

    // URL-based heuristics (fast path)
    if url_lower.contains("/doc") || url_lower.contains("/api") {
        return ContentType::Documentation;
    }
    if url_lower.contains("/forum")
        || url_lower.contains("/thread")
        || url_lower.contains("/discussion")
    {
        return ContentType::Forum;
    }
    if url_lower.contains("/product")
        || url_lower.contains("/shop")
        || url_lower.contains("/store")
    {
        return ContentType::Product;
    }

    // Content-based heuristic
    let word_count = count_words(content);
    if word_count > 500 {
        return ContentType::Article;
    }

    ContentType::Other
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_words_empty() {
        assert_eq!(count_words(""), 0);
    }

    #[test]
    fn test_count_words_simple() {
        assert_eq!(count_words("hello world foo bar"), 4);
    }

    #[test]
    fn test_count_words_multiline() {
        let text = "line one\nline two\nline three";
        assert_eq!(count_words(text), 6);
    }

    #[test]
    fn test_estimate_reading_time_zero() {
        assert_eq!(estimate_reading_time(0), 1);
    }

    #[test]
    fn test_estimate_reading_time_under_minute() {
        assert_eq!(estimate_reading_time(50), 1);
    }

    #[test]
    fn test_estimate_reading_time_exact() {
        assert_eq!(estimate_reading_time(200), 1);
        assert_eq!(estimate_reading_time(201), 2);
        assert_eq!(estimate_reading_time(400), 2);
    }

    #[test]
    fn test_detect_language_english() {
        let text = "This is a clear English sentence with common words.";
        let lang = detect_language(text);
        assert_eq!(lang, "en");
    }

    #[test]
    fn test_detect_language_spanish() {
        let text = "Este es un claro ejemplo de texto en español con palabras comunes.";
        let lang = detect_language(text);
        assert_eq!(lang, "es");
    }

    #[test]
    fn test_detect_language_too_short() {
        let text = "xyz";
        let lang = detect_language(text);
        assert_eq!(lang, "unknown");
    }

    #[test]
    fn test_infer_content_type_documentation() {
        assert_eq!(
            infer_content_type("", "https://example.com/docs/api"),
            ContentType::Documentation
        );
    }

    #[test]
    fn test_infer_content_type_forum() {
        assert_eq!(
            infer_content_type("", "https://example.com/forum/thread/123"),
            ContentType::Forum
        );
    }

    #[test]
    fn test_infer_content_type_article() {
        let content = "word ".repeat(600);
        assert_eq!(
            infer_content_type(&content, "https://example.com/blog/post"),
            ContentType::Article
        );
    }

    #[test]
    fn test_infer_content_type_other() {
        assert_eq!(
            infer_content_type("short", "https://example.com/page"),
            ContentType::Other
        );
    }

    #[test]
    fn test_content_type_display() {
        assert_eq!(ContentType::Article.to_string(), "article");
        assert_eq!(ContentType::Documentation.to_string(), "documentation");
        assert_eq!(ContentType::Other.to_string(), "other");
    }

    #[test]
    fn test_rich_metadata_generation() {
        let content = "word ".repeat(400);
        let meta = ObsidianRichMetadata::from_content(&content, "https://example.com/article");

        assert_eq!(meta.word_count, 400);
        assert_eq!(meta.reading_time, 2);
        assert_eq!(meta.language, "en");
        assert_eq!(meta.content_type, ContentType::Article);
        assert_eq!(meta.source, "https://example.com/article");
        assert!(!meta.scrape_date.is_empty());
    }
}
```

### 2.3 `uri.rs` — Obsidian URI Protocol

```rust
//! Obsidian URI protocol support.
//!
//! Opens notes directly in Obsidian using the `obsidian://` URI scheme.
//! Supports both vault-specific and vault-agnostic opening.
//!
//! URI format: `obsidian://open?vault=<vault_name>&file=<file_path>`
//!
//! Platform support:
//! - Linux: `xdg-open`
//! - macOS: `open`
//! - Windows: `start`

use std::path::Path;
use std::process::Command;

/// Open a note in Obsidian using the URI protocol.
///
/// # Arguments
/// - `vault_name` — Name of the Obsidian vault (folder name, not full path)
/// - `file_path` — Path to the note relative to the vault root (without extension)
///
/// # Returns
/// - `Ok(())` — URI was opened successfully
/// - `Err(String)` — Platform command failed or vault not found
///
/// # Example
/// ```ignore
/// open_in_obsidian("MyVault", "Inbox/example-com-article")?;
/// // Opens: obsidian://open?vault=MyVault&file=Inbox/example-com-article
/// ```
pub fn open_in_obsidian(vault_name: &str, file_path: &Path) -> Result<(), String> {
    let file_str = file_path
        .to_string_lossy()
        .replace('\\', "/")
        .trim_end_matches(".md")
        .to_string();

    let uri = format!(
        "obsidian://open?vault={}&file={}",
        urlencoding::encode(vault_name),
        urlencoding::encode(&file_str),
    );

    open_uri(&uri)
}

/// Open a URI using the platform-specific handler.
fn open_uri(uri: &str) -> Result<(), String> {
    let (cmd, args) = if cfg!(target_os = "windows") {
        ("cmd", vec!["/C", "start", uri])
    } else if cfg!(target_os = "macos") {
        ("open", vec![uri])
    } else {
        ("xdg-open", vec![uri])
    };

    Command::new(cmd)
        .args(&args)
        .output()
        .map_err(|e| format!("failed to open URI: {}", e))
        .map(|_| ())
}

/// Extract vault name from a vault path (last directory component).
///
/// # Example
/// `/home/user/Obsidian/MyVault` → `MyVault`
#[must_use]
pub fn extract_vault_name(vault_path: &Path) -> String {
    vault_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_vault_name() {
        assert_eq!(
            extract_vault_name(Path::new("/home/user/Obsidian/MyVault")),
            "MyVault"
        );
    }

    #[test]
    fn test_extract_vault_name_single() {
        assert_eq!(extract_vault_name(Path::new("MyVault")), "MyVault");
    }

    #[test]
    fn test_extract_vault_name_empty() {
        assert_eq!(extract_vault_name(Path::new("")), "Unknown");
    }
}
```

---

## 3. Data Structures

### 3.1 New: `ObsidianRichMetadata` (see §2.2)

### 3.2 New: `VaultDetection` (see §2.1)

### 3.3 New: `ContentType` enum (see §2.2)

### 3.4 Modified: `Frontmatter` in `frontmatter.rs`

The existing `Frontmatter` struct gains optional rich metadata fields:

```rust
// src/infrastructure/output/frontmatter.rs — modified

#[derive(Debug, Serialize)]
struct Frontmatter {
    // Existing fields (unchanged)
    title: String,
    url: String,
    date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    excerpt: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,

    // NEW: Rich metadata fields
    #[serde(skip_serializing_if = "Option::is_none")]
    word_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reading_time: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scrape_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
}
```

### 3.5 Modified: `generate()` function signature in `frontmatter.rs`

```rust
// BEFORE:
pub fn generate(
    title: &str,
    url: &str,
    date: Option<&str>,
    author: Option<&str>,
    excerpt: Option<&str>,
    tags: &[String],
) -> String

// AFTER:
pub fn generate(
    title: &str,
    url: &str,
    date: Option<&str>,
    author: Option<&str>,
    excerpt: Option<&str>,
    tags: &[String],
    rich_meta: Option<&ObsidianRichMetadata>,  // NEW
) -> String
```

### 3.6 Modified: `Args` in `lib.rs`

New CLI flags added to the `Args` struct:

```rust
// src/lib.rs — new fields in Args struct

// ========== Obsidian Vault ==========
/// Path to Obsidian vault (auto-detects if not provided)
///
/// The vault must contain a `.obsidian/` directory.
/// If not provided, searches: OBSIDIAN_VAULT env var,
/// config file, then common locations (~/Obsidian, etc.)
#[arg(long, env = "RUST_SCRAPER_OBSIDIAN_VAULT")]
#[clap(next_help_heading = "Output")]
pub obsidian_vault: Option<std::path::PathBuf>,

/// Quick-save mode: save directly to vault Inbox folder
///
/// Bypasses TUI selection and saves all scraped content
/// to `<vault>/Inbox/`. Ideal for one-command clipping.
///
/// Requires --obsidian-wiki-links to be enabled.
#[arg(long, default_value = "false", env = "RUST_SCRAPER_OBSIDIAN_QUICK_SAVE")]
#[clap(next_help_heading = "Output")]
pub obsidian_quick_save: bool,

/// Open saved notes in Obsidian after scraping (Linux/macOS/Windows)
///
/// Uses the obsidian:// URI protocol to open each saved note
/// directly in the Obsidian app.
#[arg(long, default_value = "false", env = "RUST_SCRAPER_OBSIDIAN_OPEN")]
#[clap(next_help_heading = "Output")]
pub obsidian_open: bool,

/// Enable rich metadata in frontmatter (word count, reading time, language)
///
/// Adds wordCount, readingTime, language, contentType, scrapeDate,
/// and source fields to YAML frontmatter for Dataview compatibility.
#[arg(long, default_value = "false", env = "RUST_SCRAPER_OBSIDIAN_RICH_META")]
#[clap(next_help_heading = "Output")]
pub obsidian_rich_metadata: bool,
```

### 3.7 Modified: `ConfigDefaults` in `config.rs`

```rust
// src/cli/config.rs — new field

/// Default Obsidian vault path
pub vault_path: Option<String>,
```

### 3.8 Modified: `ObsidianOptions` in `file_saver.rs`

```rust
// src/infrastructure/output/file_saver.rs — modified

#[derive(Debug, Clone, Default)]
pub struct ObsidianOptions {
    pub wiki_links: bool,
    pub relative_assets: bool,
    pub tags: Vec<String>,
    // NEW:
    pub rich_metadata: bool,
    pub quick_save: bool,
    pub open_after_save: bool,
    pub vault_path: Option<PathBuf>,
}
```

---

## 4. Algorithm Design

### 4.1 Vault Detection Algorithm

```
detect_vault(cli_path, config_path):
    1. If cli_path exists AND contains .obsidian/ → return Explicit(cli_path)
    2. If OBSIDIAN_VAULT env exists AND contains .obsidian/ → return Explicit(env_path)
    3. If config_path exists AND contains .obsidian/ → return Explicit(config_path)
    4. Auto-scan common locations:
       a. ~/Obsidian/
       b. ~/Documents/Obsidian/
       c. ~/.obsidian/
       d. Current working directory
       First valid → return AutoDetected(path)
    5. Return NotFound
```

**Performance:** Steps 1-3 are O(1) filesystem checks. Step 4 scans at most 4 directories — each check is a single `stat()` call for `.obsidian/`. Total cost: ~4 stat calls, <1ms.

### 4.2 Metadata Generation Algorithm

```
generate_rich_metadata(content, url):
    1. word_count = content.split_whitespace().count()     // O(n) single pass
    2. reading_time = ceil(word_count / 200).max(1)        // O(1)
    3. language = whatlang::detect(content[..1024])        // O(min(n, 1024))
       a. If confidence < 0.3 → "unknown"
       b. Else → ISO 639-1 code
    4. content_type = infer from URL patterns + word count  // O(1)
       a. URL heuristics: /doc, /api, /forum, /product
       b. If words > 500 → Article
       c. Else → Other
    5. scrape_date = Utc::now().to_iso8601()              // O(1)
    6. source = url                                        // O(1)
```

**Performance:** Dominated by word count (single pass over content) and language detection (capped at 1024 bytes). For a 10,000-word article: ~0.5ms word count + ~0.3ms language detection = **<1ms total**.

### 4.3 Quick-Save Flow

```
main() pipeline with --quick-save:
    1. detect_vault() → if NotFound, warn and fall back to --output
    2. If vault found:
       a. resolve_vault_output(vault, quick_save=true, domain)
          → <vault>/Inbox/
       b. Skip TUI selector entirely (bypass step 13)
       c. Scrape all discovered URLs
       d. Save to Inbox with rich metadata
       e. If --obsidian-open: open each note via URI
    3. If vault not found:
       a. Warn user
       b. Fall back to --output directory
       c. Continue normal flow
```

### 4.4 URI Construction

```
open_in_obsidian(vault_name, file_path):
    1. file_str = file_path.to_string()
       .replace('\\', '/')
       .trim_end_matches(".md")
    2. uri = "obsidian://open?vault=" + url_encode(vault_name)
           + "&file=" + url_encode(file_str)
    3. Platform command:
       Linux:   xdg-open <uri>
       macOS:   open <uri>
       Windows: cmd /C start <uri>
```

---

## 5. Integration Points

### 5.1 CLI Flag Flow

```
Args (lib.rs)
    ↓
apply_config_defaults() (main.rs)
    ↓ merge vault_path from config
    ↓
detect_vault(args.obsidian_vault, config.vault_path)
    ↓
resolve_vault_output(vault, quick_save, domain)
    ↓
ObsidianOptions { rich_metadata, quick_save, open_after_save, vault_path }
    ↓
save_results(results, output_dir, format, obsidian_options)
    ↓
if obsidian_open: open_in_obsidian(vault_name, saved_path)
```

### 5.2 Config Merge in `apply_config_defaults()`

```rust
// src/main.rs — new section in apply_config_defaults()

// Vault path from config
if let Some(ref v) = config.vault_path {
    if args.obsidian_vault.is_none() {
        args.obsidian_vault = Some(PathBuf::from(v));
    }
}

// Rich metadata from config (future: could be config flag)
// Quick-save: no config override — always explicit
// Open after save: no config override — always explicit
```

### 5.3 Quick-Save Bypasses TUI

In `main.rs`, the quick-save flag creates a new branch **before** the TUI selector (step 13):

```rust
// After step 12 (dry-run check), before step 13:

// =========================================================================
// 12b. Quick-save mode: bypass TUI, save to vault Inbox
// =========================================================================
let urls_to_scrape = if args.obsidian_quick_save {
    info!("Quick-save mode: bypassing TUI, saving to vault Inbox");
    discovered_urls  // Use all discovered URLs
} else if args.interactive {
    // ... existing TUI logic ...
};
```

### 5.4 Frontmatter Integration

In `save_as_markdown()` in `file_saver.rs`, the frontmatter generation call is extended:

```rust
// BEFORE:
let fm = frontmatter::generate(
    &item.title,
    item.url.as_str(),
    item.date.as_deref(),
    item.author.as_deref(),
    item.excerpt.as_deref(),
    &obsidian.tags,
);

// AFTER:
let rich_meta = if obsidian.rich_metadata {
    Some(ObsidianRichMetadata::from_content(
        &processed,
        item.url.as_str(),
    ))
} else {
    None
};

let fm = frontmatter::generate(
    &item.title,
    item.url.as_str(),
    item.date.as_deref(),
    item.author.as_deref(),
    item.excerpt.as_deref(),
    &obsidian.tags,
    rich_meta.as_ref(),
);
```

### 5.5 Post-Save URI Opening

After `save_results()` completes, if `--obsidian-open` is set:

```rust
// After save_results(), before summary:

if args.obsidian_open {
    if let Some(ref vault_path) = vault_detection.path() {
        let vault_name = obsidian::uri::extract_vault_name(vault_path);
        for item in &results {
            let file_path = /* compute saved file path relative to vault */;
            match obsidian::uri::open_in_obsidian(&vault_name, &file_path) {
                Ok(()) => info!("Opened in Obsidian: {}", item.title),
                Err(e) => warn!("Failed to open in Obsidian: {}", e),
            }
        }
    } else {
        warn!("Cannot open in Obsidian: no vault detected");
    }
}
```

---

## 6. Testing Strategy

### 6.1 Unit Tests

| Module | Test | What It Verifies |
|--------|------|-----------------|
| `vault_detector` | `test_is_valid_vault_true` | `.obsidian/` directory detection |
| `vault_detector` | `test_is_valid_vault_false` | Non-vault directory rejection |
| `vault_detector` | `test_detect_vault_explicit_path` | CLI flag priority |
| `vault_detector` | `test_detect_vault_not_found` | Fallback behavior |
| `vault_detector` | `test_detect_vault_env_var` | Environment variable priority |
| `vault_detector` | `test_resolve_vault_output_quick_save` | Inbox path resolution |
| `vault_detector` | `test_resolve_vault_output_domain` | Domain path resolution |
| `metadata` | `test_count_words_*` | Word counting edge cases |
| `metadata` | `test_estimate_reading_time_*` | Rounding, zero, minimum |
| `metadata` | `test_detect_language_english` | English detection |
| `metadata` | `test_detect_language_spanish` | Spanish detection |
| `metadata` | `test_detect_language_too_short` | Fallback for short text |
| `metadata` | `test_infer_content_type_*` | URL and content heuristics |
| `metadata` | `test_rich_metadata_generation` | End-to-end metadata |
| `uri` | `test_extract_vault_name` | Vault name extraction |
| `frontmatter` | `test_generate_with_rich_metadata` | Rich fields in YAML output |
| `frontmatter` | `test_generate_without_rich_metadata` | Backward compatibility |

### 6.2 Integration Tests

| Test | Setup | Assertion |
|------|-------|-----------|
| `test_quick_save_to_vault` | Create temp vault with `.obsidian/`, run with `--quick-save` | Files saved to `<vault>/Inbox/` |
| `test_quick_save_without_vault` | No vault, run with `--quick-save` | Falls back to `--output` dir, warning logged |
| `test_rich_metadata_in_frontmatter` | Scrape known content, enable `--obsidian-rich-metadata` | Frontmatter contains `wordCount`, `readingTime`, `language` |
| `test_vault_detection_priority` | Set env var + CLI flag | CLI flag wins |
| `test_backward_compatibility` | Run without any new flags | Existing behavior unchanged |

### 6.3 Edge Cases

| Edge Case | Handling |
|-----------|----------|
| Vault path doesn't exist | Warn, fall back to `--output` |
| Vault path exists but no `.obsidian/` | Warn, fall back to `--output` |
| Content is empty | `wordCount: 0`, `readingTime: 1`, `language: "unknown"` |
| Content < 1024 bytes | Language detection on full content |
| URL has no recognizable pattern | `contentType: "other"` |
| `--quick-save` without `--obsidian-wiki-links` | Warn, continue (wiki-links is independent) |
| `--obsidian-open` on headless server | Command fails gracefully, warning logged |
| Vault name with spaces | URL-encoded in URI |
| Cross-platform path separators | Normalized to `/` in URI |

---

## 7. Performance Considerations

### 7.1 Language Detection Overhead

| Content Size | whatlang Time | Impact |
|-------------|--------------|--------|
| < 100 bytes | ~0.01ms | Negligible |
| 1,024 bytes (cap) | ~0.3ms | Negligible |
| 10,000 words | ~0.3ms (capped) | No impact |
| 100,000 words | ~0.3ms (capped) | No impact |

**Key insight:** The 1024-byte cap ensures language detection is O(1) regardless of content size. On our HDD-optimized Haswell i5, this adds <0.5ms per page — completely negligible compared to network I/O (100ms+ per request).

### 7.2 Vault Scan Performance

| Scenario | Stat Calls | Time |
|----------|-----------|------|
| CLI flag provided | 1 | <0.1ms |
| Env var provided | 1 | <0.1ms |
| Config provided | 1 | <0.1ms |
| Auto-scan (all 4 locations) | 4 | <0.5ms |

**Key insight:** All filesystem checks are `stat()` calls on known paths — no recursive directory walking. Total cost is bounded at 4 stat calls.

### 7.3 Word Count Performance

Word counting uses `split_whitespace()` which is a single-pass iterator over the content string. For a 50,000-character article: ~0.2ms on Haswell. This is already done during readability extraction, so the incremental cost is minimal.

### 7.4 Overall Impact

| Operation | Added Time | Frequency |
|-----------|-----------|-----------|
| Vault detection | <0.5ms | Once per run |
| Language detection | <0.5ms | Per scraped page |
| Word count | <0.3ms | Per scraped page |
| Content type inference | <0.1ms | Per scraped page |
| URI opening | ~50ms | Per scraped page (optional) |

**Total per-page overhead:** <1.5ms (without URI opening), ~52ms (with URI opening). Compared to ~2000ms for a typical HTTP request + readability extraction, this is **<0.1% overhead**.

---

## 8. Dependencies

### 8.1 `whatlang` Crate Analysis

| Property | Value |
|----------|-------|
| **Crate** | `whatlang` |
| **Version** | `0.16` (as specified in issue) |
| **Latest** | `0.18.0` (Oct 2025) |
| **License** | MIT |
| **Size** | 81.2 KB |
| **Downloads** | 1.6M total, 344K in 90 days |
| **MSRV** | 1.70+ (compatible with our 1.88) |
| **Dependencies** | Zero external deps — pure Rust |

**Why 0.16 vs 0.18:** The issue specifies `0.16`. Version 0.18 uses Rust 2024 edition. Since our MSRV is 1.88 and we use edition 2021, `0.16` is the safest choice. However, `0.18` should also compile fine on 1.88. **Recommendation:** Use `0.16` as specified; upgrade to `0.18` in a future maintenance pass if needed.

**Alternatives considered:**

| Crate | Pros | Cons | Verdict |
|-------|------|------|---------|
| `whatlang` | Zero deps, fast, well-maintained | Less accurate than lingua | ✅ Selected |
| `lingua` | Most accurate (90%+), 75 languages | Heavy (15MB models), slower | ❌ Overkill |
| `langram` | New, accurate | Small ecosystem, 2024 edition | ❌ Too new |

**Cargo.toml addition:**
```toml
# Language detection for Obsidian rich metadata
whatlang = "0.16"
```

### 8.2 `urlencoding` Crate (for URI)

The `uri.rs` module needs URL encoding for vault names and file paths. The `url` crate's `form_urlencoded` module can handle this without an additional dependency:

```rust
use url::form_urlencoded::byte_serialize;

fn encode(s: &str) -> String {
    byte_serialize(s.as_bytes()).collect()
}
```

**No new dependency needed** — `url` is already in Cargo.toml.

### 8.3 Dependency Summary

| Dependency | Reason | Feature Gate |
|-----------|--------|-------------|
| `whatlang = "0.16"` | Language detection | None (always available) |
| `url` (existing) | URI encoding | None |
| `chrono` (existing) | Timestamps | None |
| `serde` (existing) | Serialization | None |
| `dirs` (existing) | Home directory | None |

**No new transitive dependencies beyond `whatlang`.** The crate has zero external dependencies, so the total dependency tree impact is minimal.

---

## 9. File Change Summary

| File | Action | Changes |
|------|--------|---------|
| `Cargo.toml` | Modify | Add `whatlang = "0.16"` |
| `src/lib.rs` | Modify | Add 4 new CLI flags to `Args` |
| `src/cli/config.rs` | Modify | Add `vault_path` to `ConfigDefaults` |
| `src/main.rs` | Modify | Vault detection, quick-save branch, URI opening, config merge |
| `src/infrastructure/mod.rs` | Modify | Add `pub mod obsidian;` |
| `src/infrastructure/obsidian/mod.rs` | **New** | Module declaration |
| `src/infrastructure/obsidian/vault_detector.rs` | **New** | Vault detection logic |
| `src/infrastructure/obsidian/metadata.rs` | **New** | Rich metadata generation |
| `src/infrastructure/obsidian/uri.rs` | **New** | Obsidian URI protocol |
| `src/infrastructure/output/frontmatter.rs` | Modify | Add rich metadata fields to `Frontmatter` and `generate()` |
| `src/infrastructure/output/file_saver.rs` | Modify | Add fields to `ObsidianOptions`, pass rich metadata |

**Total:** 3 new files, 6 modified files, 1 new dependency.

---

## 10. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| `whatlang` fails to detect language | Low | Low | Falls back to "unknown" — graceful degradation |
| Vault auto-detect finds wrong vault | Low | Medium | Warns user, explicit CLI flag always wins |
| `xdg-open` not available on Linux | Low | Low | Graceful error, warning logged |
| Frontmatter backward compatibility | Low | High | All new fields use `skip_serializing_if` |
| Quick-save with no vault | Medium | Low | Falls back to `--output` with warning |
| URI opening on headless server | Medium | Low | Command fails gracefully, warning logged |

---

## 11. Future Considerations (Out of Scope for #23)

The following items are identified in the user research but are **explicitly out of scope** for this issue:

- **Duplicate URL detection** — Requires state store integration (P2-2)
- **Image download to vault assets** — Requires asset pipeline changes (P2-5)
- **Template system** — Requires configurable output templates (P2-11)
- **Auto-generated MOCs** — Requires content relationship analysis (P2-17)
- **Git-aware vault support** — Requires git2 integration (P2-18)
- **Semantic duplicate detection** — Requires AI embeddings (P2-13)

These should be tracked as separate issues in the P2 roadmap.
