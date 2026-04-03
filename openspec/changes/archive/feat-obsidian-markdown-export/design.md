# Technical Design: Obsidian-compatible Markdown Export

**Change:** `feat(v1.5): Obsidian-compatible Markdown export #22`  
**Author:** SDD Design Agent  
**Date:** 2026-04-03  
**Status:** Draft

---

## 1. Architecture Overview

### 1.1 Placement in Clean Architecture

The new feature lives entirely in the **Infrastructure layer**, respecting the dependency rule:

```
Domain (ScrapedContent, DownloadedAsset)    ← no changes to domain entities
    ↑
Application (scrape pipeline)               ← no changes to application logic
    ↑
Infrastructure ─────────────────────────────┐
  converter/                                │
    html_to_markdown.rs                     │
    syntax_highlight.rs                     │
    obsidian.rs              ← NEW MODULE   │  Pure string transformations
  output/                                   │
    frontmatter.rs           ← MODIFIED     │  Adds tags field
    file_saver.rs            ← MODIFIED     │  Wires obsidian conversions
    ↑
Adapters (CLI, TUI)
  lib.rs (Args)              ← MODIFIED     │  Adds --obsidian-* flags
  cli/config.rs              ← MODIFIED     │  Adds [obsidian] TOML section
```

**Key principle:** All changes are additive. No existing function signatures change — only new parameters are threaded through.

### 1.2 Data Flow Diagram

```
CLI Args (lib.rs)
  ├── --format markdown
  ├── --obsidian-wiki-links          ─┐
  ├── --obsidian-tags "tag1,tag2"     │  ObsidianOptions
  └── --obsidian-relative-assets     ─┘
                  │
                  ▼
        ConfigDefaults (config.rs)
        merges CLI + TOML config
                  │
                  ▼
        save_results() in file_saver.rs
          │
          ├─── OutputFormat::Markdown branch
          │      │
          │      ▼
          │    save_as_markdown(results, output_dir, obsidian_opts)
          │      │
          │      ├── For each ScrapedContent:
          │      │   │
          │      │   ├── 1. HTML → Markdown (existing)
          │      │   ├── 2. Syntax highlight (existing)
          │      │   ├── 3. ┌──────────────────────────────┐
          │      │   │      │ IF obsidian_wiki_links:       │
          │      │   │      │   convert_wiki_links(md, url) │  ← obsidian.rs
          │      │   │      └──────────────────────────────┘
          │      │   ├── 4. ┌──────────────────────────────┐
          │      │   │      │ IF obsidian_relative_assets:  │
          │      │   │      │   resolve_asset_paths(...)    │  ← obsidian.rs
          │      │   │      └──────────────────────────────┘
          │      │   └── 5. Generate frontmatter (with tags if set)
          │      │         └── write file
          │      │
          │      └── Output unchanged if no flags set
          │
          └─── Other formats (Text, JSON) — no changes
```

---

## 2. Module Design

### 2.1 New Module: `src/infrastructure/converter/obsidian.rs`

This module contains pure string-transformation functions. No IO, no async, no side effects.

#### Module Declaration

```rust
// src/infrastructure/converter/mod.rs — ADD this line:
pub mod obsidian;
```

#### Function Signatures

