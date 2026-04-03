# Tasks: CLI UX Improvement

**Phase**: SDD Phase 6 — Tasks  
**Based on**: design (sdd/cli-ux-improvement/design.md) + spec (sdd/cli-ux-improvement/spec.md)  
**Date**: 2026-04-02

---

## Task Breakdown Checklist

### Phase 0: Dependencies & Build Setup

- [ ] **T-001** — Add `indicatif` 0.17 to `[dependencies]` in Cargo.toml
  - Version: `"0.17"` with features: `["improved_unicode"]`
  - Verify: `cargo check` passes

- [ ] **T-002** — Add `clap_complete` 4 to `[dependencies]` in Cargo.toml
  - Version: `"4"`
  - Verify: `cargo check` passes

- [ ] **T-003** — Add `toml` 0.8 to `[dependencies]` in Cargo.toml
  - Version: `"0.8"`
  - Verify: `cargo check` passes

- [ ] **T-004** — Add `built` 0.7 to `[build-dependencies]` in Cargo.toml
  - Version: `"0.7"` with features: `["git2", "chrono"]`
  - Section: `[build-dependencies]`
  - Verify: `cargo check` passes

- [ ] **T-005** — Extend `build.rs` to call `built::write_built_file()`
  - Add `built::write_built_file()` call
  - Generate `src/built.rs` at build time
  - Add `include!("built.rs")` directive handling
  - Verify: `cargo build` creates `src/built.rs`

### Phase 1: Logging & NO_COLOR (R1, R3, R4)

- [ ] **T-010** — Add `is_no_color()` and `should_emit_emoji()` to `src/config.rs`
  - Check `NO_COLOR` env var (non-empty = true)
  - Public function `is_no_color() -> bool`
  - Public function `should_emit_emoji() -> bool`
  - Verify: UT-07 (NO_COLOR set → false), UT-08 (not set → true)

- [ ] **T-011** — Add `init_logging_dual()` to `src/config.rs`
  - Signature: `init_logging_dual(level: &str, quiet: bool, no_color: bool)`
  - Force all tracing to stderr via `.with_writer(|| std::io::stderr)`
  - If `quiet=true`: EnvFilter set to `"rust_scraper=warn,tokio=warn,reqwest=warn"` (info/debug suppressed)
  - If `quiet=false`: EnvFilter set to `"rust_scraper={level},tokio=warn,reqwest=warn"`
  - If `no_color=true`: `.with_ansi(false)` on fmt layer
  - Keep existing `init_logging()` for backward compat (delegates to `init_logging_dual` with defaults)
  - Verify: `cargo test` passes, all logs go to stderr

- [ ] **T-012** — Add EmojiStripLayer or emoji-stripping formatter to tracing
  - Add custom `format_event` closure in `fmt::layer()` when NO_COLOR
  - Use `LazyLock<Regex>` for emoji pattern compilation
  - Pattern covers: emojis, dingbats, variation selectors
  - Verify: UT-13, IT-13 (no emoji in output when NO_COLOR=1)

### Phase 2: CLI Arguments (R3, R5, R9, R10, R12)

- [ ] **T-020** — Add `--quiet` / `-q` flag to `Args` struct in `src/lib.rs`
  - Field: `#[arg(short = 'q', long, default_value = "false", env = "RUST_SCRAPER_QUIET")]`
  - Type: `pub quiet: bool`
  - Help heading: `"Display"`
  - Verify: IT-01 (long), IT-03 (short)

- [ ] **T-021** — Add `--dry-run` / `-n` flag to `Args` struct in `src/lib.rs`
  - Field: `#[arg(short = 'n', long, default_value = "false", env = "RUST_SCRAPER_DRY_RUN")]`
  - Type: `pub dry_run: bool`
  - Help heading: `"Display"`
  - Verify: IT-02 (long), IT-04 (short)

