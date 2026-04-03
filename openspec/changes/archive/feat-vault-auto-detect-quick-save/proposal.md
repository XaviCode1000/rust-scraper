# Proposal: Vault Auto-Detect, Quick-Save Mode, and Rich Metadata for Obsidian

## Intent

Obsidian users need a frictionless path from URL to vault note. Currently (#22 merged), Obsidian export requires manual vault path specification and produces minimal frontmatter. This change adds: (1) automatic vault detection via CLI > env > config > filesystem scan, (2) a `--quick-save` mode that skips TUI and writes directly to the vault inbox, and (3) rich frontmatter metadata (readingTime, language, wordCount, contentType, status) for Dataview compatibility.

## Scope

### In Scope
- **Vault auto-detect**: Resolution order `--vault` flag > `OBSIDIAN_VAULT` env > `config.toml` > auto-scan parent dirs for `.obsidian/app.json`
- **Quick-Save mode**: `--obsidian --quick-save` branch in `main.rs` that bypasses TUI, scrapes single URL, saves directly to vault inbox
- **Rich metadata**: Extend `Frontmatter` struct with `readingTime`, `language`, `wordCount`, `contentType`, `status`
- **Obsidian URI**: Best-effort `obsidian://open?vault=...&file=...` via `xdg-open` (Linux only)
- **New dependency**: `whatlang = "0.16"` for language detection (lightweight, ~200KB, zero deps)

### Out of Scope
- Windows/Mac URI handlers (deferred — Linux only for now)
- Vault sync/backup logic
- Dataview query generation
- Image/document download to vault assets folder (already works via `--download-images`)

## Capabilities

### New Capabilities
- `vault-auto-detect`: Automatic Obsidian vault path resolution with 4-tier fallback
- `quick-save-mode`: Headless single-URL scrape-to-vault pipeline bypassing TUI
- `rich-metadata`: Extended YAML frontmatter with readingTime, language, wordCount, contentType, status
- `obsidian-uri`: Open saved note in Obsidian via URI scheme (Linux)

### Modified Capabilities
- `obsidian-export`: Existing `ObsidianOptions` struct gains `vault_path` field; `Frontmatter` gains new fields; `save_results` gains quick-save branch

## Approach

**Layer-respecting, backward-compatible.** All new code follows Clean Architecture strictly:

1. **Infrastructure** (`src/infrastructure/obsidian/`): New module with pure logic
   - `vault_detector.rs` — `detect_vault_path(cli_path, env_var, config_path)` → `Option<PathBuf>`
   - `metadata.rs` — `compute_metadata(content: &str)` → `ArticleMetadata { reading_time, language, word_count, content_type }`
   - `uri.rs` — `build_obsidian_uri(vault_name, file_path)` → `String` + `open_in_obsidian(uri)` via `std::process::Command`

2. **Infrastructure** (`src/infrastructure/output/`):
   - Extend `Frontmatter` struct in `frontmatter.rs` with new optional fields
   - Extend `generate()` signature to accept `ArticleMetadata`
   - `ObsidianOptions` gains `vault_path: Option<PathBuf>`

3. **CLI** (`src/lib.rs`):
   - Add `--vault` (path), `--quick-save` (bool) flags to `Args`
   - Add `vault_path` to `ConfigDefaults` in `src/cli/config.rs`

4. **Binary** (`src/main.rs`):
   - Vault detection logic after config loading
   - Quick-save branch: detect → scrape single URL → save to vault inbox → open URI
   - Existing pipeline unchanged (full backward compatibility)

**Word count**: Manual character-to-word split (no new crate needed).
**Reading time**: `word_count / 200` (average adult reading speed).
**Language**: `whatlang::detect()` with confidence threshold 0.5, fallback to `"en"`.
**Content type**: Heuristic based on HTML structure (article, list, documentation).

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/lib.rs` | Modified | Add `--vault`, `--quick-save` CLI flags |
| `src/cli/config.rs` | Modified | Add `vault_path: Option<String>` to `ConfigDefaults` |
| `src/main.rs` | Modified | Vault detection, quick-save branch, URI opening |
| `src/infrastructure/mod.rs` | Modified | Add `pub mod obsidian;` |
| `src/infrastructure/obsidian/mod.rs` | New | Module root |
| `src/infrastructure/obsidian/vault_detector.rs` | New | 4-tier vault path resolution |
| `src/infrastructure/obsidian/metadata.rs` | New | Language, word count, reading time, content type |
| `src/infrastructure/obsidian/uri.rs` | New | Obsidian URI builder + xdg-open |
| `src/infrastructure/output/frontmatter.rs` | Modified | Extend `Frontmatter` struct + `generate()` signature |
| `src/infrastructure/output/file_saver.rs` | Modified | `ObsidianOptions` gains `vault_path`, pass metadata to frontmatter |
| `Cargo.toml` | Modified | Add `whatlang = "0.16"` dependency |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| False positive vault scan (`.obsidian/` exists but not a vault) | Low | Validate `.obsidian/app.json` exists, not just directory |
| Language detection inaccurate on short content | Medium | Confidence threshold 0.5, fallback to `"en"`, mark as `"unknown"` if below threshold |
| Obsidian not running when URI opens | Medium | Non-blocking `Command::spawn()`, log warning on failure, never block pipeline |
| `whatlang` crate compatibility with MSRV 1.88 | Low | `whatlang 0.16` supports Rust 1.56+, well below MSRV |
| Breaking change to `generate()` function signature | Low | Add new `generate_with_metadata()` function, keep existing `generate()` as wrapper |

## Rollback Plan

1. `git revert` the merge commit — all changes are additive, no existing behavior modified
2. Remove `whatlang` from `Cargo.toml`
3. Delete `src/infrastructure/obsidian/` directory
4. Revert modifications to `frontmatter.rs`, `file_saver.rs`, `lib.rs`, `main.rs`, `config.rs`
5. Run `cargo check` to verify clean build

## Dependencies

- **Issue #22** (Obsidian Markdown export) — already merged, provides base infrastructure
- **`whatlang 0.16`** — new crate for language detection (pure Rust, no system deps)
- **`xdg-open`** — system binary for Linux URI handling (standard on all Linux desktops)

## Success Criteria

- [ ] `--vault /path/to/vault` overrides all other detection methods
- [ ] `OBSIDIAN_VAULT=/path` env var works when `--vault` not specified
- [ ] `config.toml` `[obsidian]` section with `vault_path` works as fallback
- [ ] Auto-scan finds vault when `.obsidian/app.json` exists in parent directories
- [ ] `--obsidian --quick-save` scrapes single URL and saves to vault inbox without TUI
- [ ] Frontmatter includes `readingTime`, `language`, `wordCount`, `contentType`, `status` fields
- [ ] `obsidian://open` URI fires on Linux after save (non-blocking, best-effort)
- [ ] Existing behavior unchanged when no Obsidian flags used
- [ ] `cargo check --all-features` passes
- [ ] `cargo nextest run --test-threads 2` passes
