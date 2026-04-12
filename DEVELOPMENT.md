# Development Workflow — Rust Scraper

## 🎉 Latest Achievements

**Tests:** **~271 passing** (nextest)
**Status:** ✅ **All tests passing, 0 failing**
**Version:** v1.1.0 — Vault Auto-Detect & Quick-Save

### v1.1.0 Highlights
- **Vault auto-detect:** 4-tier resolution (CLI > env > config > auto-scan)
- **Quick-save mode:** `--obsidian --quick-save` bypasses TUI, saves to vault inbox
- **Rich metadata:** readingTime, language, wordCount, contentType, status for Dataview
- **Obsidian URI:** Opens saved notes in Obsidian via `obsidian://open` (Linux)
- **Obsidian Markdown export:** Wiki-links, relative asset paths, tags in frontmatter
- **36 new tests** covering vault detection, metadata generation, and URI building
- **New modules:** `src/infrastructure/converter/obsidian.rs`, `src/infrastructure/obsidian/`
- **New dependencies:** `pathdiff = "0.2"`, `whatlang = "0.18"`, `urlencoding = "2.1"`, `slug = "0.1"`
- **Backward compatible:** All flags optional, zero breaking changes

### SPA Detection (unreleased)
- **SPA Detection:** `detect_spa_content()` heuristic in ScraperService warns when pages return minimal content
- **JsRenderer Trait:** Forward-compatible domain trait for headless browser rendering
- **6 new tests:** SPA detection unit tests covering threshold, markers, and edge cases

### v1.0.7 Highlights
- **SRE Hardening:** WAF/CAPTCHA detection (19 signatures), fs2 file locking, OOM protection, TUI panic safety
- **Pure Rust:** Zero FFI dependencies (removed zvec-sys stub, bumpalo dead code)
- **AI Safety:** Fixed P0 bug — `debug_assert_eq!` → `assert_eq!` in `ModelInput::new()` (was silent in --release)
- **Network Hardening:** `connect_timeout(10s)` + `pool_max_idle_per_host` for resilient scraping

---

## 🚀 Quick Start

```bash
# Install tools (one-time)
cargo binstall just cargo-nextest cargo-llvm-cov cargo-audit cargo-machete

# Run tests (HDD-optimized)
just test

# Run full check (fmt + clippy)
just check

# Run background checker
bacon
```

---

## 📦 Stack Óptimo 2025-26

| Herramienta | Versión | Propósito |
|-------------|---------|-----------|
| **Rust** | 1.88 | MSRV del proyecto |
| **just** | 1.49 | Task runner (orquesta tareas) |
| **cargo-nextest** | 0.9+ | Test runner (paraleliza) |
| **cargo-llvm-cov** | latest | Cobertura nativa LLVM |
| **cargo-audit** | latest | Vulnerabilidades conocidas |
| **cargo-deny** | 0.16+ | Licencias y deps |
| **cargo-machete** | 0.9+ | Deps huérfanas |
| **sccache** | 0.14 | Cache de compilación |
| **bacon** | latest | Background checker |
| **mold** | 2.40 | Linker rápido |

---

## 🛠️ Commands

### Just Recipes (preferred)

```bash
just                # default: check (fmt + clippy)
just check          # fmt + clippy --all-targets --all-features
just check-fast     # cargo check
just test           # cargo nextest run --test-threads 2
just test-ai        # nextest with --features ai
just audit          # cargo audit + cargo deny check + cargo machete
just cov            # cargo llvm-cov --html
just fmt            # cargo fmt
just build-release  # cargo build --release
```

### Raw Commands

### Tests

```bash
# Nextest (preferred)
cargo nextest run --test-threads 2

# Run only failed tests
cargo nextest run --failed

# Run ignored tests (real sites)
cargo nextest run --run-ignored ignored-only
```

### Cobertura

```bash
# LLVM-Cov (preferred)
cargo llvm-cov nextest --html --output-dir coverage-llvm
```

### Build

```bash
# Standard build
cargo build --release

# With sccache (auto-configured via RUSTC_WRAPPER)
sccache --show-stats  # View cache stats
```

### Linting

```bash
# Clippy with warnings as errors
cargo clippy -- -D warnings

# Auto-fix
cargo clippy --fix -- -D warnings
```

### Formatting

```bash
# Check format
cargo fmt --check

# Format code
cargo fmt
```

### Background Checking (Bacon)

```bash
# Run bacon (auto-runs clippy on changes)
bacon

# Keybindings: t=nextest, c=clippy, n=nextest, r=run
```

### Audit

```bash
# Security vulnerabilities
cargo audit

# License and dependency policy
cargo deny check

# Unused dependencies
cargo machete
```

---

## 📊 Performance Comparison

| Task | Traditional | Optimized 2025-26 | Mejora |
|------|-------------|-------------------|--------|
| **Tests** | `cargo test` (~30s) | `cargo nextest` (~6s) | **~5x** |
| **Coverage** | `tarpaulin` (5min) | `llvm-cov` (30s) | **~10x** |
| **Build** | Clean (60s) | `sccache` (10s) | **~6x** |
| **Linting** | Manual | `bacon` (instant) | **Instant** |
| **Multi-step** | Manual secuencial | `just audit` (un paso) | **Orquestado** |

