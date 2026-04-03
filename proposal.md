# Proposal: CLI UX Improvement

## Intent
The rust-scraper CLI sends all tracing output to stdout (breaking piping), lacks visual progress feedback, has no CI-friendly modes, and bundles 17 flags with no organization. This change makes the tool usable for both interactive users and automated pipelines.

## Scope

### In Scope (P0-P2)
- **P0**: stdout/stderr separation, progress bars (`indicatif`), `--quiet` mode
- **P1**: flag grouping in help text, rich error formatting, summary output, `NO_COLOR` support
- **P2**: shell completions, config file (`~/.config/rust-scraper/config.toml`), `RUST_SCRAPER_*` env vars, semantic exit codes, `--dry-run`, build-info version, pre-flight URL validation

### Out of Scope
- P3: Subcommand architecture — separate change
- TUI modifications, AI semantic cleaning, core scraping logic

## Capabilities

### New Capabilities
- `output-routing`: stdout/stderr separation, quiet mode, NO_COLOR
- `progress-feedback`: progress bars for discovery/scraping, summary output
- `cli-ux`: clap flag groups, rich error messages, exit codes, dry-run, pre-flight checks, version info
- `config-file`: persistent defaults via config.toml
- `shell-completions`: bash/zsh/fish via clap_complete
- `env-vars`: RUST_SCRAPER_* env var support on all flags

## Approach

**Phase 1: Output & Feedback (P0)**
- All `tracing::info!/warn!/error!` route to stderr; stdout reserved exclusively for scrape data
- Add `indicatif` with `ProgressBar` wrapping `discover_urls_for_tui()` and `scrape_urls_for_tui()` 
- `--quiet`: suppresses progress bars and non-error output; CI/cron friendly
- `NO_COLOR`: guard emoji/styling behind `std::env::var("NO_COLOR")` check

**Phase 2: CLI Structure (P1)**
- Reorganize `Args` with `#[arg(help_heading)]` for groups: Target, Output, Discovery, Behavior, Display
- `CliError` with structured messages: error type, context, suggested fix
- Structured `ScrapeSummary` printed at end (stderr by default, stdout if interactive)

**Phase 3: Supporting Infrastructure (P2)**
- `clap_complete` for shell completions
- `#[arg(env = "RUST_SCRAPER_...")]` on all Args fields
- Config file via `toml` crate; precedence: CLI > env > config > defaults
- `--dry-run`: prints planned actions without scraping
- Exit codes: 0=success, 1=general, 2=config error, 3=network, 4=partial success
- Build-time version via `built` crate in `build.rs`
- Pre-flight: validate URL scheme, HEAD connectivity check

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/config.rs` | Modified | Dual output tracer setup, NO_COLOR detection |
| `src/main.rs` | Modified | Progress bars, quiet mode, summary, dry-run, exit codes |
| `src/lib.rs` Args | Modified | Flag grouping, quiet/dry-run flags, `env=` on all args |
| `Cargo.toml` | Modified | Add: `indicatif`, `clap_complete`, `toml`, `built` |
| `build.rs` | New | Build-time version info |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Progress bars interfere with file output | Low | Bars write to stderr; data goes to files only |
| Config/CLI precedence confusion | Low | Strict hierarchy: CLI > env > config > defaults |
| Breaking scripts parsing stdout | Low | Tracing currently pollutes stdout; separating is corrective |

## Rollback Plan
All changes additive (new flags with defaults, internal behavior). Revert the commit — no API or data format changes. Config file is opt-in. Run `--verbose` to restore old trace density.

## Dependencies
`indicatif` 0.17, `clap_complete` 4, `toml` 0.8, `built` 0.7 (build-script only)

## Success Criteria
- [ ] `2>/dev/null` produces empty stdout (no trace lines)
- [ ] `--quiet` suppresses progress bars and non-error output
- [ ] Progress bars show URL count, ETA, rate during discovery/scraping
- [ ] Help shows grouped flags by category
- [ ] Errors include context + suggested fix
- [ ] `NO_COLOR=1` disables all color/emoji
- [ ] `RUST_SCRAPER_MAX_PAGES=5` works without `--max-pages`
- [ ] `--dry-run` prints planned actions without scraping
- [ ] Exit codes differentiate failure types