- [ ] **T-022** — Add `next_help_heading` groups to all Args fields
  - `"Target"`: `--url`, `--selector`
  - `"Output"`: `--output`, `--format`, `--export-format`
  - `"Discovery"`: `--max-pages`, `--delay-ms`, `--use-sitemap`, `--sitemap-url`, `--concurrency`
  - `"Behavior"`: `--interactive`, `--resume`, `--state-dir`, `--download-images`, `--download-documents`, `--force-js-render`, `--clean-ai`
  - `"Display"`: `--verbose`, `--quiet`, `--dry-run`
  - Verify: IT-09 (help output contains all 5 headings)

- [ ] **T-023** — Add `env = "RUST_SCRAPER_<NAME>"` to all existing Args fields
  - `url` → `RUST_SCRAPER_URL`
  - `selector` → `RUST_SCRAPER_SELECTOR`
  - `output` → `RUST_SCRAPER_OUTPUT`
  - `format` → `RUST_SCRAPER_FORMAT`
  - `export_format` → `RUST_SCRAPER_EXPORT_FORMAT`
  - `resume` → `RUST_SCRAPER_RESUME`
  - `state_dir` → `RUST_SCRAPER_STATE_DIR`
  - `delay_ms` → `RUST_SCRAPER_DELAY_MS`
  - `max_pages` → `RUST_SCRAPER_MAX_PAGES`
  - `download_images` → `RUST_SCRAPER_DOWNLOAD_IMAGES`
  - `download_documents` → `RUST_SCRAPER_DOWNLOAD_DOCUMENTS`
  - `verbose` → `RUST_SCRAPER_VERBOSE`
  - `concurrency` → `RUST_SCRAPER_CONCURRENCY`
  - `use_sitemap` → `RUST_SCRAPER_USE_SITEMAP`
  - `sitemap_url` → `RUST_SCRAPER_SITEMAP_URL`
  - `interactive` → `RUST_SCRAPER_INTERACTIVE`
  - `force_js_render` → `RUST_SCRAPER_FORCE_JS_RENDER`
  - `clean_ai` → `RUST_SCRAPER_CLEAN_AI`
  - Verify: IT-05 (env var overrides default), IT-06 (CLI overrides env)

- [ ] **T-024** — Update version string to include git commit hash
  - Use `built::BUILT_TIME_YYYYMMDD` and `built::GIT_COMMIT_HASH_SHORT` from generated `built.rs`
  - Format: `rust-scraper 1.0.7 (commit: abc1234, build: 2026-04-02)`
  - Add `#[command(version = ...)]` attribute or `#[command(long_version = ...)]`
  - May need to use `lazy_static` or runtime version formatting
  - Verify: `rust-scraper --version` shows extended format

### Phase 3: Config File System (R9)

- [ ] **T-030** — Create `src/cli/config.rs` — Config file loading
  - Define `ConfigDefaults` struct with serde `#[derive(Deserialize)]`
  - Fields: `output_dir`, `max_pages`, `delay_ms`, `concurrency`, `export_format`, `format`, `download_images`, `download_documents`, `use_sitemap`, `resume`, `quiet`, `verbose`, `dry_run`
  - All fields `Option<T>` so missing keys use clap defaults
  - Function `load_config(path: &Path) -> Result<ConfigDefaults>`
  - Default path: `~/.config/rust-scraper/config.toml`
  - Missing file: return `Ok(defaults)` (no error)
  - Invalid TOML: return `Err(CliError::ConfigFile)`
  - Verify: UT-09 (valid TOML), UT-10 (invalid TOML), UT-11 (missing file)

- [ ] **T-031** — Add config merge logic in `src/cli/config.rs`
  - Function `apply_config(config: &ConfigDefaults, args: &mut Args)`
  - For each field in config: if `Some(value)` and args field is at default, override
  - Precedence: CLI (already in args) > env (already in args via clap) > config > struct default
  - Strategy: Only override if args value equals the hardcoded default
  - Verify: IT-12 (config + env + CLI precedence)