---

## 🔧 Configuration

### nextest.toml

Located at project root. Optimizado para HDD:

```toml
[profile.default]
threads-required = 2
retries = 2
slow-timeout = { period = "60s", terminate-after = 3 }

[profile.ci]
threads-required = 4
retries = 0
```

### bacon.toml

Background checker config:

```toml
summary = true

[keybindings]
t = "nextest"
f = "nextest --failed"
c = "clippy"
r = "build --release"
```

### `.cargo/config.toml`

```toml
# Cargo configuration for local development
# sccache is configured globally (~/.cargo/config.toml) via rustc-wrapper
# CI uses Swatinem/rust-cache@v2 instead
```

Local config is intentionally minimal — `sccache`, `mold`, and `sparse registry` are configured **globally** in `~/.cargo/config.toml` because they're machine-level optimizations, not project-specific settings.

### justfile

Task runner for multi-step orchestration:

```makefile
just check        # fmt + clippy strict
just test         # nextest --test-threads 2
just audit        # audit + deny + machete
just cov          # coverage report
```

Complements `bacon` (inner loop watch mode) — `just` is for explicit one-off commands.

### sccache Stats

```bash
# Start server (usually auto-started)
sccache --start-server

# View stats
sccache --show-stats

# Zero stats
sccache --zero-stats
```

---

## 📁 Project Structure

```
rust_scraper/
├── .cargo/
│   └── config.toml          # Minimal — sccache/mold are global
├── src/
│   ├── application/
│   │   └── http_client.rs   # HttpClient wrapper (Option A)
│   ├── domain/
│   ├── infrastructure/
│   └── adapters/
├── tests/
│   ├── http_client_integration.rs  # Real site tests (ignored)
│   └── ai_integration.rs            # AI tests (feature-gated)
├── docs/
├── justfile                 # Task runner recipes
├── nextest.toml             # Test configuration
├── bacon.toml              # Background checker
├── deny.toml               # License/dependency policy
└── Cargo.toml
```

---

## 🎯 Testing Workflow

### 1. Daily Development

```bash
# Terminal 1: Run bacon (auto-runs clippy + tests)
bacon

# Terminal 2: Edit code → see results instantly
```

### 2. Before Commit

```bash
# Option A: Just recipe (preferred)
just check && just test

# Option B: Manual
cargo fmt
cargo clippy -- -D warnings
cargo nextest run
```

### 3. Coverage Check

```bash
# Generate and open coverage report
cargo llvm-cov nextest --html
open coverage-llvm/index.html
```

---

## 🐛 Troubleshooting

### sccache no funciona

```bash
# Verificar servidor
sccache --show-stats

# Reiniciar
sccache --stop-server
sccache --start-server
```

### Nextest falla

```bash
# Limpiar build
cargo clean

# Reintentar
cargo nextest run
```

### Cobertura no genera

```bash
# Limpiar artifacts
cargo clean

# Regenerar
cargo llvm-cov nextest --clean --html
```

## ⚡ Build Performance

### Why the first build takes ~7 minutes

The initial compilation is dominated by heavy crates that compile native code:

| Crate | Time | Reason |
|-------|------|--------|
| `tract-onnx` | ~3 min | ONNX runtime (C++ codegen) |
| `syntect` | ~1-2 min | Oniguruma regex engine (C bindings) |
| `tokenizers` | ~1 min | NLP tokenization |
| `ring` | ~30s | Cryptography (C bindings) |

### Speed it up with sccache

```bash
# Set sccache as the Rust compiler wrapper
export RUSTC_WRAPPER=sccache

# Start the sccache server
sccache --start-server

# Now build — first time is slow, subsequent builds are instant
cargo build --release
```

**Expected improvement:**
- First build: ~7 min (unchanged — must compile everything)
- Rebuild after small change: **~10-30 seconds** (sccache hits)
- Rebuild after `git pull`: **~1-2 min** (only changed crates recompile)

### Build without AI features (saves ~3 minutes)

```bash
# Standard build (no AI/ONNX)
cargo build --release

# With all stable features (images, documents)
cargo build --release --features full
```

### Duplicate dependencies (intentional)

`cargo tree` shows duplicate versions of `selectors`, `dashmap`, `lru`, and `quick-xml`.
This is **expected and unavoidable** — they come from different upstream crates that we all need.
See comments in `Cargo.toml` for details. Do NOT try to unify them.

---

## 📚 Resources

- [cargo-nextest docs](https://nexte.st/)
- [cargo-llvm-cov docs](https://github.com/taiki-e/cargo-llvm-cov)
- [sccache docs](https://github.com/mozilla/sccache)
- [bacon docs](https://dystroy.org/bacon/)
- [just docs](https://github.com/casey/just)
- [cargo-deny docs](https://embarkstudios.github.io/cargo-deny/)
- [cargo-audit docs](https://github.com/rustsec/rustsec/tree/main/cargo-audit)
- [cargo-machete docs](https://github.com/bnjbvr/cargo-machete)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

---

**Last updated**: 2026-04-12
**Rust version**: 1.88
**Stack**: sccache + mold + nextest + just + bacon + cargo-deny + cargo-audit + cargo-machete
**Tests**: ~271 passing (nextest)