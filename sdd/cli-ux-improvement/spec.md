# Specification: CLI UX Improvement

**Phase**: SDD Phase 4 — Spec  
**Status**: Complete  
**Date**: 2026-04-02  
**Based on**: proposal (sdd/cli-ux-improvement/proposal)

## Overview
Transform rust-scraper from a tool with flat output and 17 ungrouped flags into a polished CLI with proper output routing, visual progress, structured errors, and supporting infrastructure. Additive only — no breaking changes.

---

## 1. Requirements

### Output Routing (P0)

**R1** — stdout/stderr Separation
- **ID**: R1 | **Type**: Functional | **Priority**: P0 (MUST)
- All tracing output (info, debug, warn, error) MUST route to stderr exclusively
- stdout MUST be reserved exclusively for scrape data output
- Enables `rust-scraper --url ... 2>/dev/null` to produce clean data on stdout

**R2** — Progress Bars (Discovery & Scraping)
- **ID**: R2 | **Type**: Functional | **Priority**: P0 (MUST)
- `indicatif::ProgressBar` wrapping `discover_urls_for_tui()` (URL count, current URL, ETA, rate)
- Second `ProgressBar` wrapping `scrape_urls_for_tui()` (pages completed/total, current URL, ETA, rate)
- Progress bars MUST write to stderr via `ProgressDrawTarget::stderr()`

**R3** — Quiet Mode (`-q` / `--quiet`)
- **ID**: R3 | **Type**: Functional | **Priority**: P0 (MUST)
- Suppress all progress bars, suppress info!/debug! tracing, keep warn!/error!
- Exit code 0 on success (no summary printed)

**R4** — NO_COLOR Detection
- **ID**: R4 | **Type**: Functional | **Priority**: P1 (SHOULD)
- If `NO_COLOR` env var is set (any non-empty value), all emoji suppressed from log output
- Progress bar styling disabled; `indicatif` respects `NO_COLOR` natively
- Emoji stripping in tracing requires custom layer

### CLI Structure (P1)

**R5** — Flag Grouping
- **ID**: R5 | **Type**: Functional | **Priority**: P1 (SHOULD)
- All flags organized via `#[arg(next_help_heading = "...")]`:
  - `Target`: `--url`, `--selector`
  - `Output`: `--output`, `--format`, `--export-format`
  - `Discovery`: `--max-pages`, `--delay-ms`, `--use-sitemap`, `--sitemap-url`, `--concurrency`
  - `Behavior`: `--interactive`, `--resume`, `--state-dir`, `--download-images`, `--download-documents`, `--force-js-render`, `--clean-ai`
  - `Display`: `--verbose`, `--quiet`, `--dry-run`
- `--version` MUST include git commit hash

**R6** — Structured Error Formatting
- **ID**: R6 | **Type**: Functional | **Priority**: P1 (SHOULD)
- CLI errors display: error type header, context message, suggested fix
- Format: `Error: {category}\n  {message}\n  Suggestion: {fix}`

**R7** — Summary Output
- **ID**: R7 | **Type**: Functional | **Priority**: P1 (SHOULD)
- End-of-run structured summary to stderr (URLs discovered/scraped/failed/skipped, elements extracted, assets, duration)
- Suppressed in `--quiet` mode; uses emoji unless `NO_COLOR` set

### Supporting Infrastructure (P2)

**R8** — Shell Completions | **P2** — Generate bash/zsh/fish/elvish/powershell via `clap_complete`

**R9** — Config File | **P2** — `~/.config/rust-scraper/config.toml`; precedence: CLI > env > config > defaults

**R10** — Environment Variables | **P2** — `RUST_SCRAPER_*` on all flags via `#[arg(env = "...")]`

**R11** — Semantic Exit Codes | **P2** — sysexits: 0=OK, 64=usage, 69=network/partial, 74=I/O, 76=protocol, 78=config

**R12** — Dry-Run Mode | **P2** — `--dry-run` discovers URLs, prints planned actions, exits without scraping

**R13** — Pre-flight URL Validation | **P2** — Validate scheme + HEAD request before discovery; fail fast on DNS/connection errors

---

## 2. Scenarios

### S1: stdout/stderr Separation
**Given** `rust-scraper --url https://example.com --output ./out`
**When** scraper executes
**Then** all tracing goes to stderr; stdout is clean; `2>/dev/null` produces no stdout

### S2: Quiet Mode in CI
**Given** CI runs `rust-scraper --url https://example.com --quiet`
**When** job executes
**Then** no progress bars, no info/debug, warnings/errors still visible, exit 0 on success

### S3: Progress During Discovery
**Given** `rust-scraper --url https://large-site.com --max-pages 50`
**When** discovery begins
**Then** spinner bar shows: `[████████░░] 25/52 URLs found` with rate and ETA