### Phase 4: Shell Completions (R8)

- [ ] **T-040** — Create `src/cli/completions.rs`
  - Function `generate_completions(shell: Shell) -> String`
  - Use `clap_complete::generate()` with `Shell::Bash`, `Shell::Zsh`, `Shell::Fish`, `Shell::Elvish`, `Shell::PowerShell`
  - Write to a buffer, return as String
  - Verify: IT-07 (bash), IT-08 (fish)

- [ ] **T-041** — Add `completions` subcommand to Args
  - Use `#[derive(Subcommand)]` with `Completions { shell: Shell }` variant
  - `Shell` derived from `clap_complete::Shell` via `ValueEnum`
  - Handle in `main()` — when completions subcommand, print and exit 0
  - Use `#[command(args_conflicts_with_subcommands = true)]` on Args struct
  - Verify: IT-10 (completions prints valid script)

### Phase 5: Error Formatting & Exit Codes (R6, R11)

- [ ] **T-050** — Create `src/cli/error.rs` — CliError enum and format_cli_error
  - `enum CliError` with variants: `ConfigFile`, `NetworkError`, `PartialSuccess`, `PreflightFailed`
  - Each variant has `msg: String, suggestion: String` (PartialSuccess has `success`/`failed` u32 instead of msg)
  - Function `format_cli_error(err: &CliError, no_color: bool) -> String`
  - Format: `Error: {category}\n  {message}\n  Suggestion: {fix}`
  - Category mapping: ConfigFile→"Configuration", NetworkError→"Network", PartialSuccess→"Partial Success", PreflightFailed→"Preflight Check Failed"
  - Verify: UT-01 (ConfigFile formatting), UT-02 (NetworkError formatting)

- [ ] **T-051** — Implement `CliExit` enum with `Termination` trait
  - Variants: `Success`, `UsageError(String)`, `NetworkError(String)`, `IoError(String)`, `ProtocolError(String)`, `ConfigError(String)`, `PartialSuccess { success: usize, failed: usize }`
  - Implement `std::process::Termination` mapping to sysexits codes
  - `Success` → 0, `UsageError` → 64, `NetworkError` → 69, `IoError` → 74, `ProtocolError` → 76, `ConfigError` → 78, `PartialSuccess` → 69
  - Verify: UT-03 (PartialSuccess = 69), UT-04 (Success = 0), UT-05 (ConfigError = 78), UT-06 (NetworkError = 69)

### Phase 6: Summary Output (R7)

- [ ] **T-060** — Create `src/cli/summary.rs` — ScrapeSummary struct
  - Struct `ScrapeSummary` with fields: `urls_discovered: usize`, `urls_scraped: usize`, `urls_failed: usize`, `urls_skipped: usize`, `elements_extracted: usize`, `assets_downloaded: usize`, `duration: Duration`
  - Method `new(...)` constructor
  - Method `display(&self, no_color: bool) -> String`
  - Formatting: table-like layout with emoji icons (or ASCII if no_color)
  - Emoji icons: ✅, ⚠️, ❌, 📊 (suppressed when no_color)
  - ASCII fallback: `[OK]`, `[WARN]`, `[FAIL]`, `[SUMMARY]`
  - Verify: UT-12 (formatted output), UT-13 (ASCII only when no_color)

### Phase 7: Pre-flight Validation (R13)

- [ ] **T-070** — Add pre-flight HEAD check in `src/main.rs`
  - After URL validation, before discovery
  - Create HTTP client, send HEAD request to `args.url`
  - DNS error / timeout → return `CliError::PreflightFailed` with suggestion
  - 2xx/3xx/4xx → continue (not connectivity failure)
  - 5xx → log warning, continue
  - On failure: format error with suggestion, exit 69
  - Verify: UT-14 (unreachable host fails), UT-15 (2xx succeeds), UT-16 (404 warns but continues)