```rust
/// Convert Markdown links to Obsidian wiki-links for same-domain URLs.
///
/// Transforms `[link text](https://same-domain.com/page)` → `[[page-slug|link text]]`
/// External links (different domain) are left unchanged.
///
/// # Arguments
/// * `content` — Markdown content to process
/// * `base_domain` — The domain of the scraped page (e.g. "example.com")
///
/// # Returns
/// Markdown with same-domain links converted to wiki-link syntax
pub fn convert_wiki_links(content: &str, base_domain: &str) -> String;

/// Rewrite Markdown image/document references to use relative paths.
///
/// Transforms `![](absolute/local/path)` → `![](../../relative/path)`
/// based on the `.md` file's location and the asset's `local_path`.
///
/// # Arguments
/// * `content` — Markdown content with `![]()` references
/// * `md_file_dir` — Directory containing the output `.md` file
/// * `assets` — DownloadedAsset list with `local_path` and original `url`
///
/// # Returns
/// Markdown with asset paths rewritten as relative
pub fn resolve_asset_paths(
    content: &str,
    md_file_dir: &Path,
    assets: &[DownloadedAsset],
) -> String;

/// Extract a URL-safe slug from a URL path.
///
/// Strips query strings, fragments, and file extensions.
/// Converts path segments to kebab-case slug.
///
/// # Examples
/// * `/blog/my-post` → `my-post`
/// * `/docs/api/v2.html?page=1` → `api-v2`
/// * `/` → `index`
pub fn slug_from_url(url_path: &str) -> String;
```

#### Regex Design

```rust
use regex::Regex;
use std::sync::LazyLock;

/// Matches Markdown links: [text](url)
/// Uses alternation to skip code blocks and inline code:
///   (```[\s\S]*?```|`[^`]+`) — skip fenced/inline code
///   | \[([^\]]+)\]\(([^)]+)\) — capture link text and URL
///
/// The alternation ensures we don't convert links inside code blocks.
static MARKDOWN_LINK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)(```[\s\S]*?```|`[^`]+`)|\[([^\]]+)\]\(([^)]+)\)")
        .expect("BUG: invalid regex for markdown links")
});

/// Matches Markdown image references: ![alt](path)
/// Same code-block-skipping strategy.
static MARKDOWN_IMAGE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)(```[\s\S]*?```|`[^`]+`)|!\[([^\]]*)\]\(([^)]+)\)")
        .expect("BUG: invalid regex for markdown images")
});
```

**Why this regex pattern:** The alternation `(code_pattern) | (link_pattern)` is a well-established technique. When the regex engine matches the code-block branch, the capture groups for the link branch are empty, so the replacement function can check `caps.get(2).is_some()` to decide whether to transform. This avoids the complexity of negative lookbehind (not supported in Rust regex) and is O(n) single-pass.

### 2.2 Modified: `src/infrastructure/output/frontmatter.rs`

#### Changes to `Frontmatter` struct

```rust
#[derive(Debug, Serialize)]
struct Frontmatter {
    title: String,
    url: String,
    date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    excerpt: Option<String>,
    // NEW: Obsidian tags
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
}
```

#### Changes to `generate()` function

```rust
// BEFORE:
pub fn generate(
    title: &str,
    url: &str,
    date: Option<&str>,
    author: Option<&str>,
    excerpt: Option<&str>,
) -> String

// AFTER (add parameter, backward compatible via default):
pub fn generate(
    title: &str,
    url: &str,
    date: Option<&str>,
    author: Option<&str>,
    excerpt: Option<&str>,
    tags: &[String],  // NEW — pass empty slice for no tags
) -> String
```

**Note:** This is a breaking change to the function signature, but the only call site is `file_saver.rs` which we control. No public API exposure.

### 2.3 Modified: `src/infrastructure/output/file_saver.rs`

#### New struct: `ObsidianOptions`

```rust
/// Configuration for Obsidian-compatible output.
#[derive(Debug, Clone, Default)]
pub struct ObsidianOptions {
    /// Convert same-domain links to [[wiki-link]] syntax
    pub wiki_links: bool,
    /// Rewrite asset paths as relative to the .md file
    pub relative_assets: bool,
    /// Tags to include in YAML frontmatter
    pub tags: Vec<String>,
}
```

#### Modified `save_as_markdown()` signature

```rust
// BEFORE:
fn save_as_markdown(results: &[ScrapedContent], output_dir: &Path) -> Result<()>

// AFTER:
fn save_as_markdown(
    results: &[ScrapedContent],
    output_dir: &Path,
    obsidian: &ObsidianOptions,
) -> Result<()>
```

#### Modified `save_results()` signature

```rust
// BEFORE:
pub fn save_results(
    results: &[ScrapedContent],
    output_dir: &Path,
    format: &OutputFormat,
) -> Result<()>