### S4: Progress During Scraping
**Given** 20 URLs to scrape
**When** scraping starts
**Then** bar shows: `[████████████████░░] 14/20 | 3.1 URLs/s | ETA: 2s`

### S5: NO_COLOR Mode
**Given** `NO_COLOR=1` set
**When** scraper runs
**Then** no emoji in output, ASCII-only progress bars, plain text separators

### S6: Grouped Help Text
**Given** `rust-scraper --help`
**When** help displayed
**Then** flags grouped under headings: Target, Output, Discovery, Behavior, Display

### S7: Structured Error with Suggestion
**Given** `rust-scraper --url "ftp://invalid"`
**When** validation fails
**Then** error shows category, message, and suggestion for using http/https

### S8: Config File Precedence
**Given** config.toml has `max_pages = 20`, env has `RUST_SCRAPER_MAX_PAGES=15`
**When** run without `--max-pages`
**Then** max_pages = 15; when run with `--max-pages 5` then max_pages = 5

### S9: Dry-Run Mode
**Given** `rust-scraper --url https://example.com --dry-run`
**When** executed
**Then** discovery runs, URLs listed to stdout, no scrapes, no file writes, exit 0

### S10: Shell Completions
**Given** `rust-scraper completions bash`
**When** executed
**Then** bash completion script printed to stdout, exit 0

### S11: Pre-flight Connectivity
**Given** unreachable server
**When** HEAD request times out
**Then** fails fast with NetworkError suggestion, exit 69

### S12: Partial Success
**Given** 10 URLs, 3 fail
**When** run completes
**Then** exit 69, summary shows 7 succeeded / 3 failed, failures logged to stderr

---

## 3. API Contracts

### New Flags
| Flag | Short | Default | Env Var |
|------|-------|---------|---------|
| `--quiet` | `-q` | false | `RUST_SCRAPER_QUIET` |
| `--dry-run` | `-n` | false | `RUST_SCRAPER_DRY_RUN` |
| `completions <SHELL>` | N/A | N/A | N/A |

### All Flags Get Env Var Support
Every existing arg gets `env = "RUST_SCRAPER_<NAME>"` attribute.

### Config File Schema (~/.config/rust-scraper/config.toml)
```toml
[defaults]
output_dir = "output"
max_pages = 10
delay_ms = 1000
concurrency = "auto"
export_format = "jsonl"
format = "markdown"
download_images = false
download_documents = false
use_sitemap = false
resume = false
quiet = false
verbose = 0
dry_run = false
```

### Version Output
`rust-scraper 1.0.7 (commit: abc1234, build: 2026-04-02)`

### Exit Codes
| Code | Meaning |
|------|---------|
| 0 | EX_OK — all operations succeeded |
| 64 | EX_USAGE — invalid CLI args |
| 69 | EX_UNAVAILABLE/EX_PARTIAL — network failure or partial success |
| 74 | EX_IOERR — file system error |
| 76 | EX_PROTOCOL — WAF/CAPTCHA, all blocked |
| 78 | EX_CONFIG — config file parse error |

---

## 4. Error Contracts

### New CliError Enum
```rust
pub enum CliError {
    ConfigFile { msg: String, suggestion: String },
    NetworkError { msg: String, suggestion: String },
    PartialSuccess { success: usize, failed: usize, suggestion: String },
    PreflightFailed { msg: String, suggestion: String },
}
```

### Error Formatting
All CLI errors use `format_cli_error()`:
1. Extract root cause from anyhow chain
2. Map to category (Invalid URL, Configuration, Network, I/O, WAF)
3. Display: `Error: {category}\n  {message}\n  Suggestion: {fix}`
4. In `--quiet`, errors still print to stderr

### New Error Matrix
| Error | Trigger | Exit | Suggestion |
|-------|---------|------|-------------|
| ConfigFile | Invalid config.toml | 78 | Check TOML syntax |
| ConfigFileMissing | Config dir missing | — (skip) | N/A |
| PreflightFailed | HEAD fails connection | 69 | Check network/URL |
| PartialSuccess | Some URLs fail | 69 | Review stderr |
| WAFAllBlocked | WAF, no success | 76 | Rotate UA, increase delay |

---

## 5. File Changes

### Cargo.toml — New Dependencies
```
indicatif = { version = "0.17", features = ["improved_unicode"] }
clap_complete = "4"
toml = "0.8"
built = { version = "0.7", features = ["git2", "chrono"] }
```

### build.rs — Extended
Add `built::write_built_file()` call to generate `src/built.rs` with git commit hash and build timestamp.

### src/config.rs — Dual Output Tracer
1. Add `init_logging_dual(level: &str, stderr_only: bool)`
2. Force tracing to stderr via `.with_writer(std::io::stderr)`
3. Add `should_emit_emoji() -> bool` (checks NO_COLOR)
4. Add EmojiStripLayer for NO_COLOR mode
5. In quiet mode, EnvFilter level set to `warn,error` only