### Phase 8: Progress Bars (R2)

- [ ] **T-080** — Add progress bar wrapping for URL discovery in `main.rs`
  - After logging init, if not quiet: create spinner `ProgressBar::new_spinner()`
  - Set `ProgressDrawTarget::stderr()`
  - `enable_steady_tick(Duration::from_millis(100))`
  - Custom spinner style with URL count if available
  - `pb.finish_with_message("Found {} URLs")` after discovery
  - If quiet: skip progress bar creation entirely
  - Verify: IT-16 (progress bars write to stderr, stdout clean)

- [ ] **T-081** — Add per-URL scraping progress in `main.rs`
  - After discovery, create bounded `ProgressBar::new(urls_to_scrape.len() as u64)`
  - Set style: `[====>     ] 3/10 | 2.1 URL/s | ETA: 3s`
  - Loop: set_message(url), call, inc(1), track success/failure
  - `pb.finish_with_message(...)` after loop
  - If quiet: skip progress bar
  - Verify: IT-16 (progress bars to stderr)

- [ ] **T-082** — Expose `scrape_single_url_for_tui()` from application layer
  - Extract per-URL logic from `scrape_urls_for_tui()` into a new public function
  - Signature: `pub async fn scrape_single_url_for_tui(url: &Url, config: &ScraperConfig) -> Result<ScrapedContent>`
  - Re-export from `lib.rs`
  - Update `scrape_urls_for_tui()` to internally call the new function (keep backward compat)
  - Verify: existing tests for `scrape_urls_for_tui` still pass

### Phase 9: Main.rs Refactor & Integration

- [ ] **T-090** — Refactor `main()` to use `CliExit` return type
  - Change signature: `async fn main() -> CliExit`
  - Replace `anyhow::Result<()>` early returns with `return CliExit::...`
  - Wrap clap parsing: `Args::try_parse()` → catch errors → `CliExit::UsageError`
  - Add config file loading after arg parsing
  - Replace `.context()?` calls with explicit error mapping to CliExit variants
  - Verify: all exit codes match spec (0, 64, 69, 74, 76, 78)

- [ ] **T-091** — Implement dry-run mode in `main.rs`
  - After URL discovery: if `args.dry_run`, print URLs to stdout (one per line), print summary to stderr, return `CliExit::Success`
  - Skip scraping, file writing, export
  - Verify: IT-11 (no files written), S9 scenario

- [ ] **T-092** — Add summary printing at end of main.rs
  - After scraping, construct `ScrapeSummary`
  - If not quiet: `eprint!("{}", summary.display(is_no_color()))`
  - Return `CliExit::PartialSuccess` if failures > 0, else `CliExit::Success`
  - Verify: IT-10 (quiet suppresses summary), IT-15 (partial failure exit 69)

- [ ] **T-093** — Replace emoji in main.rs log messages with conditional helpers
  - Create helper function or macro for emoji/ASCII selection
  - Replace all hardcoded emojis with `if should_emit_emoji() { "..." } else { "..." }`
  - Apply to: startup messages, progress messages, completion messages
  - Verify: IT-13 (NO_COLOR=1 produces no emoji)

### Phase 10: Tests & Verification