// AFTER:
pub fn save_results(
    results: &[ScrapedContent],
    output_dir: &Path,
    format: &OutputFormat,
    obsidian: &ObsidianOptions,  // NEW — defaults to ObsidianOptions::default()
) -> Result<()>
```

#### Integration flow in `save_as_markdown()`

```rust
fn save_as_markdown(
    results: &[ScrapedContent],
    output_dir: &Path,
    obsidian: &ObsidianOptions,
) -> Result<()> {
    for item in results {
        // ... existing path resolution ...

        // 1. HTML → Markdown (existing)
        let markdown_content = item.html
            .as_ref()
            .map(|html| html_to_markdown::convert_to_markdown(html))
            .unwrap_or_else(|| item.content.clone());

        // 2. Syntax highlight (existing)
        let mut highlighted = syntax_highlight::highlight_code_blocks(&markdown_content);

        // 3. NEW: Wiki-link conversion (conditional)
        if obsidian.wiki_links {
            let base_domain = item.url.domain_str(); // from ValidUrl
            highlighted = obsidian::convert_wiki_links(&highlighted, base_domain);
        }

        // 4. NEW: Relative asset paths (conditional)
        if obsidian.relative_assets && !item.assets.is_empty() {
            highlighted = obsidian::resolve_asset_paths(
                &highlighted,
                full_path.parent().unwrap(),
                &item.assets,
            );
        }

        // 5. Frontmatter (modified — includes tags)
        let fm = frontmatter::generate(
            &item.title,
            item.url.as_str(),
            item.date.as_deref(),
            item.author.as_deref(),
            item.excerpt.as_deref(),
            &obsidian.tags,  // NEW
        );

        let final_content = format!("---\n{}---\n\n{}", fm.trim(), highlighted);
        fs::write(&full_path, final_content)?;
    }
    Ok(())
}
```

---

## 3. Data Structures

### 3.1 `ObsidianOptions` (new, in `file_saver.rs`)

```rust
#[derive(Debug, Clone, Default)]
pub struct ObsidianOptions {
    pub wiki_links: bool,
    pub relative_assets: bool,
    pub tags: Vec<String>,
}
```

**Design rationale:** A dedicated struct rather than three separate parameters. This keeps `save_results()` and `save_as_markdown()` signatures clean, makes it easy to add future Obsidian options (e.g., `aliases`, `cssclasses`), and maps cleanly to the `[obsidian]` TOML config section.

### 3.2 `Frontmatter` (modified, in `frontmatter.rs`)

```rust
#[derive(Debug, Serialize)]
struct Frontmatter {
    title: String,
    url: String,
    date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    excerpt: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,  // NEW
}
```

**Obsidian-compatible frontmatter output example:**

```yaml
---
title: "My Article"
url: "https://example.com/blog/my-post"
date: "2026-04-03"
tags:
  - scraped
  - rust
  - web-dev
---
```

### 3.3 Error Types

No new error types needed. All functions in `obsidian.rs` are pure string transformations that cannot fail. If regex compilation fails (impossible with static patterns), the `expect()` in `LazyLock` will panic at startup — which is the correct behavior for a bug in our regex patterns.

---

## 4. Algorithm Design

### 4.1 Wiki-link Conversion Algorithm

```
Input:  markdown_content, base_domain (e.g. "example.com")
Output: markdown_content with same-domain links as [[wiki-links]]

For each match of MARKDOWN_LINK_RE in content:
  1. If code_block_group matched → return original text unchanged
     (We're inside a fenced code block or inline code — don't touch it)

  2. Extract link_text = caps[2], url_str = caps[3]

  3. Parse url_str with url::Url::parse()
     If parse fails → return original text (e.g. relative links, anchors)

  4. Extract link_host from parsed URL
     If link_host != base_domain → return original text (external link)

  5. Extract path from parsed URL
     slug = slug_from_url(path)

  6. Return [[slug|link_text]]

The Regex::replace_all() closure handles this in a single O(n) pass.
```

**Edge cases handled:**
- External links (`https://other.com/page`) → left as `[text](url)`
- Anchor links (`#section`) → left as `[text](url)` (parse fails or no host)
- Relative links (`/page`) → left as `[text](url)` (no host to compare)
- Links in code blocks → skipped by alternation
- Identical links appearing multiple times → all converted (replace_all, not replace)
- URL-encoded paths → `url::Url::parse()` handles normalization

### 4.2 Slug Extraction Algorithm