### src/lib.rs — Args Struct
1. Add `--quiet` (short `-q`, env `RUST_SCRAPER_QUIET`)
2. Add `--dry-run` (short `-n`, env `RUST_SCRAPER_DRY_RUN`)
3. Add `next_help_heading` to all args with 5 groups
4. Add `env = "RUST_SCRAPER_..."` to all 17+ existing fields
5. Update version to show extended format via built crate

### src/main.rs — Orchestrator
1. Check `args.quiet` early, pass to logging init
2. Wrap `discover_urls_for_tui()` with `ProgressBar::new_spinner()`
3. Wrap `scrape_urls_for_tui()` with progress bar (per-URL tracking)
4. If `--dry-run`: print planned actions, exit 0
5. Pre-flight HEAD check before scraping
6. Track per-URL failures (don't bail on first failure)
7. Print `ScrapeSummary` to stderr at end
8. Return custom `CliExit` type mapping to sysexits codes

### src/error.rs — Extended
Add `CliError` enum with ConfigFile, NetworkError, PartialSuccess, PreflightFailed.
Add `format_cli_error()` helper function.

### New Files
| File | Purpose |
|------|---------|
| `src/cli/mod.rs` | Re-export cli submodules |
| `src/cli/error.rs` | CliError, format_cli_error, CliExit (Termination) |
| `src/cli/completions.rs` | Shell completion generation |
| `src/cli/config.rs` | Config file parsing |
| `src/cli/summary.rs` | ScrapeSummary struct and printing |
| `src/built.rs` | Generated by build.rs at build time |

### Modified Files Summary
| File | Change |
|------|--------|
| Cargo.toml | +indicatif, +clap_complete, +toml, +built |
| build.rs | +built::write_built_file() |
| src/lib.rs | +quiet, +dry-run, help headings, env vars |
| src/main.rs | progress bars, summary, dry-run, exit codes, pre-flight |
| src/config.rs | stderr logging, NO_COLOR, quiet mode |
| src/error.rs | +CliError enum, format_cli_error |

---

## 6. Test Requirements

### Unit Tests
| ID | Target | Assertion |
|----|--------|-----------|
| UT-01 | format_cli_error(ConfigFile) | Contains "Error: Configuration", "Suggestion:" |
| UT-02 | format_cli_error(NetworkError) | Contains "Error: Network", suggestion |
| UT-03 | CliExit::PartialSuccess | Returns 69 |
| UT-04 | CliExit::Success | Returns 0 |
| UT-05 | CliExit::ConfigError | Returns 78 |
| UT-06 | CliExit::NetworkError | Returns 69 |
| UT-07 | should_emit_emoji() with NO_COLOR | Returns false |
| UT-08 | should_emit_emoji() without NO_COLOR | Returns true |
| UT-09 | load_config() valid TOML | Correct ConfigDefaults |
| UT-10 | load_config() invalid TOML | Returns error |
| UT-11 | load_config() missing file | Returns defaults (no error) |
| UT-12 | ScrapeSummary::display() | Formatted table output |
| UT-13 | ScrapeSummary::display() NO_COLOR | ASCII only, no emoji |
| UT-14 | Pre-flight unreachable host | PreflightFailed error |
| UT-15 | Pre-flight 2xx response | Ok(()) |
| UT-16 | Pre-flight 404 response | Ok(()) with warning |

### Integration Tests
| ID | Target | Assertion |
|----|--------|-----------|
| IT-01 | Args parse with --quiet | quiet == true |
| IT-02 | Args parse with --dry-run | dry_run == true |
| IT-03 | Args parse with -q | quiet == true |
| IT-04 | Args parse with -n | dry_run == true |
| IT-05 | RUST_SCRAPER_MAX_PAGES=25 env, no CLI flag | max_pages == 25 |
| IT-06 | RUST_SCRAPER_MAX_PAGES=25 + --max-pages 10 | max_pages == 10 (CLI wins) |
| IT-07 | completions bash | Valid bash script output |
| IT-08 | completions fish | Valid fish script |
| IT-09 | --help output | Contains all 5 group headings |
| IT-10 | --quiet suppresses summary | No summary visible |
| IT-11 | --dry-run: no scrape, no files | Only discovery runs |
| IT-12 | Config file + env + CLI precedence | CLI > env > config > default |
| IT-13 | NO_COLOR=1 output | No emoji anywhere |
| IT-14 | 2>/dev/null captures no data on stdout | Clean data pipable |
| IT-15 | Partial failure exit code | Exit 69, not 0 or 1 |
| IT-16 | Progress bars write to stderr | Stdout remains clean during scraping |
