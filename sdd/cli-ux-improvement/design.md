# Technical Design: CLI UX Improvement

**Phase**: SDD Phase 5 — Design  
**Based on**: spec (sdd/cli-ux-improvement/spec.md)  
**Date**: 2026-04-02

---

## 1. Architecture Decisions

### 1.1 Overall Approach: Additive Layering

All changes are additive — no existing public API signatures change. The design introduces a new `cli/` module namespace that houses all new CLI UX components, leaving `lib.rs` (the library interface) untouched except for new field additions to the `Args` struct. The `main.rs` orchestrator gains progress tracking, exit code handling, and dry-run logic without touching the application layer functions.

### 1.2 Output Routing Decision

**Decision**: Tracing subscriber is reconfigured early in `main()` to use a custom `MakeWriter` that always writes to `stderr`. This is a one-time configuration before any log emission.

**Rationale**: The existing `init_logging()` in `src/config.rs` uses the default `fmt::Layer` which writes to stdout. By adding `.with_writer(std::io::stderr)`, all tracing automatically routes to stderr. This is the cleanest approach because:
- No changes needed at every `info!`/`debug!` call site
- Works with existing tracing layers
- Progress bars from `indicatif` naturally render to stderr

### 1.3 Progress Bar Integration Decision

**Decision**: Progress bars wrap the two main application-layer calls in `main.rs`:
- `discover_urls_for_tui()` → spinner-style `ProgressBar` (count unknown during discovery)
- `scrape_urls_for_tui()` → bounded `ProgressBar::new(total)` (count known beforehand)

**Rationale**: The application layer returns `Vec<Url>` / `Vec<ScrapedContent>` — it doesn't expose per-item callbacks. Rather than modifying the application layer (violating Clean Architecture), progress is tracked at the orchestrator level by:
- For discovery: using a spinner that updates its message with the current count as items are discovered internally (note: the function returns a completed Vec, so the spinner runs during the async call and updates via periodic ticks)
- For scraping: wrapping individual URL processing inside `scrape_urls_for_tui` requires a slight modification — OR alternatively, the progress bar wraps the outer call and the function's internals are enhanced to emit tracing events that the progress bar can observe

**Refined decision**: Since `scrape_urls_for_tui` is an application-layer function, passing a progress bar reference would couple it to `indicatif`. Instead:
1. Discovery progress: Use a spinner during the `.await`, then set finished count when the call returns
2. Scraping progress: Create a new `scrape_urls_with_progress()` wrapper in `main.rs` that iterates URLs individually, calls the per-URL application function, and updates the bar. This requires either a per-URL scrape function exposed from the application layer, or cloning the scraping loop into main.rs.

**Final decision**: Expose `scrape_single_url_for_tui(url, config) -> Result<ScrapedContent>` from the application layer (new public function). `main.rs` then loops over URLs with progress tracking, collecting results. This keeps the application layer pure (no indicatif imports) while giving the orchestrator per-URL control.

### 1.4 Error Formatting Decision

**Decision**: `CliError` is a custom enum in `src/cli/error.rs` with a `format_cli_error()` function. The `main()` function catches `anyhow::Error`, walks the error chain, and maps known patterns to `CliError` variants. Unknown errors fall back to a default `NetworkError` with the anyhow chain as context.

**Rationale**: Using `std::process::Termination` trait via `CliExit` allows `main()` to return exit codes naturally without calling `std::process::exit()` directly (which bypasses cleanup).

### 1.5 Config File Decision

**Decision**: Config file uses the `toml` crate with a `ConfigDefaults` struct that mirrors the clap `Args` defaults. Loading happens after Args parsing. Config values are applied via `config.override_args(&args)` pattern where CLI args take final precedence.