- [ ] **T-100** — Unit tests for CliError and format_cli_error (UT-01 through UT-02)
- [ ] **T-101** — Unit tests for CliExit exit codes (UT-03 through UT-06)
- [ ] **T-102** — Unit tests for NO_COLOR emoji stripping (UT-07, UT-08)
- [ ] **T-103** — Unit tests for config file loading (UT-09 through UT-11)
- [ ] **T-104** — Unit tests for ScrapeSummary display (UT-12, UT-13)
- [ ] **T-105** — Unit tests for pre-flight validation (UT-14 through UT-16)
- [ ] **T-106** — Integration tests: arg parsing with new flags (IT-01 through IT-04)
- [ ] **T-107** — Integration tests: env var precedence (IT-05, IT-06)
- [ ] **T-108** — Integration tests: shell completions (IT-07, IT-08)
- [ ] **T-109** — Integration tests: help text grouping (IT-09)
- [ ] **T-110** — Integration tests: quiet mode, dry-run, config precedence (IT-10 through IT-12)
- [ ] **T-111** — Integration tests: NO_COLOR, stdout/stderr separation, exit codes (IT-13 through IT-16)
- [ ] **T-112** — Full test suite pass: `cargo test` with zero failures
- [ ] **T-113** — `cargo clippy` passes with no warnings
- [ ] **T-114** — `cargo doc --no-deps` passes with no warnings
- [ ] **T-115** — Manual verification: `2>/dev/null` produces clean stdout only
- [ ] **T-116** — Manual verification: `--help` shows grouped flags
- [ ] **T-117** — Manual verification: `--version` shows extended format

---

## Task Execution Order

```
T-001 → T-005     (dependencies, can be done in parallel)
T-010 → T-012     (logging/NO_COLOR, depends on nothing)
T-020 → T-024     (CLI args, depends on nothing for quiet/dry-run)
T-030 → T-031     (config file, depends on T-050 for CliError types)
T-040 → T-041     (completions, depends on T-022 for args structure)
T-050 → T-051     (error types, depends on nothing)
T-060             (summary, depends on T-051 for CliExit)
T-070             (pre-flight, depends on T-051)
T-080 → T-082     (progress bars, depends on T-010-T-012, T-082 app layer change)
T-090 → T-093     (main.rs integration, depends on ALL previous)
T-100 → T-117     (tests, interleaved with features)
```

### Parallelizable Groups

- **Group A**: T-001, T-002, T-003, T-004, T-010 (indep deps + emoji detection)
- **Group B**: T-020, T-021 (new flags, independent of each other)
- **Group C**: T-040, T-041 (completions after args have subcommands)
- **Group D**: T-100-T-105 (unit tests, write alongside features)

### Total Task Count: 47

- Dependencies & Build: 5 tasks
- Logging & NO_COLOR: 3 tasks
- CLI Arguments: 5 tasks
- Config File: 2 tasks
- Shell Completions: 2 tasks
- Error & Exit Codes: 2 tasks
- Summary Output: 1 task
- Pre-flight: 1 task
- Progress Bars: 3 tasks
- Main Integration: 4 tasks
- Tests & Verification: 19 tasks

---

## Spec Requirements Coverage Matrix

| Spec Req | Tasks | Status in Design |
|----------|-------|-----------------|
| R1 (stdout/stderr) | T-011, T-080, IT-14, IT-16 | Covered: stderr writer + progress bars |
| R2 (Progress bars) | T-080, T-081, T-082 | Covered: spinner + bounded bar |
| R3 (Quiet mode) | T-020, T-011, T-092 | Covered: -q flag + warn-only filter |
| R4 (NO_COLOR) | T-010, T-011, T-012, T-093 | Covered: env detection + emoji strip |
| R5 (Flag grouping) | T-022, T-024 | Covered: 5 help headings + version hash |
| R6 (Error format) | T-050, T-090 | Covered: CliError + format_cli_error |
| R7 (Summary) | T-060, T-092 | Covered: ScrapeSummary struct |
| R8 (Completions) | T-040, T-041 | Covered: clap_complete subcommand |
| R9 (Config file) | T-030, T-031 | Covered: TOML parsing + merge logic |
| R10 (Env vars) | T-023 | Covered: env attribute on all args |
| R11 (Exit codes) | T-051, T-090 | Covered: CliExit with Termination |
| R12 (Dry run) | T-021, T-091 | Covered: -n flag + early exit |
| R13 (Pre-flight) | T-070 | Covered: HEAD request check |
