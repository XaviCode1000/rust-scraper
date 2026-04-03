# Specification: Vault Auto-Detect, Quick-Save Mode, and Rich Metadata for Obsidian

## Requirements

### Vault Detection

**R1: CLI flag takes highest priority**
- The `--vault <path>` flag MUST override all other detection methods
- If the path does not contain `.obsidian/`, a warning is logged but the path is still used
- MUST: If the path does not exist, fall back to next detection method

**R2: 4-tier detection order**
- Tier 1: `--vault` CLI flag
- Tier 2: `OBSIDIAN_VAULT` environment variable
- Tier 3: `config.toml` `[obsidian]` section `vault_path` field
- Tier 4: Auto-scan directories for `.obsidian/app.json`
  - Current working directory
  - `~/Obsidian/`
  - `~/Documents/`
  - `~/` (home directory)
- If no vault is found, return `None` and use default output directory

### Quick-Save Mode

**R3: Quick-save bypasses TUI**
- When `--obsidian --quick-save` is used, the TUI MUST NOT be launched
- The scraper MUST scrape the single URL provided via `--url`
- Results MUST be saved to `{vault}/_inbox/{YYYY-MM-DD}-{slug}.md`
- If vault is not detected, fall back to `--output` directory (existing behavior)

**R4: Quick-save file naming**
- Filename format: `{YYYY-MM-DD}-{slug}.md` where slug is derived from URL path
- If a file with the same name exists, append `-1`, `-2`, etc.
- The `_inbox/` directory MUST be created if it does not exist

### Rich Metadata

**R5: Extended frontmatter fields**
- The following fields MUST be added to YAML frontmatter when rich metadata is enabled:
  - `readingTime`: Integer, minutes, calculated as `(word_count / 200.0).ceil()`, minimum 1
  - `language`: String, ISO 639-1 code (e.g., "en", "es", "fr"), detected via `whatlang`
  - `wordCount`: Integer, total word count of content
  - `contentType`: String, one of: "article", "product", "recipe", "paper", "documentation"
  - `status`: String, default "unread"
- All fields MUST use `#[serde(skip_serializing_if = "Option::is_none")]` for backward compatibility

**R6: Language detection**
- Use `whatlang` crate for language detection
- If confidence < 0.5, set language to `"unknown"`
- Cap input to first 1024 bytes for performance
- MUST NOT block the scraping pipeline

### Obsidian URI

**R7: Open note in Obsidian**
- After saving, construct `obsidian://open?vault={vault_name}&file={file_path}` URI
- Open URI via `xdg-open` on Linux
- MUST be non-blocking (fire-and-forget)
- MUST NOT fail the pipeline if Obsidian is not running
- MUST log a warning if URI opening fails

## Scenarios

### S1: Vault detected via CLI flag
- **Given** user passes `--vault ~/Obsidian/MyVault`
- **When** vault path is resolved
- **Then** `~/Obsidian/MyVault` is returned without checking other sources

### S2: Vault detected via environment variable
- **Given** `OBSIDIAN_VAULT=~/Documents/Knowledge` is set and `--vault` is not passed
- **When** vault path is resolved
- **Then** `~/Documents/Knowledge` is returned if `.obsidian/app.json` exists

### S3: Vault detected via config file
- **Given** `config.toml` has `[obsidian]\nvault_path = "~/Obsidian/Brain"` and no CLI/env var
- **When** vault path is resolved
- **Then** `~/Obsidian/Brain` is returned if `.obsidian/app.json` exists

### S4: Vault detected via auto-scan
- **Given** no CLI flag, no env var, no config
- **When** vault path is resolved
- **Then** first directory containing `.obsidian/app.json` is returned

### S5: No vault found
- **Given** no vault exists in any search location
- **When** vault path is resolved
- **Then** `None` is returned and default output directory is used

### S6: Quick-save with vault detected
- **Given** `--obsidian --quick-save --url https://example.com/article` and vault detected
- **When** scraping completes
- **Then** file is saved to `{vault}/_inbox/2026-04-03-article.md`
- **And** TUI is not launched