```
Input:  URL path (e.g. "/blog/my-post/index.html?foo=bar#section")
Output: Slug string (e.g. "my-post")

1. Strip query string: everything after '?'
2. Strip fragment: everything after '#'
3. Strip trailing slash
4. Strip file extension: remove `.html`, `.htm`, `.php`, etc.
5. Take last path segment (after final '/')
6. If empty (root path `/`) → return "index"
7. Convert to lowercase
8. Replace non-alphanumeric characters (except hyphens) with hyphens
9. Collapse multiple hyphens → single hyphen
10. Trim leading/trailing hyphens
```

**Examples:**
| URL Path | Slug |
|----------|------|
| `/blog/my-post` | `my-post` |
| `/blog/my-post/` | `my-post` |
| `/docs/api/v2.html` | `api-v2` |
| `/` | `index` |
| `/2026/04/03/hello-world/` | `hello-world` |
| `/page?id=123#section` | `page` |
| `/My%20Post%20Title` | `my-post-title` |

### 4.3 Asset Path Resolution Algorithm

```
Input:  markdown_content, md_file_dir, assets: &[DownloadedAsset]
Output: markdown_content with ![](paths) rewritten as relative

For each asset in assets:
  1. original_url = asset.url (the URL that appeared in the HTML)
  2. local_path = asset.local_path (absolute path where file was saved)

  3. Compute relative_path = pathdiff::diff_paths(
       local_path,
       md_file_dir
     )

  4. For each occurrence of original_url in markdown image refs:
     Replace with relative_path (normalized to forward slashes)

The Regex::replace_all() closure handles this in a single pass,
checking each match against the asset URL map.
```

**Why fuzzy matching:** The `html-to-markdown-rs` crate may URL-encode characters in image URLs (e.g., spaces become `%20`). We build a lookup map from `asset.url` → `relative_path`, and for each image reference in the Markdown, we try exact match first, then try URL-decoded variants.

**Algorithm refinement:**

```rust
pub fn resolve_asset_paths(
    content: &str,
    md_file_dir: &Path,
    assets: &[DownloadedAsset],
) -> String {
    // Build a map: original_url → relative_path
    let asset_map: HashMap<&str, String> = assets
        .iter()
        .filter_map(|a| {
            let rel = pathdiff::diff_paths(&a.local_path, md_file_dir)?;
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            Some((a.url.as_str(), rel_str))
        })
        .collect();

    MARKDOWN_IMAGE_RE.replace_all(content, |caps: &regex::Captures| {
        // If code block matched, return unchanged
        if caps.get(1).is_some() {
            return caps[0].to_string();
        }

        let alt = &caps[2];
        let path = &caps[3];

        // Try exact match, then URL-decoded match
        let replacement = asset_map.get(path)
            .or_else(|| {
                // Fuzzy: try URL-decoding the path
                let decoded = urlencoding::decode(path).ok()?;
                asset_map.get(decoded.as_ref())
            });

        match replacement {
            Some(rel) => format!("![{alt}]({rel})"),
            None => caps[0].to_string(), // Not an asset we downloaded
        }
    })
    .to_string()
}
```

**Note:** The `urlencoding` crate is already a transitive dependency via `url`. If not directly available, we can use `url::percent_encoding::percent_decode_str()`.

---

## 5. Integration Points

### 5.1 CLI Flag Flow

```
main.rs
  ├── parse Args (clap)
  │     ├── --obsidian-wiki-links: bool
  │     ├── --obsidian-tags: Option<String>   → split by ',' → Vec<String>
  │     └── --obsidian-relative-assets: bool
  │
  ├── load ConfigDefaults from TOML
  │     └── [obsidian] section:
  │           wiki_links = true
  │           tags = ["scraped", "auto-tagged"]
  │           relative_assets = true
  │
  ├── Merge: CLI flags override TOML config
  │     obsidian_opts = ObsidianOptions {
  │         wiki_links: args.obsidian_wiki_links
  │             || config.obsidian_wiki_links.unwrap_or(false),
  │         relative_assets: args.obsidian_relative_assets
  │             || config.obsidian_relative_assets.unwrap_or(false),
  │         tags: parse_tags(args.obsidian_tags.as_deref())
  │             .or(config.obsidian_tags.clone())
  │             .unwrap_or_default(),
  │     }
  │
  └── Pass obsidian_opts to save_results()
```

### 5.2 CLI Flags in `Args` (src/lib.rs)

