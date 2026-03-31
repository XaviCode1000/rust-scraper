# Skill Registry — rust_scraper

Generated: 2026-03-31

## Project Conventions

- **AGENTS.md**: `/home/gazadev/Dev/apps/rust_scraper/AGENTS.md` — Full project instructions with Clean Architecture rules, error handling, memory optimization, async patterns
- **DEVELOPMENT.md**: `/home/gazadev/Dev/apps/rust_scraper/DEVELOPMENT.md` — Development workflow, tooling, test commands

## Compact Rules

### Rust (rust-skills)
- `own-borrow-over-clone`: Prefer `&T` borrowing over `.clone()`. If clone needed in hot paths, explain WHY.
- `own-slice-over-vec`: Accept `&[T]` not `&Vec<T>`, `&str` not `&String`.
- `err-thiserror-lib`: Use `thiserror` for Domain and Infrastructure error types.
- `err-anyhow-app`: Use `anyhow` for Application and CLI/Binary level.
- `err-no-unwrap-prod`: NEVER use `.unwrap()` in production code. Use `?` or `match`.
- `api-builder-pattern`: Use Builder pattern for complex config construction.
- `api-newtype-safety`: Use newtypes for type-safe distinctions.
- `async-tokio-runtime`: Use Tokio exclusively. No other runtimes.
- `async-no-lock-await`: NEVER hold `Mutex`/`RwLock` across `.await`.
- `async-spawn-blocking`: Use `spawn_blocking` for CPU-intensive work.
- `mem-with-capacity`: Use `with_capacity()` when final size is known or estimable.
- `test-nextest`: Never use `cargo test` — always `cargo nextest run`.
- `test-llvm-cov`: Never use `cargo tarpaulin` — always `cargo llvm-cov`.
- `anti-unwrap-abuse`: No `.unwrap()` in production code. Ever.
- `anti-lock-across-await`: No locks held across `.await` — deadlock guarantee.

### GitHub Actions / CI
- Optimize for runner minutes — avoid redundant installs
- Use concurrency groups to cancel superseded runs
- Cache aggressively (cargo, system deps)
- Use `--frozen` or `--locked` for reproducible builds
- Separate fast-fail jobs from slow jobs

## User Skills (trigger table)

| Skill | Trigger |
|-------|---------|
| rust-skills | Writing/reviewing Rust code |
| sdd-explore | Investigating codebase before changes |
| sdd-propose | Creating change proposals |
| sdd-spec | Writing specifications |
| sdd-design | Technical design documents |
| sdd-tasks | Breaking down into tasks |
| sdd-apply | Implementing tasks |
| sdd-verify | Validating implementation |
| sdd-archive | Archiving completed changes |
| branch-pr | Creating pull requests |
| issue-creation | Creating GitHub issues |