### S7: Quick-save without vault
- **Given** `--obsidian --quick-save --url https://example.com/article` and no vault detected
- **When** scraping completes
- **Then** file is saved to `--output` directory (existing behavior)
- **And** a warning is logged

### S8: Quick-save with filename collision
- **Given** `{vault}/_inbox/2026-04-03-article.md` already exists
- **When** quick-save creates a new file with the same slug
- **Then** file is saved as `2026-04-03-article-1.md`

### S9: Rich metadata generation
- **Given** content with 1234 words in English
- **When** frontmatter is generated with rich metadata
- **Then** `readingTime: 7`, `language: "en"`, `wordCount: 1234`

### S10: Rich metadata with short content
- **Given** content with 10 words
- **When** language detection runs
- **Then** `language: "unknown"` (confidence too low)
- **And** `readingTime: 1` (minimum)

### S11: Obsidian URI opens note
- **Given** file saved to `{vault}/_inbox/2026-04-03-article.md`
- **When** URI opening is triggered
- **Then** `xdg-open "obsidian://open?vault=MyVault&file=_inbox/2026-04-03-article.md"` is executed
- **And** the pipeline continues regardless of success/failure

### S12: Invalid vault path
- **Given** `--vault /nonexistent/path`
- **When** vault path is resolved
- **Then** fall back to next detection method
- **And** a warning is logged

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| E1 | `.obsidian/` exists but no `app.json` | Not a valid vault, continue scanning |
| E2 | Multiple vaults found in auto-scan | Use first one found, log warning |
| E3 | `OBSIDIAN_VAULT` points to non-directory | Fall back to next method |
| E4 | Config file has invalid TOML | Log warning, use defaults |
| E5 | Content is empty | `wordCount: 0`, `readingTime: 1`, `language: "unknown"` |
| E6 | Content is mixed language | Use language with highest confidence |
| E7 | `xdg-open` not installed | Log warning, continue pipeline |
| E8 | Obsidian not running | URI fails silently, log warning |
| E9 | Vault path has special characters | URL-encode in URI |
| E10 | `_inbox/` creation fails | Fall back to vault root |
| E11 | `--quick-save` without `--obsidian` | Ignore quick-save flag, log warning |
| E12 | Permission denied on vault directory | Fall back to default output |
| E13 | Very large content (>1MB) | Cap language detection at 1024 bytes |
| E14 | Slug extraction yields empty string | Use "untitled" as fallback |

## Acceptance Criteria

| ID | Criterion |
|----|-----------|
| AC-01 | `--vault /path/to/vault` overrides all other detection methods |
| AC-02 | `OBSIDIAN_VAULT=/path` env var works when `--vault` not specified |
| AC-03 | `config.toml` `[obsidian]` section with `vault_path` works as fallback |
| AC-04 | Auto-scan finds vault when `.obsidian/app.json` exists in parent directories |
| AC-05 | `--obsidian --quick-save` scrapes single URL and saves to vault inbox without TUI |
| AC-06 | Frontmatter includes `readingTime`, `language`, `wordCount`, `contentType`, `status` fields |
| AC-07 | `obsidian://open` URI fires on Linux after save (non-blocking, best-effort) |
| AC-08 | Existing behavior unchanged when no Obsidian flags used |
| AC-09 | `cargo check --all-features` passes |
| AC-10 | `cargo nextest run --test-threads 2` passes |
| AC-11 | `cargo clippy -- -D warnings` passes clean |
| AC-12 | `cargo fmt --check` passes |
| AC-13 | Language detection works for: en, es, fr, de, pt, zh, ja |
| AC-14 | Reading time estimated correctly (word_count / 200 WPM, minimum 1) |
| AC-15 | Filename collision handled with `-1`, `-2` suffix |
| AC-16 | Invalid vault path falls back gracefully |
| AC-17 | Empty content produces `wordCount: 0`, `readingTime: 1`, `language: "unknown"` |
| AC-18 | `--quick-save` without `--obsidian` logs warning and ignores flag |
| AC-19 | `_inbox/` directory created automatically if not exists |
| AC-20 | Vault detection completes in <100ms |
| AC-21 | Language detection completes in <50ms per page |
| AC-22 | URI opening does not block or fail the pipeline |
