# Proposal: Obsidian-compatible Markdown export

## Intent

Scraped Markdown files are not optimized for Obsidian vaults. Users must manually convert links to wiki-links, add tags, and fix asset paths. This change adds native Obsidian support so scraped content drops directly into an Obsidian vault and works out-of-the-box with backlinks, tags, and embedded assets.

## Scope

### In Scope
- **Tags in frontmatter** — Add `tags: Vec<String>` to `Frontmatter` struct, populated from CLI `--obsidian-tags` flag
- **Wiki-links conversion** — New module `src/infrastructure/converter/obsidian.rs` with `convert_wiki_links()` that transforms `[text](url)` → `[[slug|text]]` for same-domain links
- **Relative asset paths** — Calculate relative paths from `.md` file to downloaded assets, rewrite `![](path)` references
- **CLI flags** — `--obsidian-tags`, `--obsidian-wiki-links`, `--obsidian-relative-assets` (all optional, backward compatible)
- **Config file support** — Persist Obsidian options in `config.toml`

### Out of Scope
- Obsidian plugin development
- Automatic vault creation/sync
- Embedding generation for Obsidian
- Canvas/dataview integration
- Auto-tagging from HTML meta keywords (deferred to future iteration)

## Capabilities

### New Capabilities
- `obsidian-export`: Obsidian-compatible Markdown output with wiki-links, tags, and relative asset paths

### Modified Capabilities
- `markdown-output`: Extended to support Obsidian options while preserving existing behavior

## Approach

1. **Extend `Frontmatter`** — Add optional `tags: Vec<String>` field with `#[serde(skip_serializing_if = "Vec::is_empty")]`
2. **Create `obsidian.rs` converter** — Pure function `convert_wiki_links(content: &str, base_domain: &str) -> String` using regex to find markdown links, compare domain, convert to `[[slug|text]]` format
3. **Add `resolve_relative_asset_path()`** — Helper in `file_saver.rs` to compute relative paths from output `.md` location to asset `local_path`
4. **Extend `Args`** — Add three optional boolean/string flags to `Args` struct in `src/lib.rs`
5. **Wire in `save_as_markdown()`** — Conditionally apply wiki-link conversion and asset path rewriting when flags are set
6. **Update config** — Add `[obsidian]` section support in `ConfigDefaults`

All changes are additive and gated behind flags. Existing Markdown output is unchanged when flags are not set.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/infrastructure/output/frontmatter.rs` | Modified | Add `tags` field to `Frontmatter` struct |
| `src/infrastructure/output/file_saver.rs` | Modified | Wire wiki-link conversion, relative asset paths |
| `src/infrastructure/converter/obsidian.rs` | New | Wiki-link conversion logic |
| `src/infrastructure/converter/mod.rs` | Modified | Export new `obsidian` module |
| `src/lib.rs` (Args) | Modified | Add `--obsidian-*` CLI flags |
| `src/cli/config.rs` | Modified | Add `[obsidian]` config section support |
| `src/main.rs` | Modified | Pass Obsidian flags through scrape pipeline |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Regex performance on large documents | Medium | Use compiled `once_cell::sync::Lazy` regex; benchmark on 1MB+ docs |
| Wiki-link false positives (external links converted) | Low | Strict domain comparison: only convert when link host matches scraped page host |
| Relative path calculation errors on nested URLs | Medium | Use `pathdiff` crate for reliable cross-platform relative paths; add tests for edge cases |
| Config file backward compatibility | Low | New `[obsidian]` section is optional; missing section = defaults (disabled) |

## Rollback Plan

1. `git revert` the change commit — all changes are additive, no deletions
2. Users with `--obsidian-*` flags will see "unknown option" errors (expected, harmless)
3. No database or migration to rollback
4. No breaking changes to existing Markdown output format

## Dependencies

- `regex` crate (already in project dependencies)
- `pathdiff` crate (new dependency for cross-platform relative path calculation)

## Success Criteria

- [ ] `--obsidian-tags "tag1,tag2"` produces YAML frontmatter with `tags: [tag1, tag2]`
- [ ] `--obsidian-wiki-links` converts same-domain `[text](https://example.com/page)` to `[[page-slug|text]]`
- [ ] `--obsidian-relative-assets` rewrites `![](absolute/path)` to `![](../assets/image.png)` relative to `.md` file
- [ ] All three flags work independently and in combination
- [ ] Existing Markdown output (without flags) produces identical output to before
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo nextest run --test-threads 2` passes with all new tests
- [ ] `cargo fmt --check` passes