```rust
// In lib.rs, add to Args struct (after existing output flags):

/// Convert same-domain links to Obsidian [[wiki-link]] syntax
///
/// Only links pointing to the same domain as the scraped page
/// are converted. External links remain as standard Markdown links.
///
/// Example: [Read more](https://example.com/about) → [[about|Read more]]
#[arg(long, default_value = "false", env = "RUST_SCRAPER_OBSIDIAN_WIKI_LINKS")]
#[clap(next_help_heading = "Output")]
pub obsidian_wiki_links: bool,

/// Tags to include in YAML frontmatter (comma-separated)
///
/// Tags are added to the frontmatter of each Markdown file,
/// making them discoverable in Obsidian's tag pane.
///
/// Example: --obsidian-tags "scraped,rust,web-dev"
#[arg(long, env = "RUST_SCRAPER_OBSIDIAN_TAGS", value_delimiter = ',')]
#[clap(next_help_heading = "Output")]
pub obsidian_tags: Option<Vec<String>>,

/// Rewrite downloaded asset paths as relative to the .md file
///
/// When images or documents are downloaded during scraping,
/// this flag rewrites their Markdown references to use relative
/// paths so they display correctly in Obsidian.
///
/// Requires --download-images or --download-documents to have effect.
#[arg(long, default_value = "false", env = "RUST_SCRAPER_OBSIDIAN_RELATIVE_ASSETS")]
#[clap(next_help_heading = "Output")]
pub obsidian_relative_assets: bool,
```

### 5.3 Config File Changes (src/cli/config.rs)

```rust
#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(default)]
pub struct ConfigDefaults {
    // ... existing fields ...

    // NEW: Obsidian options
    pub obsidian_wiki_links: Option<bool>,
    pub obsidian_tags: Option<Vec<String>>,
    pub obsidian_relative_assets: Option<bool>,
}
```

**Example `config.toml`:**

```toml
format = "markdown"

[obsidian]
wiki_links = true
relative_assets = true
tags = ["scraped", "knowledge-base"]
```

### 5.4 Merge Logic

The merge happens in `main.rs` (or wherever `save_results()` is called):

```rust
let obsidian_opts = ObsidianOptions {
    wiki_links: args.obsidian_wiki_links
        || defaults.obsidian_wiki_links.unwrap_or(false),
    relative_assets: args.obsidian_relative_assets
        || defaults.obsidian_relative_assets.unwrap_or(false),
    tags: args.obsidian_tags
        .clone()
        .or(defaults.obsidian_tags.clone())
        .unwrap_or_default(),
};

save_results(&results, &args.output, &args.format, &obsidian_opts)?;
```

**Merge priority:** CLI flag > TOML config > default (disabled/empty).

---

## 6. Testing Strategy