**Precedence chain** (applied in main.rs):
1. Struct defaults (hardcoded in Args derive macros)
2. Config file TOML (loaded, applied as middleware)
3. Environment variables (handled by clap's `env` attribute automatically)
4. CLI arguments (parsed last, highest precedence by clap's behavior)

**Wait** — clap env vars are applied *during* parsing, which happens before config file loading. The correct precedence order for clap-with-env + config file:
1. Struct defaults
2. Config file TOML
3. Environment variables (clap reads these during `Args::parse()`)
4. CLI arguments

So the flow must be: parse args first (which gives us cli+env+defaults), then load config, then manually override with CLI args. This requires tracking which fields were explicitly provided by CLI vs defaulted.

**Final approach**: Use `Option<T>` for all arg fields where config file override is desired, then apply config in between. However, this is intrusive. Simpler approach: clap handles env+CLI, then config defaults fill in what's still at default value. We define a `merge_config(config, args)` function in `src/cli/config.rs` that returns a merged struct.

Actually, the spec says: precedence CLI > env > config > defaults. Since clap handles CLI and env together, the flow is:
1. Parse args with clap (gives CLI + env or defaults)
2. Load config from TOML
3. For each field where the parsed value equals the default, check if config has a value and use it

This requires knowing the default for each field. We implement this via `ConfigDefaults` struct and a `apply_config_file(args, config)` function.

### 1.6 Dry-Run Decision

**Decision**: After URL discovery, if `--dry-run` is active, print the discovered URLs to stdout and exit 0. No scraping, no file writes. This is a simple branch in `main.rs` after the discovery step.

### 1.7 Shell Completions Decision

**Decision**: Add a subcommand `completions <SHELL>` via clap subcommand. When invoked, generate the completion script using `clap_complete` and print to stdout. This is mutually exclusive with the main scraping operation.

---

## 2. Module Structure

### New File Layout

```
src/
├── main.rs           (modified — progress bars, dry-run, exit codes, summary)
├── lib.rs            (modified — Args: +quiet, +dry-run, env vars, help headings)
├── config.rs         (modified — stderr logging, NO_COLOR, quiet mode filter)
├── error.rs          (modified — +CliError enum, format_cli_error)
├── cli/
│   ├── mod.rs        (NEW — re-export cli submodule types)
│   ├── error.rs      (NEW — CliError, CliExit, format_cli_error)
│   ├── completions.rs (NEW — shell completion generation)
│   ├── config.rs     (NEW — ConfigDefaults, load_config, merge with Args)
│   └── summary.rs    (NEW — ScrapeSummary struct, display formatting)
├── built.rs          (GENERATED by build.rs — git hash, build timestamp)
```

### Existing Files Modified

| File | Change |
|------|--------|
| `Cargo.toml` | +indicatif 0.17, +clap_complete 4, +toml 0.8, +built 0.7 |
| `build.rs` | +built::write_built_file() with git2+chrono features |
| `src/lib.rs` | +quiet, +dry-run fields, `env` on all args, `next_help_heading` groups |
| `src/main.rs` | progress bars, dry-run branch, summary printing, pre-flight, exit codes |
| `src/config.rs` | `init_logging_dual()`, NO_COLOR emoji stripping, quiet mode filtering |
| `src/error.rs` | +CliError enum, +format_cli_error() |

### Module Exports (src/cli/mod.rs)

```rust
pub mod completions;
pub mod config;
pub mod error;
pub mod summary;

pub use error::{CliError, CliExit, format_cli_error};
pub use config::{ConfigDefaults, load_config};
pub use summary::ScrapeSummary;
```

### lib.rs Re-exports

```rust
// In src/lib.rs
pub mod cli;  // NEW — expose cli module publicly for testing
```

---

## 3. Dependency Changes

### New Dependencies in Cargo.toml

```toml
# Progress bars with Unicode support
indicatif = { version = "0.17", features = ["improved_unicode"] }
# Why: spec requires ProgressBar for discovery and scraping phases.
# 0.17 is the stable series with ProgressDrawTarget::stderr() support.
# "improved_unicode" enables proper Unicode width calculation.

# Shell completion generation
clap_complete = "4"
# Why: spec R8 requires bash/zsh/fish/elvish/powershell completions.
# Major version 4 aligns with existing clap 4 — same minor version series.

# TOML config file parsing
toml = "0.8"
# Why: spec R9 requires config.toml support. 0.8 is the current stable,
# compatible with serde 1.x used project-wide.

# Build-time metadata (git hash, timestamps)
built = { version = "0.7", features = ["git2", "chrono"] }
# Why: spec R5 requires version string include git commit hash.
# "git2" extracts current commit, "chrono" adds build timestamp.
# build.rs calls built::write_built_file() to generate src/built.rs.
```

### No Version Changes

All existing dependencies remain at their current versions. No conflicts expected:
- `indicatif 0.17` has no transitive overlap with existing deps
- `clap_complete 4` uses the same `clap 4` as the project
- `toml 0.8` brings its own serde-compatible parser, no conflict with serde_json 1
- `built 0.7` is a build-time dep only (dev-dependency or build-dependency)

**Note**: `built` goes in `[build-dependencies]`, not `[dependencies]`:

```toml
[build-dependencies]
built = { version = "0.7", features = ["git2", "chrono"] }
```

---

## 4. Data Flow

### 4.1 Tracing to stderr

```
main() starts
  │
  ├─ Args::parse()
  │
  ├─ init_logging_dual(level, quiet, no_color)
  │     │
  │     ├─ EnvFilter: "rust_scraper={level},tokio=warn,reqwest=warn"
  │     │   (if quiet: "rust_scraper=warn,tokio=warn,reqwest=warn")
  │     │
  │     ├─ fmt::layer()
  │     │   .with_writer(|| std::io::stderr())  ← ALL output to stderr
  │     │   .with_target(true)
  │     │
  │     └─ (if NO_COLOR) EmojiStripLayer
  │         Intercepts each record, strips emoji from message
  │
  ├─ Rest of main() — all info!/debug!/warn!/error! go to stderr
  │
  └─ Progress bars → ProgressDrawTarget::stderr() (also stderr)
```

### 4.2 Progress Bar Flow (Discovery)

```
info!("Discovering URLs...")
  │
  ├─ let pb = ProgressBar::new_spinner()  ← spinner, total unknown
  │   pb.set_draw_target(ProgressDrawTarget::stderr())
  │   pb.enable_steady_tick(Duration::from_millis(100))
  │   pb.set_style(spinner_style())
  │
  ├─ let urls = discover_urls_for_tui(...).await
  │   (spinner ticks during await via steady_tick)
  │
  ├─ pb.finish_with_message("Found {} URLs")
  │
  └─ (if quiet) pb abandon / don't create pb at all
```

### 4.3 Progress Bar Flow (Scraping)

```
// main.rs loop (NOT inside application layer):
let pb = ProgressBar::new(urls_to_scrape.len() as u64)
pb.set_style(progress_style())  // [====>     ] 3/10 | 2.1 URL/s | ETA: 3s

for url in urls_to_scrape {
    pb.set_message(format!("Scraping: {}", url));
    match scrape_single_url(url, &scraper_config).await {
        Ok(content) => { results.push(content); pb.inc(1); }
        Err(e) => { failures.push(e); pb.inc(1); }
    }
}
pb.finish_with_message("Scraping complete: {} succeeded, {} failed", ...)
```

This requires exposing a per-URL scrape function. We add `scrape_single_url_for_tui(url, config) -> Result<ScrapedContent>` to the public API in `lib.rs`, re-exported from the application layer.

### 4.4 Summary Output Flow

```
After scraping completes:
  │
  ├─ ScrapeSummary::new(discovered, succeeded, failed, extracted, assets, duration)
  │
  ├─ if !args.quiet:
  │     print!("{}", summary.display(no_color))  ← to stderr
  │
  └─ Return CliExit based on failure count
```

### 4.5 Pre-flight Check Flow

```
After URL validation, before discovery:
  │
  ├─ let client = create_http_client()?
  ├─ let response = client.head(&args.url).send().await?
  ├─ match response.status() {
  │     2xx, 3xx, 4xx → Ok(()) (continue — 4xx is not connectivity failure)
  │     5xx → warn, continue
  │     DNS error / timeout → PreflightFailed(error) → exit 69
  │   }
```

---

## 5. Backward Compatibility

### 5.1 Additive-Only Changes

All changes are additive — no breaking changes:
- **Args struct**: New fields have defaults (`quiet: false`, `dry_run: false`)
- **env attributes**: Additive to existing clap args, don't change behavior when env vars unset
- **next_help_heading**: Clap metadata only, no behavioral change
- **New public functions**: `scrape_single_url_for_tui()`, `format_cli_error()`, etc.
- **New modules**: `cli/` is entirely new, no existing module signatures change

### 5.2 Config File Graceful Degradation

- Missing config file → defaults used, no error
- Invalid config file → CliError(ConfigFile) with exit 78
- Partial config (only some defaults set) → unspecified fields remain at clap defaults

### 5.3 Exit Code Compatibility

The current `main()` returns `anyhow::Result<()>` which maps to exit 0 on success, exit 1 on error. The new `CliExit` type extends this with sysexits codes but:
- Success still returns 0
- Failures still return non-zero (just more specific)
- Exit 1 is no longer used — replaced with specific codes

This is backward compatible since non-zero means error regardless of exact value.

### 5.4 Version Output

The extended version string includes more info but remains parseable by tools looking for the leading `rust-scraper 1.0.7` prefix.

---

## 6. Performance Analysis

### 6.1 Progress Bar Overhead

**Progress bars (indicatif)**:
- `ProgressBar::new_spinner()`: negligible, uses a timer thread that ticks at 100ms
- `ProgressBar::new(n)`: O(1) creation, O(1) per `inc()` call
- Rendering cost: terminal refresh at ~10fps by default, negligible CPU
- Memory: ~2KB per progress bar (internal state + style)

**Impact**: Imperceptible. The dominant cost is HTTP requests (network I/O), not progress bar rendering.

### 6.2 Config File Parsing Overhead

**TOML parsing (`toml` crate)**:
- File size: expected <1KB (simple key-value pairs)
- Parse time: <10μs for typical config
- Deserialization via serde: <50μs
- Only parsed once at startup

**Impact**: Negligible. The file is read and parsed once during startup before any network operations.

### 6.3 Emoji Stripping Overhead

**EmojiStripLayer** (NO_COLOR mode):
- Runs as a tracing layer on every log record
- Uses regex: `[\u{1F300}-\u{1F9FF}\u{2600}-\u{26FF}\u{2700}-\u{27BF}\u{FE00}-\u{FE0F}\u{1F000}-\u{1F02F}\u{1F0A0}-\u{1F0FF}]`
- Regex compilation: done once at startup via `LazyLock`
- Per-record cost: O(n) where n is message length, but messages are short (<200 chars)

**Impact**: Negligible. Even at trace-level verbosity with hundreds of messages per second, regex matching on short strings is sub-microsecond.

### 6.4 Built Metadata Overhead

**`built` crate**:
- Runs at compile time only, not at runtime
- Adds ~5s to compile time for git2 operations (one-time)
- Zero runtime overhead — `src/built.rs` is a static file with const strings

### 6.5 Memory Impact

New data structures added to the hot path:
- `ScrapeSummary`: 5 `usize` fields + `Duration` = ~48 bytes
- `CliError` enum: 2 Strings each variant (configurable)
- `ConfigDefaults`: ~10 fields, mostly primitives = ~64 bytes

**Impact**: Negligible compared to existing allocations (ScrapedContent ~several KB, HTTP response bodies ~MB range).

---

## 7. NO_COLOR Implementation Approach

### 7.1 Detection

```rust
// In src/config.rs
pub fn is_no_color() -> bool {
    std::env::var_os("NO_COLOR").map_or(false, |v| !v.is_empty())
}

pub fn should_emit_emoji() -> bool {
    !is_no_color()
}
```

### 7.2 Tracing Emoji Stripping

Implement `EmojiStripLayer` as a tracing-subscriber layer:

```rust
struct EmojiStripLayer;

impl<S> Layer<S> for EmojiStripLayer
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        // Intercept the event's message, strip emojis, forward
        // Uses a Visitor pattern to capture and transform the formatted message
    }
}
```

**Practical approach**: Instead of a full layer (complex), use a formatting function at the layer level:
- `fmt::layer().with_ansi(!is_no_color())` — indicatif respects this natively
- For emoji in log messages: use a `format_event` closure in `fmt::layer()` that strips emoji before rendering

**Simpler approach per spec**: Replace emoji in log messages with ASCII equivalents:
```rust
// In config.rs, when building the layer:
let no_color = is_no_color();
let fmt_layer = fmt::layer()
    .with_writer(|| std::io::stderr())
    .with_ansi(!no_color);
    // Custom event formatter strips emojis
```

### 7.3 Progress Bar NO_COLOR

`indicatif` natively respects `NO_COLOR`:
- When `NO_COLOR` is set, `ProgressDrawTarget::stderr()` automatically disables ANSI styling
- Unicode spinner characters are replaced with ASCII equivalents
- No additional code needed beyond not explicitly forcing ANSI

### 7.4 Summary Output NO_COLOR

`ScrapeSummary::display(no_color: bool)` takes a flag:
- `no_color = true`: uses `[OK]`, `[WARN]`, `[ERROR]` instead of ✅, ⚠️, ❌
- `no_color = true`: ASCII separator lines `---` instead of `━`
- `no_color = false`: full emoji and Unicode box-drawing characters

### 7.5 NO_COLOR Checkpoints

Emoji is present in these locations and must be suppressed:
1. `main.rs` log messages: `"🚀"`, `"📌"`, `"📁"`, `"🔄"`, `"✅"`, `"🖼️"`, `"📄"`, `"🔍"`, `"⚠️"`, `"🎮"`, `"📡"`, `"🎯"`, `"⏭️"`, `"🕷️"`, `"💾"`, `"📦"`, `"🎉"`, `"📊"`, `"📈"` → replace with ASCII text via helper
2. `src/cli/summary.rs`: emoji icons → conditional on `no_color` param
3. `src/cli/error.rs`: error prefix emoji → conditional on `no_color`

**Refactoring approach**: Create `msg::` helper module or use inline conditionals. Given scope, inline `if should_emit_emoji() { "✅" } else { "OK" }` pattern is simplest.

---

## 8. Exit Code Flow

### 8.1 CliExit Type

```rust
#[derive(Debug)]
pub enum CliExit {
    Success,                    // exit 0
    UsageError(String),         // exit 64 (EX_USAGE)
    NetworkError(String),       // exit 69 (EX_UNAVAILABLE)
    IoError(String),            // exit 74 (EX_IOERR)
    ProtocolError(String),      // exit 76 (EX_PROTOCOL)
    ConfigError(String),        // exit 78 (EX_CONFIG)
    PartialSuccess { success: usize, failed: usize }, // exit 69
}

impl std::process::Termination for CliExit {
    fn report(self) -> std::process::ExitCode {
        match self {
            Self::Success => ExitCode::SUCCESS,
            Self::UsageError(_) => ExitCode::from(64),
            Self::NetworkError(_) => ExitCode::from(69),
            Self::IoError(_) => ExitCode::from(74),
            Self::ProtocolError(_) => ExitCode::from(76),
            Self::ConfigError(_) => ExitCode::from(78),
            Self::PartialSuccess { .. } => ExitCode::from(69),
        }
    }
}
```

### 8.2 main() Signature Change

```rust
// OLD:
async fn main() -> anyhow::Result<()> { ... }

// NEW:
async fn main() -> CliExit { ... }
```

### 8.3 Exit Code Decision Tree in main()

```
main() -> CliExit
  │
  ├─ Args::parse() fails (clap) → clap prints error, exits 64 automatically
  │
  ├─ Config file parse error → CliExit::ConfigError(msg) → exit 78
  │
  ├─ URL validation fails → CliExit::UsageError(msg) → exit 64
  │
  ├─ Pre-flight HEAD fails → CliExit::NetworkError(msg) → exit 69
  │
  ├─ User agent load warning → continue (non-fatal)
  │
  ├─ Discovery returns 0 URLs:
  │   ├─ --dry-run → print nothing (or "0 URLs"), exit 0
  │   └─ normal → warn, exit 0 (not a failure, just no content)
  │
  ├─ --dry-run after discovery → print URLs, exit 0
  │
  ├─ Scraping:
  │   ├─ 0 failures → CliExit::Success → exit 0
  │   ├─ Some failures, some successes → CliExit::PartialSuccess → exit 69
  │   └─ All failures (WAF):
  │       ├─ WAF detected → CliExit::ProtocolError → exit 76
  │       └─ Network errors → CliExit::NetworkError → exit 69
  │
  └─ File export error → CliExit::IoError(msg) → exit 74
```

### 8.4 Clap Error Handling

When clap encounters invalid arguments (missing required `--url`, invalid value types), it automatically:
- Prints usage to stderr
- Exits with code 2 (clap's default for errors)

The spec requires exit code 64 (EX_USAGE) for invalid CLI args. Solution: wrap clap parsing:

```rust
let args = match Args::try_parse() {
    Ok(args) => args,
    Err(e) => {
        e.exit(); // clap's exit() already uses code 2; we need 64
        // Alternative: print the error ourselves and return UsageError
    }
};
```

**Better approach**: Use `Args::parse()` but then set `Args::error()` to use exit code 64:

```rust
#[command(name = "rust-scraper", args_conflicts_with_subcommands = true)]
// In main:
let args = match Args::try_parse() {
    Ok(args) => args,
    Err(e) => {
        eprintln!("{}", e);
        return CliExit::UsageError("Invalid arguments".into());
    }
};
```

This ensures exit code 64 instead of clap's default 2.

### 8.5 Completions Subcommand

```rust
#[derive(Subcommand)]
enum CliCommand {
    /// Generate shell completions
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
}
```

When `completions <shell>` is invoked:
1. Generate completion script via `clap_complete`
2. Print to stdout
3. Return `CliExit::Success` (exit 0)
4. Do NOT run scraping pipeline

This is handled via `args_conflicts_with_subcommands = true` on the `Args` struct, or by making `completions` a separate subcommand with its own handler in `main()`.

---

## 9. Risk Analysis

### 9.1 Risk: Modifying Application Layer for Per-URL Scraping

**Risk**: Exposing `scrape_single_url_for_tui` couples the application layer to per-URL orchestration.

**Mitigation**: The function already exists or can be trivially extracted from the current `scrape_urls_for_tui` loop (which internally iterates with `futures::stream::iter().map().buffer_unordered()`). Making the per-URL function public is a minor refactor.

### 9.2 Risk: Tracing Initialization Order

**Risk**: Any `info!` before `init_logging_dual()` is silently dropped.

**Mitigation**: Ensure logging init is the very first call after `Args::parse()`. No other code in `main()` emits logs before this point.

### 9.2 Risk: Config File Schema Drift

**Risk**: As new flags are added, the config file schema must be updated.

**Mitigation**: Use struct-level serde with `#[serde(default)]` — unknown config keys are ignored, missing keys get defaults. The `ConfigDefaults` struct is defined once and reused.

### 9.3 Risk: Emoji Regex Performance

**Risk**: Complex regex for emoji stripping could be slow.

**Mitigation**: Use `once_cell::sync::LazyLock` to compile the regex once. The regex is only active when `NO_COLOR` is set (rare case).

### 9.4 Risk: Progress Bar and Pipe Output

**Risk**: If user pipes stdout (`rust-scraper ... | jq .`), progress bars on stderr don't interfere but could still cause issues with combined stderr+stdout display.

**Mitigation**: `indicatif` with `ProgressDrawTarget::stderr()` is specifically designed for this. Progress bars use carriage returns (`\r`) to overwrite in place on stderr while stdout remains clean.