### 6.1 Unit Tests for `obsidian.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // === convert_wiki_links tests ===

    #[test]
    fn test_convert_same_domain_link() {
        let md = "[Read more](https://example.com/about)";
        let result = convert_wiki_links(md, "example.com");
        assert_eq!(result, "[[about|Read more]]");
    }

    #[test]
    fn test_skip_external_domain_link() {
        let md = "[Google](https://google.com)";
        let result = convert_wiki_links(md, "example.com");
        assert_eq!(result, "[Google](https://google.com)"); // unchanged
    }

    #[test]
    fn test_skip_links_in_code_block() {
        let md = "```\n[not a link](https://example.com/foo)\n```";
        let result = convert_wiki_links(md, "example.com");
        assert!(result.contains("[not a link]")); // not converted
    }

    #[test]
    fn test_skip_inline_code_link() {
        let md = "Use `[link](https://example.com)` for docs";
        let result = convert_wiki_links(md, "example.com");
        assert!(result.contains("[link](https://example.com)"));
    }

    #[test]
    fn test_multiple_links_mixed() {
        let md = "[internal](https://example.com/a) and [external](https://other.com/b)";
        let result = convert_wiki_links(md, "example.com");
        assert!(result.contains("[[a|internal]"));
        assert!(result.contains("[external](https://other.com/b)"));
    }

    #[test]
    fn test_identical_links_all_converted() {
        // replace_all must handle duplicates
        let md = "[link](https://example.com/x) and [link](https://example.com/x)";
        let result = convert_wiki_links(md, "example.com");
        assert_eq!(result.matches("[[x|link]]").count(), 2);
    }

    // === slug_from_url tests ===

    #[test]
    fn test_slug_simple_path() {
        assert_eq!(slug_from_url("/blog/my-post"), "my-post");
    }

    #[test]
    fn test_slug_with_query_and_fragment() {
        assert_eq!(slug_from_url("/page?id=1#section"), "page");
    }

    #[test]
    fn test_slug_root_path() {
        assert_eq!(slug_from_url("/"), "index");
    }

    #[test]
    fn test_slug_with_extension() {
        assert_eq!(slug_from_url("/docs/api.html"), "api");
    }

    #[test]
    fn test_slug_url_encoded() {
        assert_eq!(slug_from_url("/My%20Post%20Title"), "my-post-title");
    }

    #[test]
    fn test_slug_nested_with_date() {
        assert_eq!(slug_from_url("/2026/04/03/hello-world/"), "hello-world");
    }

    // === resolve_asset_paths tests ===

    #[test]
    fn test_resolve_single_asset() {
        let md = "![image](https://example.com/img/photo.png)";
        let assets = vec![DownloadedAsset {
            url: "https://example.com/img/photo.png".to_string(),
            local_path: "/home/user/output/example.com/images/photo.png".to_string(),
            asset_type: "image".to_string(),
            size: 1024,
        }];
        let md_dir = Path::new("/home/user/output/example.com");
        let result = resolve_asset_paths(md, md_dir, &assets);
        assert!(result.contains("images/photo.png"));
    }

    #[test]
    fn test_resolve_no_assets() {
        let md = "No images here";
        let result = resolve_asset_paths(md, Path::new("/tmp"), &[]);
        assert_eq!(result, md);
    }

    #[test]
    fn test_resolve_asset_in_nested_dir() {
        // .md file in subdirectory, asset in sibling images/ dir
        let md = "![chart](https://example.com/charts/data.png)";
        let assets = vec![DownloadedAsset {
            url: "https://example.com/charts/data.png".to_string(),
            local_path: "/tmp/output/example.com/blog/images/data.png".to_string(),
            asset_type: "image".to_string(),
            size: 2048,
        }];
        let md_dir = Path::new("/tmp/output/example.com/blog");
        let result = resolve_asset_paths(md, md_dir, &assets);
        assert!(result.contains("images/data.png"));
    }
}
```

### 6.2 Integration Tests for `file_saver.rs`

```rust
#[test]
fn test_save_with_obsidian_wiki_links() {
    let temp_dir = TempDir::new().unwrap();
    let results = vec![ScrapedContent {
        title: "Test".to_string(),
        content: "See [about](https://example.com/about)".to_string(),
        url: ValidUrl::parse("https://example.com/page").unwrap(),
        excerpt: None,
        author: None,
        date: None,
        html: None,
        assets: Vec::new(),
    }];

    let obsidian = ObsidianOptions {
        wiki_links: true,
        ..Default::default()
    };

    save_as_markdown(&results, temp_dir.path(), &obsidian).unwrap();

    // Read file and verify wiki-link conversion
    let content = std::fs::read_to_string(
        temp_dir.path().join("example.com/page/index.md")
    ).unwrap();
    assert!(content.contains("[[about|about]]"));
}

#[test]
fn test_save_with_obsidian_tags() {
    // ... verify frontmatter contains tags: [tag1, tag2]
}

#[test]
fn test_save_without_obsidian_flags_unchanged() {
    // Verify that ObsidianOptions::default() produces
    // identical output to the pre-feature version
}
```

### 6.3 Edge Cases to Cover

| Edge Case | Expected Behavior |
|-----------|------------------|
| Link with no text: `[](https://example.com/foo)` | Convert to `[[foo|]]` (valid Obsidian syntax) |
| Link with same text as slug: `[about](https://example.com/about)` | Convert to `[[about]]` (Obsidian shorthand, no pipe needed) |
| Empty content string | Return empty string (no-op) |
| Content with no links | Return unchanged content |
| Asset URL not found in Markdown | No replacement (graceful skip) |
| Asset with URL-encoded characters in path | Fuzzy match via URL decoding |
| Tags with spaces: `"web dev, rust-lang"` | Preserved as-is (Obsidian supports multi-word tags) |
| Tags with special chars: `"#important"` | Strip leading `#` if present (Obsidian doesn't want `#` in frontmatter tags) |
| Nested code blocks with links inside | Links inside code blocks NOT converted |
| Links with same domain but different subdomain | NOT converted (e.g., `blog.example.com` ≠ `example.com`) |
| Relative links in Markdown: `[link](/about)` | NOT converted (no host to compare) |
| Anchor-only links: `[link](#section)` | NOT converted |

---

## 7. Dependencies

### 7.1 New Dependency

```toml
# Cargo.toml — add to [dependencies]
pathdiff = "0.2"
```

**Why `pathdiff`:** Cross-platform relative path calculation. `std::fs` has no built-in relative path computation between two arbitrary paths. `pathdiff::diff_paths()` handles Windows drive letters, UNC paths, and Unix paths correctly.

### 7.2 Existing Dependencies (no changes)

| Crate | Current Version | Usage |
|-------|----------------|-------|
| `regex` | `1` | Link/image pattern matching |
| `serde_yaml` | `0.9` | Frontmatter YAML serialization |
| `url` | `2` | URL parsing, percent decoding |
| `thiserror` | `2` | Error types (none new needed) |

**No new crates beyond `pathdiff`.** The `url` crate already provides percent decoding via `url::percent_encoding::percent_decode_str()`, so no separate `urlencoding` crate is needed.

---

## 8. Performance Considerations

### 8.1 Regex Compilation

All regex patterns use `LazyLock<Regex>` for one-time compilation at first use:

```rust
static MARKDOWN_LINK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"...").expect("BUG: invalid regex")
});
```

**Cost:** ~1-5ms on first call, zero on subsequent calls. This is the same pattern already used in `syntax_highlight.rs` (`CODE_BLOCK_RE`, `SYNTAX_SET`, `THEME_SET`).

### 8.2 String Allocation Strategy

| Operation | Strategy | Rationale |
|-----------|----------|-----------|
| `convert_wiki_links()` | `Regex::replace_all()` → `Cow<str>` → `String` | `replace_all()` returns `Cow`, avoiding allocation if no matches found |
| `resolve_asset_paths()` | `Regex::replace_all()` → `Cow<str>` → `String` | Same pattern — zero allocation when no assets to rewrite |
| `slug_from_url()` | `String::with_capacity(path.len())` | Pre-allocate based on input size |
| Frontmatter tags | `Vec::with_capacity(tags.len())` | Known size from CLI/config |

### 8.3 Memory Usage for Large Documents

For a 1MB Markdown document:
- `Regex::replace_all()` processes in a single pass — O(n) time, O(n) output
- No intermediate allocations for code-block skipping (alternation handles it inline)
- Total memory overhead: ~2x input size (original + output string)
- For 1MB input: ~2MB peak allocation, well within the 8GB RAM target

### 8.4 HDD-Optimized Considerations

The regex operations are CPU-bound, not IO-bound. On the target Haswell i5-4590:
- `convert_wiki_links()` on 100KB document: <5ms
- `resolve_asset_paths()` on 100KB document with 10 assets: <10ms
- These are negligible compared to the IO cost of writing files to HDD

---

## 9. Files Changed Summary

| File | Change | Lines Impact |
|------|--------|-------------|
| `src/infrastructure/converter/obsidian.rs` | **NEW** | ~150 LOC |
| `src/infrastructure/converter/mod.rs` | Add `pub mod obsidian` | +1 |
| `src/infrastructure/output/frontmatter.rs` | Add `tags` field + param | +10 |
| `src/infrastructure/output/file_saver.rs` | Add `ObsidianOptions`, wire conversions | +60 |
| `src/lib.rs` (Args) | Add 3 CLI flags | +30 |
| `src/cli/config.rs` | Add `[obsidian]` config fields | +5 |
| `Cargo.toml` | Add `pathdiff = "0.2"` | +1 |
| `src/main.rs` | Pass `ObsidianOptions` to `save_results()` | +15 |

**Total estimated impact:** ~270 new lines, zero deletions, zero breaking changes.
